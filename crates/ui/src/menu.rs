//! Menu system.

/// Menu bar.
pub struct MenuBar {
    menus: Vec<Menu>,
}

impl MenuBar {
    pub fn new() -> Self {
        Self { menus: Vec::new() }
    }

    pub fn add_menu(&mut self, menu: Menu) {
        self.menus.push(menu);
    }

    pub fn menus(&self) -> &[Menu] {
        &self.menus
    }

    pub fn create_default() -> Self {
        let mut bar = Self::new();

        // File menu
        bar.add_menu(Menu::new("File")
            .add_item(MenuItem::action("New Tab", "Ctrl+T"))
            .add_item(MenuItem::action("New Window", "Ctrl+N"))
            .add_item(MenuItem::action("New Incognito Window", "Ctrl+Shift+N"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Open File...", "Ctrl+O"))
            .add_item(MenuItem::action("Open Location...", "Ctrl+L"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Save Page As...", "Ctrl+S"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Print...", "Ctrl+P"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Exit", "Alt+F4")));

        // Edit menu
        bar.add_menu(Menu::new("Edit")
            .add_item(MenuItem::action("Undo", "Ctrl+Z"))
            .add_item(MenuItem::action("Redo", "Ctrl+Y"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Cut", "Ctrl+X"))
            .add_item(MenuItem::action("Copy", "Ctrl+C"))
            .add_item(MenuItem::action("Paste", "Ctrl+V"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Find...", "Ctrl+F"))
            .add_item(MenuItem::action("Find Next", "F3")));

        // View menu
        bar.add_menu(Menu::new("View")
            .add_item(MenuItem::action("Reload", "F5"))
            .add_item(MenuItem::action("Hard Reload", "Ctrl+F5"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Zoom In", "Ctrl++"))
            .add_item(MenuItem::action("Zoom Out", "Ctrl+-"))
            .add_item(MenuItem::action("Reset Zoom", "Ctrl+0"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Full Screen", "F11"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("View Source", "Ctrl+U"))
            .add_item(MenuItem::action("Developer Tools", "F12")));

        // History menu
        bar.add_menu(Menu::new("History")
            .add_item(MenuItem::action("Back", "Alt+Left"))
            .add_item(MenuItem::action("Forward", "Alt+Right"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Show All History", "Ctrl+H")));

        // Bookmarks menu
        bar.add_menu(Menu::new("Bookmarks")
            .add_item(MenuItem::action("Bookmark This Page", "Ctrl+D"))
            .add_item(MenuItem::action("Show All Bookmarks", "Ctrl+Shift+O")));

        // Help menu
        bar.add_menu(Menu::new("Help")
            .add_item(MenuItem::action("About", "")));

        bar
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::create_default()
    }
}

/// A menu.
#[derive(Clone, Debug)]
pub struct Menu {
    label: String,
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            items: Vec::new(),
        }
    }

    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }
}

/// Menu item.
#[derive(Clone, Debug)]
pub struct MenuItem {
    pub item_type: MenuItemType,
    pub label: String,
    pub shortcut: String,
    pub enabled: bool,
    pub checked: bool,
    pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
    pub fn action(label: &str, shortcut: &str) -> Self {
        Self {
            item_type: MenuItemType::Action,
            label: label.to_string(),
            shortcut: shortcut.to_string(),
            enabled: true,
            checked: false,
            submenu: None,
        }
    }

    pub fn separator() -> Self {
        Self {
            item_type: MenuItemType::Separator,
            label: String::new(),
            shortcut: String::new(),
            enabled: true,
            checked: false,
            submenu: None,
        }
    }

    pub fn checkbox(label: &str, checked: bool) -> Self {
        Self {
            item_type: MenuItemType::Checkbox,
            label: label.to_string(),
            shortcut: String::new(),
            enabled: true,
            checked,
            submenu: None,
        }
    }

    pub fn submenu(label: &str, items: Vec<MenuItem>) -> Self {
        Self {
            item_type: MenuItemType::Submenu,
            label: label.to_string(),
            shortcut: String::new(),
            enabled: true,
            checked: false,
            submenu: Some(items),
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Menu item type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItemType {
    Action,
    Separator,
    Checkbox,
    Radio,
    Submenu,
}
