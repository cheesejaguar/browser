//! Browser tab.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Tab identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

/// Browser tab.
pub struct Tab {
    /// Tab ID.
    id: TabId,
    /// Current URL.
    url: String,
    /// Page title.
    title: String,
    /// Favicon URL.
    favicon: Option<String>,
    /// Loading state.
    loading: bool,
    /// Load progress (0.0 - 1.0).
    progress: f32,
    /// Navigation history.
    history: NavigationHistory,
    /// Zoom level.
    zoom: f32,
    /// Is pinned.
    pinned: bool,
    /// Is muted.
    muted: bool,
    /// Is playing audio.
    playing_audio: bool,
    /// Security state.
    security: SecurityState,
    /// Last active time.
    last_active: Instant,
    /// Creation time.
    created: Instant,
}

impl Tab {
    /// Create a new tab.
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            url: "about:blank".to_string(),
            title: "New Tab".to_string(),
            favicon: None,
            loading: false,
            progress: 0.0,
            history: NavigationHistory::new(),
            zoom: 1.0,
            pinned: false,
            muted: false,
            playing_audio: false,
            security: SecurityState::None,
            last_active: Instant::now(),
            created: Instant::now(),
        }
    }

    /// Get the tab ID.
    pub fn id(&self) -> TabId {
        self.id
    }

    /// Get the current URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the page title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the page title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the favicon URL.
    pub fn favicon(&self) -> Option<&str> {
        self.favicon.as_deref()
    }

    /// Set the favicon URL.
    pub fn set_favicon(&mut self, favicon: Option<String>) {
        self.favicon = favicon;
    }

    /// Check if loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get load progress.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Set load progress.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Navigate to a URL.
    pub fn navigate(&mut self, url: &str) {
        self.url = url.to_string();
        self.loading = true;
        self.progress = 0.0;
        self.history.push(url.to_string());
        self.last_active = Instant::now();
    }

    /// Stop loading.
    pub fn stop(&mut self) {
        self.loading = false;
    }

    /// Reload the page.
    pub fn reload(&mut self) {
        self.loading = true;
        self.progress = 0.0;
    }

    /// Reload bypassing cache.
    pub fn hard_reload(&mut self) {
        self.reload();
        // Would also clear cached resources for this page
    }

    /// Go back in history.
    pub fn go_back(&mut self) -> bool {
        if let Some(url) = self.history.go_back() {
            self.url = url.clone();
            self.loading = true;
            self.progress = 0.0;
            true
        } else {
            false
        }
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) -> bool {
        if let Some(url) = self.history.go_forward() {
            self.url = url.clone();
            self.loading = true;
            self.progress = 0.0;
            true
        } else {
            false
        }
    }

    /// Check if can go back.
    pub fn can_go_back(&self) -> bool {
        self.history.can_go_back()
    }

    /// Check if can go forward.
    pub fn can_go_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    /// Get the zoom level.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Set the zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 5.0);
    }

    /// Zoom in.
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * 1.1);
    }

    /// Zoom out.
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / 1.1);
    }

    /// Reset zoom.
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }

    /// Check if pinned.
    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    /// Set pinned state.
    pub fn set_pinned(&mut self, pinned: bool) {
        self.pinned = pinned;
    }

    /// Check if muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Set muted state.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Check if playing audio.
    pub fn is_playing_audio(&self) -> bool {
        self.playing_audio
    }

    /// Set playing audio state.
    pub fn set_playing_audio(&mut self, playing: bool) {
        self.playing_audio = playing;
    }

    /// Get security state.
    pub fn security(&self) -> SecurityState {
        self.security
    }

    /// Set security state.
    pub fn set_security(&mut self, security: SecurityState) {
        self.security = security;
    }

    /// Get last active time.
    pub fn last_active(&self) -> Instant {
        self.last_active
    }

    /// Mark as active.
    pub fn mark_active(&mut self) {
        self.last_active = Instant::now();
    }

    /// Get creation time.
    pub fn created(&self) -> Instant {
        self.created
    }

    /// Called when page load completes.
    pub fn on_load_complete(&mut self, title: String, favicon: Option<String>) {
        self.loading = false;
        self.progress = 1.0;
        self.title = title;
        self.favicon = favicon;
    }

    /// Called when page load fails.
    pub fn on_load_error(&mut self, error: &str) {
        self.loading = false;
        self.title = format!("Error: {}", error);
    }
}

/// Navigation history.
pub struct NavigationHistory {
    /// History entries.
    entries: Vec<String>,
    /// Current index.
    current: isize,
    /// Maximum history size.
    max_size: usize,
}

impl NavigationHistory {
    /// Create a new navigation history.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current: -1,
            max_size: 50,
        }
    }

    /// Push a new entry.
    pub fn push(&mut self, url: String) {
        // Truncate forward history
        if self.current >= 0 {
            self.entries.truncate((self.current + 1) as usize);
        }

        self.entries.push(url);
        self.current = (self.entries.len() - 1) as isize;

        // Enforce max size
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.current -= 1;
        }
    }

    /// Go back and return the URL.
    pub fn go_back(&mut self) -> Option<&String> {
        if self.current > 0 {
            self.current -= 1;
            self.entries.get(self.current as usize)
        } else {
            None
        }
    }

    /// Go forward and return the URL.
    pub fn go_forward(&mut self) -> Option<&String> {
        if (self.current as usize) < self.entries.len() - 1 {
            self.current += 1;
            self.entries.get(self.current as usize)
        } else {
            None
        }
    }

    /// Check if can go back.
    pub fn can_go_back(&self) -> bool {
        self.current > 0
    }

    /// Check if can go forward.
    pub fn can_go_forward(&self) -> bool {
        (self.current as usize) < self.entries.len().saturating_sub(1)
    }

    /// Get the current entry.
    pub fn current(&self) -> Option<&String> {
        if self.current >= 0 {
            self.entries.get(self.current as usize)
        } else {
            None
        }
    }

    /// Get all entries.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Get the current index.
    pub fn current_index(&self) -> isize {
        self.current
    }

    /// Clear history.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current = -1;
    }
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Security state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityState {
    /// No security information.
    None,
    /// Secure (HTTPS with valid certificate).
    Secure,
    /// Secure with EV certificate.
    SecureEV,
    /// Insecure (HTTP).
    Insecure,
    /// Warning (HTTPS with issues).
    Warning,
    /// Dangerous (known bad site).
    Dangerous,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(TabId(1));

        assert_eq!(tab.id(), TabId(1));
        assert_eq!(tab.url(), "about:blank");
        assert_eq!(tab.title(), "New Tab");
        assert!(!tab.is_loading());
    }

    #[test]
    fn test_navigation() {
        let mut tab = Tab::new(TabId(1));

        tab.navigate("https://example.com");
        assert_eq!(tab.url(), "https://example.com");
        assert!(tab.is_loading());

        tab.navigate("https://example.com/page2");
        assert!(tab.can_go_back());
        assert!(!tab.can_go_forward());

        tab.go_back();
        assert_eq!(tab.url(), "https://example.com");
        assert!(tab.can_go_forward());
    }

    #[test]
    fn test_zoom() {
        let mut tab = Tab::new(TabId(1));

        assert_eq!(tab.zoom(), 1.0);

        tab.zoom_in();
        assert!(tab.zoom() > 1.0);

        tab.zoom_out();
        tab.zoom_out();
        assert!(tab.zoom() < 1.0);

        tab.reset_zoom();
        assert_eq!(tab.zoom(), 1.0);
    }

    #[test]
    fn test_navigation_history() {
        let mut history = NavigationHistory::new();

        history.push("https://page1.com".to_string());
        history.push("https://page2.com".to_string());
        history.push("https://page3.com".to_string());

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        let url = history.go_back().unwrap();
        assert_eq!(url, "https://page2.com");

        assert!(history.can_go_forward());

        let url = history.go_forward().unwrap();
        assert_eq!(url, "https://page3.com");
    }

    #[test]
    fn test_history_truncation() {
        let mut history = NavigationHistory::new();

        history.push("https://page1.com".to_string());
        history.push("https://page2.com".to_string());
        history.push("https://page3.com".to_string());

        history.go_back();
        history.push("https://page4.com".to_string());

        // page3 should be gone
        assert!(!history.can_go_forward());
        assert_eq!(history.entries().len(), 3);
    }
}
