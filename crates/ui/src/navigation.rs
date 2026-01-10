//! Navigation bar component.

/// Navigation bar.
pub struct NavigationBar {
    /// Can go back.
    can_go_back: bool,
    /// Can go forward.
    can_go_forward: bool,
    /// Is loading.
    loading: bool,
    /// Reload button hovered.
    reload_hovered: bool,
    /// Stop button hovered.
    stop_hovered: bool,
    /// Back button hovered.
    back_hovered: bool,
    /// Forward button hovered.
    forward_hovered: bool,
}

impl NavigationBar {
    /// Create a new navigation bar.
    pub fn new() -> Self {
        Self {
            can_go_back: false,
            can_go_forward: false,
            loading: false,
            reload_hovered: false,
            stop_hovered: false,
            back_hovered: false,
            forward_hovered: false,
        }
    }

    /// Set can go back.
    pub fn set_can_go_back(&mut self, can: bool) {
        self.can_go_back = can;
    }

    /// Check if can go back.
    pub fn can_go_back(&self) -> bool {
        self.can_go_back
    }

    /// Set can go forward.
    pub fn set_can_go_forward(&mut self, can: bool) {
        self.can_go_forward = can;
    }

    /// Check if can go forward.
    pub fn can_go_forward(&self) -> bool {
        self.can_go_forward
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Check if loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Handle mouse enter on back button.
    pub fn on_back_hover(&mut self, hovered: bool) {
        self.back_hovered = hovered;
    }

    /// Handle mouse enter on forward button.
    pub fn on_forward_hover(&mut self, hovered: bool) {
        self.forward_hovered = hovered;
    }

    /// Handle mouse enter on reload button.
    pub fn on_reload_hover(&mut self, hovered: bool) {
        self.reload_hovered = hovered;
    }

    /// Handle mouse enter on stop button.
    pub fn on_stop_hover(&mut self, hovered: bool) {
        self.stop_hovered = hovered;
    }

    /// Check if back button is hovered.
    pub fn is_back_hovered(&self) -> bool {
        self.back_hovered
    }

    /// Check if forward button is hovered.
    pub fn is_forward_hovered(&self) -> bool {
        self.forward_hovered
    }

    /// Check if reload button is hovered.
    pub fn is_reload_hovered(&self) -> bool {
        self.reload_hovered
    }

    /// Check if stop button is hovered.
    pub fn is_stop_hovered(&self) -> bool {
        self.stop_hovered
    }
}

impl Default for NavigationBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationAction {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_bar() {
        let mut bar = NavigationBar::new();

        assert!(!bar.can_go_back());
        assert!(!bar.can_go_forward());
        assert!(!bar.is_loading());

        bar.set_can_go_back(true);
        bar.set_loading(true);

        assert!(bar.can_go_back());
        assert!(bar.is_loading());
    }
}
