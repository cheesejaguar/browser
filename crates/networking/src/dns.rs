//! DNS resolution and caching.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

/// DNS resolver with caching.
pub struct DnsResolver {
    /// Cache of resolved addresses.
    cache: RwLock<HashMap<String, DnsCacheEntry>>,
    /// Configuration.
    config: DnsConfig,
}

/// DNS configuration.
#[derive(Clone, Debug)]
pub struct DnsConfig {
    /// Cache TTL (time to live).
    pub cache_ttl: Duration,
    /// Negative cache TTL (for failed lookups).
    pub negative_cache_ttl: Duration,
    /// Enable IPv6.
    pub ipv6: bool,
    /// Prefer IPv4 over IPv6.
    pub prefer_ipv4: bool,
    /// Timeout for DNS resolution.
    pub timeout: Duration,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(300),
            negative_cache_ttl: Duration::from_secs(60),
            ipv6: true,
            prefer_ipv4: true,
            timeout: Duration::from_secs(5),
        }
    }
}

/// A DNS cache entry.
#[derive(Clone, Debug)]
struct DnsCacheEntry {
    /// Resolved addresses.
    addresses: Vec<IpAddr>,
    /// Expiration time.
    expires: Instant,
    /// Whether this was a failed lookup.
    negative: bool,
}

impl DnsCacheEntry {
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires
    }
}

impl DnsResolver {
    /// Create a new DNS resolver.
    pub fn new() -> Self {
        Self::with_config(DnsConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: DnsConfig) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Resolve a hostname to IP addresses.
    pub async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, DnsError> {
        // Check cache first
        if let Some(addresses) = self.get_cached(host) {
            return if addresses.is_empty() {
                Err(DnsError::NoAddresses(host.to_string()))
            } else {
                Ok(addresses)
            };
        }

        // Perform resolution
        let result = self.do_resolve(host).await;

        // Cache the result
        match &result {
            Ok(addresses) => self.cache_positive(host, addresses.clone()),
            Err(_) => self.cache_negative(host),
        }

        result
    }

    /// Get socket addresses for a host and port.
    pub async fn resolve_socket_addrs(
        &self,
        host: &str,
        port: u16,
    ) -> Result<Vec<SocketAddr>, DnsError> {
        let addresses = self.resolve(host).await?;
        Ok(addresses
            .into_iter()
            .map(|addr| SocketAddr::new(addr, port))
            .collect())
    }

    /// Get cached addresses.
    fn get_cached(&self, host: &str) -> Option<Vec<IpAddr>> {
        let cache = self.cache.read();
        if let Some(entry) = cache.get(host) {
            if !entry.is_expired() {
                return Some(entry.addresses.clone());
            }
        }
        None
    }

    /// Cache a successful resolution.
    fn cache_positive(&self, host: &str, addresses: Vec<IpAddr>) {
        let entry = DnsCacheEntry {
            addresses,
            expires: Instant::now() + self.config.cache_ttl,
            negative: false,
        };
        self.cache.write().insert(host.to_string(), entry);
    }

    /// Cache a failed resolution.
    fn cache_negative(&self, host: &str) {
        let entry = DnsCacheEntry {
            addresses: Vec::new(),
            expires: Instant::now() + self.config.negative_cache_ttl,
            negative: true,
        };
        self.cache.write().insert(host.to_string(), entry);
    }

    /// Perform the actual DNS resolution.
    async fn do_resolve(&self, host: &str) -> Result<Vec<IpAddr>, DnsError> {
        // Use tokio's DNS resolver
        use tokio::net::lookup_host;

        let addrs: Vec<SocketAddr> = lookup_host(format!("{}:0", host))
            .await
            .map_err(|e| DnsError::Resolution(e.to_string()))?
            .collect();

        if addrs.is_empty() {
            return Err(DnsError::NoAddresses(host.to_string()));
        }

        let mut addresses: Vec<IpAddr> = addrs.into_iter().map(|a| a.ip()).collect();

        // Filter by configuration
        if !self.config.ipv6 {
            addresses.retain(|a| a.is_ipv4());
        }

        // Sort by preference
        if self.config.prefer_ipv4 {
            addresses.sort_by_key(|a| !a.is_ipv4());
        }

        // Remove duplicates
        addresses.dedup();

        if addresses.is_empty() {
            return Err(DnsError::NoAddresses(host.to_string()));
        }

        Ok(addresses)
    }

    /// Clear the cache.
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Remove expired entries from the cache.
    pub fn cleanup_cache(&self) {
        self.cache.write().retain(|_, entry| !entry.is_expired());
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> DnsCacheStats {
        let cache = self.cache.read();
        let total = cache.len();
        let negative = cache.values().filter(|e| e.negative).count();
        DnsCacheStats {
            total_entries: total,
            positive_entries: total - negative,
            negative_entries: negative,
        }
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// DNS error.
#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("DNS resolution failed: {0}")]
    Resolution(String),
    #[error("No addresses found for host: {0}")]
    NoAddresses(String),
    #[error("DNS timeout")]
    Timeout,
}

/// DNS cache statistics.
#[derive(Clone, Debug)]
pub struct DnsCacheStats {
    pub total_entries: usize,
    pub positive_entries: usize,
    pub negative_entries: usize,
}

/// Prefetch DNS for a list of hosts.
pub async fn prefetch_dns(resolver: &DnsResolver, hosts: &[&str]) {
    let futures: Vec<_> = hosts
        .iter()
        .map(|host| resolver.resolve(host))
        .collect();

    futures::future::join_all(futures).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_config_default() {
        let config = DnsConfig::default();
        assert!(config.ipv6);
        assert!(config.prefer_ipv4);
    }

    #[test]
    fn test_dns_cache_entry_expiration() {
        let entry = DnsCacheEntry {
            addresses: vec![],
            expires: Instant::now() - Duration::from_secs(1),
            negative: false,
        };
        assert!(entry.is_expired());

        let entry = DnsCacheEntry {
            addresses: vec![],
            expires: Instant::now() + Duration::from_secs(100),
            negative: false,
        };
        assert!(!entry.is_expired());
    }
}
