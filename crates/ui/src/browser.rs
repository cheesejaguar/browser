//! Main browser controller.

use crate::tab::Tab;
use crate::window::BrowserWindow;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Browser application.
pub struct Browser {
    /// Browser windows.
    windows: HashMap<WindowId, BrowserWindow>,
    /// Window ID counter.
    window_counter: u64,
    /// Default profile.
    profile: Profile,
    /// Browser settings.
    settings: BrowserSettings,
    /// Extension manager.
    extensions: ExtensionManager,
    /// Whether the browser is running.
    running: bool,
}

impl Browser {
    /// Create a new browser instance.
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_counter: 0,
            profile: Profile::default(),
            settings: BrowserSettings::default(),
            extensions: ExtensionManager::new(),
            running: false,
        }
    }

    /// Start the browser.
    pub fn start(&mut self) {
        self.running = true;

        // Create initial window
        if self.windows.is_empty() {
            self.create_window();
        }
    }

    /// Stop the browser.
    pub fn stop(&mut self) {
        self.running = false;

        // Close all windows
        for (_, window) in self.windows.drain() {
            // Window cleanup
        }
    }

    /// Create a new browser window.
    pub fn create_window(&mut self) -> WindowId {
        self.window_counter += 1;
        let id = WindowId(self.window_counter);

        let window = BrowserWindow::new(id, &self.settings);
        self.windows.insert(id, window);

        id
    }

    /// Close a window.
    pub fn close_window(&mut self, id: WindowId) {
        self.windows.remove(&id);

        // Quit if all windows are closed
        if self.windows.is_empty() {
            self.running = false;
        }
    }

    /// Get a window.
    pub fn window(&self, id: WindowId) -> Option<&BrowserWindow> {
        self.windows.get(&id)
    }

    /// Get a mutable window.
    pub fn window_mut(&mut self, id: WindowId) -> Option<&mut BrowserWindow> {
        self.windows.get_mut(&id)
    }

    /// Get all windows.
    pub fn windows(&self) -> impl Iterator<Item = &BrowserWindow> {
        self.windows.values()
    }

    /// Get browser settings.
    pub fn settings(&self) -> &BrowserSettings {
        &self.settings
    }

    /// Get mutable browser settings.
    pub fn settings_mut(&mut self) -> &mut BrowserSettings {
        &mut self.settings
    }

    /// Get the profile.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Check if the browser is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Navigate the active tab of the focused window.
    pub fn navigate(&mut self, url: &str) {
        if let Some(window) = self.windows.values_mut().next() {
            if let Some(tab) = window.active_tab_mut() {
                tab.navigate(url);
            }
        }
    }

    /// Open a URL in a new tab.
    pub fn open_url_in_new_tab(&mut self, url: &str) {
        if let Some(window) = self.windows.values_mut().next() {
            let tab_id = window.new_tab();
            if let Some(tab) = window.tab_mut(tab_id) {
                tab.navigate(url);
            }
        }
    }

    /// Handle keyboard shortcut.
    pub fn handle_shortcut(&mut self, shortcut: KeyboardShortcut) {
        match shortcut {
            KeyboardShortcut::NewTab => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.new_tab();
                }
            }
            KeyboardShortcut::CloseTab => {
                if let Some(window) = self.windows.values_mut().next() {
                    let tab_id = window.active_tab_id();
                    if let Some(id) = tab_id {
                        window.close_tab(id);
                    }
                }
            }
            KeyboardShortcut::NewWindow => {
                self.create_window();
            }
            KeyboardShortcut::CloseWindow => {
                if let Some(&id) = self.windows.keys().next() {
                    self.close_window(id);
                }
            }
            KeyboardShortcut::Reload => {
                if let Some(window) = self.windows.values_mut().next() {
                    if let Some(tab) = window.active_tab_mut() {
                        tab.reload();
                    }
                }
            }
            KeyboardShortcut::Back => {
                if let Some(window) = self.windows.values_mut().next() {
                    if let Some(tab) = window.active_tab_mut() {
                        tab.go_back();
                    }
                }
            }
            KeyboardShortcut::Forward => {
                if let Some(window) = self.windows.values_mut().next() {
                    if let Some(tab) = window.active_tab_mut() {
                        tab.go_forward();
                    }
                }
            }
            KeyboardShortcut::FocusAddressBar => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.focus_address_bar();
                }
            }
            KeyboardShortcut::Find => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.show_find_bar();
                }
            }
            KeyboardShortcut::DevTools => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.toggle_devtools();
                }
            }
            KeyboardShortcut::ZoomIn => {
                if let Some(window) = self.windows.values_mut().next() {
                    if let Some(tab) = window.active_tab_mut() {
                        tab.zoom_in();
                    }
                }
            }
            KeyboardShortcut::ZoomOut => {
                if let Some(window) = self.windows.values_mut().next() {
                    if let Some(tab) = window.active_tab_mut() {
                        tab.zoom_out();
                    }
                }
            }
            KeyboardShortcut::ResetZoom => {
                if let Some(window) = self.windows.values_mut().next() {
                    if let Some(tab) = window.active_tab_mut() {
                        tab.reset_zoom();
                    }
                }
            }
            KeyboardShortcut::Fullscreen => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.toggle_fullscreen();
                }
            }
            _ => {}
        }
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new()
    }
}

/// Window identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// Browser profile.
#[derive(Clone, Debug)]
pub struct Profile {
    /// Profile name.
    pub name: String,
    /// Profile directory.
    pub directory: std::path::PathBuf,
    /// Whether this is the default profile.
    pub is_default: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            directory: std::path::PathBuf::from("~/.browser/profiles/default"),
            is_default: true,
        }
    }
}

/// Browser settings.
#[derive(Clone, Debug)]
pub struct BrowserSettings {
    /// Home page URL.
    pub home_page: String,
    /// Search engine.
    pub search_engine: SearchEngine,
    /// Default zoom level.
    pub default_zoom: f32,
    /// Enable JavaScript.
    pub javascript_enabled: bool,
    /// Enable cookies.
    pub cookies_enabled: bool,
    /// Block popups.
    pub block_popups: bool,
    /// Enable do not track.
    pub do_not_track: bool,
    /// Clear browsing data on exit.
    pub clear_on_exit: bool,
    /// Theme.
    pub theme: Theme,
    /// Font settings.
    pub fonts: FontSettings,
    /// Download directory.
    pub download_directory: std::path::PathBuf,
    /// Ask where to save downloads.
    pub ask_download_location: bool,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            home_page: "about:blank".to_string(),
            search_engine: SearchEngine::default(),
            default_zoom: 1.0,
            javascript_enabled: true,
            cookies_enabled: true,
            block_popups: true,
            do_not_track: false,
            clear_on_exit: false,
            theme: Theme::System,
            fonts: FontSettings::default(),
            download_directory: std::path::PathBuf::from("~/Downloads"),
            ask_download_location: false,
        }
    }
}

/// Search engine configuration.
#[derive(Clone, Debug)]
pub struct SearchEngine {
    /// Name.
    pub name: String,
    /// Search URL template.
    pub url_template: String,
    /// Suggestion URL template.
    pub suggestion_url: Option<String>,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self {
            name: "Google".to_string(),
            url_template: "https://www.google.com/search?q=%s".to_string(),
            suggestion_url: Some("https://www.google.com/complete/search?client=chrome&q=%s".to_string()),
        }
    }
}

/// Theme setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Font settings.
#[derive(Clone, Debug)]
pub struct FontSettings {
    /// Standard font family.
    pub standard: String,
    /// Serif font family.
    pub serif: String,
    /// Sans-serif font family.
    pub sans_serif: String,
    /// Monospace font family.
    pub monospace: String,
    /// Default font size.
    pub default_size: u32,
    /// Minimum font size.
    pub minimum_size: u32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            standard: "Arial".to_string(),
            serif: "Times New Roman".to_string(),
            sans_serif: "Arial".to_string(),
            monospace: "Courier New".to_string(),
            default_size: 16,
            minimum_size: 10,
        }
    }
}

/// Extension manager.
pub struct ExtensionManager {
    /// Installed extensions.
    extensions: Vec<Extension>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    pub fn install(&mut self, extension: Extension) {
        self.extensions.push(extension);
    }

    pub fn uninstall(&mut self, id: &str) {
        self.extensions.retain(|e| e.id != id);
    }

    pub fn get(&self, id: &str) -> Option<&Extension> {
        self.extensions.iter().find(|e| e.id == id)
    }

    pub fn all(&self) -> &[Extension] {
        &self.extensions
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Browser extension.
#[derive(Clone, Debug)]
pub struct Extension {
    /// Extension ID.
    pub id: String,
    /// Name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Description.
    pub description: String,
    /// Whether enabled.
    pub enabled: bool,
}

/// Keyboard shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardShortcut {
    NewTab,
    CloseTab,
    NewWindow,
    CloseWindow,
    Reload,
    HardReload,
    Back,
    Forward,
    FocusAddressBar,
    Find,
    FindNext,
    FindPrevious,
    DevTools,
    ViewSource,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    Fullscreen,
    Print,
    Save,
    NextTab,
    PreviousTab,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = Browser::new();
        assert!(!browser.is_running());
        assert!(browser.windows.is_empty());
    }

    #[test]
    fn test_browser_start() {
        let mut browser = Browser::new();
        browser.start();

        assert!(browser.is_running());
        assert_eq!(browser.windows.len(), 1);
    }

    #[test]
    fn test_create_window() {
        let mut browser = Browser::new();
        let id1 = browser.create_window();
        let id2 = browser.create_window();

        assert_ne!(id1, id2);
        assert_eq!(browser.windows.len(), 2);
    }

    #[test]
    fn test_close_window() {
        let mut browser = Browser::new();
        let id = browser.create_window();
        browser.running = true;

        browser.close_window(id);
        assert!(browser.windows.is_empty());
        assert!(!browser.is_running()); // Should quit when last window closes
    }
}
