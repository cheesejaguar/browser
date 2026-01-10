//! Browser window.

use crate::address_bar::AddressBar;
use crate::browser::{BrowserSettings, WindowId};
use crate::find_bar::FindBar;
use crate::navigation::NavigationBar;
use crate::tab::{Tab, TabId};
use crate::tab_bar::TabBar;
use std::collections::HashMap;

/// Browser window.
pub struct BrowserWindow {
    /// Window ID.
    id: WindowId,
    /// Tab bar.
    tab_bar: TabBar,
    /// Navigation bar.
    navigation_bar: NavigationBar,
    /// Address bar.
    address_bar: AddressBar,
    /// Find bar.
    find_bar: FindBar,
    /// Tabs.
    tabs: HashMap<TabId, Tab>,
    /// Active tab ID.
    active_tab: Option<TabId>,
    /// Tab ID counter.
    tab_counter: u64,
    /// Window dimensions.
    dimensions: WindowDimensions,
    /// Window state.
    state: WindowState,
    /// Is fullscreen.
    fullscreen: bool,
    /// DevTools visible.
    devtools_visible: bool,
    /// Find bar visible.
    find_bar_visible: bool,
}

impl BrowserWindow {
    /// Create a new browser window.
    pub fn new(id: WindowId, settings: &BrowserSettings) -> Self {
        let mut window = Self {
            id,
            tab_bar: TabBar::new(),
            navigation_bar: NavigationBar::new(),
            address_bar: AddressBar::new(),
            find_bar: FindBar::new(),
            tabs: HashMap::new(),
            active_tab: None,
            tab_counter: 0,
            dimensions: WindowDimensions::default(),
            state: WindowState::Normal,
            fullscreen: false,
            devtools_visible: false,
            find_bar_visible: false,
        };

        // Create initial tab
        let tab_id = window.new_tab();
        if let Some(tab) = window.tab_mut(tab_id) {
            tab.navigate(&settings.home_page);
        }

        window
    }

    /// Get the window ID.
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Create a new tab.
    pub fn new_tab(&mut self) -> TabId {
        self.tab_counter += 1;
        let id = TabId(self.tab_counter);

        let tab = Tab::new(id);
        self.tabs.insert(id, tab);
        self.tab_bar.add_tab(id, "New Tab".to_string());
        self.active_tab = Some(id);

        id
    }

    /// Close a tab.
    pub fn close_tab(&mut self, id: TabId) {
        self.tabs.remove(&id);
        self.tab_bar.remove_tab(id);

        // Update active tab
        if self.active_tab == Some(id) {
            self.active_tab = self.tabs.keys().next().copied();
        }

        // Create a new tab if all tabs are closed
        if self.tabs.is_empty() {
            self.new_tab();
        }
    }

    /// Get a tab.
    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.get(&id)
    }

    /// Get a mutable tab.
    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.get_mut(&id)
    }

    /// Get the active tab.
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|id| self.tabs.get(&id))
    }

    /// Get the active tab mutably.
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|id| self.tabs.get_mut(&id))
    }

    /// Get the active tab ID.
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.active_tab
    }

    /// Set the active tab.
    pub fn set_active_tab(&mut self, id: TabId) {
        if self.tabs.contains_key(&id) {
            self.active_tab = Some(id);
            self.tab_bar.set_active(id);

            // Update address bar
            if let Some(tab) = self.tabs.get(&id) {
                self.address_bar.set_url(&tab.url());
            }
        }
    }

    /// Get all tabs.
    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.values()
    }

    /// Get the tab count.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Focus the address bar.
    pub fn focus_address_bar(&mut self) {
        self.address_bar.focus();
    }

    /// Show the find bar.
    pub fn show_find_bar(&mut self) {
        self.find_bar_visible = true;
        self.find_bar.focus();
    }

    /// Hide the find bar.
    pub fn hide_find_bar(&mut self) {
        self.find_bar_visible = false;
        self.find_bar.clear();
    }

    /// Toggle DevTools.
    pub fn toggle_devtools(&mut self) {
        self.devtools_visible = !self.devtools_visible;
    }

    /// Toggle fullscreen.
    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    /// Check if fullscreen.
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Get window dimensions.
    pub fn dimensions(&self) -> &WindowDimensions {
        &self.dimensions
    }

    /// Set window dimensions.
    pub fn set_dimensions(&mut self, dimensions: WindowDimensions) {
        self.dimensions = dimensions;
    }

    /// Get window state.
    pub fn state(&self) -> WindowState {
        self.state
    }

    /// Set window state.
    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }

    /// Minimize the window.
    pub fn minimize(&mut self) {
        self.state = WindowState::Minimized;
    }

    /// Maximize the window.
    pub fn maximize(&mut self) {
        self.state = WindowState::Maximized;
    }

    /// Restore the window.
    pub fn restore(&mut self) {
        self.state = WindowState::Normal;
    }

    /// Handle address bar submission.
    pub fn on_address_submit(&mut self, input: &str) {
        if let Some(tab) = self.active_tab_mut() {
            // Check if it's a URL or search query
            let url = if input.contains('.') && !input.contains(' ') {
                // Likely a URL
                if input.starts_with("http://") || input.starts_with("https://") {
                    input.to_string()
                } else {
                    format!("https://{}", input)
                }
            } else {
                // Search query
                format!("https://www.google.com/search?q={}", urlencoding::encode(input))
            };

            tab.navigate(&url);
        }
    }

    /// Handle navigation button clicks.
    pub fn on_back(&mut self) {
        if let Some(tab) = self.active_tab_mut() {
            tab.go_back();
        }
    }

    pub fn on_forward(&mut self) {
        if let Some(tab) = self.active_tab_mut() {
            tab.go_forward();
        }
    }

    pub fn on_reload(&mut self) {
        if let Some(tab) = self.active_tab_mut() {
            tab.reload();
        }
    }

    pub fn on_stop(&mut self) {
        if let Some(tab) = self.active_tab_mut() {
            tab.stop();
        }
    }

    /// Update the UI based on tab state.
    pub fn update_ui(&mut self) {
        if let Some(tab) = self.active_tab() {
            self.address_bar.set_url(&tab.url());
            self.navigation_bar.set_can_go_back(tab.can_go_back());
            self.navigation_bar.set_can_go_forward(tab.can_go_forward());
            self.navigation_bar.set_loading(tab.is_loading());
            self.tab_bar.set_tab_title(tab.id(), tab.title().to_string());
        }
    }
}

/// Window dimensions.
#[derive(Clone, Debug)]
pub struct WindowDimensions {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowDimensions {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1280,
            height: 800,
        }
    }
}

/// Window state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let settings = BrowserSettings::default();
        let window = BrowserWindow::new(WindowId(1), &settings);

        assert_eq!(window.id(), WindowId(1));
        assert_eq!(window.tab_count(), 1);
        assert!(window.active_tab().is_some());
    }

    #[test]
    fn test_new_tab() {
        let settings = BrowserSettings::default();
        let mut window = BrowserWindow::new(WindowId(1), &settings);

        let initial_count = window.tab_count();
        let new_id = window.new_tab();

        assert_eq!(window.tab_count(), initial_count + 1);
        assert_eq!(window.active_tab_id(), Some(new_id));
    }

    #[test]
    fn test_close_tab() {
        let settings = BrowserSettings::default();
        let mut window = BrowserWindow::new(WindowId(1), &settings);

        let tab1 = window.new_tab();
        let tab2 = window.new_tab();

        window.close_tab(tab2);
        assert_eq!(window.tab_count(), 2); // Initial tab + tab1
        assert!(window.tab(tab2).is_none());
    }

    #[test]
    fn test_close_last_tab_creates_new() {
        let settings = BrowserSettings::default();
        let mut window = BrowserWindow::new(WindowId(1), &settings);

        let initial_tab = window.active_tab_id().unwrap();
        window.close_tab(initial_tab);

        // Should have created a new tab
        assert_eq!(window.tab_count(), 1);
        assert!(window.active_tab().is_some());
    }

    #[test]
    fn test_window_state() {
        let settings = BrowserSettings::default();
        let mut window = BrowserWindow::new(WindowId(1), &settings);

        assert_eq!(window.state(), WindowState::Normal);

        window.maximize();
        assert_eq!(window.state(), WindowState::Maximized);

        window.minimize();
        assert_eq!(window.state(), WindowState::Minimized);

        window.restore();
        assert_eq!(window.state(), WindowState::Normal);
    }
}
