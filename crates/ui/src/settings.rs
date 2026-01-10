//! Settings UI.

/// Settings category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Privacy,
    Security,
    Appearance,
    SearchEngine,
    Downloads,
    Languages,
    Advanced,
}

/// Settings page.
pub struct SettingsPage {
    category: SettingsCategory,
}

impl SettingsPage {
    pub fn new(category: SettingsCategory) -> Self {
        Self { category }
    }

    pub fn category(&self) -> SettingsCategory {
        self.category
    }
}
