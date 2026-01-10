//! Cookie management.

use indexmap::IndexMap;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

/// A cookie.
#[derive(Clone, Debug)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Domain.
    pub domain: Option<String>,
    /// Path.
    pub path: Option<String>,
    /// Expiration time (Unix timestamp).
    pub expires: Option<u64>,
    /// Max-Age in seconds.
    pub max_age: Option<u64>,
    /// Secure flag.
    pub secure: bool,
    /// HttpOnly flag.
    pub http_only: bool,
    /// SameSite attribute.
    pub same_site: SameSite,
    /// Creation time.
    pub created: u64,
}

/// SameSite attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SameSite {
    None,
    Lax,
    Strict,
}

impl Cookie {
    /// Create a new cookie.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            created: now,
        }
    }

    /// Parse a Set-Cookie header.
    pub fn parse(header: &str) -> Option<Self> {
        let mut parts = header.split(';').map(|s| s.trim());

        // First part is name=value
        let name_value = parts.next()?;
        let (name, value) = name_value.split_once('=')?;

        let mut cookie = Cookie::new(name.trim(), value.trim());

        // Parse attributes
        for attr in parts {
            let (attr_name, attr_value) = attr
                .split_once('=')
                .map(|(n, v)| (n.trim().to_lowercase(), Some(v.trim())))
                .unwrap_or_else(|| (attr.trim().to_lowercase(), None));

            match attr_name.as_str() {
                "domain" => cookie.domain = attr_value.map(|s| s.to_string()),
                "path" => cookie.path = attr_value.map(|s| s.to_string()),
                "expires" => {
                    if let Some(date_str) = attr_value {
                        cookie.expires = parse_http_date(date_str);
                    }
                }
                "max-age" => {
                    cookie.max_age = attr_value.and_then(|s| s.parse().ok());
                }
                "secure" => cookie.secure = true,
                "httponly" => cookie.http_only = true,
                "samesite" => {
                    cookie.same_site = match attr_value.map(|s| s.to_lowercase()).as_deref() {
                        Some("strict") => SameSite::Strict,
                        Some("lax") => SameSite::Lax,
                        Some("none") => SameSite::None,
                        _ => SameSite::Lax,
                    };
                }
                _ => {}
            }
        }

        Some(cookie)
    }

    /// Check if the cookie is expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Check max-age first
        if let Some(max_age) = self.max_age {
            if self.created + max_age < now {
                return true;
            }
        }

        // Check expires
        if let Some(expires) = self.expires {
            if expires < now {
                return true;
            }
        }

        false
    }

    /// Check if cookie is valid for a URL.
    pub fn matches_url(&self, url: &Url) -> bool {
        // Check secure
        if self.secure && url.scheme() != "https" {
            return false;
        }

        // Check domain
        if let Some(domain) = &self.domain {
            if let Some(host) = url.host_str() {
                if !host.ends_with(domain) && host != domain.trim_start_matches('.') {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check path
        if let Some(path) = &self.path {
            let url_path = url.path();
            if !url_path.starts_with(path) {
                return false;
            }
        }

        true
    }

    /// Serialize to Set-Cookie format.
    pub fn to_set_cookie(&self) -> String {
        let mut result = format!("{}={}", self.name, self.value);

        if let Some(domain) = &self.domain {
            result.push_str(&format!("; Domain={}", domain));
        }

        if let Some(path) = &self.path {
            result.push_str(&format!("; Path={}", path));
        }

        if let Some(max_age) = self.max_age {
            result.push_str(&format!("; Max-Age={}", max_age));
        }

        if self.secure {
            result.push_str("; Secure");
        }

        if self.http_only {
            result.push_str("; HttpOnly");
        }

        match self.same_site {
            SameSite::Strict => result.push_str("; SameSite=Strict"),
            SameSite::Lax => result.push_str("; SameSite=Lax"),
            SameSite::None => result.push_str("; SameSite=None"),
        }

        result
    }
}

/// Cookie jar for storing cookies.
#[derive(Clone, Debug, Default)]
pub struct CookieJar {
    /// Cookies indexed by domain and name.
    cookies: HashMap<String, IndexMap<String, Cookie>>,
}

impl CookieJar {
    /// Create a new cookie jar.
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Add a cookie from a Set-Cookie response header.
    pub fn add_from_response(&mut self, url: &Url, header: &str) {
        if let Some(mut cookie) = Cookie::parse(header) {
            // Set domain from URL if not specified
            if cookie.domain.is_none() {
                cookie.domain = url.host_str().map(|s| s.to_string());
            }

            // Set path from URL if not specified
            if cookie.path.is_none() {
                cookie.path = Some(url.path().to_string());
            }

            self.add(cookie);
        }
    }

    /// Add a cookie.
    pub fn add(&mut self, cookie: Cookie) {
        let domain = cookie.domain.clone().unwrap_or_default();
        let cookies = self.cookies.entry(domain).or_insert_with(IndexMap::new);
        cookies.insert(cookie.name.clone(), cookie);
    }

    /// Get cookies for a URL.
    pub fn get_cookies(&self, url: &Url) -> Vec<&Cookie> {
        let host = url.host_str().unwrap_or("");

        let mut result = Vec::new();

        for (domain, cookies) in &self.cookies {
            // Check if domain matches
            if host.ends_with(domain) || host == domain.trim_start_matches('.') {
                for cookie in cookies.values() {
                    if !cookie.is_expired() && cookie.matches_url(url) {
                        result.push(cookie);
                    }
                }
            }
        }

        result
    }

    /// Get the Cookie header value for a URL.
    pub fn get_cookie_header(&self, url: &Url) -> String {
        let cookies = self.get_cookies(url);
        cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Remove a cookie.
    pub fn remove(&mut self, domain: &str, name: &str) {
        if let Some(cookies) = self.cookies.get_mut(domain) {
            cookies.shift_remove(name);
        }
    }

    /// Remove expired cookies.
    pub fn cleanup_expired(&mut self) {
        for cookies in self.cookies.values_mut() {
            cookies.retain(|_, cookie| !cookie.is_expired());
        }
    }

    /// Clear all cookies.
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Get total number of cookies.
    pub fn len(&self) -> usize {
        self.cookies.values().map(|c| c.len()).sum()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty() || self.cookies.values().all(|c| c.is_empty())
    }

    /// Get all cookies.
    pub fn all_cookies(&self) -> Vec<&Cookie> {
        self.cookies.values().flat_map(|c| c.values()).collect()
    }
}

/// Parse an HTTP date string to Unix timestamp.
fn parse_http_date(date_str: &str) -> Option<u64> {
    // Simplified HTTP date parsing
    // Real implementation would handle multiple formats

    // Try parsing common formats
    // For now, just return None and rely on max-age
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_parse() {
        let cookie = Cookie::parse("session_id=abc123; Path=/; Secure; HttpOnly").unwrap();
        assert_eq!(cookie.name, "session_id");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.path, Some("/".to_string()));
        assert!(cookie.secure);
        assert!(cookie.http_only);
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();
        let url = Url::parse("https://example.com/path").unwrap();

        jar.add_from_response(&url, "session=abc123; Path=/");
        jar.add_from_response(&url, "user=john; Path=/path");

        let cookies = jar.get_cookies(&url);
        assert_eq!(cookies.len(), 2);

        let header = jar.get_cookie_header(&url);
        assert!(header.contains("session=abc123"));
        assert!(header.contains("user=john"));
    }

    #[test]
    fn test_same_site() {
        let cookie = Cookie::parse("id=123; SameSite=Strict").unwrap();
        assert_eq!(cookie.same_site, SameSite::Strict);

        let cookie = Cookie::parse("id=123; SameSite=None; Secure").unwrap();
        assert_eq!(cookie.same_site, SameSite::None);
    }
}
