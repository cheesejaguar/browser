//! DOM Window object implementation.

use crate::document::DocumentRef;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

/// Browser window object.
pub struct Window {
    /// Associated document.
    pub document: Option<DocumentRef>,
    /// Window name.
    pub name: String,
    /// Window location.
    pub location: Location,
    /// Window history.
    pub history: History,
    /// Navigator.
    pub navigator: Navigator,
    /// Screen info.
    pub screen: Screen,
    /// Inner dimensions.
    pub inner_width: u32,
    pub inner_height: u32,
    /// Outer dimensions.
    pub outer_width: u32,
    pub outer_height: u32,
    /// Scroll position.
    pub scroll_x: f64,
    pub scroll_y: f64,
    /// Device pixel ratio.
    pub device_pixel_ratio: f64,
    /// Storage.
    pub local_storage: Storage,
    pub session_storage: Storage,
    /// Timers.
    timers: HashMap<u32, Timer>,
    next_timer_id: u32,
    /// Animation frames.
    animation_frames: HashMap<u32, AnimationFrameCallback>,
    next_frame_id: u32,
    /// Opener window.
    pub opener: Option<Arc<RwLock<Window>>>,
    /// Parent window (for frames).
    pub parent: Option<Arc<RwLock<Window>>>,
    /// Top window.
    pub top: Option<Arc<RwLock<Window>>>,
    /// Closed flag.
    pub closed: bool,
}

impl Window {
    pub fn new() -> Self {
        Self {
            document: None,
            name: String::new(),
            location: Location::new(),
            history: History::new(),
            navigator: Navigator::new(),
            screen: Screen::new(),
            inner_width: 1920,
            inner_height: 1080,
            outer_width: 1920,
            outer_height: 1080,
            scroll_x: 0.0,
            scroll_y: 0.0,
            device_pixel_ratio: 1.0,
            local_storage: Storage::new(),
            session_storage: Storage::new(),
            timers: HashMap::new(),
            next_timer_id: 1,
            animation_frames: HashMap::new(),
            next_frame_id: 1,
            opener: None,
            parent: None,
            top: None,
            closed: false,
        }
    }

    /// Set timeout.
    pub fn set_timeout(&mut self, callback: TimerCallback, delay: u32) -> u32 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;

        let timer = Timer {
            callback,
            delay: Duration::from_millis(delay as u64),
            repeat: false,
            scheduled: Instant::now(),
        };

        self.timers.insert(id, timer);
        id
    }

    /// Clear timeout.
    pub fn clear_timeout(&mut self, id: u32) {
        self.timers.remove(&id);
    }

    /// Set interval.
    pub fn set_interval(&mut self, callback: TimerCallback, delay: u32) -> u32 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;

        let timer = Timer {
            callback,
            delay: Duration::from_millis(delay as u64),
            repeat: true,
            scheduled: Instant::now(),
        };

        self.timers.insert(id, timer);
        id
    }

    /// Clear interval.
    pub fn clear_interval(&mut self, id: u32) {
        self.timers.remove(&id);
    }

    /// Request animation frame.
    pub fn request_animation_frame(&mut self, callback: AnimationFrameCallback) -> u32 {
        let id = self.next_frame_id;
        self.next_frame_id += 1;
        self.animation_frames.insert(id, callback);
        id
    }

    /// Cancel animation frame.
    pub fn cancel_animation_frame(&mut self, id: u32) {
        self.animation_frames.remove(&id);
    }

    /// Process timers (called by event loop).
    pub fn process_timers(&mut self) -> Vec<TimerCallback> {
        let now = Instant::now();
        let mut callbacks = Vec::new();
        let mut to_remove = Vec::new();

        for (&id, timer) in &self.timers {
            if now.duration_since(timer.scheduled) >= timer.delay {
                callbacks.push(timer.callback.clone());
                if !timer.repeat {
                    to_remove.push(id);
                }
            }
        }

        for id in to_remove {
            self.timers.remove(&id);
        }

        // Reset repeat timers
        for timer in self.timers.values_mut() {
            if timer.repeat {
                timer.scheduled = now;
            }
        }

        callbacks
    }

    /// Process animation frames.
    pub fn process_animation_frames(&mut self, timestamp: f64) -> Vec<AnimationFrameCallback> {
        let callbacks: Vec<_> = self.animation_frames.drain().map(|(_, cb)| cb).collect();
        callbacks
    }

    /// Scroll to position.
    pub fn scroll_to(&mut self, x: f64, y: f64) {
        self.scroll_x = x.max(0.0);
        self.scroll_y = y.max(0.0);
    }

    /// Scroll by amount.
    pub fn scroll_by(&mut self, dx: f64, dy: f64) {
        self.scroll_to(self.scroll_x + dx, self.scroll_y + dy);
    }

    /// Resize window.
    pub fn resize_to(&mut self, width: u32, height: u32) {
        self.outer_width = width;
        self.outer_height = height;
        // Inner size would be outer minus chrome
        self.inner_width = width;
        self.inner_height = height;
    }

    /// Move window.
    pub fn move_to(&mut self, _x: i32, _y: i32) {
        // Platform-specific implementation
    }

    /// Focus window.
    pub fn focus(&mut self) {
        // Platform-specific implementation
    }

    /// Blur window.
    pub fn blur(&mut self) {
        // Platform-specific implementation
    }

    /// Close window.
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Alert dialog.
    pub fn alert(&self, message: &str) {
        // Would show native dialog
        eprintln!("Alert: {}", message);
    }

    /// Confirm dialog.
    pub fn confirm(&self, message: &str) -> bool {
        eprintln!("Confirm: {}", message);
        false
    }

    /// Prompt dialog.
    pub fn prompt(&self, message: &str, default: &str) -> Option<String> {
        eprintln!("Prompt: {} (default: {})", message, default);
        None
    }

    /// Get computed style.
    pub fn get_computed_style(&self, _element_id: crate::node::NodeId) -> ComputedStyle {
        ComputedStyle::default()
    }

    /// Match media query.
    pub fn match_media(&self, query: &str) -> MediaQueryList {
        MediaQueryList {
            media: query.to_string(),
            matches: self.evaluate_media_query(query),
        }
    }

    fn evaluate_media_query(&self, query: &str) -> bool {
        // Basic media query evaluation
        if query.contains("prefers-color-scheme: dark") {
            return false; // Assume light mode
        }
        if query.contains("prefers-color-scheme: light") {
            return true;
        }
        if query.contains("prefers-reduced-motion") {
            return false;
        }

        // Width queries
        if let Some(captures) = query.find("min-width:") {
            // Basic parsing - real implementation would be more robust
            return true;
        }

        true
    }

    /// Open new window.
    pub fn open(&self, url: &str, target: &str, features: &str) -> Option<Arc<RwLock<Window>>> {
        // Would create new window/tab
        None
    }

    /// Post message to window.
    pub fn post_message(&self, _message: &str, _target_origin: &str) {
        // Would post message via event
    }

    /// Print window.
    pub fn print(&self) {
        // Would trigger print dialog
    }

    /// Get selection.
    pub fn get_selection(&self) -> Option<Selection> {
        None
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer callback type.
pub type TimerCallback = Arc<dyn Fn() + Send + Sync>;

/// Animation frame callback type.
pub type AnimationFrameCallback = Arc<dyn Fn(f64) + Send + Sync>;

/// Timer data.
struct Timer {
    callback: TimerCallback,
    delay: Duration,
    repeat: bool,
    scheduled: Instant,
}

/// Window location.
#[derive(Clone, Debug, Default)]
pub struct Location {
    pub href: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub origin: String,
}

impl Location {
    pub fn new() -> Self {
        Self::from_url(&Url::parse("about:blank").unwrap())
    }

    pub fn from_url(url: &Url) -> Self {
        Self {
            href: url.to_string(),
            protocol: format!("{}:", url.scheme()),
            host: url.host_str().unwrap_or("").to_string()
                + url.port().map(|p| format!(":{}", p)).as_deref().unwrap_or(""),
            hostname: url.host_str().unwrap_or("").to_string(),
            port: url.port().map(|p| p.to_string()).unwrap_or_default(),
            pathname: url.path().to_string(),
            search: url.query().map(|q| format!("?{}", q)).unwrap_or_default(),
            hash: url.fragment().map(|f| format!("#{}", f)).unwrap_or_default(),
            origin: url.origin().ascii_serialization(),
        }
    }

    pub fn assign(&mut self, url: &str) {
        if let Ok(parsed) = Url::parse(url) {
            *self = Self::from_url(&parsed);
        }
    }

    pub fn replace(&mut self, url: &str) {
        self.assign(url);
    }

    pub fn reload(&self) {
        // Would reload page
    }
}

/// Browser history.
#[derive(Clone, Debug, Default)]
pub struct History {
    entries: Vec<HistoryEntry>,
    current: usize,
    pub length: usize,
}

#[derive(Clone, Debug)]
struct HistoryEntry {
    url: String,
    title: String,
    state: Option<String>,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: vec![HistoryEntry {
                url: "about:blank".to_string(),
                title: String::new(),
                state: None,
            }],
            current: 0,
            length: 1,
        }
    }

    pub fn go(&mut self, delta: i32) {
        let new_idx = self.current as i32 + delta;
        if new_idx >= 0 && (new_idx as usize) < self.entries.len() {
            self.current = new_idx as usize;
        }
    }

    pub fn back(&mut self) {
        self.go(-1);
    }

    pub fn forward(&mut self) {
        self.go(1);
    }

    pub fn push_state(&mut self, state: Option<String>, title: &str, url: &str) {
        // Remove forward entries
        self.entries.truncate(self.current + 1);

        self.entries.push(HistoryEntry {
            url: url.to_string(),
            title: title.to_string(),
            state,
        });

        self.current = self.entries.len() - 1;
        self.length = self.entries.len();
    }

    pub fn replace_state(&mut self, state: Option<String>, title: &str, url: &str) {
        if let Some(entry) = self.entries.get_mut(self.current) {
            entry.url = url.to_string();
            entry.title = title.to_string();
            entry.state = state;
        }
    }

    pub fn state(&self) -> Option<&str> {
        self.entries.get(self.current).and_then(|e| e.state.as_deref())
    }
}

/// Navigator object.
#[derive(Clone, Debug)]
pub struct Navigator {
    pub user_agent: String,
    pub platform: String,
    pub language: String,
    pub languages: Vec<String>,
    pub online: bool,
    pub cookie_enabled: bool,
    pub hardware_concurrency: usize,
    pub max_touch_points: u32,
    pub vendor: String,
}

impl Navigator {
    pub fn new() -> Self {
        Self {
            user_agent: "OxideBrowser/1.0".to_string(),
            platform: std::env::consts::OS.to_string(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            online: true,
            cookie_enabled: true,
            hardware_concurrency: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4),
            max_touch_points: 0,
            vendor: "Oxide".to_string(),
        }
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen object.
#[derive(Clone, Debug)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub avail_width: u32,
    pub avail_height: u32,
    pub color_depth: u32,
    pub pixel_depth: u32,
    pub orientation: ScreenOrientation,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            avail_width: 1920,
            avail_height: 1040, // Minus taskbar
            color_depth: 24,
            pixel_depth: 24,
            orientation: ScreenOrientation::default(),
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen orientation.
#[derive(Clone, Debug)]
pub struct ScreenOrientation {
    pub angle: u16,
    pub orientation_type: String,
}

impl Default for ScreenOrientation {
    fn default() -> Self {
        Self {
            angle: 0,
            orientation_type: "landscape-primary".to_string(),
        }
    }
}

/// Web storage.
#[derive(Clone, Debug, Default)]
pub struct Storage {
    data: HashMap<String, String>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn length(&self) -> usize {
        self.data.len()
    }

    pub fn key(&self, index: usize) -> Option<&str> {
        self.data.keys().nth(index).map(|s| s.as_str())
    }

    pub fn get_item(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set_item(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn remove_item(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// Computed style (placeholder).
#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    values: HashMap<String, String>,
}

impl ComputedStyle {
    pub fn get_property_value(&self, property: &str) -> String {
        self.values.get(property).cloned().unwrap_or_default()
    }
}

/// Media query list.
#[derive(Clone, Debug)]
pub struct MediaQueryList {
    pub media: String,
    pub matches: bool,
}

/// Text selection.
#[derive(Clone, Debug)]
pub struct Selection {
    pub anchor_node: Option<crate::node::NodeId>,
    pub anchor_offset: usize,
    pub focus_node: Option<crate::node::NodeId>,
    pub focus_offset: usize,
    pub is_collapsed: bool,
    pub range_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let window = Window::new();
        assert!(!window.closed);
        assert_eq!(window.inner_width, 1920);
    }

    #[test]
    fn test_location() {
        let loc = Location::from_url(&Url::parse("https://example.com:8080/path?query#hash").unwrap());
        assert_eq!(loc.protocol, "https:");
        assert_eq!(loc.hostname, "example.com");
        assert_eq!(loc.port, "8080");
        assert_eq!(loc.pathname, "/path");
        assert_eq!(loc.search, "?query");
        assert_eq!(loc.hash, "#hash");
    }

    #[test]
    fn test_storage() {
        let mut storage = Storage::new();
        storage.set_item("key", "value");
        assert_eq!(storage.get_item("key"), Some("value"));
        storage.remove_item("key");
        assert_eq!(storage.get_item("key"), None);
    }

    #[test]
    fn test_history() {
        let mut history = History::new();
        assert_eq!(history.length, 1);

        history.push_state(None, "Page 2", "/page2");
        assert_eq!(history.length, 2);

        history.back();
        assert_eq!(history.current, 0);

        history.forward();
        assert_eq!(history.current, 1);
    }
}
