//! HTTP response handling.

use crate::client::ClientError;
use crate::headers::HeaderMap;
use bytes::Bytes;
use encoding_rs::Encoding;
use http::StatusCode;
use mime::Mime;
use serde::de::DeserializeOwned;
use url::Url;

/// An HTTP response.
pub struct Response {
    /// Response status code.
    pub status: StatusCode,
    /// Response headers.
    pub headers: HeaderMap,
    /// Final URL (after redirects).
    pub url: Url,
    /// Response body.
    body: Option<Bytes>,
    /// Content type.
    content_type: Option<Mime>,
}

impl Response {
    /// Create a response from reqwest response.
    pub(crate) async fn from_reqwest(response: reqwest::Response) -> Result<Self, ClientError> {
        let status = response.status();
        let url = response.url().clone();

        // Convert headers
        let mut headers = HeaderMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.as_str().to_string(), v.to_string());
            }
        }

        // Get content type
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        // Get body
        let body = response
            .bytes()
            .await
            .map_err(|e| ClientError::Response(e.to_string()))?;

        Ok(Self {
            status,
            headers,
            url,
            body: Some(body),
            content_type,
        })
    }

    /// Get the response status code.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Check if the response was successful (2xx).
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the response was a redirect (3xx).
    pub fn is_redirect(&self) -> bool {
        self.status.is_redirection()
    }

    /// Check if the response was a client error (4xx).
    pub fn is_client_error(&self) -> bool {
        self.status.is_client_error()
    }

    /// Check if the response was a server error (5xx).
    pub fn is_server_error(&self) -> bool {
        self.status.is_server_error()
    }

    /// Get the response headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a specific header.
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Get the final URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the content type.
    pub fn content_type(&self) -> Option<&Mime> {
        self.content_type.as_ref()
    }

    /// Get the content length from headers.
    pub fn content_length(&self) -> Option<u64> {
        self.headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// Get the body as bytes.
    pub fn bytes(self) -> Result<Bytes, ClientError> {
        self.body.ok_or_else(|| ClientError::Response("Body already consumed".to_string()))
    }

    /// Get the body as text.
    pub fn text(self) -> Result<String, ClientError> {
        let bytes = self.bytes()?;

        // Detect encoding from content-type or BOM
        let encoding = self.detect_encoding(&bytes);

        let (text, _, _) = encoding.decode(&bytes);
        Ok(text.into_owned())
    }

    /// Parse the body as JSON.
    pub fn json<T: DeserializeOwned>(self) -> Result<T, ClientError> {
        let bytes = self.bytes()?;
        serde_json::from_slice(&bytes).map_err(|e| ClientError::Response(e.to_string()))
    }

    /// Detect character encoding.
    fn detect_encoding(&self, bytes: &[u8]) -> &'static Encoding {
        // Check content-type charset
        if let Some(mime) = &self.content_type {
            if let Some(charset) = mime.get_param("charset") {
                if let Some(encoding) = Encoding::for_label(charset.as_str().as_bytes()) {
                    return encoding;
                }
            }
        }

        // Check for BOM
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return encoding_rs::UTF_8;
        }
        if bytes.starts_with(&[0xFF, 0xFE]) {
            return encoding_rs::UTF_16LE;
        }
        if bytes.starts_with(&[0xFE, 0xFF]) {
            return encoding_rs::UTF_16BE;
        }

        // Default to UTF-8
        encoding_rs::UTF_8
    }

    /// Get a reference to the body bytes.
    pub fn body_ref(&self) -> Option<&Bytes> {
        self.body.as_ref()
    }
}

/// Response metadata for caching.
#[derive(Clone, Debug)]
pub struct ResponseMetadata {
    /// Status code.
    pub status: u16,
    /// Content type.
    pub content_type: Option<String>,
    /// Content length.
    pub content_length: Option<u64>,
    /// ETag.
    pub etag: Option<String>,
    /// Last-Modified.
    pub last_modified: Option<String>,
    /// Cache-Control.
    pub cache_control: Option<CacheControl>,
    /// Expires.
    pub expires: Option<String>,
}

impl ResponseMetadata {
    /// Create from response.
    pub fn from_response(response: &Response) -> Self {
        Self {
            status: response.status.as_u16(),
            content_type: response.content_type.as_ref().map(|m| m.to_string()),
            content_length: response.content_length(),
            etag: response.header("etag").cloned(),
            last_modified: response.header("last-modified").cloned(),
            cache_control: response
                .header("cache-control")
                .map(|s| CacheControl::parse(s)),
            expires: response.header("expires").cloned(),
        }
    }

    /// Check if response can be cached.
    pub fn is_cacheable(&self) -> bool {
        // Only cache successful responses
        if self.status < 200 || self.status >= 300 {
            return false;
        }

        // Check cache-control
        if let Some(cc) = &self.cache_control {
            if cc.no_store || cc.no_cache {
                return false;
            }
        }

        true
    }

    /// Get cache max age in seconds.
    pub fn max_age(&self) -> Option<u64> {
        self.cache_control.as_ref().and_then(|cc| cc.max_age)
    }
}

/// Parsed Cache-Control header.
#[derive(Clone, Debug, Default)]
pub struct CacheControl {
    pub max_age: Option<u64>,
    pub s_maxage: Option<u64>,
    pub no_cache: bool,
    pub no_store: bool,
    pub no_transform: bool,
    pub must_revalidate: bool,
    pub proxy_revalidate: bool,
    pub public: bool,
    pub private: bool,
    pub immutable: bool,
}

impl CacheControl {
    /// Parse a Cache-Control header value.
    pub fn parse(value: &str) -> Self {
        let mut result = Self::default();

        for directive in value.split(',').map(|s| s.trim()) {
            let parts: Vec<&str> = directive.splitn(2, '=').collect();
            let name = parts[0].to_lowercase();
            let value = parts.get(1).map(|s| s.trim_matches('"'));

            match name.as_str() {
                "max-age" => result.max_age = value.and_then(|v| v.parse().ok()),
                "s-maxage" => result.s_maxage = value.and_then(|v| v.parse().ok()),
                "no-cache" => result.no_cache = true,
                "no-store" => result.no_store = true,
                "no-transform" => result.no_transform = true,
                "must-revalidate" => result.must_revalidate = true,
                "proxy-revalidate" => result.proxy_revalidate = true,
                "public" => result.public = true,
                "private" => result.private = true,
                "immutable" => result.immutable = true,
                _ => {}
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_parse() {
        let cc = CacheControl::parse("max-age=3600, public, no-transform");
        assert_eq!(cc.max_age, Some(3600));
        assert!(cc.public);
        assert!(cc.no_transform);
        assert!(!cc.no_cache);
    }

    #[test]
    fn test_cache_control_no_store() {
        let cc = CacheControl::parse("no-store");
        assert!(cc.no_store);
        assert_eq!(cc.max_age, None);
    }
}
