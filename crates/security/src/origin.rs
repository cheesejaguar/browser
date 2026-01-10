//! Same-Origin Policy implementation.

use std::fmt;
use url::Url;

/// Represents an origin (scheme, host, port tuple).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
}

impl Origin {
    /// Create a new origin from components.
    pub fn new(scheme: &str, host: &str, port: Option<u16>) -> Self {
        Self {
            scheme: scheme.to_lowercase(),
            host: host.to_lowercase(),
            port,
        }
    }

    /// Parse an origin from a URL.
    pub fn from_url(url: &Url) -> Option<Self> {
        let scheme = url.scheme().to_lowercase();

        // Opaque origins for certain schemes
        if matches!(scheme.as_str(), "data" | "file" | "blob" | "javascript") {
            return None;
        }

        let host = url.host_str()?.to_lowercase();
        let port = url.port_or_known_default();

        Some(Self { scheme, host, port })
    }

    /// Parse an origin from a string URL.
    pub fn parse(url_str: &str) -> Option<Self> {
        let url = Url::parse(url_str).ok()?;
        Self::from_url(&url)
    }

    /// Check if this origin is the same as another.
    pub fn is_same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme
            && self.host == other.host
            && self.effective_port() == other.effective_port()
    }

    /// Check if this origin is same-origin with a URL.
    pub fn is_same_origin_with_url(&self, url: &Url) -> bool {
        if let Some(other) = Origin::from_url(url) {
            self.is_same_origin(&other)
        } else {
            false
        }
    }

    /// Get the effective port (using default ports for known schemes).
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or_else(|| match self.scheme.as_str() {
            "http" => 80,
            "https" => 443,
            "ws" => 80,
            "wss" => 443,
            "ftp" => 21,
            _ => 0,
        })
    }

    /// Check if this is an opaque origin.
    pub fn is_opaque(&self) -> bool {
        false // Non-opaque origins are represented by this struct
    }

    /// Serialize the origin to a string.
    pub fn serialize(&self) -> String {
        format!("{}", self)
    }

    /// Check if two origins are same-site.
    pub fn is_same_site(&self, other: &Origin) -> bool {
        // Same site means same registrable domain
        let self_domain = self.registrable_domain();
        let other_domain = other.registrable_domain();
        self_domain == other_domain
    }

    /// Get the registrable domain (eTLD+1).
    pub fn registrable_domain(&self) -> &str {
        // Simplified: just return the host
        // In a real implementation, this would use the Public Suffix List
        &self.host
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default_port = match self.scheme.as_str() {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        };

        if self.port.is_some() && self.port != default_port {
            write!(f, "{}://{}:{}", self.scheme, self.host, self.port.unwrap())
        } else {
            write!(f, "{}://{}", self.scheme, self.host)
        }
    }
}

/// Opaque origin for data:, file:, etc.
#[derive(Clone, Debug)]
pub struct OpaqueOrigin {
    /// Internal identifier for the opaque origin.
    id: u64,
}

impl OpaqueOrigin {
    /// Create a new unique opaque origin.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            id: COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl Default for OpaqueOrigin {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for OpaqueOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Origin policy for same-origin checks.
#[derive(Clone, Debug)]
pub struct OriginPolicy {
    /// The document's origin.
    document_origin: Option<Origin>,
    /// Whether same-origin policy is enabled.
    enabled: bool,
}

impl OriginPolicy {
    /// Create a new origin policy.
    pub fn new(document_origin: Option<Origin>) -> Self {
        Self {
            document_origin,
            enabled: true,
        }
    }

    /// Disable same-origin policy (for testing only).
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if a request to a URL is allowed.
    pub fn can_request(&self, url: &Url) -> bool {
        if !self.enabled {
            return true;
        }

        match (&self.document_origin, Origin::from_url(url)) {
            (Some(doc), Some(target)) => doc.is_same_origin(&target),
            (None, _) => true, // Opaque origin can't make cross-origin requests
            (_, None) => true, // Requests to opaque origins are allowed
        }
    }

    /// Check if DOM access is allowed to another window.
    pub fn can_access_dom(&self, other_origin: &Option<Origin>) -> bool {
        if !self.enabled {
            return true;
        }

        match (&self.document_origin, other_origin) {
            (Some(doc), Some(other)) => doc.is_same_origin(other),
            (None, None) => false, // Two opaque origins are never same-origin
            _ => false,
        }
    }

    /// Check if reading from a cross-origin resource is allowed.
    pub fn can_read_response(&self, url: &Url, cors_allowed: bool) -> bool {
        if !self.enabled || cors_allowed {
            return true;
        }

        self.can_request(url)
    }

    /// Get the document origin.
    pub fn document_origin(&self) -> Option<&Origin> {
        self.document_origin.as_ref()
    }
}

/// Cross-origin isolation state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossOriginIsolation {
    /// Not isolated.
    None,
    /// Cross-origin isolated (COOP + COEP).
    Isolated,
}

impl CrossOriginIsolation {
    /// Check if SharedArrayBuffer is allowed.
    pub fn allows_shared_array_buffer(&self) -> bool {
        matches!(self, CrossOriginIsolation::Isolated)
    }

    /// Check if high-resolution timers are allowed.
    pub fn allows_high_resolution_timers(&self) -> bool {
        matches!(self, CrossOriginIsolation::Isolated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_from_url() {
        let origin = Origin::parse("https://example.com/path").unwrap();
        assert_eq!(origin.scheme, "https");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.effective_port(), 443);
    }

    #[test]
    fn test_same_origin() {
        let origin1 = Origin::parse("https://example.com/path1").unwrap();
        let origin2 = Origin::parse("https://example.com/path2").unwrap();
        let origin3 = Origin::parse("http://example.com/path").unwrap();
        let origin4 = Origin::parse("https://other.com/path").unwrap();

        assert!(origin1.is_same_origin(&origin2));
        assert!(!origin1.is_same_origin(&origin3)); // Different scheme
        assert!(!origin1.is_same_origin(&origin4)); // Different host
    }

    #[test]
    fn test_origin_with_port() {
        let origin1 = Origin::parse("https://example.com:443/path").unwrap();
        let origin2 = Origin::parse("https://example.com/path").unwrap();
        let origin3 = Origin::parse("https://example.com:8443/path").unwrap();

        assert!(origin1.is_same_origin(&origin2)); // Same effective port
        assert!(!origin1.is_same_origin(&origin3)); // Different port
    }

    #[test]
    fn test_origin_policy() {
        let origin = Origin::parse("https://example.com").unwrap();
        let policy = OriginPolicy::new(Some(origin));

        let same_origin_url = Url::parse("https://example.com/api").unwrap();
        let cross_origin_url = Url::parse("https://other.com/api").unwrap();

        assert!(policy.can_request(&same_origin_url));
        assert!(!policy.can_request(&cross_origin_url));
    }
}
