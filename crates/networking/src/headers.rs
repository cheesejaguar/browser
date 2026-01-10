//! HTTP header handling.

use indexmap::IndexMap;
use std::fmt;

/// HTTP header map (case-insensitive keys, order-preserving).
#[derive(Clone, Debug, Default)]
pub struct HeaderMap {
    headers: IndexMap<String, String>,
}

impl HeaderMap {
    /// Create a new empty header map.
    pub fn new() -> Self {
        Self {
            headers: IndexMap::new(),
        }
    }

    /// Insert a header.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_lowercase();
        self.headers.insert(name, value.into());
    }

    /// Get a header value.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// Check if a header exists.
    pub fn contains(&self, name: &str) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    /// Remove a header.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.headers.shift_remove(&name.to_lowercase())
    }

    /// Get number of headers.
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Iterate over headers.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.headers.iter()
    }

    /// Clear all headers.
    pub fn clear(&mut self) {
        self.headers.clear();
    }

    /// Get Content-Type header.
    pub fn content_type(&self) -> Option<&String> {
        self.get("content-type")
    }

    /// Get Content-Length header.
    pub fn content_length(&self) -> Option<u64> {
        self.get("content-length").and_then(|v| v.parse().ok())
    }

    /// Get Content-Encoding header.
    pub fn content_encoding(&self) -> Option<&String> {
        self.get("content-encoding")
    }
}

impl fmt::Display for HeaderMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, value) in &self.headers {
            writeln!(f, "{}: {}", name, value)?;
        }
        Ok(())
    }
}

/// Common HTTP headers.
pub mod names {
    pub const ACCEPT: &str = "accept";
    pub const ACCEPT_ENCODING: &str = "accept-encoding";
    pub const ACCEPT_LANGUAGE: &str = "accept-language";
    pub const AUTHORIZATION: &str = "authorization";
    pub const CACHE_CONTROL: &str = "cache-control";
    pub const CONNECTION: &str = "connection";
    pub const CONTENT_ENCODING: &str = "content-encoding";
    pub const CONTENT_LENGTH: &str = "content-length";
    pub const CONTENT_TYPE: &str = "content-type";
    pub const COOKIE: &str = "cookie";
    pub const DATE: &str = "date";
    pub const ETAG: &str = "etag";
    pub const EXPIRES: &str = "expires";
    pub const HOST: &str = "host";
    pub const IF_MODIFIED_SINCE: &str = "if-modified-since";
    pub const IF_NONE_MATCH: &str = "if-none-match";
    pub const LAST_MODIFIED: &str = "last-modified";
    pub const LOCATION: &str = "location";
    pub const ORIGIN: &str = "origin";
    pub const PRAGMA: &str = "pragma";
    pub const REFERER: &str = "referer";
    pub const SET_COOKIE: &str = "set-cookie";
    pub const USER_AGENT: &str = "user-agent";
    pub const VARY: &str = "vary";
    pub const X_CONTENT_TYPE_OPTIONS: &str = "x-content-type-options";
    pub const X_FRAME_OPTIONS: &str = "x-frame-options";
    pub const X_XSS_PROTECTION: &str = "x-xss-protection";
}

/// Content type utilities.
pub mod content_type {
    pub const HTML: &str = "text/html";
    pub const XHTML: &str = "application/xhtml+xml";
    pub const XML: &str = "application/xml";
    pub const TEXT_XML: &str = "text/xml";
    pub const JSON: &str = "application/json";
    pub const JAVASCRIPT: &str = "application/javascript";
    pub const TEXT_JAVASCRIPT: &str = "text/javascript";
    pub const CSS: &str = "text/css";
    pub const PLAIN: &str = "text/plain";
    pub const FORM: &str = "application/x-www-form-urlencoded";
    pub const MULTIPART: &str = "multipart/form-data";
    pub const OCTET_STREAM: &str = "application/octet-stream";

    /// Check if content type is HTML.
    pub fn is_html(content_type: &str) -> bool {
        content_type.starts_with(HTML) || content_type.starts_with(XHTML)
    }

    /// Check if content type is XML.
    pub fn is_xml(content_type: &str) -> bool {
        content_type.starts_with(XML)
            || content_type.starts_with(TEXT_XML)
            || content_type.ends_with("+xml")
    }

    /// Check if content type is JSON.
    pub fn is_json(content_type: &str) -> bool {
        content_type.starts_with(JSON) || content_type.ends_with("+json")
    }

    /// Check if content type is JavaScript.
    pub fn is_javascript(content_type: &str) -> bool {
        content_type.starts_with(JAVASCRIPT) || content_type.starts_with(TEXT_JAVASCRIPT)
    }

    /// Check if content type is CSS.
    pub fn is_css(content_type: &str) -> bool {
        content_type.starts_with(CSS)
    }

    /// Check if content type is text.
    pub fn is_text(content_type: &str) -> bool {
        content_type.starts_with("text/")
    }

    /// Check if content type is an image.
    pub fn is_image(content_type: &str) -> bool {
        content_type.starts_with("image/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_map() {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "text/html");
        headers.insert("Accept", "application/json");

        assert_eq!(headers.get("content-type"), Some(&"text/html".to_string()));
        assert_eq!(headers.get("CONTENT-TYPE"), Some(&"text/html".to_string()));
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_content_type_detection() {
        assert!(content_type::is_html("text/html; charset=utf-8"));
        assert!(content_type::is_json("application/json"));
        assert!(content_type::is_xml("application/xml"));
        assert!(content_type::is_javascript("application/javascript"));
    }
}
