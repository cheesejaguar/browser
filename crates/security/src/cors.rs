//! Cross-Origin Resource Sharing (CORS) implementation.

use std::collections::HashSet;
use url::Url;
use crate::origin::Origin;

/// CORS configuration for a server.
#[derive(Clone, Debug, Default)]
pub struct CorsConfig {
    /// Allowed origins.
    pub allowed_origins: AllowedOrigins,
    /// Allowed methods.
    pub allowed_methods: HashSet<String>,
    /// Allowed headers.
    pub allowed_headers: HashSet<String>,
    /// Exposed headers.
    pub exposed_headers: HashSet<String>,
    /// Whether credentials are allowed.
    pub allow_credentials: bool,
    /// Max age for preflight cache.
    pub max_age: Option<u32>,
}

impl CorsConfig {
    /// Create a new permissive CORS config.
    pub fn permissive() -> Self {
        Self {
            allowed_origins: AllowedOrigins::Any,
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: HashSet::new(),
            exposed_headers: HashSet::new(),
            allow_credentials: false,
            max_age: Some(86400),
        }
    }

    /// Check if an origin is allowed.
    pub fn is_origin_allowed(&self, origin: &Origin) -> bool {
        self.allowed_origins.is_allowed(origin)
    }

    /// Check if a method is allowed.
    pub fn is_method_allowed(&self, method: &str) -> bool {
        self.allowed_methods.contains(&method.to_uppercase())
    }

    /// Check if headers are allowed.
    pub fn are_headers_allowed(&self, headers: &[String]) -> bool {
        if self.allowed_headers.is_empty() {
            return true;
        }

        for header in headers {
            let header_lower = header.to_lowercase();
            // CORS-safelisted headers are always allowed
            if is_cors_safelisted_header(&header_lower) {
                continue;
            }
            if !self.allowed_headers.contains(&header_lower) {
                return false;
            }
        }
        true
    }
}

/// Allowed origins specification.
#[derive(Clone, Debug)]
pub enum AllowedOrigins {
    /// Allow any origin.
    Any,
    /// Allow specific origins.
    List(HashSet<String>),
    /// Allow origins matching a pattern.
    Pattern(regex::Regex),
}

impl Default for AllowedOrigins {
    fn default() -> Self {
        AllowedOrigins::List(HashSet::new())
    }
}

impl AllowedOrigins {
    /// Check if an origin is allowed.
    pub fn is_allowed(&self, origin: &Origin) -> bool {
        match self {
            AllowedOrigins::Any => true,
            AllowedOrigins::List(origins) => origins.contains(&origin.serialize()),
            AllowedOrigins::Pattern(pattern) => pattern.is_match(&origin.serialize()),
        }
    }
}

/// CORS request type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorsRequestType {
    /// Simple request (no preflight needed).
    Simple,
    /// Preflight request.
    Preflight,
    /// Actual request after preflight.
    Actual,
}

/// CORS request information.
#[derive(Clone, Debug)]
pub struct CorsRequest {
    /// Request origin.
    pub origin: Option<Origin>,
    /// Request method.
    pub method: String,
    /// Request headers.
    pub headers: Vec<String>,
    /// Whether credentials are included.
    pub with_credentials: bool,
    /// Request type.
    pub request_type: CorsRequestType,
}

impl CorsRequest {
    /// Create a new CORS request.
    pub fn new(origin: Option<Origin>, method: &str) -> Self {
        Self {
            origin,
            method: method.to_uppercase(),
            headers: Vec::new(),
            with_credentials: false,
            request_type: CorsRequestType::Simple,
        }
    }

    /// Check if this is a cross-origin request.
    pub fn is_cross_origin(&self, target_origin: &Origin) -> bool {
        match &self.origin {
            Some(origin) => !origin.is_same_origin(target_origin),
            None => false,
        }
    }

    /// Check if preflight is required.
    pub fn requires_preflight(&self) -> bool {
        // Simple methods don't require preflight
        let simple_methods = ["GET", "HEAD", "POST"];
        if !simple_methods.contains(&self.method.as_str()) {
            return true;
        }

        // Check for non-safelisted headers
        for header in &self.headers {
            if !is_cors_safelisted_header(&header.to_lowercase()) {
                return true;
            }
        }

        false
    }

    /// Create a preflight request.
    pub fn to_preflight(&self) -> Self {
        Self {
            origin: self.origin.clone(),
            method: "OPTIONS".to_string(),
            headers: self.headers.clone(),
            with_credentials: self.with_credentials,
            request_type: CorsRequestType::Preflight,
        }
    }
}

/// CORS response.
#[derive(Clone, Debug)]
pub struct CorsResponse {
    /// Access-Control-Allow-Origin.
    pub allow_origin: Option<String>,
    /// Access-Control-Allow-Methods.
    pub allow_methods: Vec<String>,
    /// Access-Control-Allow-Headers.
    pub allow_headers: Vec<String>,
    /// Access-Control-Expose-Headers.
    pub expose_headers: Vec<String>,
    /// Access-Control-Allow-Credentials.
    pub allow_credentials: bool,
    /// Access-Control-Max-Age.
    pub max_age: Option<u32>,
}

impl CorsResponse {
    /// Parse CORS headers from response headers.
    pub fn from_headers(headers: &[(String, String)]) -> Self {
        let mut response = Self {
            allow_origin: None,
            allow_methods: Vec::new(),
            allow_headers: Vec::new(),
            expose_headers: Vec::new(),
            allow_credentials: false,
            max_age: None,
        };

        for (name, value) in headers {
            match name.to_lowercase().as_str() {
                "access-control-allow-origin" => {
                    response.allow_origin = Some(value.clone());
                }
                "access-control-allow-methods" => {
                    response.allow_methods = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                "access-control-allow-headers" => {
                    response.allow_headers = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                "access-control-expose-headers" => {
                    response.expose_headers = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                "access-control-allow-credentials" => {
                    response.allow_credentials = value.eq_ignore_ascii_case("true");
                }
                "access-control-max-age" => {
                    response.max_age = value.parse().ok();
                }
                _ => {}
            }
        }

        response
    }

    /// Check if a request is allowed based on this response.
    pub fn allows_request(&self, request: &CorsRequest) -> CorsCheckResult {
        // Check origin
        let origin_str = match &request.origin {
            Some(o) => o.serialize(),
            None => return CorsCheckResult::Allowed,
        };

        match &self.allow_origin {
            Some(allowed) => {
                if allowed != "*" && allowed != &origin_str {
                    return CorsCheckResult::OriginNotAllowed;
                }
                // Wildcard with credentials is not allowed
                if allowed == "*" && request.with_credentials {
                    return CorsCheckResult::OriginNotAllowed;
                }
            }
            None => return CorsCheckResult::OriginNotAllowed,
        }

        // Check credentials
        if request.with_credentials && !self.allow_credentials {
            return CorsCheckResult::CredentialsNotAllowed;
        }

        // For preflight, check method and headers
        if request.request_type == CorsRequestType::Preflight {
            if !self.allow_methods.iter().any(|m| m.eq_ignore_ascii_case(&request.method)) {
                return CorsCheckResult::MethodNotAllowed;
            }

            for header in &request.headers {
                if !is_cors_safelisted_header(&header.to_lowercase())
                    && !self.allow_headers.iter().any(|h| h.eq_ignore_ascii_case(header))
                {
                    return CorsCheckResult::HeaderNotAllowed(header.clone());
                }
            }
        }

        CorsCheckResult::Allowed
    }

    /// Get headers that should be exposed to JavaScript.
    pub fn get_exposed_headers(&self) -> &[String] {
        &self.expose_headers
    }
}

/// Result of CORS check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorsCheckResult {
    /// Request is allowed.
    Allowed,
    /// Origin is not allowed.
    OriginNotAllowed,
    /// Method is not allowed.
    MethodNotAllowed,
    /// Header is not allowed.
    HeaderNotAllowed(String),
    /// Credentials are not allowed.
    CredentialsNotAllowed,
}

impl CorsCheckResult {
    /// Check if the request is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, CorsCheckResult::Allowed)
    }
}

/// Check if a header is CORS-safelisted.
fn is_cors_safelisted_header(header: &str) -> bool {
    matches!(
        header,
        "accept"
            | "accept-language"
            | "content-language"
            | "content-type"
            | "dpr"
            | "downlink"
            | "save-data"
            | "viewport-width"
            | "width"
    )
}

/// CORS preflight cache.
#[derive(Debug, Default)]
pub struct PreflightCache {
    entries: parking_lot::RwLock<std::collections::HashMap<String, PreflightCacheEntry>>,
}

impl PreflightCache {
    /// Create a new preflight cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a cached preflight response.
    pub fn get(&self, url: &Url, origin: &Origin) -> Option<CorsResponse> {
        let key = format!("{}|{}", origin.serialize(), url);
        let entries = self.entries.read();
        entries.get(&key).and_then(|entry| {
            if entry.is_valid() {
                Some(entry.response.clone())
            } else {
                None
            }
        })
    }

    /// Store a preflight response.
    pub fn put(&self, url: &Url, origin: &Origin, response: CorsResponse) {
        if let Some(max_age) = response.max_age {
            let key = format!("{}|{}", origin.serialize(), url);
            let entry = PreflightCacheEntry {
                response,
                created: std::time::Instant::now(),
                max_age,
            };
            self.entries.write().insert(key, entry);
        }
    }

    /// Clear expired entries.
    pub fn clear_expired(&self) {
        self.entries.write().retain(|_, entry| entry.is_valid());
    }
}

/// Preflight cache entry.
#[derive(Debug)]
struct PreflightCacheEntry {
    response: CorsResponse,
    created: std::time::Instant,
    max_age: u32,
}

impl PreflightCacheEntry {
    fn is_valid(&self) -> bool {
        self.created.elapsed().as_secs() < self.max_age as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_simple_request() {
        let request = CorsRequest::new(Origin::parse("https://example.com"), "GET");
        assert!(!request.requires_preflight());
    }

    #[test]
    fn test_cors_preflight_required() {
        let mut request = CorsRequest::new(Origin::parse("https://example.com"), "PUT");
        assert!(request.requires_preflight());

        request.method = "POST".to_string();
        request.headers = vec!["X-Custom-Header".to_string()];
        assert!(request.requires_preflight());
    }

    #[test]
    fn test_cors_response_allows_origin() {
        let response = CorsResponse {
            allow_origin: Some("https://example.com".to_string()),
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec![],
            expose_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };

        let allowed_request = CorsRequest::new(Origin::parse("https://example.com"), "GET");
        let blocked_request = CorsRequest::new(Origin::parse("https://evil.com"), "GET");

        assert!(response.allows_request(&allowed_request).is_allowed());
        assert!(!response.allows_request(&blocked_request).is_allowed());
    }

    #[test]
    fn test_cors_wildcard_with_credentials() {
        let response = CorsResponse {
            allow_origin: Some("*".to_string()),
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            expose_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };

        let mut request = CorsRequest::new(Origin::parse("https://example.com"), "GET");
        assert!(response.allows_request(&request).is_allowed());

        request.with_credentials = true;
        assert!(!response.allows_request(&request).is_allowed());
    }
}
