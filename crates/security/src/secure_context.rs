//! Secure context detection.

use url::Url;
use crate::origin::Origin;

/// Secure context checker.
#[derive(Clone, Debug)]
pub struct SecureContext {
    /// Whether this context is secure.
    is_secure: bool,
    /// The URL of the context.
    url: Option<Url>,
    /// Parent context is secure.
    parent_secure: bool,
}

impl SecureContext {
    /// Create a new secure context.
    pub fn new(url: &Url) -> Self {
        let is_secure = Self::is_url_potentially_trustworthy(url);
        Self {
            is_secure,
            url: Some(url.clone()),
            parent_secure: true,
        }
    }

    /// Create a secure context for a worker.
    pub fn for_worker(url: &Url, parent_secure: bool) -> Self {
        let url_secure = Self::is_url_potentially_trustworthy(url);
        Self {
            is_secure: url_secure && parent_secure,
            url: Some(url.clone()),
            parent_secure,
        }
    }

    /// Check if the context is secure.
    pub fn is_secure(&self) -> bool {
        self.is_secure
    }

    /// Check if an API requires a secure context.
    pub fn requires_secure_context(api_name: &str) -> bool {
        // APIs that require secure context
        matches!(
            api_name,
            "Geolocation"
                | "getUserMedia"
                | "ServiceWorker"
                | "CacheStorage"
                | "Bluetooth"
                | "Credential"
                | "PaymentRequest"
                | "PushManager"
                | "ScreenOrientation"
                | "StorageManager"
                | "Notification"
                | "WebUSB"
                | "WebNFC"
                | "DeviceMotion"
                | "DeviceOrientation"
                | "Clipboard"
        )
    }

    /// Check if an API is allowed in this context.
    pub fn allows_api(&self, api_name: &str) -> bool {
        if Self::requires_secure_context(api_name) {
            self.is_secure
        } else {
            true
        }
    }

    /// Check if a URL is potentially trustworthy.
    pub fn is_url_potentially_trustworthy(url: &Url) -> bool {
        // HTTPS URLs are secure
        if url.scheme() == "https" || url.scheme() == "wss" {
            return true;
        }

        // Localhost is secure
        if let Some(host) = url.host_str() {
            if Self::is_localhost(host) {
                return true;
            }
        }

        // File URLs are secure
        if url.scheme() == "file" {
            return true;
        }

        // Data URLs depend on parent
        if url.scheme() == "data" {
            return false; // Need parent context to determine
        }

        // Blob URLs inherit from their creator
        if url.scheme() == "blob" {
            return false; // Need creator context to determine
        }

        false
    }

    /// Check if a host is localhost.
    fn is_localhost(host: &str) -> bool {
        host == "localhost"
            || host == "127.0.0.1"
            || host == "[::1]"
            || host == "::1"
            || host.ends_with(".localhost")
    }

    /// Check if an origin is potentially trustworthy.
    pub fn is_origin_potentially_trustworthy(origin: &Origin) -> bool {
        // HTTPS origins are secure
        if origin.scheme == "https" || origin.scheme == "wss" {
            return true;
        }

        // Localhost is secure
        if Self::is_localhost(&origin.host) {
            return true;
        }

        // File origins are secure
        if origin.scheme == "file" {
            return true;
        }

        false
    }
}

impl Default for SecureContext {
    fn default() -> Self {
        Self {
            is_secure: false,
            url: None,
            parent_secure: false,
        }
    }
}

/// Secure context for different environment types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextType {
    /// Window/document context.
    Window,
    /// Dedicated worker.
    DedicatedWorker,
    /// Shared worker.
    SharedWorker,
    /// Service worker.
    ServiceWorker,
    /// Worklet.
    Worklet,
}

impl ContextType {
    /// Check if this context type requires secure context for registration.
    pub fn requires_secure_registration(&self) -> bool {
        matches!(self, ContextType::ServiceWorker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_is_secure() {
        let url = Url::parse("https://example.com").unwrap();
        let ctx = SecureContext::new(&url);
        assert!(ctx.is_secure());
    }

    #[test]
    fn test_http_is_not_secure() {
        let url = Url::parse("http://example.com").unwrap();
        let ctx = SecureContext::new(&url);
        assert!(!ctx.is_secure());
    }

    #[test]
    fn test_localhost_is_secure() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let ctx = SecureContext::new(&url);
        assert!(ctx.is_secure());

        let url2 = Url::parse("http://127.0.0.1:3000").unwrap();
        let ctx2 = SecureContext::new(&url2);
        assert!(ctx2.is_secure());
    }

    #[test]
    fn test_file_is_secure() {
        let url = Url::parse("file:///home/user/file.html").unwrap();
        let ctx = SecureContext::new(&url);
        assert!(ctx.is_secure());
    }

    #[test]
    fn test_api_requires_secure_context() {
        assert!(SecureContext::requires_secure_context("Geolocation"));
        assert!(SecureContext::requires_secure_context("ServiceWorker"));
        assert!(!SecureContext::requires_secure_context("localStorage"));
    }
}
