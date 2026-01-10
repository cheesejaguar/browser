//! DOM bindings for JavaScript.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::{builtins::JsFunction, ObjectInitializer, JsObject},
    property::{Attribute, PropertyDescriptor},
    class::{Class, ClassBuilder},
};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// DOM binding registry.
pub struct DomBindings {
    /// Node cache (DOM node ID -> JS object).
    node_cache: HashMap<u64, JsValue>,
    /// Event handlers.
    event_handlers: HashMap<(u64, String), Vec<JsValue>>,
}

impl DomBindings {
    /// Create new DOM bindings.
    pub fn new() -> Self {
        Self {
            node_cache: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }

    /// Register all DOM classes and constructors.
    pub fn register(&self, context: &mut Context) {
        // Register Node class
        register_node_class(context);

        // Register Element class
        register_element_class(context);

        // Register Document class
        register_document_class(context);

        // Register Event class
        register_event_class(context);

        // Register HTMLElement and subclasses
        register_html_element_classes(context);
    }

    /// Create a JavaScript object for a DOM node.
    pub fn create_node_object(
        &mut self,
        node_id: u64,
        node_type: NodeType,
        context: &mut Context,
    ) -> JsValue {
        // Check cache first
        if let Some(obj) = self.node_cache.get(&node_id) {
            return obj.clone();
        }

        // Create new object based on node type
        let obj = match node_type {
            NodeType::Element(tag) => create_element_object(node_id, &tag, context),
            NodeType::Text => create_text_node_object(node_id, context),
            NodeType::Comment => create_comment_node_object(node_id, context),
            NodeType::Document => create_document_object(node_id, context),
            NodeType::DocumentFragment => create_document_fragment_object(node_id, context),
        };

        // Cache the object
        self.node_cache.insert(node_id, obj.clone());

        obj
    }

    /// Add an event listener.
    pub fn add_event_listener(
        &mut self,
        node_id: u64,
        event_type: &str,
        handler: JsValue,
    ) {
        let key = (node_id, event_type.to_string());
        self.event_handlers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Remove an event listener.
    pub fn remove_event_listener(
        &mut self,
        node_id: u64,
        event_type: &str,
        handler: &JsValue,
    ) {
        let key = (node_id, event_type.to_string());
        if let Some(handlers) = self.event_handlers.get_mut(&key) {
            handlers.retain(|h| !js_value_equals(h, handler));
        }
    }

    /// Dispatch an event to handlers.
    pub fn dispatch_event(
        &self,
        node_id: u64,
        event_type: &str,
        event: JsValue,
        context: &mut Context,
    ) -> bool {
        let key = (node_id, event_type.to_string());

        if let Some(handlers) = self.event_handlers.get(&key) {
            for handler in handlers {
                if let Some(callable) = handler.as_callable() {
                    let this = self.node_cache.get(&node_id).cloned().unwrap_or(JsValue::undefined());
                    let _ = callable.call(&this, &[event.clone()], context);
                }
            }
        }

        true // Would return false if preventDefault was called
    }

    /// Remove a node from the cache.
    pub fn remove_node(&mut self, node_id: u64) {
        self.node_cache.remove(&node_id);
        // Remove associated event handlers
        self.event_handlers.retain(|(id, _), _| *id != node_id);
    }

    /// Clear all bindings.
    pub fn clear(&mut self) {
        self.node_cache.clear();
        self.event_handlers.clear();
    }
}

impl Default for DomBindings {
    fn default() -> Self {
        Self::new()
    }
}

/// Node type for creating appropriate JS objects.
#[derive(Clone, Debug)]
pub enum NodeType {
    Element(String),
    Text,
    Comment,
    Document,
    DocumentFragment,
}

/// Compare two JsValues for equality.
fn js_value_equals(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (JsValue::Object(a), JsValue::Object(b)) => std::ptr::eq(a.as_ref(), b.as_ref()),
        _ => false,
    }
}

/// Register the Node class.
fn register_node_class(context: &mut Context) {
    let node = ObjectInitializer::new(context)
        // Node type constants
        .property(js_string!("ELEMENT_NODE"), 1, Attribute::READONLY)
        .property(js_string!("ATTRIBUTE_NODE"), 2, Attribute::READONLY)
        .property(js_string!("TEXT_NODE"), 3, Attribute::READONLY)
        .property(js_string!("CDATA_SECTION_NODE"), 4, Attribute::READONLY)
        .property(js_string!("PROCESSING_INSTRUCTION_NODE"), 7, Attribute::READONLY)
        .property(js_string!("COMMENT_NODE"), 8, Attribute::READONLY)
        .property(js_string!("DOCUMENT_NODE"), 9, Attribute::READONLY)
        .property(js_string!("DOCUMENT_TYPE_NODE"), 10, Attribute::READONLY)
        .property(js_string!("DOCUMENT_FRAGMENT_NODE"), 11, Attribute::READONLY)
        .build();

    context
        .register_global_property(js_string!("Node"), node, Attribute::all())
        .expect("Failed to register Node");
}

/// Register the Element class.
fn register_element_class(context: &mut Context) {
    let element_proto = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(element_get_attribute), js_string!("getAttribute"), 1)
        .function(NativeFunction::from_fn_ptr(element_set_attribute), js_string!("setAttribute"), 2)
        .function(NativeFunction::from_fn_ptr(element_remove_attribute), js_string!("removeAttribute"), 1)
        .function(NativeFunction::from_fn_ptr(element_has_attribute), js_string!("hasAttribute"), 1)
        .function(NativeFunction::from_fn_ptr(element_query_selector), js_string!("querySelector"), 1)
        .function(NativeFunction::from_fn_ptr(element_query_selector_all), js_string!("querySelectorAll"), 1)
        .function(NativeFunction::from_fn_ptr(element_get_elements_by_class_name), js_string!("getElementsByClassName"), 1)
        .function(NativeFunction::from_fn_ptr(element_get_elements_by_tag_name), js_string!("getElementsByTagName"), 1)
        .function(NativeFunction::from_fn_ptr(element_append), js_string!("append"), 1)
        .function(NativeFunction::from_fn_ptr(element_prepend), js_string!("prepend"), 1)
        .function(NativeFunction::from_fn_ptr(element_remove), js_string!("remove"), 0)
        .function(NativeFunction::from_fn_ptr(node_append_child), js_string!("appendChild"), 1)
        .function(NativeFunction::from_fn_ptr(node_remove_child), js_string!("removeChild"), 1)
        .function(NativeFunction::from_fn_ptr(node_insert_before), js_string!("insertBefore"), 2)
        .function(NativeFunction::from_fn_ptr(node_replace_child), js_string!("replaceChild"), 2)
        .function(NativeFunction::from_fn_ptr(node_clone_node), js_string!("cloneNode"), 1)
        .function(NativeFunction::from_fn_ptr(node_contains), js_string!("contains"), 1)
        .function(NativeFunction::from_fn_ptr(event_target_add_event_listener), js_string!("addEventListener"), 2)
        .function(NativeFunction::from_fn_ptr(event_target_remove_event_listener), js_string!("removeEventListener"), 2)
        .function(NativeFunction::from_fn_ptr(event_target_dispatch_event), js_string!("dispatchEvent"), 1)
        .build();

    context
        .register_global_property(js_string!("Element"), element_proto, Attribute::all())
        .expect("Failed to register Element");
}

/// Register the Document class.
fn register_document_class(context: &mut Context) {
    let document = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(document_get_element_by_id), js_string!("getElementById"), 1)
        .function(NativeFunction::from_fn_ptr(document_create_element), js_string!("createElement"), 1)
        .function(NativeFunction::from_fn_ptr(document_create_text_node), js_string!("createTextNode"), 1)
        .function(NativeFunction::from_fn_ptr(document_create_comment), js_string!("createComment"), 1)
        .function(NativeFunction::from_fn_ptr(document_create_document_fragment), js_string!("createDocumentFragment"), 0)
        .function(NativeFunction::from_fn_ptr(element_query_selector), js_string!("querySelector"), 1)
        .function(NativeFunction::from_fn_ptr(element_query_selector_all), js_string!("querySelectorAll"), 1)
        .function(NativeFunction::from_fn_ptr(element_get_elements_by_class_name), js_string!("getElementsByClassName"), 1)
        .function(NativeFunction::from_fn_ptr(element_get_elements_by_tag_name), js_string!("getElementsByTagName"), 1)
        .build();

    context
        .register_global_property(js_string!("document"), document, Attribute::all())
        .expect("Failed to register document");
}

/// Register the Event class.
fn register_event_class(context: &mut Context) {
    let event = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(event_prevent_default), js_string!("preventDefault"), 0)
        .function(NativeFunction::from_fn_ptr(event_stop_propagation), js_string!("stopPropagation"), 0)
        .function(NativeFunction::from_fn_ptr(event_stop_immediate_propagation), js_string!("stopImmediatePropagation"), 0)
        .build();

    context
        .register_global_property(js_string!("Event"), event, Attribute::all())
        .expect("Failed to register Event");
}

/// Register HTMLElement and common subclasses.
fn register_html_element_classes(context: &mut Context) {
    // HTMLElement (extends Element)
    let html_element = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(html_element_focus), js_string!("focus"), 0)
        .function(NativeFunction::from_fn_ptr(html_element_blur), js_string!("blur"), 0)
        .function(NativeFunction::from_fn_ptr(html_element_click), js_string!("click"), 0)
        .build();

    context
        .register_global_property(js_string!("HTMLElement"), html_element, Attribute::all())
        .expect("Failed to register HTMLElement");
}

// === Native function implementations ===

fn element_get_attribute(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(ctx)?;
    // Would look up attribute on the DOM node
    Ok(JsValue::null())
}

fn element_set_attribute(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(ctx)?;
    let _value = args.get_or_undefined(1).to_string(ctx)?;
    // Would set attribute on the DOM node
    Ok(JsValue::undefined())
}

fn element_remove_attribute(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(ctx)?;
    // Would remove attribute from the DOM node
    Ok(JsValue::undefined())
}

fn element_has_attribute(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(ctx)?;
    // Would check if attribute exists on the DOM node
    Ok(JsValue::from(false))
}

fn element_query_selector(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _selector = args.get_or_undefined(0).to_string(ctx)?;
    // Would query the DOM
    Ok(JsValue::null())
}

fn element_query_selector_all(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _selector = args.get_or_undefined(0).to_string(ctx)?;
    // Would query the DOM and return NodeList
    Ok(JsValue::undefined()) // Would return NodeList
}

fn element_get_elements_by_class_name(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _class_name = args.get_or_undefined(0).to_string(ctx)?;
    Ok(JsValue::undefined()) // Would return HTMLCollection
}

fn element_get_elements_by_tag_name(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _tag_name = args.get_or_undefined(0).to_string(ctx)?;
    Ok(JsValue::undefined()) // Would return HTMLCollection
}

fn element_append(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn element_prepend(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn element_remove(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn node_append_child(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn node_remove_child(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn node_insert_before(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn node_replace_child(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn node_clone_node(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn node_contains(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(false))
}

fn event_target_add_event_listener(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _event_type = args.get_or_undefined(0).to_string(ctx)?;
    let _handler = args.get_or_undefined(1);
    Ok(JsValue::undefined())
}

fn event_target_remove_event_listener(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _event_type = args.get_or_undefined(0).to_string(ctx)?;
    let _handler = args.get_or_undefined(1);
    Ok(JsValue::undefined())
}

fn event_target_dispatch_event(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(true))
}

fn document_get_element_by_id(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _id = args.get_or_undefined(0).to_string(ctx)?;
    Ok(JsValue::null())
}

fn document_create_element(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _tag_name = args.get_or_undefined(0).to_string(ctx)?;
    Ok(JsValue::undefined()) // Would create and return element
}

fn document_create_text_node(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _text = args.get_or_undefined(0).to_string(ctx)?;
    Ok(JsValue::undefined()) // Would create and return text node
}

fn document_create_comment(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let _text = args.get_or_undefined(0).to_string(ctx)?;
    Ok(JsValue::undefined()) // Would create and return comment node
}

fn document_create_document_fragment(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined()) // Would create and return document fragment
}

fn event_prevent_default(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn event_stop_propagation(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn event_stop_immediate_propagation(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn html_element_focus(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn html_element_blur(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn html_element_click(_: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

/// Create an Element JavaScript object.
fn create_element_object(node_id: u64, tag_name: &str, context: &mut Context) -> JsValue {
    let obj = ObjectInitializer::new(context)
        .property(js_string!("nodeType"), 1, Attribute::READONLY)
        .property(js_string!("nodeName"), js_string!(tag_name.to_uppercase()), Attribute::READONLY)
        .property(js_string!("tagName"), js_string!(tag_name.to_uppercase()), Attribute::READONLY)
        .property(js_string!("__nodeId"), node_id as i32, Attribute::empty())
        .build();

    obj.into()
}

/// Create a Text node JavaScript object.
fn create_text_node_object(node_id: u64, context: &mut Context) -> JsValue {
    let obj = ObjectInitializer::new(context)
        .property(js_string!("nodeType"), 3, Attribute::READONLY)
        .property(js_string!("nodeName"), js_string!("#text"), Attribute::READONLY)
        .property(js_string!("__nodeId"), node_id as i32, Attribute::empty())
        .build();

    obj.into()
}

/// Create a Comment node JavaScript object.
fn create_comment_node_object(node_id: u64, context: &mut Context) -> JsValue {
    let obj = ObjectInitializer::new(context)
        .property(js_string!("nodeType"), 8, Attribute::READONLY)
        .property(js_string!("nodeName"), js_string!("#comment"), Attribute::READONLY)
        .property(js_string!("__nodeId"), node_id as i32, Attribute::empty())
        .build();

    obj.into()
}

/// Create a Document JavaScript object.
fn create_document_object(node_id: u64, context: &mut Context) -> JsValue {
    let obj = ObjectInitializer::new(context)
        .property(js_string!("nodeType"), 9, Attribute::READONLY)
        .property(js_string!("nodeName"), js_string!("#document"), Attribute::READONLY)
        .property(js_string!("__nodeId"), node_id as i32, Attribute::empty())
        .build();

    obj.into()
}

/// Create a DocumentFragment JavaScript object.
fn create_document_fragment_object(node_id: u64, context: &mut Context) -> JsValue {
    let obj = ObjectInitializer::new(context)
        .property(js_string!("nodeType"), 11, Attribute::READONLY)
        .property(js_string!("nodeName"), js_string!("#document-fragment"), Attribute::READONLY)
        .property(js_string!("__nodeId"), node_id as i32, Attribute::empty())
        .build();

    obj.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_bindings_creation() {
        let bindings = DomBindings::new();
        assert!(bindings.node_cache.is_empty());
    }

    #[test]
    fn test_create_element_object() {
        let mut context = Context::default();
        let obj = create_element_object(1, "div", &mut context);
        assert!(obj.is_object());
    }
}
