//! HTTP request handling.

use crate::client::{ClientError, HttpClient};
use crate::headers::HeaderMap;
use crate::response::Response;
use bytes::Bytes;
use http::Method;
use serde::Serialize;
use std::time::Duration;
use url::Url;

/// An HTTP request.
#[derive(Clone, Debug)]
pub struct Request {
    /// Request method.
    pub method: Method,
    /// Request URL.
    pub url: Url,
    /// Request headers.
    pub headers: HeaderMap,
    /// Request body.
    pub body: Option<Bytes>,
    /// Request timeout.
    pub timeout: Option<Duration>,
}

impl Request {
    /// Create a new request.
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
        }
    }

    /// Set a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set multiple headers.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        for (name, value) in headers.iter() {
            self.headers.insert(name.clone(), value.clone());
        }
        self
    }

    /// Set the request body.
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Get the URL as a string.
    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }
}

/// Request builder for constructing HTTP requests.
pub struct RequestBuilder<'a> {
    client: &'a HttpClient,
    method: Method,
    url: String,
    headers: HeaderMap,
    body: Option<Bytes>,
    timeout: Option<Duration>,
}

impl<'a> RequestBuilder<'a> {
    pub(crate) fn new(client: &'a HttpClient, method: Method, url: &str) -> Self {
        Self {
            client,
            method,
            url: url.to_string(),
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
        }
    }

    /// Add a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Add multiple headers.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        for (name, value) in headers.iter() {
            self.headers.insert(name.clone(), value.clone());
        }
        self
    }

    /// Set the Content-Type header.
    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    /// Set the Accept header.
    pub fn accept(self, accept: &str) -> Self {
        self.header("Accept", accept)
    }

    /// Set basic authentication.
    pub fn basic_auth(self, username: &str, password: Option<&str>) -> Self {
        let credentials = match password {
            Some(p) => format!("{}:{}", username, p),
            None => username.to_string(),
        };
        let encoded = base64_encode(&credentials);
        self.header("Authorization", format!("Basic {}", encoded))
    }

    /// Set bearer token authentication.
    pub fn bearer_auth(self, token: &str) -> Self {
        self.header("Authorization", format!("Bearer {}", token))
    }

    /// Set the request body.
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set a JSON body.
    pub fn json<T: Serialize>(self, json: &T) -> Result<Self, ClientError> {
        let body = serde_json::to_vec(json)
            .map_err(|e| ClientError::Request(e.to_string()))?;
        Ok(self.content_type("application/json").body(body))
    }

    /// Set a form body.
    pub fn form<T: Serialize>(self, form: &T) -> Result<Self, ClientError> {
        let body = serde_urlencoded::to_string(form)
            .map_err(|e| ClientError::Request(e.to_string()))?;
        Ok(self
            .content_type("application/x-www-form-urlencoded")
            .body(body))
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the request.
    pub fn build(self) -> Result<Request, ClientError> {
        let url = Url::parse(&self.url)
            .map_err(|e| ClientError::InvalidUrl(e.to_string()))?;

        Ok(Request {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout,
        })
    }

    /// Send the request.
    pub async fn send(self) -> Result<Response, ClientError> {
        let request = self.build()?;
        self.client.execute(request).await
    }
}

/// Simple base64 encoding.
fn base64_encode(input: &str) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = base64_writer(&mut buf);
        encoder.write_all(input.as_bytes()).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

/// Base64 writer (simplified).
fn base64_writer(output: &mut Vec<u8>) -> impl std::io::Write + '_ {
    struct Base64Writer<'a>(&'a mut Vec<u8>);

    impl<'a> std::io::Write for Base64Writer<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

            for chunk in buf.chunks(3) {
                let b0 = chunk[0] as usize;
                let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
                let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

                self.0.push(CHARS[b0 >> 2]);
                self.0.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)]);

                if chunk.len() > 1 {
                    self.0.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)]);
                } else {
                    self.0.push(b'=');
                }

                if chunk.len() > 2 {
                    self.0.push(CHARS[b2 & 0x3f]);
                } else {
                    self.0.push(b'=');
                }
            }

            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    Base64Writer(output)
}

/// Request method aliases.
pub mod method {
    pub use http::Method::{DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let url = Url::parse("https://example.com").unwrap();
        let request = Request::new(Method::GET, url)
            .header("Accept", "text/html");

        assert_eq!(request.method, Method::GET);
        assert!(request.headers.get("Accept").is_some());
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode("hello");
        assert_eq!(encoded, "aGVsbG8=");
    }
}
