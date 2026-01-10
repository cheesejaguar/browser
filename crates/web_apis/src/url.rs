//! URL API implementation.

use boa_engine::{
    Context, JsArgs, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};

/// URL API implementation.
#[derive(Clone, Debug)]
pub struct URL {
    /// Internal URL.
    url: url::Url,
}

impl URL {
    /// Create a new URL.
    pub fn new(url: &str) -> Result<Self, url::ParseError> {
        let url = url::Url::parse(url)?;
        Ok(Self { url })
    }

    /// Create a new URL with a base.
    pub fn new_with_base(url: &str, base: &str) -> Result<Self, url::ParseError> {
        let base_url = url::Url::parse(base)?;
        let url = base_url.join(url)?;
        Ok(Self { url })
    }

    /// Get the href (full URL).
    pub fn href(&self) -> String {
        self.url.to_string()
    }

    /// Set the href.
    pub fn set_href(&mut self, href: &str) -> Result<(), url::ParseError> {
        self.url = url::Url::parse(href)?;
        Ok(())
    }

    /// Get the origin.
    pub fn origin(&self) -> String {
        self.url.origin().ascii_serialization()
    }

    /// Get the protocol.
    pub fn protocol(&self) -> String {
        format!("{}:", self.url.scheme())
    }

    /// Set the protocol.
    pub fn set_protocol(&mut self, protocol: &str) {
        let protocol = protocol.trim_end_matches(':');
        let _ = self.url.set_scheme(protocol);
    }

    /// Get the username.
    pub fn username(&self) -> String {
        self.url.username().to_string()
    }

    /// Set the username.
    pub fn set_username(&mut self, username: &str) {
        let _ = self.url.set_username(username);
    }

    /// Get the password.
    pub fn password(&self) -> Option<String> {
        self.url.password().map(|s| s.to_string())
    }

    /// Set the password.
    pub fn set_password(&mut self, password: Option<&str>) {
        let _ = self.url.set_password(password);
    }

    /// Get the host.
    pub fn host(&self) -> String {
        match self.url.port() {
            Some(port) => format!("{}:{}", self.url.host_str().unwrap_or(""), port),
            None => self.url.host_str().unwrap_or("").to_string(),
        }
    }

    /// Set the host.
    pub fn set_host(&mut self, host: &str) {
        if let Some((hostname, port)) = host.split_once(':') {
            let _ = self.url.set_host(Some(hostname));
            if let Ok(port) = port.parse() {
                let _ = self.url.set_port(Some(port));
            }
        } else {
            let _ = self.url.set_host(Some(host));
        }
    }

    /// Get the hostname.
    pub fn hostname(&self) -> String {
        self.url.host_str().unwrap_or("").to_string()
    }

    /// Set the hostname.
    pub fn set_hostname(&mut self, hostname: &str) {
        let _ = self.url.set_host(Some(hostname));
    }

    /// Get the port.
    pub fn port(&self) -> String {
        self.url.port().map(|p| p.to_string()).unwrap_or_default()
    }

    /// Set the port.
    pub fn set_port(&mut self, port: &str) {
        if port.is_empty() {
            let _ = self.url.set_port(None);
        } else if let Ok(port) = port.parse() {
            let _ = self.url.set_port(Some(port));
        }
    }

    /// Get the pathname.
    pub fn pathname(&self) -> String {
        self.url.path().to_string()
    }

    /// Set the pathname.
    pub fn set_pathname(&mut self, pathname: &str) {
        self.url.set_path(pathname);
    }

    /// Get the search (query string with ?).
    pub fn search(&self) -> String {
        self.url.query().map(|q| format!("?{}", q)).unwrap_or_default()
    }

    /// Set the search.
    pub fn set_search(&mut self, search: &str) {
        let search = search.trim_start_matches('?');
        self.url.set_query(if search.is_empty() { None } else { Some(search) });
    }

    /// Get the search params.
    pub fn search_params(&self) -> URLSearchParams {
        URLSearchParams::from_url(&self.url)
    }

    /// Get the hash.
    pub fn hash(&self) -> String {
        self.url.fragment().map(|f| format!("#{}", f)).unwrap_or_default()
    }

    /// Set the hash.
    pub fn set_hash(&mut self, hash: &str) {
        let hash = hash.trim_start_matches('#');
        self.url.set_fragment(if hash.is_empty() { None } else { Some(hash) });
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> String {
        self.href()
    }

    /// Register the URL class on the global object.
    pub fn register(context: &mut Context) {
        let url_constructor = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(url_can_parse), js_string!("canParse"), 2)
            .function(NativeFunction::from_fn_ptr(url_create_object_url), js_string!("createObjectURL"), 1)
            .function(NativeFunction::from_fn_ptr(url_revoke_object_url), js_string!("revokeObjectURL"), 1)
            .build();

        context
            .register_global_property(js_string!("URL"), url_constructor, Attribute::all())
            .expect("Failed to register URL");

        // Also register URLSearchParams
        URLSearchParams::register(context);
    }
}

/// URL search params.
#[derive(Clone, Debug, Default)]
pub struct URLSearchParams {
    params: Vec<(String, String)>,
}

impl URLSearchParams {
    /// Create empty search params.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a URL.
    pub fn from_url(url: &url::Url) -> Self {
        let params: Vec<(String, String)> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        Self { params }
    }

    /// Parse from a string.
    pub fn parse(init: &str) -> Self {
        let init = init.trim_start_matches('?');
        let params: Vec<(String, String)> = form_urlencoded::parse(init.as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        Self { params }
    }

    /// Append a parameter.
    pub fn append(&mut self, name: &str, value: &str) {
        self.params.push((name.to_string(), value.to_string()));
    }

    /// Delete parameters by name.
    pub fn delete(&mut self, name: &str) {
        self.params.retain(|(n, _)| n != name);
    }

    /// Get the first value for a name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Get all values for a name.
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

    /// Set a parameter (replaces all).
    pub fn set(&mut self, name: &str, value: &str) {
        self.delete(name);
        self.append(name, value);
    }

    /// Sort parameters by name.
    pub fn sort(&mut self) {
        self.params.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    /// Get size.
    pub fn size(&self) -> usize {
        self.params.len()
    }

    /// Convert to string.
    pub fn to_string(&self) -> String {
        form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&self.params)
            .finish()
    }

    /// Iterate entries.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Iterate keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.params.iter().map(|(k, _)| k.as_str())
    }

    /// Iterate values.
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.params.iter().map(|(_, v)| v.as_str())
    }

    /// Register URLSearchParams on the global object.
    pub fn register(context: &mut Context) {
        let search_params = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(search_params_append), js_string!("append"), 2)
            .function(NativeFunction::from_fn_ptr(search_params_delete), js_string!("delete"), 1)
            .function(NativeFunction::from_fn_ptr(search_params_get), js_string!("get"), 1)
            .function(NativeFunction::from_fn_ptr(search_params_get_all), js_string!("getAll"), 1)
            .function(NativeFunction::from_fn_ptr(search_params_has), js_string!("has"), 1)
            .function(NativeFunction::from_fn_ptr(search_params_set), js_string!("set"), 2)
            .function(NativeFunction::from_fn_ptr(search_params_sort), js_string!("sort"), 0)
            .function(NativeFunction::from_fn_ptr(search_params_to_string), js_string!("toString"), 0)
            .function(NativeFunction::from_fn_ptr(search_params_entries), js_string!("entries"), 0)
            .function(NativeFunction::from_fn_ptr(search_params_keys), js_string!("keys"), 0)
            .function(NativeFunction::from_fn_ptr(search_params_values), js_string!("values"), 0)
            .function(NativeFunction::from_fn_ptr(search_params_for_each), js_string!("forEach"), 1)
            .build();

        context
            .register_global_property(js_string!("URLSearchParams"), search_params, Attribute::all())
            .expect("Failed to register URLSearchParams");
    }
}

// Native function implementations
fn url_can_parse(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url_str = args.get_or_undefined(0).to_string(context)?;
    let base = args.get(1);

    let result = if let Some(base) = base {
        if !base.is_undefined() {
            let base_str = base.to_string(context)?;
            URL::new_with_base(&url_str.to_std_string_escaped(), &base_str.to_std_string_escaped()).is_ok()
        } else {
            URL::new(&url_str.to_std_string_escaped()).is_ok()
        }
    } else {
        URL::new(&url_str.to_std_string_escaped()).is_ok()
    };

    Ok(JsValue::from(result))
}

fn url_create_object_url(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    // Would create a blob URL
    Ok(js_string!("blob:null/00000000-0000-0000-0000-000000000000").into())
}

fn url_revoke_object_url(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_append(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_delete(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_get(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::null())
}

fn search_params_get_all(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined()) // Would return array
}

fn search_params_has(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(false))
}

fn search_params_set(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_sort(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_to_string(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(js_string!("").into())
}

fn search_params_entries(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_keys(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_values(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn search_params_for_each(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let url = URL::new("https://user:pass@example.com:8080/path?query=value#hash").unwrap();

        assert_eq!(url.protocol(), "https:");
        assert_eq!(url.username(), "user");
        assert_eq!(url.password(), Some("pass".to_string()));
        assert_eq!(url.hostname(), "example.com");
        assert_eq!(url.port(), "8080");
        assert_eq!(url.pathname(), "/path");
        assert_eq!(url.search(), "?query=value");
        assert_eq!(url.hash(), "#hash");
    }

    #[test]
    fn test_url_with_base() {
        let url = URL::new_with_base("/path/to/resource", "https://example.com/base/").unwrap();
        assert_eq!(url.href(), "https://example.com/path/to/resource");

        let url = URL::new_with_base("../sibling", "https://example.com/base/current/").unwrap();
        assert_eq!(url.href(), "https://example.com/base/sibling");
    }

    #[test]
    fn test_url_search_params() {
        let mut params = URLSearchParams::parse("?foo=bar&baz=qux");

        assert_eq!(params.get("foo"), Some("bar"));
        assert!(params.has("baz"));

        params.append("foo", "another");
        assert_eq!(params.get_all("foo"), vec!["bar", "another"]);

        params.set("foo", "replaced");
        assert_eq!(params.get_all("foo"), vec!["replaced"]);

        params.delete("baz");
        assert!(!params.has("baz"));
    }

    #[test]
    fn test_search_params_to_string() {
        let mut params = URLSearchParams::new();
        params.append("name", "John Doe");
        params.append("age", "30");

        let str = params.to_string();
        assert!(str.contains("name=John+Doe") || str.contains("name=John%20Doe"));
        assert!(str.contains("age=30"));
    }
}
