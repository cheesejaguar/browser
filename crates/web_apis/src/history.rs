//! History API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::sync::Arc;
use parking_lot::RwLock;

/// History API implementation.
pub struct History {
    /// History entries.
    entries: Vec<HistoryEntry>,
    /// Current index in history.
    current_index: usize,
    /// Maximum history length.
    max_length: usize,
}

impl History {
    /// Create a new History instance.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            max_length: 50, // Typical browser limit
        }
    }

    /// Get the number of entries.
    pub fn length(&self) -> usize {
        self.entries.len()
    }

    /// Get the current scroll restoration mode.
    pub fn scroll_restoration(&self) -> ScrollRestoration {
        self.entries
            .get(self.current_index)
            .map(|e| e.scroll_restoration)
            .unwrap_or(ScrollRestoration::Auto)
    }

    /// Set the scroll restoration mode.
    pub fn set_scroll_restoration(&mut self, mode: ScrollRestoration) {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            entry.scroll_restoration = mode;
        }
    }

    /// Get the current state.
    pub fn state(&self) -> Option<&serde_json::Value> {
        self.entries
            .get(self.current_index)
            .and_then(|e| e.state.as_ref())
    }

    /// Navigate back in history.
    pub fn back(&mut self) -> Option<&HistoryEntry> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }

    /// Navigate forward in history.
    pub fn forward(&mut self) -> Option<&HistoryEntry> {
        if self.current_index + 1 < self.entries.len() {
            self.current_index += 1;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }

    /// Navigate to a specific position.
    pub fn go(&mut self, delta: i32) -> Option<&HistoryEntry> {
        let new_index = if delta >= 0 {
            self.current_index.saturating_add(delta as usize)
        } else {
            self.current_index.saturating_sub((-delta) as usize)
        };

        if new_index < self.entries.len() {
            self.current_index = new_index;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }

    /// Push a new state onto the history stack.
    pub fn push_state(
        &mut self,
        state: Option<serde_json::Value>,
        title: String,
        url: Option<String>,
    ) {
        // Truncate any forward history
        self.entries.truncate(self.current_index + 1);

        // Create new entry
        let entry = HistoryEntry {
            state,
            title,
            url: url.unwrap_or_default(),
            scroll_restoration: ScrollRestoration::Auto,
        };

        self.entries.push(entry);
        self.current_index = self.entries.len() - 1;

        // Enforce max length
        if self.entries.len() > self.max_length {
            self.entries.remove(0);
            self.current_index = self.current_index.saturating_sub(1);
        }
    }

    /// Replace the current state.
    pub fn replace_state(
        &mut self,
        state: Option<serde_json::Value>,
        title: String,
        url: Option<String>,
    ) {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            entry.state = state;
            entry.title = title;
            if let Some(url) = url {
                entry.url = url;
            }
        }
    }

    /// Get the current entry.
    pub fn current(&self) -> Option<&HistoryEntry> {
        self.entries.get(self.current_index)
    }

    /// Register the History API on the global object.
    pub fn register(history: Arc<RwLock<History>>, context: &mut Context) {
        let history_obj = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(history_back), js_string!("back"), 0)
            .function(NativeFunction::from_fn_ptr(history_forward), js_string!("forward"), 0)
            .function(NativeFunction::from_fn_ptr(history_go), js_string!("go"), 1)
            .function(NativeFunction::from_fn_ptr(history_push_state), js_string!("pushState"), 3)
            .function(NativeFunction::from_fn_ptr(history_replace_state), js_string!("replaceState"), 3)
            .build();

        context
            .register_global_property(js_string!("history"), history_obj, Attribute::all())
            .expect("Failed to register history");
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

/// A single history entry.
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    /// State object.
    pub state: Option<serde_json::Value>,
    /// Page title.
    pub title: String,
    /// URL.
    pub url: String,
    /// Scroll restoration mode.
    pub scroll_restoration: ScrollRestoration,
}

/// Scroll restoration mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollRestoration {
    /// Automatic scroll restoration.
    Auto,
    /// Manual scroll restoration.
    Manual,
}

// Native function implementations
fn history_back(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn history_forward(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn history_go(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _delta = args.get_or_undefined(0).to_i32(context)?;
    Ok(JsValue::undefined())
}

fn history_push_state(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _state = args.get_or_undefined(0);
    let _title = args.get_or_undefined(1).to_string(context)?;
    let _url = args.get(2);
    Ok(JsValue::undefined())
}

fn history_replace_state(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _state = args.get_or_undefined(0);
    let _title = args.get_or_undefined(1).to_string(context)?;
    let _url = args.get(2);
    Ok(JsValue::undefined())
}

/// PopState event data.
#[derive(Clone, Debug)]
pub struct PopStateEvent {
    /// The state object.
    pub state: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_creation() {
        let history = History::new();
        assert_eq!(history.length(), 0);
    }

    #[test]
    fn test_history_push_state() {
        let mut history = History::new();

        history.push_state(None, "Page 1".to_string(), Some("/page1".to_string()));
        assert_eq!(history.length(), 1);
        assert_eq!(history.current().unwrap().url, "/page1");

        history.push_state(None, "Page 2".to_string(), Some("/page2".to_string()));
        assert_eq!(history.length(), 2);
        assert_eq!(history.current().unwrap().url, "/page2");
    }

    #[test]
    fn test_history_navigation() {
        let mut history = History::new();

        history.push_state(None, "Page 1".to_string(), Some("/page1".to_string()));
        history.push_state(None, "Page 2".to_string(), Some("/page2".to_string()));
        history.push_state(None, "Page 3".to_string(), Some("/page3".to_string()));

        history.back();
        assert_eq!(history.current().unwrap().url, "/page2");

        history.back();
        assert_eq!(history.current().unwrap().url, "/page1");

        history.forward();
        assert_eq!(history.current().unwrap().url, "/page2");

        history.go(1);
        assert_eq!(history.current().unwrap().url, "/page3");

        history.go(-2);
        assert_eq!(history.current().unwrap().url, "/page1");
    }

    #[test]
    fn test_history_replace_state() {
        let mut history = History::new();

        history.push_state(None, "Page 1".to_string(), Some("/page1".to_string()));
        history.replace_state(None, "Updated Page 1".to_string(), Some("/updated-page1".to_string()));

        assert_eq!(history.length(), 1);
        assert_eq!(history.current().unwrap().url, "/updated-page1");
        assert_eq!(history.current().unwrap().title, "Updated Page 1");
    }

    #[test]
    fn test_history_truncation() {
        let mut history = History::new();

        history.push_state(None, "1".to_string(), Some("/1".to_string()));
        history.push_state(None, "2".to_string(), Some("/2".to_string()));
        history.push_state(None, "3".to_string(), Some("/3".to_string()));

        // Go back and push new state
        history.go(-2);
        history.push_state(None, "new".to_string(), Some("/new".to_string()));

        // Should have truncated forward history
        assert_eq!(history.length(), 2);
        assert_eq!(history.current().unwrap().url, "/new");
    }
}
