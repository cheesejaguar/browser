//! HTTP client implementation.

use crate::connection::ConnectionPool;
use crate::cookies::CookieJar;
use crate::headers::HeaderMap;
use crate::request::{Request, RequestBuilder};
use crate::response::Response;
use bytes::Bytes;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Semaphore;
use url::Url;

/// HTTP client errors.
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Timeout")]
    Timeout,
    #[error("Too many redirects")]
    TooManyRedirects,
    #[error("Request error: {0}")]
    Request(String),
    #[error("Response error: {0}")]
    Response(String),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("DNS error: {0}")]
    Dns(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// HTTP client for making requests.
pub struct HttpClient {
    /// Inner reqwest client.
    inner: reqwest::Client,
    /// Cookie jar.
    cookies: Arc<RwLock<CookieJar>>,
    /// Default headers.
    default_headers: HeaderMap,
    /// Client configuration.
    config: ClientConfig,
    /// Connection semaphore.
    connection_semaphore: Arc<Semaphore>,
}

/// Client configuration.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Request timeout.
    pub timeout: Duration,
    /// Connect timeout.
    pub connect_timeout: Duration,
    /// Maximum redirects.
    pub max_redirects: u32,
    /// User agent string.
    pub user_agent: String,
    /// Accept encoding.
    pub accept_encoding: Vec<String>,
    /// Maximum connections per host.
    pub max_connections_per_host: usize,
    /// Total maximum connections.
    pub max_total_connections: usize,
    /// Enable HTTP/2.
    pub http2: bool,
    /// Enable cookie storage.
    pub store_cookies: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_redirects: 20,
            user_agent: format!(
                "RustBrowser/1.0 ({})",
                std::env::consts::OS
            ),
            accept_encoding: vec![
                "gzip".to_string(),
                "deflate".to_string(),
                "br".to_string(),
            ],
            max_connections_per_host: 6,
            max_total_connections: 100,
            http2: true,
            store_cookies: true,
        }
    }
}

impl HttpClient {
    /// Create a new HTTP client.
    pub fn new() -> Result<Self, ClientError> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a client with custom configuration.
    pub fn with_config(config: ClientConfig) -> Result<Self, ClientError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            config.user_agent.parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                .parse()
                .unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            "en-US,en;q=0.5".parse().unwrap(),
        );

        if !config.accept_encoding.is_empty() {
            headers.insert(
                reqwest::header::ACCEPT_ENCODING,
                config.accept_encoding.join(", ").parse().unwrap(),
            );
        }

        let mut builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects as usize))
            .default_headers(headers)
            .pool_max_idle_per_host(config.max_connections_per_host)
            .gzip(true)
            .brotli(true)
            .deflate(true);

        if config.http2 {
            builder = builder.http2_prior_knowledge();
        }

        let inner = builder.build().map_err(|e| ClientError::Request(e.to_string()))?;

        Ok(Self {
            inner,
            cookies: Arc::new(RwLock::new(CookieJar::new())),
            default_headers: HeaderMap::new(),
            config: config.clone(),
            connection_semaphore: Arc::new(Semaphore::new(config.max_total_connections)),
        })
    }

    /// Create a GET request builder.
    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self, http::Method::GET, url)
    }

    /// Create a POST request builder.
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self, http::Method::POST, url)
    }

    /// Create a PUT request builder.
    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self, http::Method::PUT, url)
    }

    /// Create a DELETE request builder.
    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self, http::Method::DELETE, url)
    }

    /// Create a HEAD request builder.
    pub fn head(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self, http::Method::HEAD, url)
    }

    /// Execute a request.
    pub async fn execute(&self, request: Request) -> Result<Response, ClientError> {
        // Acquire connection permit
        let _permit = self
            .connection_semaphore
            .acquire()
            .await
            .map_err(|_| ClientError::Connection("Connection limit reached".to_string()))?;

        // Build reqwest request
        let mut req_builder = self.inner.request(
            request.method.clone(),
            request.url.clone(),
        );

        // Add headers
        for (name, value) in request.headers.iter() {
            req_builder = req_builder.header(name.as_str(), value.as_str());
        }

        // Add cookies if enabled
        if self.config.store_cookies {
            let cookies = self.cookies.read();
            let cookie_header = cookies.get_cookie_header(&request.url);
            if !cookie_header.is_empty() {
                req_builder = req_builder.header("Cookie", cookie_header);
            }
        }

        // Add body
        if let Some(body) = request.body {
            req_builder = req_builder.body(body);
        }

        // Execute request
        let response = req_builder
            .send()
            .await
            .map_err(|e| ClientError::Request(e.to_string()))?;

        // Store cookies from response
        if self.config.store_cookies {
            let mut cookies = self.cookies.write();
            for cookie in response.cookies() {
                cookies.add_from_response(&request.url, &cookie.to_string());
            }
        }

        // Convert to our response type
        Response::from_reqwest(response).await
    }

    /// Fetch a URL and return the body bytes.
    pub async fn fetch(&self, url: &str) -> Result<Bytes, ClientError> {
        let response = self.get(url).send().await?;
        response.bytes().await
    }

    /// Fetch a URL and return the body as text.
    pub async fn fetch_text(&self, url: &str) -> Result<String, ClientError> {
        let response = self.get(url).send().await?;
        response.text().await
    }

    /// Get the cookie jar.
    pub fn cookies(&self) -> Arc<RwLock<CookieJar>> {
        self.cookies.clone()
    }

    /// Clear all cookies.
    pub fn clear_cookies(&self) {
        self.cookies.write().clear();
    }

    /// Get client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

/// HTTP client builder.
pub struct HttpClientBuilder {
    config: ClientConfig,
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Set request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set connect timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set maximum redirects.
    pub fn max_redirects(mut self, max: u32) -> Self {
        self.config.max_redirects = max;
        self
    }

    /// Set user agent.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Enable or disable HTTP/2.
    pub fn http2(mut self, enabled: bool) -> Self {
        self.config.http2 = enabled;
        self
    }

    /// Enable or disable cookie storage.
    pub fn store_cookies(mut self, enabled: bool) -> Self {
        self.config.store_cookies = enabled;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<HttpClient, ClientError> {
        HttpClient::with_config(self.config)
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.max_redirects, 20);
        assert!(config.http2);
    }

    #[test]
    fn test_client_builder() {
        let builder = HttpClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .max_redirects(10)
            .http2(false);

        assert_eq!(builder.config.timeout, Duration::from_secs(60));
        assert_eq!(builder.config.max_redirects, 10);
        assert!(!builder.config.http2);
    }
}
