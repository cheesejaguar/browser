//! Fetch API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::{builtins::JsPromise, ObjectInitializer},
    property::Attribute,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Fetch API implementation.
pub struct Fetch {
    /// Default headers.
    default_headers: HashMap<String, String>,
    /// Base URL for relative requests.
    base_url: Option<String>,
}

impl Fetch {
    /// Create a new Fetch API instance.
    pub fn new() -> Self {
        Self {
            default_headers: HashMap::new(),
            base_url: None,
        }
    }

    /// Set the base URL.
    pub fn set_base_url(&mut self, url: impl Into<String>) {
        self.base_url = Some(url.into());
    }

    /// Add a default header.
    pub fn add_default_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.default_headers.insert(name.into(), value.into());
    }

    /// Register the fetch function on the global object.
    pub fn register(context: &mut Context) {
        context
            .register_global_builtin_callable(
                js_string!("fetch"),
                2,
                NativeFunction::from_fn_ptr(fetch_fn),
            )
            .expect("Failed to register fetch");

        // Register Request class
        register_request_class(context);

        // Register Response class
        register_response_class(context);

        // Register Headers class
        register_headers_class(context);
    }
}

impl Default for Fetch {
    fn default() -> Self {
        Self::new()
    }
}

/// Native fetch function.
fn fetch_fn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.get_or_undefined(0);
    let options = args.get(1);

    // Parse URL
    let url_str = url.to_string(context)?;

    // Create a promise for the fetch operation
    let (promise, resolvers) = JsPromise::new_pending(context);

    // In a real implementation, we would:
    // 1. Parse options (method, headers, body, etc.)
    // 2. Make the HTTP request asynchronously
    // 3. Resolve or reject the promise based on the result

    // For now, reject with "not implemented"
    // In production, this would spawn an async task to perform the fetch

    Ok(promise.into())
}

/// Register the Request class.
fn register_request_class(context: &mut Context) {
    let request_proto = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(request_clone), js_string!("clone"), 0)
        .function(NativeFunction::from_fn_ptr(request_array_buffer), js_string!("arrayBuffer"), 0)
        .function(NativeFunction::from_fn_ptr(request_blob), js_string!("blob"), 0)
        .function(NativeFunction::from_fn_ptr(request_form_data), js_string!("formData"), 0)
        .function(NativeFunction::from_fn_ptr(request_json), js_string!("json"), 0)
        .function(NativeFunction::from_fn_ptr(request_text), js_string!("text"), 0)
        .build();

    context
        .register_global_property(js_string!("Request"), request_proto, Attribute::all())
        .expect("Failed to register Request");
}

/// Register the Response class.
fn register_response_class(context: &mut Context) {
    let response_proto = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(response_clone), js_string!("clone"), 0)
        .function(NativeFunction::from_fn_ptr(response_array_buffer), js_string!("arrayBuffer"), 0)
        .function(NativeFunction::from_fn_ptr(response_blob), js_string!("blob"), 0)
        .function(NativeFunction::from_fn_ptr(response_form_data), js_string!("formData"), 0)
        .function(NativeFunction::from_fn_ptr(response_json), js_string!("json"), 0)
        .function(NativeFunction::from_fn_ptr(response_text), js_string!("text"), 0)
        .build();

    // Static methods
    let response_constructor = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(response_error), js_string!("error"), 0)
        .function(NativeFunction::from_fn_ptr(response_redirect), js_string!("redirect"), 2)
        .function(NativeFunction::from_fn_ptr(response_json_static), js_string!("json"), 2)
        .build();

    context
        .register_global_property(js_string!("Response"), response_constructor, Attribute::all())
        .expect("Failed to register Response");
}

/// Register the Headers class.
fn register_headers_class(context: &mut Context) {
    let headers_proto = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(headers_append), js_string!("append"), 2)
        .function(NativeFunction::from_fn_ptr(headers_delete), js_string!("delete"), 1)
        .function(NativeFunction::from_fn_ptr(headers_get), js_string!("get"), 1)
        .function(NativeFunction::from_fn_ptr(headers_has), js_string!("has"), 1)
        .function(NativeFunction::from_fn_ptr(headers_set), js_string!("set"), 2)
        .function(NativeFunction::from_fn_ptr(headers_entries), js_string!("entries"), 0)
        .function(NativeFunction::from_fn_ptr(headers_keys), js_string!("keys"), 0)
        .function(NativeFunction::from_fn_ptr(headers_values), js_string!("values"), 0)
        .function(NativeFunction::from_fn_ptr(headers_for_each), js_string!("forEach"), 1)
        .build();

    context
        .register_global_property(js_string!("Headers"), headers_proto, Attribute::all())
        .expect("Failed to register Headers");
}

// Request methods
fn request_clone(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn request_array_buffer(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn request_blob(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn request_form_data(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn request_json(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn request_text(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

// Response methods
fn response_clone(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn response_array_buffer(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn response_blob(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn response_form_data(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn response_json(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn response_text(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

// Response static methods
fn response_error(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn response_redirect(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn response_json_static(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

// Headers methods
fn headers_append(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn headers_delete(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn headers_get(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::null())
}

fn headers_has(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(false))
}

fn headers_set(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn headers_entries(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn headers_keys(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn headers_values(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn headers_for_each(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

/// Fetch request configuration.
#[derive(Clone, Debug)]
pub struct FetchRequest {
    /// Request URL.
    pub url: String,
    /// HTTP method.
    pub method: String,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body.
    pub body: Option<Vec<u8>>,
    /// Request mode (cors, no-cors, same-origin).
    pub mode: RequestMode,
    /// Credentials mode.
    pub credentials: CredentialsMode,
    /// Cache mode.
    pub cache: CacheMode,
    /// Redirect mode.
    pub redirect: RedirectMode,
    /// Referrer.
    pub referrer: String,
    /// Referrer policy.
    pub referrer_policy: ReferrerPolicy,
    /// Request integrity.
    pub integrity: String,
    /// Keep-alive.
    pub keepalive: bool,
    /// Signal for abort controller.
    pub signal: Option<AbortSignal>,
}

impl FetchRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            mode: RequestMode::Cors,
            credentials: CredentialsMode::SameOrigin,
            cache: CacheMode::Default,
            redirect: RedirectMode::Follow,
            referrer: "about:client".to_string(),
            referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            integrity: String::new(),
            keepalive: false,
            signal: None,
        }
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_vec(value)?;
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json);
        Ok(self)
    }
}

/// Fetch response.
#[derive(Clone, Debug)]
pub struct FetchResponse {
    /// Response URL.
    pub url: String,
    /// HTTP status code.
    pub status: u16,
    /// Status text.
    pub status_text: String,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Vec<u8>,
    /// Whether the response was redirected.
    pub redirected: bool,
    /// Response type.
    pub response_type: ResponseType,
}

impl FetchResponse {
    /// Check if the response was successful (status 200-299).
    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Get the response as text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Get the response as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

/// Request mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestMode {
    Cors,
    NoCors,
    SameOrigin,
    Navigate,
}

/// Credentials mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialsMode {
    Omit,
    SameOrigin,
    Include,
}

/// Cache mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

/// Redirect mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual,
}

/// Referrer policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    SameOrigin,
    Origin,
    StrictOrigin,
    OriginWhenCrossOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

/// Response type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseType {
    Basic,
    Cors,
    Default,
    Error,
    Opaque,
    OpaqueRedirect,
}

/// Abort signal for cancellation.
#[derive(Clone, Debug)]
pub struct AbortSignal {
    /// Whether the signal is aborted.
    pub aborted: bool,
    /// Abort reason.
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_request_builder() {
        let request = FetchRequest::new("https://api.example.com/data")
            .method("POST")
            .header("Authorization", "Bearer token");

        assert_eq!(request.url, "https://api.example.com/data");
        assert_eq!(request.method, "POST");
        assert!(request.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_fetch_response_ok() {
        let response = FetchResponse {
            url: "https://example.com".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            redirected: false,
            response_type: ResponseType::Basic,
        };

        assert!(response.ok());
    }
}
