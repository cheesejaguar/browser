//! Tab bar component.

use crate::tab::TabId;
use std::collections::HashMap;

/// Tab bar.
pub struct TabBar {
    /// Tab entries.
    tabs: Vec<TabEntry>,
    /// Active tab.
    active: Option<TabId>,
    /// Hovered tab.
    hovered: Option<TabId>,
    /// Dragging tab.
    dragging: Option<DragState>,
}

impl TabBar {
    /// Create a new tab bar.
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
            hovered: None,
            dragging: None,
        }
    }

    /// Add a tab.
    pub fn add_tab(&mut self, id: TabId, title: String) {
        self.tabs.push(TabEntry {
            id,
            title,
            favicon: None,
            close_hovered: false,
            loading: false,
            playing_audio: false,
            muted: false,
            pinned: false,
        });
    }

    /// Remove a tab.
    pub fn remove_tab(&mut self, id: TabId) {
        self.tabs.retain(|t| t.id != id);
    }

    /// Set the active tab.
    pub fn set_active(&mut self, id: TabId) {
        self.active = Some(id);
    }

    /// Get the active tab.
    pub fn active(&self) -> Option<TabId> {
        self.active
    }

    /// Set tab title.
    pub fn set_tab_title(&mut self, id: TabId, title: String) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.title = title;
        }
    }

    /// Set tab favicon.
    pub fn set_tab_favicon(&mut self, id: TabId, favicon: Option<String>) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.favicon = favicon;
        }
    }

    /// Set tab loading state.
    pub fn set_tab_loading(&mut self, id: TabId, loading: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.loading = loading;
        }
    }

    /// Set tab audio state.
    pub fn set_tab_audio(&mut self, id: TabId, playing: bool, muted: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.playing_audio = playing;
            tab.muted = muted;
        }
    }

    /// Set tab pinned state.
    pub fn set_tab_pinned(&mut self, id: TabId, pinned: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.pinned = pinned;
        }

        // Sort tabs so pinned tabs come first
        self.tabs.sort_by(|a, b| b.pinned.cmp(&a.pinned));
    }

    /// Get tabs.
    pub fn tabs(&self) -> &[TabEntry] {
        &self.tabs
    }

    /// Get tab count.
    pub fn count(&self) -> usize {
        self.tabs.len()
    }

    /// Move a tab to a new position.
    pub fn move_tab(&mut self, id: TabId, to_index: usize) {
        if let Some(from_index) = self.tabs.iter().position(|t| t.id == id) {
            let tab = self.tabs.remove(from_index);
            let to_index = to_index.min(self.tabs.len());
            self.tabs.insert(to_index, tab);
        }
    }

    /// Start dragging a tab.
    pub fn start_drag(&mut self, id: TabId, x: f32, y: f32) {
        if let Some(index) = self.tabs.iter().position(|t| t.id == id) {
            self.dragging = Some(DragState {
                tab_id: id,
                original_index: index,
                current_x: x,
                current_y: y,
            });
        }
    }

    /// Update drag position.
    pub fn update_drag(&mut self, x: f32, y: f32) {
        if let Some(ref mut drag) = self.dragging {
            drag.current_x = x;
            drag.current_y = y;
        }
    }

    /// End dragging.
    pub fn end_drag(&mut self) -> Option<(TabId, usize)> {
        self.dragging.take().map(|drag| {
            // Calculate new position based on drag position
            // For now, return the original position
            (drag.tab_id, drag.original_index)
        })
    }

    /// Check if dragging.
    pub fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    /// Handle mouse enter on tab.
    pub fn on_mouse_enter(&mut self, id: TabId) {
        self.hovered = Some(id);
    }

    /// Handle mouse leave.
    pub fn on_mouse_leave(&mut self) {
        self.hovered = None;
    }

    /// Handle mouse enter on close button.
    pub fn on_close_button_enter(&mut self, id: TabId) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.close_hovered = true;
        }
    }

    /// Handle mouse leave from close button.
    pub fn on_close_button_leave(&mut self, id: TabId) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.close_hovered = false;
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab entry in the tab bar.
#[derive(Clone, Debug)]
pub struct TabEntry {
    /// Tab ID.
    pub id: TabId,
    /// Title.
    pub title: String,
    /// Favicon URL.
    pub favicon: Option<String>,
    /// Close button hovered.
    pub close_hovered: bool,
    /// Is loading.
    pub loading: bool,
    /// Is playing audio.
    pub playing_audio: bool,
    /// Is muted.
    pub muted: bool,
    /// Is pinned.
    pub pinned: bool,
}

/// Drag state for tab reordering.
#[derive(Clone, Debug)]
struct DragState {
    tab_id: TabId,
    original_index: usize,
    current_x: f32,
    current_y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_creation() {
        let tab_bar = TabBar::new();
        assert_eq!(tab_bar.count(), 0);
        assert!(tab_bar.active().is_none());
    }

    #[test]
    fn test_add_remove_tab() {
        let mut tab_bar = TabBar::new();

        tab_bar.add_tab(TabId(1), "Tab 1".to_string());
        tab_bar.add_tab(TabId(2), "Tab 2".to_string());

        assert_eq!(tab_bar.count(), 2);

        tab_bar.remove_tab(TabId(1));
        assert_eq!(tab_bar.count(), 1);
    }

    #[test]
    fn test_active_tab() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab(TabId(1), "Tab 1".to_string());
        tab_bar.add_tab(TabId(2), "Tab 2".to_string());

        tab_bar.set_active(TabId(2));
        assert_eq!(tab_bar.active(), Some(TabId(2)));
    }

    #[test]
    fn test_move_tab() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab(TabId(1), "Tab 1".to_string());
        tab_bar.add_tab(TabId(2), "Tab 2".to_string());
        tab_bar.add_tab(TabId(3), "Tab 3".to_string());

        tab_bar.move_tab(TabId(3), 0);

        assert_eq!(tab_bar.tabs()[0].id, TabId(3));
        assert_eq!(tab_bar.tabs()[1].id, TabId(1));
        assert_eq!(tab_bar.tabs()[2].id, TabId(2));
    }

    #[test]
    fn test_pinned_tabs_sorted_first() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab(TabId(1), "Tab 1".to_string());
        tab_bar.add_tab(TabId(2), "Tab 2".to_string());
        tab_bar.add_tab(TabId(3), "Tab 3".to_string());

        tab_bar.set_tab_pinned(TabId(2), true);

        assert_eq!(tab_bar.tabs()[0].id, TabId(2));
        assert!(tab_bar.tabs()[0].pinned);
    }
}
