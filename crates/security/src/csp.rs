//! Content Security Policy implementation.

use std::collections::{HashMap, HashSet};
use url::Url;
use crate::origin::Origin;

/// Content Security Policy.
#[derive(Clone, Debug, Default)]
pub struct ContentSecurityPolicy {
    /// Directives in this policy.
    directives: HashMap<String, CspDirective>,
    /// Whether this is a report-only policy.
    report_only: bool,
    /// Report URI for violations.
    report_uri: Option<String>,
}

impl ContentSecurityPolicy {
    /// Create a new empty CSP.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a CSP header value.
    pub fn parse(header: &str) -> Self {
        let mut csp = ContentSecurityPolicy::new();

        for directive_str in header.split(';') {
            let directive_str = directive_str.trim();
            if directive_str.is_empty() {
                continue;
            }

            let mut parts = directive_str.split_whitespace();
            if let Some(name) = parts.next() {
                let values: Vec<String> = parts.map(|s| s.to_string()).collect();
                let directive = CspDirective::new(name, values);
                csp.directives.insert(name.to_lowercase(), directive);
            }
        }

        csp
    }

    /// Set report-only mode.
    pub fn set_report_only(&mut self, report_only: bool) {
        self.report_only = report_only;
    }

    /// Set report URI.
    pub fn set_report_uri(&mut self, uri: &str) {
        self.report_uri = Some(uri.to_string());
    }

    /// Check if a script source is allowed.
    pub fn allows_script(&self, url: &Url, nonce: Option<&str>, hash: Option<&str>) -> CspCheck {
        self.check_source("script-src", url, nonce, hash)
    }

    /// Check if a style source is allowed.
    pub fn allows_style(&self, url: &Url, nonce: Option<&str>, hash: Option<&str>) -> CspCheck {
        self.check_source("style-src", url, nonce, hash)
    }

    /// Check if an image source is allowed.
    pub fn allows_image(&self, url: &Url) -> CspCheck {
        self.check_source("img-src", url, None, None)
    }

    /// Check if a font source is allowed.
    pub fn allows_font(&self, url: &Url) -> CspCheck {
        self.check_source("font-src", url, None, None)
    }

    /// Check if a connect source is allowed (fetch, XHR, WebSocket).
    pub fn allows_connect(&self, url: &Url) -> CspCheck {
        self.check_source("connect-src", url, None, None)
    }

    /// Check if a media source is allowed.
    pub fn allows_media(&self, url: &Url) -> CspCheck {
        self.check_source("media-src", url, None, None)
    }

    /// Check if a frame source is allowed.
    pub fn allows_frame(&self, url: &Url) -> CspCheck {
        self.check_source("frame-src", url, None, None)
    }

    /// Check if a child source is allowed.
    pub fn allows_child(&self, url: &Url) -> CspCheck {
        self.check_source("child-src", url, None, None)
    }

    /// Check if a worker source is allowed.
    pub fn allows_worker(&self, url: &Url) -> CspCheck {
        self.check_source("worker-src", url, None, None)
    }

    /// Check if an object source is allowed.
    pub fn allows_object(&self, url: &Url) -> CspCheck {
        self.check_source("object-src", url, None, None)
    }

    /// Check if a form action is allowed.
    pub fn allows_form_action(&self, url: &Url) -> CspCheck {
        self.check_source("form-action", url, None, None)
    }

    /// Check if a base URI is allowed.
    pub fn allows_base_uri(&self, url: &Url) -> CspCheck {
        self.check_source("base-uri", url, None, None)
    }

    /// Check if inline scripts are allowed.
    pub fn allows_inline_script(&self, nonce: Option<&str>, hash: Option<&str>) -> CspCheck {
        self.check_inline("script-src", nonce, hash)
    }

    /// Check if inline styles are allowed.
    pub fn allows_inline_style(&self, nonce: Option<&str>, hash: Option<&str>) -> CspCheck {
        self.check_inline("style-src", nonce, hash)
    }

    /// Check if eval is allowed.
    pub fn allows_eval(&self) -> CspCheck {
        if let Some(directive) = self.get_effective_directive("script-src") {
            if directive.allows_unsafe_eval() {
                return CspCheck::Allowed;
            }
            return CspCheck::Blocked(CspViolation {
                directive: "script-src".to_string(),
                blocked_uri: "eval".to_string(),
                source_file: None,
                line_number: None,
                column_number: None,
            });
        }
        CspCheck::Allowed
    }

    fn check_source(
        &self,
        directive_name: &str,
        url: &Url,
        nonce: Option<&str>,
        hash: Option<&str>,
    ) -> CspCheck {
        let directive = match self.get_effective_directive(directive_name) {
            Some(d) => d,
            None => return CspCheck::Allowed,
        };

        if directive.matches_url(url) {
            return CspCheck::Allowed;
        }

        if let Some(n) = nonce {
            if directive.matches_nonce(n) {
                return CspCheck::Allowed;
            }
        }

        if let Some(h) = hash {
            if directive.matches_hash(h) {
                return CspCheck::Allowed;
            }
        }

        CspCheck::Blocked(CspViolation {
            directive: directive_name.to_string(),
            blocked_uri: url.to_string(),
            source_file: None,
            line_number: None,
            column_number: None,
        })
    }

    fn check_inline(&self, directive_name: &str, nonce: Option<&str>, hash: Option<&str>) -> CspCheck {
        let directive = match self.get_effective_directive(directive_name) {
            Some(d) => d,
            None => return CspCheck::Allowed,
        };

        if directive.allows_unsafe_inline() {
            return CspCheck::Allowed;
        }

        if let Some(n) = nonce {
            if directive.matches_nonce(n) {
                return CspCheck::Allowed;
            }
        }

        if let Some(h) = hash {
            if directive.matches_hash(h) {
                return CspCheck::Allowed;
            }
        }

        CspCheck::Blocked(CspViolation {
            directive: directive_name.to_string(),
            blocked_uri: "inline".to_string(),
            source_file: None,
            line_number: None,
            column_number: None,
        })
    }

    fn get_effective_directive(&self, name: &str) -> Option<&CspDirective> {
        // First try the specific directive
        if let Some(d) = self.directives.get(name) {
            return Some(d);
        }

        // Fall back to default-src for certain directives
        let fallback_directives = [
            "script-src",
            "style-src",
            "img-src",
            "font-src",
            "connect-src",
            "media-src",
            "object-src",
            "frame-src",
            "child-src",
            "worker-src",
        ];

        if fallback_directives.contains(&name) {
            return self.directives.get("default-src");
        }

        None
    }

    /// Check if the policy is report-only.
    pub fn is_report_only(&self) -> bool {
        self.report_only
    }

    /// Get the report URI.
    pub fn report_uri(&self) -> Option<&str> {
        self.report_uri.as_deref()
    }

    /// Get upgrade-insecure-requests directive.
    pub fn upgrade_insecure_requests(&self) -> bool {
        self.directives.contains_key("upgrade-insecure-requests")
    }

    /// Get block-all-mixed-content directive.
    pub fn block_all_mixed_content(&self) -> bool {
        self.directives.contains_key("block-all-mixed-content")
    }
}

/// CSP directive.
#[derive(Clone, Debug)]
pub struct CspDirective {
    /// Directive name.
    name: String,
    /// Source list values.
    values: Vec<String>,
    /// Parsed sources.
    sources: HashSet<CspSource>,
    /// Nonces.
    nonces: HashSet<String>,
    /// Hashes.
    hashes: HashSet<String>,
}

impl CspDirective {
    /// Create a new directive.
    pub fn new(name: &str, values: Vec<String>) -> Self {
        let mut sources = HashSet::new();
        let mut nonces = HashSet::new();
        let mut hashes = HashSet::new();

        for value in &values {
            if value.starts_with("'nonce-") && value.ends_with('\'') {
                let nonce = &value[7..value.len() - 1];
                nonces.insert(nonce.to_string());
            } else if value.starts_with("'sha256-")
                || value.starts_with("'sha384-")
                || value.starts_with("'sha512-")
            {
                let hash = &value[1..value.len() - 1];
                hashes.insert(hash.to_string());
            } else {
                sources.insert(CspSource::parse(value));
            }
        }

        Self {
            name: name.to_lowercase(),
            values,
            sources,
            nonces,
            hashes,
        }
    }

    /// Check if a URL matches this directive.
    pub fn matches_url(&self, url: &Url) -> bool {
        for source in &self.sources {
            if source.matches_url(url) {
                return true;
            }
        }
        false
    }

    /// Check if a nonce matches.
    pub fn matches_nonce(&self, nonce: &str) -> bool {
        self.nonces.contains(nonce)
    }

    /// Check if a hash matches.
    pub fn matches_hash(&self, hash: &str) -> bool {
        self.hashes.contains(hash)
    }

    /// Check if unsafe-inline is allowed.
    pub fn allows_unsafe_inline(&self) -> bool {
        self.sources.contains(&CspSource::UnsafeInline)
    }

    /// Check if unsafe-eval is allowed.
    pub fn allows_unsafe_eval(&self) -> bool {
        self.sources.contains(&CspSource::UnsafeEval)
    }
}

/// CSP source expression.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CspSource {
    /// 'none' - nothing allowed
    None,
    /// 'self' - same origin
    Self_,
    /// 'unsafe-inline'
    UnsafeInline,
    /// 'unsafe-eval'
    UnsafeEval,
    /// 'strict-dynamic'
    StrictDynamic,
    /// 'unsafe-hashes'
    UnsafeHashes,
    /// Scheme source (e.g., "https:")
    Scheme(String),
    /// Host source (e.g., "example.com", "*.example.com")
    Host {
        scheme: Option<String>,
        host: String,
        port: Option<u16>,
        path: Option<String>,
    },
}

impl CspSource {
    /// Parse a source expression.
    pub fn parse(source: &str) -> Self {
        match source.to_lowercase().as_str() {
            "'none'" => CspSource::None,
            "'self'" => CspSource::Self_,
            "'unsafe-inline'" => CspSource::UnsafeInline,
            "'unsafe-eval'" => CspSource::UnsafeEval,
            "'strict-dynamic'" => CspSource::StrictDynamic,
            "'unsafe-hashes'" => CspSource::UnsafeHashes,
            s if s.ends_with(':') => CspSource::Scheme(s[..s.len() - 1].to_string()),
            s => Self::parse_host(s),
        }
    }

    fn parse_host(source: &str) -> Self {
        let mut scheme = None;
        let mut rest = source;

        // Extract scheme if present
        if let Some(idx) = rest.find("://") {
            scheme = Some(rest[..idx].to_string());
            rest = &rest[idx + 3..];
        }

        // Extract port if present
        let mut port = None;
        let mut host_and_path = rest;
        if let Some(port_idx) = rest.rfind(':') {
            let after_colon = &rest[port_idx + 1..];
            let port_end = after_colon.find('/').unwrap_or(after_colon.len());
            if let Ok(p) = after_colon[..port_end].parse::<u16>() {
                port = Some(p);
                host_and_path = &rest[..port_idx];
                if port_end < after_colon.len() {
                    host_and_path = rest; // Has path, need to re-parse
                }
            }
        }

        // Extract path if present
        let mut path = None;
        let host = if let Some(path_idx) = host_and_path.find('/') {
            path = Some(host_and_path[path_idx..].to_string());
            &host_and_path[..path_idx]
        } else {
            host_and_path
        };

        CspSource::Host {
            scheme,
            host: host.to_string(),
            port,
            path,
        }
    }

    /// Check if this source matches a URL.
    pub fn matches_url(&self, url: &Url) -> bool {
        match self {
            CspSource::None => false,
            CspSource::Self_ => false, // Needs document origin context
            CspSource::UnsafeInline | CspSource::UnsafeEval => false,
            CspSource::StrictDynamic | CspSource::UnsafeHashes => false,
            CspSource::Scheme(scheme) => url.scheme() == scheme,
            CspSource::Host {
                scheme,
                host,
                port,
                path,
            } => {
                // Check scheme
                if let Some(s) = scheme {
                    if url.scheme() != s {
                        return false;
                    }
                }

                // Check host
                let url_host = url.host_str().unwrap_or("");
                if host.starts_with('*') {
                    let suffix = &host[1..];
                    if !url_host.ends_with(suffix) && url_host != &suffix[1..] {
                        return false;
                    }
                } else if url_host != host {
                    return false;
                }

                // Check port
                if let Some(p) = port {
                    let url_port = url.port_or_known_default().unwrap_or(0);
                    if url_port != *p {
                        return false;
                    }
                }

                // Check path
                if let Some(p) = path {
                    if !url.path().starts_with(p) {
                        return false;
                    }
                }

                true
            }
        }
    }
}

/// Result of a CSP check.
#[derive(Clone, Debug)]
pub enum CspCheck {
    /// The resource is allowed.
    Allowed,
    /// The resource is blocked.
    Blocked(CspViolation),
}

impl CspCheck {
    /// Check if the resource is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, CspCheck::Allowed)
    }

    /// Check if the resource is blocked.
    pub fn is_blocked(&self) -> bool {
        matches!(self, CspCheck::Blocked(_))
    }
}

/// CSP violation report.
#[derive(Clone, Debug)]
pub struct CspViolation {
    /// The directive that was violated.
    pub directive: String,
    /// The URI that was blocked.
    pub blocked_uri: String,
    /// Source file where the violation occurred.
    pub source_file: Option<String>,
    /// Line number.
    pub line_number: Option<u32>,
    /// Column number.
    pub column_number: Option<u32>,
}

impl CspViolation {
    /// Create a violation report JSON.
    pub fn to_report(&self, document_uri: &str, policy: &str) -> serde_json::Value {
        serde_json::json!({
            "csp-report": {
                "document-uri": document_uri,
                "violated-directive": self.directive,
                "blocked-uri": self.blocked_uri,
                "source-file": self.source_file,
                "line-number": self.line_number,
                "column-number": self.column_number,
                "original-policy": policy,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_parse() {
        let csp = ContentSecurityPolicy::parse(
            "default-src 'self'; script-src 'self' https://cdn.example.com; style-src 'unsafe-inline'",
        );

        assert!(csp.directives.contains_key("default-src"));
        assert!(csp.directives.contains_key("script-src"));
        assert!(csp.directives.contains_key("style-src"));
    }

    #[test]
    fn test_csp_allows_script() {
        let csp = ContentSecurityPolicy::parse("script-src https://cdn.example.com");

        let allowed_url = Url::parse("https://cdn.example.com/script.js").unwrap();
        let blocked_url = Url::parse("https://evil.com/script.js").unwrap();

        assert!(csp.allows_script(&allowed_url, None, None).is_allowed());
        assert!(csp.allows_script(&blocked_url, None, None).is_blocked());
    }

    #[test]
    fn test_csp_nonce() {
        let csp = ContentSecurityPolicy::parse("script-src 'nonce-abc123'");

        assert!(csp.allows_inline_script(Some("abc123"), None).is_allowed());
        assert!(csp.allows_inline_script(Some("wrong"), None).is_blocked());
        assert!(csp.allows_inline_script(None, None).is_blocked());
    }

    #[test]
    fn test_csp_wildcard_host() {
        let csp = ContentSecurityPolicy::parse("script-src *.example.com");

        let allowed1 = Url::parse("https://cdn.example.com/script.js").unwrap();
        let allowed2 = Url::parse("https://sub.cdn.example.com/script.js").unwrap();
        let blocked = Url::parse("https://example.org/script.js").unwrap();

        assert!(csp.allows_script(&allowed1, None, None).is_allowed());
        assert!(csp.allows_script(&allowed2, None, None).is_allowed());
        assert!(csp.allows_script(&blocked, None, None).is_blocked());
    }
}
