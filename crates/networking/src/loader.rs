//! Resource loading for web content.

use crate::client::{ClientError, HttpClient};
use crate::headers::content_type;
use crate::response::Response;
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, oneshot, Semaphore};
use url::Url;

/// Resource loader for fetching web resources.
pub struct ResourceLoader {
    /// HTTP client.
    client: Arc<HttpClient>,
    /// Configuration.
    config: LoaderConfig,
    /// In-flight requests.
    in_flight: RwLock<HashMap<Url, broadcast::Sender<LoadResult>>>,
    /// Request prioritization.
    priority_queue: RwLock<Vec<PendingRequest>>,
    /// Loading semaphore.
    semaphore: Arc<Semaphore>,
}

/// Loader configuration.
#[derive(Clone, Debug)]
pub struct LoaderConfig {
    /// Maximum concurrent loads.
    pub max_concurrent: usize,
    /// Priority levels.
    pub priority_levels: usize,
    /// Enable deduplication.
    pub deduplicate: bool,
    /// Request timeout.
    pub timeout: Duration,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 6,
            priority_levels: 4,
            deduplicate: true,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Load priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    /// Critical resources (main document).
    Critical = 0,
    /// High priority (CSS, render-blocking scripts).
    High = 1,
    /// Normal priority (images above fold).
    Normal = 2,
    /// Low priority (prefetch, images below fold).
    Low = 3,
}

/// A pending request.
#[derive(Debug)]
struct PendingRequest {
    url: Url,
    priority: LoadPriority,
    requested: Instant,
}

/// Result of loading a resource.
pub type LoadResult = Result<LoadedResource, LoadError>;

/// A loaded resource.
#[derive(Clone, Debug)]
pub struct LoadedResource {
    /// Final URL (after redirects).
    pub url: Url,
    /// Content type.
    pub content_type: Option<String>,
    /// Resource data.
    pub data: Bytes,
    /// HTTP status code.
    pub status: u16,
    /// Resource type.
    pub resource_type: ResourceType,
    /// Load timing.
    pub timing: LoadTiming,
}

/// Resource type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Script,
    Image,
    Font,
    Media,
    XHR,
    Fetch,
    Other,
}

impl ResourceType {
    /// Determine resource type from content type.
    pub fn from_content_type(content_type: &str) -> Self {
        if content_type::is_html(content_type) {
            ResourceType::Document
        } else if content_type::is_css(content_type) {
            ResourceType::Stylesheet
        } else if content_type::is_javascript(content_type) {
            ResourceType::Script
        } else if content_type::is_image(content_type) {
            ResourceType::Image
        } else if content_type.starts_with("font/") || content_type.contains("font") {
            ResourceType::Font
        } else if content_type.starts_with("audio/") || content_type.starts_with("video/") {
            ResourceType::Media
        } else {
            ResourceType::Other
        }
    }
}

/// Load timing information.
#[derive(Clone, Debug, Default)]
pub struct LoadTiming {
    /// When the request started.
    pub start_time: Option<Instant>,
    /// DNS lookup time.
    pub dns_time: Option<Duration>,
    /// Connection time.
    pub connect_time: Option<Duration>,
    /// TLS handshake time.
    pub tls_time: Option<Duration>,
    /// Time to first byte.
    pub ttfb: Option<Duration>,
    /// Total download time.
    pub download_time: Option<Duration>,
    /// Total load time.
    pub total_time: Option<Duration>,
}

/// Load error.
#[derive(Clone, Debug, thiserror::Error)]
pub enum LoadError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("HTTP error: {status}")]
    Http { status: u16, message: String },
    #[error("Timeout")]
    Timeout,
    #[error("Cancelled")]
    Cancelled,
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl From<ClientError> for LoadError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::Timeout => LoadError::Timeout,
            ClientError::InvalidUrl(msg) => LoadError::InvalidUrl(msg),
            _ => LoadError::Network(err.to_string()),
        }
    }
}

impl ResourceLoader {
    /// Create a new resource loader.
    pub fn new(client: Arc<HttpClient>) -> Self {
        Self::with_config(client, LoaderConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(client: Arc<HttpClient>, config: LoaderConfig) -> Self {
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
            in_flight: RwLock::new(HashMap::new()),
            priority_queue: RwLock::new(Vec::new()),
        }
    }

    /// Load a resource.
    pub async fn load(&self, url: &str) -> LoadResult {
        self.load_with_priority(url, LoadPriority::Normal).await
    }

    /// Load a resource with priority.
    pub async fn load_with_priority(&self, url: &str, priority: LoadPriority) -> LoadResult {
        let url = Url::parse(url).map_err(|e| LoadError::InvalidUrl(e.to_string()))?;

        // Check for duplicate in-flight request
        if self.config.deduplicate {
            if let Some(receiver) = self.get_in_flight(&url) {
                // Wait for the existing request
                return receiver
                    .recv()
                    .await
                    .map_err(|_| LoadError::Cancelled)?;
            }
        }

        // Create broadcast channel for this request
        let (tx, _) = broadcast::channel(1);
        self.in_flight.write().insert(url.clone(), tx.clone());

        // Acquire semaphore
        let _permit = self.semaphore.acquire().await.map_err(|_| LoadError::Cancelled)?;

        // Perform the load
        let start = Instant::now();
        let result = self.do_load(&url, priority).await;

        // Record timing
        let total_time = start.elapsed();

        // Remove from in-flight
        self.in_flight.write().remove(&url);

        // Broadcast result
        let _ = tx.send(result.clone());

        result
    }

    /// Get an in-flight request receiver.
    fn get_in_flight(&self, url: &Url) -> Option<broadcast::Receiver<LoadResult>> {
        self.in_flight.read().get(url).map(|tx| tx.subscribe())
    }

    /// Perform the actual load.
    async fn do_load(&self, url: &Url, _priority: LoadPriority) -> LoadResult {
        let start = Instant::now();

        // Make the request
        let response = self
            .client
            .get(url.as_str())
            .timeout(self.config.timeout)
            .send()
            .await
            .map_err(LoadError::from)?;

        let status = response.status().as_u16();

        // Check for HTTP errors
        if response.is_client_error() || response.is_server_error() {
            return Err(LoadError::Http {
                status,
                message: format!("HTTP {}", status),
            });
        }

        let final_url = response.url().clone();
        let content_type = response.content_type().map(|m| m.to_string());
        let resource_type = content_type
            .as_ref()
            .map(|ct| ResourceType::from_content_type(ct))
            .unwrap_or(ResourceType::Other);

        let data = response.bytes().map_err(LoadError::from)?;

        let timing = LoadTiming {
            start_time: Some(start),
            total_time: Some(start.elapsed()),
            ..Default::default()
        };

        Ok(LoadedResource {
            url: final_url,
            content_type,
            data,
            status,
            resource_type,
            timing,
        })
    }

    /// Load multiple resources in parallel.
    pub async fn load_all(&self, urls: &[&str]) -> Vec<LoadResult> {
        let futures: Vec<_> = urls.iter().map(|url| self.load(url)).collect();
        futures::future::join_all(futures).await
    }

    /// Prefetch a resource (low priority).
    pub async fn prefetch(&self, url: &str) -> LoadResult {
        self.load_with_priority(url, LoadPriority::Low).await
    }

    /// Cancel all in-flight requests.
    pub fn cancel_all(&self) {
        self.in_flight.write().clear();
    }

    /// Get number of in-flight requests.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.read().len()
    }
}

/// Resource request for batch loading.
#[derive(Clone, Debug)]
pub struct ResourceRequest {
    pub url: String,
    pub priority: LoadPriority,
    pub resource_type: Option<ResourceType>,
}

impl ResourceRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            priority: LoadPriority::Normal,
            resource_type: None,
        }
    }

    pub fn with_priority(mut self, priority: LoadPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = Some(resource_type);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_detection() {
        assert_eq!(
            ResourceType::from_content_type("text/html; charset=utf-8"),
            ResourceType::Document
        );
        assert_eq!(
            ResourceType::from_content_type("text/css"),
            ResourceType::Stylesheet
        );
        assert_eq!(
            ResourceType::from_content_type("application/javascript"),
            ResourceType::Script
        );
        assert_eq!(
            ResourceType::from_content_type("image/png"),
            ResourceType::Image
        );
    }

    #[test]
    fn test_load_priority_ordering() {
        assert!(LoadPriority::Critical < LoadPriority::High);
        assert!(LoadPriority::High < LoadPriority::Normal);
        assert!(LoadPriority::Normal < LoadPriority::Low);
    }

    #[test]
    fn test_resource_request_builder() {
        let request = ResourceRequest::new("https://example.com/style.css")
            .with_priority(LoadPriority::High)
            .with_type(ResourceType::Stylesheet);

        assert_eq!(request.priority, LoadPriority::High);
        assert_eq!(request.resource_type, Some(ResourceType::Stylesheet));
    }
}
