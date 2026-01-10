//! JavaScript engine wrapper.

use crate::context::JsContext;
use crate::runtime::Runtime;
use boa_engine::{
    Context, JsError, JsResult, JsValue, Source,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::sync::Arc;
use parking_lot::RwLock;

/// JavaScript engine.
pub struct JsEngine {
    /// Boa context.
    context: Context,
    /// Runtime environment.
    runtime: Arc<RwLock<Runtime>>,
    /// Script counter for identification.
    script_counter: u64,
}

impl JsEngine {
    /// Create a new JavaScript engine.
    pub fn new() -> Self {
        let mut context = Context::default();
        let runtime = Arc::new(RwLock::new(Runtime::new()));

        // Set up the global object with browser APIs
        Self::setup_globals(&mut context, runtime.clone());

        Self {
            context,
            runtime,
            script_counter: 0,
        }
    }

    /// Set up global browser APIs.
    fn setup_globals(context: &mut Context, runtime: Arc<RwLock<Runtime>>) {
        // Console API
        crate::console::register_console(context);

        // Timer APIs
        crate::timers::register_timers(context, runtime.clone());

        // Window object (self-referential global)
        let window = context.global_object();
        context
            .register_global_property(js_string!("window"), window.clone(), Attribute::all())
            .expect("Failed to register window");
        context
            .register_global_property(js_string!("self"), window.clone(), Attribute::all())
            .expect("Failed to register self");
        context
            .register_global_property(js_string!("globalThis"), window, Attribute::all())
            .expect("Failed to register globalThis");
    }

    /// Execute a script and return the result.
    pub fn execute(&mut self, source: &str) -> Result<JsValue, JsEngineError> {
        self.script_counter += 1;
        let script_name = format!("script_{}", self.script_counter);

        let source = Source::from_bytes(source.as_bytes());

        self.context
            .eval(source)
            .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))
    }

    /// Execute a script from a URL.
    pub fn execute_script(&mut self, source: &str, url: &str) -> Result<JsValue, JsEngineError> {
        let source = Source::from_bytes(source.as_bytes());

        self.context
            .eval(source)
            .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))
    }

    /// Execute a module.
    pub fn execute_module(&mut self, source: &str, url: &str) -> Result<JsValue, JsEngineError> {
        // Parse as module
        let source = Source::from_bytes(source.as_bytes());

        // For now, evaluate as script (full module support requires more setup)
        self.context
            .eval(source)
            .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))
    }

    /// Call a function by name.
    pub fn call_function(
        &mut self,
        name: &str,
        args: &[JsValue],
    ) -> Result<JsValue, JsEngineError> {
        let global = self.context.global_object();
        let func = global
            .get(js_string!(name.to_string()), &mut self.context)
            .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))?;

        if func.is_callable() {
            func.as_callable()
                .unwrap()
                .call(&JsValue::undefined(), args, &mut self.context)
                .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))
        } else {
            Err(JsEngineError::Execution(format!(
                "{} is not a function",
                name
            )))
        }
    }

    /// Set a global variable.
    pub fn set_global(&mut self, name: &str, value: JsValue) -> Result<(), JsEngineError> {
        self.context
            .register_global_property(js_string!(name.to_string()), value, Attribute::all())
            .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))
    }

    /// Get a global variable.
    pub fn get_global(&mut self, name: &str) -> Result<JsValue, JsEngineError> {
        let global = self.context.global_object();
        global
            .get(js_string!(name.to_string()), &mut self.context)
            .map_err(|e| JsEngineError::Execution(format_js_error(&e, &mut self.context)))
    }

    /// Process pending tasks in the event loop.
    pub fn run_pending_jobs(&mut self) {
        self.context.run_jobs();
    }

    /// Check if there are pending jobs.
    pub fn has_pending_jobs(&self) -> bool {
        // Boa handles this internally
        false
    }

    /// Get the runtime.
    pub fn runtime(&self) -> Arc<RwLock<Runtime>> {
        self.runtime.clone()
    }

    /// Get mutable access to the context.
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Get access to the context.
    pub fn context(&self) -> &Context {
        &self.context
    }
}

impl Default for JsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript engine error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum JsEngineError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Reference error: {0}")]
    ReferenceError(String),
}

/// Format a JavaScript error for display.
fn format_js_error(error: &JsError, context: &mut Context) -> String {
    error
        .try_native(context)
        .map(|e| e.message().to_std_string_escaped())
        .unwrap_or_else(|_| "Unknown error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut engine = JsEngine::new();
        let result = engine.execute("1 + 2").unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_string_execution() {
        let mut engine = JsEngine::new();
        let result = engine.execute("'hello' + ' world'").unwrap();
        assert_eq!(
            result.as_string().unwrap().to_std_string_escaped(),
            "hello world"
        );
    }

    #[test]
    fn test_function_call() {
        let mut engine = JsEngine::new();
        engine.execute("function add(a, b) { return a + b; }").unwrap();

        let result = engine
            .call_function(
                "add",
                &[JsValue::from(5), JsValue::from(3)],
            )
            .unwrap();

        assert_eq!(result.as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_global_variable() {
        let mut engine = JsEngine::new();
        engine.set_global("myValue", JsValue::from(42)).unwrap();

        let result = engine.execute("myValue * 2").unwrap();
        assert_eq!(result.as_number().unwrap(), 84.0);
    }
}
