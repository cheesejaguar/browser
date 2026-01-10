//! JavaScript execution context.

use boa_engine::{Context, JsValue, js_string, object::builtins::JsFunction, property::Attribute};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// JavaScript execution context with DOM bindings.
pub struct JsContext {
    /// Boa context.
    context: Context,
    /// DOM node bindings (node ID -> JS object reference).
    node_bindings: HashMap<u64, JsValue>,
    /// Event listeners.
    event_listeners: HashMap<(u64, String), Vec<JsValue>>,
    /// Context ID.
    id: u64,
}

impl JsContext {
    /// Create a new JavaScript context.
    pub fn new(id: u64) -> Self {
        let context = Context::default();

        Self {
            context,
            node_bindings: HashMap::new(),
            event_listeners: HashMap::new(),
            id,
        }
    }

    /// Get the context ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Bind a DOM node to a JavaScript object.
    pub fn bind_node(&mut self, node_id: u64, js_object: JsValue) {
        self.node_bindings.insert(node_id, js_object);
    }

    /// Get the JavaScript object for a DOM node.
    pub fn get_node_binding(&self, node_id: u64) -> Option<&JsValue> {
        self.node_bindings.get(&node_id)
    }

    /// Remove a DOM node binding.
    pub fn remove_node_binding(&mut self, node_id: u64) {
        self.node_bindings.remove(&node_id);
        // Also remove event listeners for this node
        self.event_listeners.retain(|(id, _), _| *id != node_id);
    }

    /// Add an event listener.
    pub fn add_event_listener(&mut self, node_id: u64, event_type: &str, callback: JsValue) {
        let key = (node_id, event_type.to_string());
        self.event_listeners
            .entry(key)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// Remove an event listener.
    pub fn remove_event_listener(&mut self, node_id: u64, event_type: &str, callback: &JsValue) {
        let key = (node_id, event_type.to_string());
        if let Some(listeners) = self.event_listeners.get_mut(&key) {
            listeners.retain(|l| !js_values_equal(l, callback));
        }
    }

    /// Get event listeners for a node and event type.
    pub fn get_event_listeners(&self, node_id: u64, event_type: &str) -> Vec<JsValue> {
        let key = (node_id, event_type.to_string());
        self.event_listeners.get(&key).cloned().unwrap_or_default()
    }

    /// Dispatch an event to listeners.
    pub fn dispatch_event(
        &mut self,
        node_id: u64,
        event_type: &str,
        event_object: JsValue,
    ) -> Result<bool, String> {
        let listeners = self.get_event_listeners(node_id, event_type);
        let mut prevented = false;

        for listener in listeners {
            if let Some(callable) = listener.as_callable() {
                let result = callable.call(&JsValue::undefined(), &[event_object.clone()], &mut self.context);

                if let Err(e) = result {
                    eprintln!("Event handler error: {:?}", e);
                }

                // Check if preventDefault was called
                if let Ok(prevented_value) = event_object.as_object().and_then(|obj| {
                    obj.get(js_string!("defaultPrevented"), &mut self.context).ok()
                }) {
                    if let Some(true) = prevented_value.as_boolean() {
                        prevented = true;
                    }
                }
            }
        }

        Ok(!prevented)
    }

    /// Get mutable access to the Boa context.
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Get access to the Boa context.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Clear all bindings (for cleanup).
    pub fn clear(&mut self) {
        self.node_bindings.clear();
        self.event_listeners.clear();
    }
}

/// Compare two JsValues for equality (by reference for objects).
fn js_values_equal(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (JsValue::Undefined, JsValue::Undefined) => true,
        (JsValue::Null, JsValue::Null) => true,
        (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
        (JsValue::Integer(a), JsValue::Integer(b)) => a == b,
        (JsValue::Rational(a), JsValue::Rational(b)) => a == b,
        (JsValue::String(a), JsValue::String(b)) => a == b,
        (JsValue::Object(a), JsValue::Object(b)) => std::ptr::eq(a.as_ref(), b.as_ref()),
        _ => false,
    }
}

/// Script origin information.
#[derive(Clone, Debug)]
pub struct ScriptOrigin {
    /// Script URL.
    pub url: String,
    /// Base URL for relative imports.
    pub base_url: Option<String>,
    /// Whether this is a module.
    pub is_module: bool,
    /// Script nonce (for CSP).
    pub nonce: Option<String>,
}

impl ScriptOrigin {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            base_url: None,
            is_module: false,
            nonce: None,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn as_module(mut self) -> Self {
        self.is_module = true;
        self
    }

    pub fn with_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = Some(nonce.into());
        self
    }
}

/// Execution options.
#[derive(Clone, Debug, Default)]
pub struct ExecutionOptions {
    /// Maximum execution time.
    pub timeout_ms: Option<u64>,
    /// Maximum memory usage.
    pub max_memory_bytes: Option<usize>,
    /// Allow eval.
    pub allow_eval: bool,
    /// Allow Function constructor.
    pub allow_function_constructor: bool,
}

impl ExecutionOptions {
    pub fn new() -> Self {
        Self {
            timeout_ms: None,
            max_memory_bytes: None,
            allow_eval: true,
            allow_function_constructor: true,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_memory_limit(mut self, max_bytes: usize) -> Self {
        self.max_memory_bytes = Some(max_bytes);
        self
    }

    pub fn disable_eval(mut self) -> Self {
        self.allow_eval = false;
        self
    }

    pub fn disable_function_constructor(mut self) -> Self {
        self.allow_function_constructor = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = JsContext::new(1);
        assert_eq!(context.id(), 1);
    }

    #[test]
    fn test_node_binding() {
        let mut context = JsContext::new(1);
        context.bind_node(42, JsValue::from(100));

        let binding = context.get_node_binding(42);
        assert!(binding.is_some());
    }

    #[test]
    fn test_script_origin() {
        let origin = ScriptOrigin::new("https://example.com/script.js")
            .with_base_url("https://example.com/")
            .as_module()
            .with_nonce("abc123");

        assert_eq!(origin.url, "https://example.com/script.js");
        assert_eq!(origin.base_url, Some("https://example.com/".to_string()));
        assert!(origin.is_module);
        assert_eq!(origin.nonce, Some("abc123".to_string()));
    }
}
