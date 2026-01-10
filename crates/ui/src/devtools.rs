//! Developer tools integration.

/// DevTools panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevToolsPanel {
    Elements,
    Console,
    Sources,
    Network,
    Performance,
    Memory,
    Application,
    Security,
}

/// DevTools state.
pub struct DevTools {
    visible: bool,
    panel: DevToolsPanel,
    docked: DockPosition,
    width: u32,
    height: u32,
}

impl DevTools {
    pub fn new() -> Self {
        Self {
            visible: false,
            panel: DevToolsPanel::Elements,
            docked: DockPosition::Right,
            width: 400,
            height: 300,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_panel(&mut self, panel: DevToolsPanel) {
        self.panel = panel;
    }

    pub fn panel(&self) -> DevToolsPanel {
        self.panel
    }

    pub fn set_dock_position(&mut self, position: DockPosition) {
        self.docked = position;
    }

    pub fn dock_position(&self) -> DockPosition {
        self.docked
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Default for DevTools {
    fn default() -> Self {
        Self::new()
    }
}

/// Dock position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockPosition {
    Right,
    Bottom,
    Left,
    Undocked,
}
