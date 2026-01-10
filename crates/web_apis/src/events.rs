//! DOM Events implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Event phase constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

/// Base event interface.
#[derive(Clone, Debug)]
pub struct Event {
    /// Event type.
    pub event_type: String,
    /// Whether the event bubbles.
    pub bubbles: bool,
    /// Whether the event is cancelable.
    pub cancelable: bool,
    /// Whether the event is composed.
    pub composed: bool,
    /// Current target.
    pub current_target: Option<u64>,
    /// Target.
    pub target: Option<u64>,
    /// Event phase.
    pub event_phase: EventPhase,
    /// Whether default is prevented.
    pub default_prevented: bool,
    /// Whether propagation is stopped.
    pub propagation_stopped: bool,
    /// Whether immediate propagation is stopped.
    pub immediate_propagation_stopped: bool,
    /// Timestamp.
    pub time_stamp: f64,
    /// Is trusted.
    pub is_trusted: bool,
}

impl Event {
    /// Create a new event.
    pub fn new(event_type: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            bubbles: false,
            cancelable: false,
            composed: false,
            current_target: None,
            target: None,
            event_phase: EventPhase::None,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            time_stamp: Self::get_timestamp(),
            is_trusted: false,
        }
    }

    /// Create with options.
    pub fn with_options(event_type: &str, options: EventInit) -> Self {
        Self {
            event_type: event_type.to_string(),
            bubbles: options.bubbles,
            cancelable: options.cancelable,
            composed: options.composed,
            current_target: None,
            target: None,
            event_phase: EventPhase::None,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            time_stamp: Self::get_timestamp(),
            is_trusted: false,
        }
    }

    fn get_timestamp() -> f64 {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }

    /// Prevent default behavior.
    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }

    /// Stop propagation.
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Stop immediate propagation.
    pub fn stop_immediate_propagation(&mut self) {
        self.propagation_stopped = true;
        self.immediate_propagation_stopped = true;
    }

    /// Composed path.
    pub fn composed_path(&self) -> Vec<u64> {
        // Would return the composed path
        Vec::new()
    }

    /// Register Event class on the global object.
    pub fn register(context: &mut Context) {
        let event = ObjectInitializer::new(context)
            // Phase constants
            .property(js_string!("NONE"), 0, Attribute::READONLY)
            .property(js_string!("CAPTURING_PHASE"), 1, Attribute::READONLY)
            .property(js_string!("AT_TARGET"), 2, Attribute::READONLY)
            .property(js_string!("BUBBLING_PHASE"), 3, Attribute::READONLY)
            // Methods
            .function(NativeFunction::from_fn_ptr(event_prevent_default), js_string!("preventDefault"), 0)
            .function(NativeFunction::from_fn_ptr(event_stop_propagation), js_string!("stopPropagation"), 0)
            .function(NativeFunction::from_fn_ptr(event_stop_immediate_propagation), js_string!("stopImmediatePropagation"), 0)
            .function(NativeFunction::from_fn_ptr(event_composed_path), js_string!("composedPath"), 0)
            .build();

        context
            .register_global_property(js_string!("Event"), event, Attribute::all())
            .expect("Failed to register Event");
    }
}

/// Event initialization options.
#[derive(Clone, Debug, Default)]
pub struct EventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
}

/// Mouse event.
#[derive(Clone, Debug)]
pub struct MouseEvent {
    /// Base event.
    pub base: Event,
    /// Screen X coordinate.
    pub screen_x: i32,
    /// Screen Y coordinate.
    pub screen_y: i32,
    /// Client X coordinate.
    pub client_x: i32,
    /// Client Y coordinate.
    pub client_y: i32,
    /// Page X coordinate.
    pub page_x: i32,
    /// Page Y coordinate.
    pub page_y: i32,
    /// Offset X coordinate.
    pub offset_x: i32,
    /// Offset Y coordinate.
    pub offset_y: i32,
    /// Movement X.
    pub movement_x: i32,
    /// Movement Y.
    pub movement_y: i32,
    /// Button pressed.
    pub button: i16,
    /// Buttons pressed.
    pub buttons: u16,
    /// Ctrl key pressed.
    pub ctrl_key: bool,
    /// Shift key pressed.
    pub shift_key: bool,
    /// Alt key pressed.
    pub alt_key: bool,
    /// Meta key pressed.
    pub meta_key: bool,
    /// Related target.
    pub related_target: Option<u64>,
}

impl MouseEvent {
    pub fn new(event_type: &str) -> Self {
        Self {
            base: Event::new(event_type),
            screen_x: 0,
            screen_y: 0,
            client_x: 0,
            client_y: 0,
            page_x: 0,
            page_y: 0,
            offset_x: 0,
            offset_y: 0,
            movement_x: 0,
            movement_y: 0,
            button: 0,
            buttons: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            related_target: None,
        }
    }

    /// Create a click event.
    pub fn click(x: i32, y: i32) -> Self {
        let mut event = Self::new("click");
        event.base.bubbles = true;
        event.base.cancelable = true;
        event.client_x = x;
        event.client_y = y;
        event.page_x = x;
        event.page_y = y;
        event
    }

    /// Check if modifier key is pressed.
    pub fn get_modifier_state(&self, key: &str) -> bool {
        match key {
            "Control" | "Ctrl" => self.ctrl_key,
            "Shift" => self.shift_key,
            "Alt" => self.alt_key,
            "Meta" => self.meta_key,
            _ => false,
        }
    }
}

/// Keyboard event.
#[derive(Clone, Debug)]
pub struct KeyboardEvent {
    /// Base event.
    pub base: Event,
    /// Key value.
    pub key: String,
    /// Key code.
    pub code: String,
    /// Location.
    pub location: KeyLocation,
    /// Ctrl key pressed.
    pub ctrl_key: bool,
    /// Shift key pressed.
    pub shift_key: bool,
    /// Alt key pressed.
    pub alt_key: bool,
    /// Meta key pressed.
    pub meta_key: bool,
    /// Repeat.
    pub repeat: bool,
    /// Is composing.
    pub is_composing: bool,
}

impl KeyboardEvent {
    pub fn new(event_type: &str, key: &str, code: &str) -> Self {
        let mut base = Event::new(event_type);
        base.bubbles = true;
        base.cancelable = true;

        Self {
            base,
            key: key.to_string(),
            code: code.to_string(),
            location: KeyLocation::Standard,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            repeat: false,
            is_composing: false,
        }
    }

    /// Check if modifier key is pressed.
    pub fn get_modifier_state(&self, key: &str) -> bool {
        match key {
            "Control" | "Ctrl" => self.ctrl_key,
            "Shift" => self.shift_key,
            "Alt" => self.alt_key,
            "Meta" => self.meta_key,
            _ => false,
        }
    }
}

/// Key location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyLocation {
    Standard = 0,
    Left = 1,
    Right = 2,
    Numpad = 3,
}

/// Focus event.
#[derive(Clone, Debug)]
pub struct FocusEvent {
    /// Base event.
    pub base: Event,
    /// Related target.
    pub related_target: Option<u64>,
}

impl FocusEvent {
    pub fn new(event_type: &str) -> Self {
        let mut base = Event::new(event_type);
        base.bubbles = matches!(event_type, "focusin" | "focusout");

        Self {
            base,
            related_target: None,
        }
    }
}

/// Input event.
#[derive(Clone, Debug)]
pub struct InputEvent {
    /// Base event.
    pub base: Event,
    /// Data.
    pub data: Option<String>,
    /// Input type.
    pub input_type: String,
    /// Is composing.
    pub is_composing: bool,
}

impl InputEvent {
    pub fn new(event_type: &str, data: Option<String>) -> Self {
        let mut base = Event::new(event_type);
        base.bubbles = true;

        Self {
            base,
            data,
            input_type: "insertText".to_string(),
            is_composing: false,
        }
    }
}

/// Wheel event.
#[derive(Clone, Debug)]
pub struct WheelEvent {
    /// Base mouse event.
    pub base: MouseEvent,
    /// Delta X.
    pub delta_x: f64,
    /// Delta Y.
    pub delta_y: f64,
    /// Delta Z.
    pub delta_z: f64,
    /// Delta mode.
    pub delta_mode: DeltaMode,
}

impl WheelEvent {
    pub fn new(delta_x: f64, delta_y: f64) -> Self {
        let mut base = MouseEvent::new("wheel");
        base.base.bubbles = true;
        base.base.cancelable = true;

        Self {
            base,
            delta_x,
            delta_y,
            delta_z: 0.0,
            delta_mode: DeltaMode::Pixel,
        }
    }
}

/// Delta mode for wheel events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaMode {
    Pixel = 0,
    Line = 1,
    Page = 2,
}

/// Touch event.
#[derive(Clone, Debug)]
pub struct TouchEvent {
    /// Base event.
    pub base: Event,
    /// Changed touches.
    pub changed_touches: Vec<Touch>,
    /// Target touches.
    pub target_touches: Vec<Touch>,
    /// All touches.
    pub touches: Vec<Touch>,
    /// Alt key.
    pub alt_key: bool,
    /// Meta key.
    pub meta_key: bool,
    /// Ctrl key.
    pub ctrl_key: bool,
    /// Shift key.
    pub shift_key: bool,
}

/// Single touch point.
#[derive(Clone, Debug)]
pub struct Touch {
    /// Touch identifier.
    pub identifier: i64,
    /// Target node.
    pub target: Option<u64>,
    /// Screen X.
    pub screen_x: f64,
    /// Screen Y.
    pub screen_y: f64,
    /// Client X.
    pub client_x: f64,
    /// Client Y.
    pub client_y: f64,
    /// Page X.
    pub page_x: f64,
    /// Page Y.
    pub page_y: f64,
    /// Radius X.
    pub radius_x: f64,
    /// Radius Y.
    pub radius_y: f64,
    /// Rotation angle.
    pub rotation_angle: f64,
    /// Force.
    pub force: f64,
}

// Native function implementations
fn event_prevent_default(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn event_stop_propagation(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn event_stop_immediate_propagation(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn event_composed_path(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined()) // Would return array
}

/// Custom event.
#[derive(Clone, Debug)]
pub struct CustomEvent {
    /// Base event.
    pub base: Event,
    /// Custom detail.
    pub detail: Option<serde_json::Value>,
}

impl CustomEvent {
    pub fn new(event_type: &str, detail: Option<serde_json::Value>) -> Self {
        Self {
            base: Event::new(event_type),
            detail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new("click");
        assert_eq!(event.event_type, "click");
        assert!(!event.bubbles);
        assert!(!event.cancelable);
        assert!(!event.default_prevented);
    }

    #[test]
    fn test_event_prevent_default() {
        let mut event = Event::with_options(
            "click",
            EventInit {
                cancelable: true,
                ..Default::default()
            },
        );

        event.prevent_default();
        assert!(event.default_prevented);
    }

    #[test]
    fn test_event_stop_propagation() {
        let mut event = Event::new("click");
        assert!(!event.propagation_stopped);

        event.stop_propagation();
        assert!(event.propagation_stopped);
        assert!(!event.immediate_propagation_stopped);

        let mut event2 = Event::new("click");
        event2.stop_immediate_propagation();
        assert!(event2.propagation_stopped);
        assert!(event2.immediate_propagation_stopped);
    }

    #[test]
    fn test_mouse_event() {
        let event = MouseEvent::click(100, 200);
        assert_eq!(event.base.event_type, "click");
        assert_eq!(event.client_x, 100);
        assert_eq!(event.client_y, 200);
        assert!(event.base.bubbles);
    }

    #[test]
    fn test_keyboard_event() {
        let mut event = KeyboardEvent::new("keydown", "a", "KeyA");
        event.ctrl_key = true;

        assert_eq!(event.key, "a");
        assert_eq!(event.code, "KeyA");
        assert!(event.get_modifier_state("Control"));
        assert!(!event.get_modifier_state("Shift"));
    }
}
