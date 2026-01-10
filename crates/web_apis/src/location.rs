//! Location API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::sync::Arc;
use parking_lot::RwLock;
use url::Url;

/// Location API implementation.
#[derive(Clone, Debug)]
pub struct Location {
    /// Current URL.
    url: Url,
    /// Ancestor origins.
    ancestor_origins: Vec<String>,
}

impl Location {
    /// Create a new Location from a URL string.
    pub fn new(url: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(url)?;
        Ok(Self {
            url,
            ancestor_origins: Vec::new(),
        })
    }

    /// Create from a parsed URL.
    pub fn from_url(url: Url) -> Self {
        Self {
            url,
            ancestor_origins: Vec::new(),
        }
    }

    /// Get the full URL as a string.
    pub fn href(&self) -> String {
        self.url.to_string()
    }

    /// Set the URL (navigate to a new page).
    pub fn set_href(&mut self, href: &str) -> Result<(), url::ParseError> {
        self.url = Url::parse(href)?;
        Ok(())
    }

    /// Get the protocol (e.g., "https:").
    pub fn protocol(&self) -> String {
        format!("{}:", self.url.scheme())
    }

    /// Set the protocol.
    pub fn set_protocol(&mut self, protocol: &str) -> Result<(), url::ParseError> {
        let protocol = protocol.trim_end_matches(':');
        self.url.set_scheme(protocol).map_err(|_| url::ParseError::InvalidScheme)
    }

    /// Get the host (hostname:port).
    pub fn host(&self) -> String {
        match self.url.port() {
            Some(port) => format!("{}:{}", self.url.host_str().unwrap_or(""), port),
            None => self.url.host_str().unwrap_or("").to_string(),
        }
    }

    /// Set the host.
    pub fn set_host(&mut self, host: &str) -> Result<(), url::ParseError> {
        if let Some((hostname, port)) = host.split_once(':') {
            let _ = self.url.set_host(Some(hostname));
            if let Ok(port) = port.parse() {
                let _ = self.url.set_port(Some(port));
            }
        } else {
            let _ = self.url.set_host(Some(host));
        }
        Ok(())
    }

    /// Get the hostname.
    pub fn hostname(&self) -> String {
        self.url.host_str().unwrap_or("").to_string()
    }

    /// Set the hostname.
    pub fn set_hostname(&mut self, hostname: &str) -> Result<(), url::ParseError> {
        let _ = self.url.set_host(Some(hostname));
        Ok(())
    }

    /// Get the port.
    pub fn port(&self) -> String {
        self.url.port().map(|p| p.to_string()).unwrap_or_default()
    }

    /// Set the port.
    pub fn set_port(&mut self, port: &str) -> Result<(), url::ParseError> {
        if port.is_empty() {
            let _ = self.url.set_port(None);
        } else if let Ok(port) = port.parse() {
            let _ = self.url.set_port(Some(port));
        }
        Ok(())
    }

    /// Get the pathname.
    pub fn pathname(&self) -> String {
        self.url.path().to_string()
    }

    /// Set the pathname.
    pub fn set_pathname(&mut self, pathname: &str) {
        self.url.set_path(pathname);
    }

    /// Get the search string (query string with ?).
    pub fn search(&self) -> String {
        self.url.query().map(|q| format!("?{}", q)).unwrap_or_default()
    }

    /// Set the search string.
    pub fn set_search(&mut self, search: &str) {
        let search = search.trim_start_matches('?');
        self.url.set_query(if search.is_empty() { None } else { Some(search) });
    }

    /// Get the hash (fragment with #).
    pub fn hash(&self) -> String {
        self.url.fragment().map(|f| format!("#{}", f)).unwrap_or_default()
    }

    /// Set the hash.
    pub fn set_hash(&mut self, hash: &str) {
        let hash = hash.trim_start_matches('#');
        self.url.set_fragment(if hash.is_empty() { None } else { Some(hash) });
    }

    /// Get the origin.
    pub fn origin(&self) -> String {
        self.url.origin().ascii_serialization()
    }

    /// Reload the current page.
    pub fn reload(&self) {
        // In a real implementation, this would trigger a page reload
    }

    /// Replace the current page (no history entry).
    pub fn replace(&mut self, url: &str) -> Result<(), url::ParseError> {
        self.set_href(url)
    }

    /// Assign a new URL (creates history entry).
    pub fn assign(&mut self, url: &str) -> Result<(), url::ParseError> {
        self.set_href(url)
    }

    /// Get ancestor origins.
    pub fn ancestor_origins(&self) -> &[String] {
        &self.ancestor_origins
    }

    /// Register the Location API on the global object.
    pub fn register(location: Arc<RwLock<Location>>, context: &mut Context) {
        let location_obj = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(location_reload), js_string!("reload"), 0)
            .function(NativeFunction::from_fn_ptr(location_replace), js_string!("replace"), 1)
            .function(NativeFunction::from_fn_ptr(location_assign), js_string!("assign"), 1)
            .function(NativeFunction::from_fn_ptr(location_to_string), js_string!("toString"), 0)
            .build();

        context
            .register_global_property(js_string!("location"), location_obj, Attribute::all())
            .expect("Failed to register location");
    }
}

// Native function implementations
fn location_reload(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn location_replace(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _url = args.get_or_undefined(0).to_string(context)?;
    Ok(JsValue::undefined())
}

fn location_assign(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _url = args.get_or_undefined(0).to_string(context)?;
    Ok(JsValue::undefined())
}

fn location_to_string(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Would return the full URL
    Ok(js_string!("about:blank").into())
}

/// URL search params.
#[derive(Clone, Debug, Default)]
pub struct URLSearchParams {
    params: Vec<(String, String)>,
}

impl URLSearchParams {
    /// Create new empty search params.
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    /// Parse from a query string.
    pub fn parse(query: &str) -> Self {
        let query = query.trim_start_matches('?');
        let params: Vec<(String, String)> = query
            .split('&')
            .filter(|s| !s.is_empty())
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or("");
                Some((
                    urlencoding::decode(key).ok()?.into_owned(),
                    urlencoding::decode(value).ok()?.into_owned(),
                ))
            })
            .collect();

        Self { params }
    }

    /// Append a parameter.
    pub fn append(&mut self, name: &str, value: &str) {
        self.params.push((name.to_string(), value.to_string()));
    }

    /// Delete all parameters with the given name.
    pub fn delete(&mut self, name: &str) {
        self.params.retain(|(n, _)| n != name);
    }

    /// Get the first value for a parameter.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Get all values for a parameter.
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.params
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// Check if a parameter exists.
    pub fn has(&self, name: &str) -> bool {
        self.params.iter().any(|(n, _)| n == name)
    }

    /// Set a parameter (replaces all existing).
    pub fn set(&mut self, name: &str, value: &str) {
        self.delete(name);
        self.append(name, value);
    }

    /// Sort parameters by name.
    pub fn sort(&mut self) {
        self.params.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    /// Get all entries.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Get all keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.params.iter().map(|(k, _)| k.as_str())
    }

    /// Get all values.
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.params.iter().map(|(_, v)| v.as_str())
    }

    /// Convert to a query string.
    pub fn to_string(&self) -> String {
        self.params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    urlencoding::encode(k),
                    urlencoding::encode(v)
                )
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_creation() {
        let location = Location::new("https://example.com:8080/path?query=value#hash").unwrap();

        assert_eq!(location.protocol(), "https:");
        assert_eq!(location.hostname(), "example.com");
        assert_eq!(location.port(), "8080");
        assert_eq!(location.host(), "example.com:8080");
        assert_eq!(location.pathname(), "/path");
        assert_eq!(location.search(), "?query=value");
        assert_eq!(location.hash(), "#hash");
        assert_eq!(location.origin(), "https://example.com:8080");
    }

    #[test]
    fn test_location_setters() {
        let mut location = Location::new("https://example.com/").unwrap();

        location.set_pathname("/new-path");
        assert_eq!(location.pathname(), "/new-path");

        location.set_search("?foo=bar");
        assert_eq!(location.search(), "?foo=bar");

        location.set_hash("#section");
        assert_eq!(location.hash(), "#section");
    }

    #[test]
    fn test_url_search_params() {
        let mut params = URLSearchParams::parse("?foo=bar&baz=qux&foo=another");

        assert_eq!(params.get("foo"), Some("bar"));
        assert_eq!(params.get_all("foo"), vec!["bar", "another"]);
        assert!(params.has("baz"));
        assert!(!params.has("nonexistent"));

        params.set("foo", "new");
        assert_eq!(params.get("foo"), Some("new"));
        assert_eq!(params.get_all("foo"), vec!["new"]);

        params.delete("baz");
        assert!(!params.has("baz"));
    }

    #[test]
    fn test_url_search_params_to_string() {
        let mut params = URLSearchParams::new();
        params.append("name", "John Doe");
        params.append("age", "30");

        let query = params.to_string();
        assert!(query.contains("name=John%20Doe"));
        assert!(query.contains("age=30"));
    }
}
