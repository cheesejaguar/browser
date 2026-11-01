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
            for set_cookie in response.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(set_cookie) = set_cookie.to_str() {
                    cookies.add_from_response(&request.url, set_cookie);
                }
            }
        }

        // Convert to our response type
        Response::from_reqwest(response).await
    }

    /// Fetch a URL and return the body bytes.
    pub async fn fetch(&self, url: &str) -> Result<Bytes, ClientError> {
        let response = self.get(url).send().await?;
        response.bytes()
    }

    /// Fetch a URL and return the body as text.
    pub async fn fetch_text(&self, url: &str) -> Result<String, ClientError> {
        let response = self.get(url).send().await?;
        response.text()
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

    /// Serve a single canned HTTP/1.1 response on a local port and return the
    /// URL plus a handle resolving to the raw request the server received.
    async fn serve_once(
        response: &'static [u8],
    ) -> (String, tokio::task::JoinHandle<String>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 1024];
            while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = socket.read(&mut chunk).await.unwrap();
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            socket.write_all(response).await.unwrap();
            socket.shutdown().await.unwrap();
            String::from_utf8_lossy(&buf).into_owned()
        });
        (format!("http://{}/page", addr), handle)
    }

    #[tokio::test]
    async fn test_fetch_text_decodes_charset_and_stores_cookies() {
        // Body is "café" encoded as ISO-8859-1 (0xE9 for 'é').
        let (url, server) = serve_once(
            b"HTTP/1.1 200 OK\r\n\
Content-Type: text/plain; charset=iso-8859-1\r\n\
Set-Cookie: session=abc123; Path=/\r\n\
Content-Length: 4\r\n\
Connection: close\r\n\
\r\n\
caf\xE9",
        )
        .await;

        let client = HttpClientBuilder::new().http2(false).build().unwrap();
        let text = client.fetch_text(&url).await.unwrap();
        assert_eq!(text, "caf\u{e9}");

        let raw_request = server.await.unwrap();
        assert!(raw_request.starts_with("GET /page HTTP/1.1\r\n"));

        let parsed = Url::parse(&url).unwrap();
        assert_eq!(client.cookies().read().get_cookie_header(&parsed), "session=abc123");
    }

    #[tokio::test]
    async fn test_stored_cookies_are_sent_on_next_request() {
        let client = HttpClientBuilder::new().http2(false).build().unwrap();

        let (url, server) = serve_once(
            b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n",
        )
        .await;
        client
            .cookies()
            .write()
            .add_from_response(&Url::parse(&url).unwrap(), "token=xyz; Path=/");

        let response = client.get(&url).send().await.unwrap();
        assert_eq!(response.status, http::StatusCode::NO_CONTENT);

        let raw_request = server.await.unwrap().to_ascii_lowercase();
        assert!(raw_request.contains("\r\ncookie: token=xyz\r\n"));
    }
}
