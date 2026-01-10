//! DOM Events implementation.

use crate::node::NodeId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Event type enumeration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    // Mouse events
    Click,
    DblClick,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseOver,
    MouseOut,
    MouseEnter,
    MouseLeave,
    ContextMenu,
    Wheel,

    // Keyboard events
    KeyDown,
    KeyUp,
    KeyPress,

    // Focus events
    Focus,
    Blur,
    FocusIn,
    FocusOut,

    // Form events
    Submit,
    Reset,
    Change,
    Input,
    Invalid,

    // Document/Window events
    Load,
    Unload,
    BeforeUnload,
    DOMContentLoaded,
    ReadyStateChange,
    Resize,
    Scroll,
    Error,

    // Touch events
    TouchStart,
    TouchEnd,
    TouchMove,
    TouchCancel,

    // Drag events
    DragStart,
    Drag,
    DragEnd,
    DragEnter,
    DragOver,
    DragLeave,
    Drop,

    // Animation events
    AnimationStart,
    AnimationEnd,
    AnimationIteration,
    TransitionEnd,

    // Other
    Custom(String),
}

impl EventType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "click" => EventType::Click,
            "dblclick" => EventType::DblClick,
            "mousedown" => EventType::MouseDown,
            "mouseup" => EventType::MouseUp,
            "mousemove" => EventType::MouseMove,
            "mouseover" => EventType::MouseOver,
            "mouseout" => EventType::MouseOut,
            "mouseenter" => EventType::MouseEnter,
            "mouseleave" => EventType::MouseLeave,
            "contextmenu" => EventType::ContextMenu,
            "wheel" => EventType::Wheel,
            "keydown" => EventType::KeyDown,
            "keyup" => EventType::KeyUp,
            "keypress" => EventType::KeyPress,
            "focus" => EventType::Focus,
            "blur" => EventType::Blur,
            "focusin" => EventType::FocusIn,
            "focusout" => EventType::FocusOut,
            "submit" => EventType::Submit,
            "reset" => EventType::Reset,
            "change" => EventType::Change,
            "input" => EventType::Input,
            "invalid" => EventType::Invalid,
            "load" => EventType::Load,
            "unload" => EventType::Unload,
            "beforeunload" => EventType::BeforeUnload,
            "domcontentloaded" => EventType::DOMContentLoaded,
            "readystatechange" => EventType::ReadyStateChange,
            "resize" => EventType::Resize,
            "scroll" => EventType::Scroll,
            "error" => EventType::Error,
            "touchstart" => EventType::TouchStart,
            "touchend" => EventType::TouchEnd,
            "touchmove" => EventType::TouchMove,
            "touchcancel" => EventType::TouchCancel,
            "dragstart" => EventType::DragStart,
            "drag" => EventType::Drag,
            "dragend" => EventType::DragEnd,
            "dragenter" => EventType::DragEnter,
            "dragover" => EventType::DragOver,
            "dragleave" => EventType::DragLeave,
            "drop" => EventType::Drop,
            "animationstart" => EventType::AnimationStart,
            "animationend" => EventType::AnimationEnd,
            "animationiteration" => EventType::AnimationIteration,
            "transitionend" => EventType::TransitionEnd,
            other => EventType::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EventType::Click => "click",
            EventType::DblClick => "dblclick",
            EventType::MouseDown => "mousedown",
            EventType::MouseUp => "mouseup",
            EventType::MouseMove => "mousemove",
            EventType::MouseOver => "mouseover",
            EventType::MouseOut => "mouseout",
            EventType::MouseEnter => "mouseenter",
            EventType::MouseLeave => "mouseleave",
            EventType::ContextMenu => "contextmenu",
            EventType::Wheel => "wheel",
            EventType::KeyDown => "keydown",
            EventType::KeyUp => "keyup",
            EventType::KeyPress => "keypress",
            EventType::Focus => "focus",
            EventType::Blur => "blur",
            EventType::FocusIn => "focusin",
            EventType::FocusOut => "focusout",
            EventType::Submit => "submit",
            EventType::Reset => "reset",
            EventType::Change => "change",
            EventType::Input => "input",
            EventType::Invalid => "invalid",
            EventType::Load => "load",
            EventType::Unload => "unload",
            EventType::BeforeUnload => "beforeunload",
            EventType::DOMContentLoaded => "DOMContentLoaded",
            EventType::ReadyStateChange => "readystatechange",
            EventType::Resize => "resize",
            EventType::Scroll => "scroll",
            EventType::Error => "error",
            EventType::TouchStart => "touchstart",
            EventType::TouchEnd => "touchend",
            EventType::TouchMove => "touchmove",
            EventType::TouchCancel => "touchcancel",
            EventType::DragStart => "dragstart",
            EventType::Drag => "drag",
            EventType::DragEnd => "dragend",
            EventType::DragEnter => "dragenter",
            EventType::DragOver => "dragover",
            EventType::DragLeave => "dragleave",
            EventType::Drop => "drop",
            EventType::AnimationStart => "animationstart",
            EventType::AnimationEnd => "animationend",
            EventType::AnimationIteration => "animationiteration",
            EventType::TransitionEnd => "transitionend",
            EventType::Custom(s) => s,
        }
    }

    /// Check if event bubbles by default.
    pub fn bubbles(&self) -> bool {
        match self {
            EventType::Focus
            | EventType::Blur
            | EventType::Load
            | EventType::Unload
            | EventType::MouseEnter
            | EventType::MouseLeave => false,
            _ => true,
        }
    }

    /// Check if event is cancelable by default.
    pub fn cancelable(&self) -> bool {
        match self {
            EventType::Load
            | EventType::Unload
            | EventType::Error
            | EventType::Resize
            | EventType::Scroll
            | EventType::Focus
            | EventType::Blur
            | EventType::MouseEnter
            | EventType::MouseLeave => false,
            _ => true,
        }
    }
}

/// Event phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

/// DOM Event.
#[derive(Clone, Debug)]
pub struct Event {
    /// Event type.
    pub event_type: EventType,
    /// Target element.
    pub target: Option<NodeId>,
    /// Current target during propagation.
    pub current_target: Option<NodeId>,
    /// Event phase.
    pub phase: EventPhase,
    /// Whether event bubbles.
    pub bubbles: bool,
    /// Whether event is cancelable.
    pub cancelable: bool,
    /// Whether default was prevented.
    pub default_prevented: bool,
    /// Whether propagation was stopped.
    pub propagation_stopped: bool,
    /// Whether immediate propagation was stopped.
    pub immediate_propagation_stopped: bool,
    /// Whether event is composed (crosses shadow DOM).
    pub composed: bool,
    /// Whether event is trusted (browser-generated).
    pub is_trusted: bool,
    /// Timestamp.
    pub timestamp: f64,
}

impl Event {
    pub fn new(event_type: EventType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64() * 1000.0;

        let bubbles = event_type.bubbles();
        let cancelable = event_type.cancelable();

        Self {
            event_type,
            target: None,
            current_target: None,
            phase: EventPhase::None,
            bubbles,
            cancelable,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            composed: false,
            is_trusted: false,
            timestamp,
        }
    }

    pub fn with_options(event_type: EventType, bubbles: bool, cancelable: bool) -> Self {
        let mut event = Self::new(event_type);
        event.bubbles = bubbles;
        event.cancelable = cancelable;
        event
    }

    /// Prevent default action.
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
        self.immediate_propagation_stopped = true;
        self.propagation_stopped = true;
    }

    /// Compose path (for shadow DOM).
    pub fn composed_path(&self) -> Vec<NodeId> {
        // TODO: Implement shadow DOM path
        match self.target {
            Some(target) => vec![target],
            None => Vec::new(),
        }
    }
}

/// Mouse event data.
#[derive(Clone, Debug, Default)]
pub struct MouseEvent {
    pub base: Event,
    pub screen_x: i32,
    pub screen_y: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub page_x: i32,
    pub page_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub movement_x: i32,
    pub movement_y: i32,
    pub button: i16,
    pub buttons: u16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub related_target: Option<NodeId>,
}

impl MouseEvent {
    pub fn new(event_type: EventType) -> Self {
        Self {
            base: Event::new(event_type),
            ..Default::default()
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new(EventType::Custom(String::new()))
    }
}

/// Keyboard event data.
#[derive(Clone, Debug)]
pub struct KeyboardEvent {
    pub base: Event,
    pub key: String,
    pub code: String,
    pub location: u32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub repeat: bool,
    pub is_composing: bool,
}

impl KeyboardEvent {
    pub fn new(event_type: EventType, key: &str, code: &str) -> Self {
        Self {
            base: Event::new(event_type),
            key: key.to_string(),
            code: code.to_string(),
            location: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            repeat: false,
            is_composing: false,
        }
    }

    /// Get character code.
    pub fn char_code(&self) -> u32 {
        self.key.chars().next().map(|c| c as u32).unwrap_or(0)
    }

    /// Get key code.
    pub fn key_code(&self) -> u32 {
        // Simplified - real implementation maps key codes
        self.char_code()
    }
}

/// Focus event data.
#[derive(Clone, Debug)]
pub struct FocusEvent {
    pub base: Event,
    pub related_target: Option<NodeId>,
}

impl FocusEvent {
    pub fn new(event_type: EventType) -> Self {
        Self {
            base: Event::new(event_type),
            related_target: None,
        }
    }
}

/// Wheel event data.
#[derive(Clone, Debug)]
pub struct WheelEvent {
    pub base: MouseEvent,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: u32,
}

impl WheelEvent {
    pub const DOM_DELTA_PIXEL: u32 = 0;
    pub const DOM_DELTA_LINE: u32 = 1;
    pub const DOM_DELTA_PAGE: u32 = 2;

    pub fn new() -> Self {
        Self {
            base: MouseEvent::new(EventType::Wheel),
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
            delta_mode: Self::DOM_DELTA_PIXEL,
        }
    }
}

impl Default for WheelEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// Event listener callback type.
pub type EventCallback = Arc<dyn Fn(&mut Event) + Send + Sync>;

/// Event listener options.
#[derive(Clone, Debug, Default)]
pub struct EventListenerOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
}

/// Event listener.
#[derive(Clone)]
pub struct EventListener {
    pub callback: EventCallback,
    pub options: EventListenerOptions,
}

/// Event target trait.
pub trait EventTarget {
    /// Add an event listener.
    fn add_event_listener(
        &mut self,
        event_type: &str,
        callback: EventCallback,
        options: EventListenerOptions,
    );

    /// Remove an event listener.
    fn remove_event_listener(&mut self, event_type: &str, callback: EventCallback);

    /// Dispatch an event.
    fn dispatch_event(&mut self, event: &mut Event) -> bool;
}

/// Event manager for handling event dispatch.
pub struct EventManager {
    /// Listeners by node and event type.
    listeners: HashMap<NodeId, HashMap<String, Vec<EventListener>>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// Add event listener for a node.
    pub fn add_listener(
        &mut self,
        node: NodeId,
        event_type: &str,
        callback: EventCallback,
        options: EventListenerOptions,
    ) {
        let node_listeners = self.listeners.entry(node).or_default();
        let type_listeners = node_listeners.entry(event_type.to_string()).or_default();
        type_listeners.push(EventListener { callback, options });
    }

    /// Remove event listener for a node.
    pub fn remove_listener(&mut self, node: NodeId, event_type: &str, _callback: EventCallback) {
        if let Some(node_listeners) = self.listeners.get_mut(&node) {
            if let Some(type_listeners) = node_listeners.get_mut(event_type) {
                // In real implementation, would compare callbacks
                type_listeners.clear();
            }
        }
    }

    /// Get listeners for a node and event type.
    pub fn get_listeners(&self, node: NodeId, event_type: &str) -> Vec<&EventListener> {
        self.listeners
            .get(&node)
            .and_then(|n| n.get(event_type))
            .map(|l| l.iter().collect())
            .unwrap_or_default()
    }

    /// Dispatch event to target.
    pub fn dispatch(&mut self, target: NodeId, event: &mut Event, path: &[NodeId]) -> bool {
        event.target = Some(target);
        event.is_trusted = true;

        // Capture phase
        event.phase = EventPhase::Capturing;
        for &node in path.iter().rev().skip(1) {
            event.current_target = Some(node);
            self.invoke_listeners(node, event, true);
            if event.propagation_stopped {
                return !event.default_prevented;
            }
        }

        // Target phase
        event.phase = EventPhase::AtTarget;
        event.current_target = Some(target);
        self.invoke_listeners(target, event, false);
        if event.propagation_stopped {
            return !event.default_prevented;
        }

        // Bubble phase
        if event.bubbles {
            event.phase = EventPhase::Bubbling;
            for &node in path.iter().skip(1) {
                event.current_target = Some(node);
                self.invoke_listeners(node, event, false);
                if event.propagation_stopped {
                    return !event.default_prevented;
                }
            }
        }

        event.phase = EventPhase::None;
        !event.default_prevented
    }

    fn invoke_listeners(&mut self, node: NodeId, event: &mut Event, capture: bool) {
        let event_type = event.event_type.as_str().to_string();

        if let Some(node_listeners) = self.listeners.get(&node) {
            if let Some(type_listeners) = node_listeners.get(&event_type) {
                for listener in type_listeners.iter() {
                    if listener.options.capture == capture || event.phase == EventPhase::AtTarget {
                        (listener.callback)(event);

                        if event.immediate_propagation_stopped {
                            break;
                        }
                    }
                }
            }
        }

        // Handle 'once' listeners
        if let Some(node_listeners) = self.listeners.get_mut(&node) {
            if let Some(type_listeners) = node_listeners.get_mut(&event_type) {
                type_listeners.retain(|l| !l.options.once);
            }
        }
    }

    /// Remove all listeners for a node.
    pub fn remove_all(&mut self, node: NodeId) {
        self.listeners.remove(&node);
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(EventType::Click);
        assert!(event.bubbles);
        assert!(event.cancelable);
        assert!(!event.default_prevented);
    }

    #[test]
    fn test_prevent_default() {
        let mut event = Event::new(EventType::Click);
        event.prevent_default();
        assert!(event.default_prevented);

        let mut uncancelable = Event::new(EventType::Load);
        uncancelable.prevent_default();
        assert!(!uncancelable.default_prevented);
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(EventType::from_str("click"), EventType::Click);
        assert_eq!(EventType::from_str("KEYDOWN"), EventType::KeyDown);
        assert_eq!(
            EventType::from_str("custom-event"),
            EventType::Custom("custom-event".to_string())
        );
    }
}
