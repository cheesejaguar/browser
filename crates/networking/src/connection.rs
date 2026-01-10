//! Connection pooling and management.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Connection pool for reusing HTTP connections.
pub struct ConnectionPool {
    /// Pool configuration.
    config: ConnectionPoolConfig,
    /// Connection semaphores per host.
    host_semaphores: Mutex<HashMap<String, Arc<Semaphore>>>,
    /// Global connection semaphore.
    global_semaphore: Arc<Semaphore>,
    /// Connection statistics.
    stats: Mutex<ConnectionStats>,
}

/// Connection pool configuration.
#[derive(Clone, Debug)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per host.
    pub max_per_host: usize,
    /// Maximum total connections.
    pub max_total: usize,
    /// Idle timeout.
    pub idle_timeout: Duration,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Enable HTTP/2.
    pub http2: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_per_host: 6,
            max_total: 100,
            idle_timeout: Duration::from_secs(90),
            connect_timeout: Duration::from_secs(10),
            http2: true,
        }
    }
}

/// Connection statistics.
#[derive(Clone, Debug, Default)]
pub struct ConnectionStats {
    /// Total connections created.
    pub connections_created: u64,
    /// Total connections reused.
    pub connections_reused: u64,
    /// Total connections closed.
    pub connections_closed: u64,
    /// Current active connections.
    pub active_connections: usize,
    /// Current idle connections.
    pub idle_connections: usize,
}

impl ConnectionPool {
    /// Create a new connection pool.
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            global_semaphore: Arc::new(Semaphore::new(config.max_total)),
            config,
            host_semaphores: Mutex::new(HashMap::new()),
            stats: Mutex::new(ConnectionStats::default()),
        }
    }

    /// Acquire a connection permit for a host.
    pub async fn acquire(&self, host: &str) -> ConnectionPermit {
        // Acquire global permit
        let global_permit = self.global_semaphore.clone().acquire_owned().await.unwrap();

        // Acquire host-specific permit
        let host_semaphore = {
            let mut semaphores = self.host_semaphores.lock();
            semaphores
                .entry(host.to_string())
                .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_per_host)))
                .clone()
        };

        let host_permit = host_semaphore.acquire_owned().await.unwrap();

        // Update stats
        {
            let mut stats = self.stats.lock();
            stats.active_connections += 1;
        }

        ConnectionPermit {
            _global_permit: global_permit,
            _host_permit: host_permit,
            host: host.to_string(),
        }
    }

    /// Get connection statistics.
    pub fn stats(&self) -> ConnectionStats {
        self.stats.lock().clone()
    }

    /// Get pool configuration.
    pub fn config(&self) -> &ConnectionPoolConfig {
        &self.config
    }
}

/// A permit for using a connection.
pub struct ConnectionPermit {
    _global_permit: tokio::sync::OwnedSemaphorePermit,
    _host_permit: tokio::sync::OwnedSemaphorePermit,
    host: String,
}

impl ConnectionPermit {
    /// Get the host.
    pub fn host(&self) -> &str {
        &self.host
    }
}

/// Connection state tracking.
#[derive(Clone, Debug)]
pub struct ConnectionState {
    /// Connection ID.
    pub id: u64,
    /// Host.
    pub host: String,
    /// Port.
    pub port: u16,
    /// Protocol (http/1.1 or h2).
    pub protocol: Protocol,
    /// Creation time.
    pub created: Instant,
    /// Last used time.
    pub last_used: Instant,
    /// Number of requests made.
    pub requests: u64,
    /// Is secure (TLS).
    pub secure: bool,
}

/// HTTP protocol version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Http11,
    Http2,
}

impl ConnectionState {
    /// Create a new connection state.
    pub fn new(id: u64, host: String, port: u16, protocol: Protocol, secure: bool) -> Self {
        let now = Instant::now();
        Self {
            id,
            host,
            port,
            protocol,
            created: now,
            last_used: now,
            requests: 0,
            secure,
        }
    }

    /// Mark the connection as used.
    pub fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.requests += 1;
    }

    /// Check if the connection is idle.
    pub fn is_idle(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }

    /// Get age of the connection.
    pub fn age(&self) -> Duration {
        self.created.elapsed()
    }
}

/// Connection key for pooling.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub host: String,
    pub port: u16,
    pub secure: bool,
}

impl ConnectionKey {
    pub fn new(host: impl Into<String>, port: u16, secure: bool) -> Self {
        Self {
            host: host.into(),
            port,
            secure,
        }
    }

    pub fn from_url(url: &url::Url) -> Self {
        let secure = url.scheme() == "https";
        let port = url.port().unwrap_or(if secure { 443 } else { 80 });
        Self {
            host: url.host_str().unwrap_or("").to_string(),
            port,
            secure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_config() {
        let config = ConnectionPoolConfig::default();
        assert_eq!(config.max_per_host, 6);
        assert_eq!(config.max_total, 100);
    }

    #[test]
    fn test_connection_key() {
        let url = url::Url::parse("https://example.com:8443/path").unwrap();
        let key = ConnectionKey::from_url(&url);

        assert_eq!(key.host, "example.com");
        assert_eq!(key.port, 8443);
        assert!(key.secure);
    }

    #[test]
    fn test_connection_state() {
        let state = ConnectionState::new(1, "example.com".to_string(), 443, Protocol::Http2, true);
        assert_eq!(state.requests, 0);
        assert!(state.secure);
    }
}
