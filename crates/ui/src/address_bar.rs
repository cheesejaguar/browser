//! Address bar component.

use crate::tab::SecurityState;

/// Address bar.
pub struct AddressBar {
    /// Current URL.
    url: String,
    /// Input text.
    input: String,
    /// Is focused.
    focused: bool,
    /// Cursor position.
    cursor: usize,
    /// Selection range.
    selection: Option<(usize, usize)>,
    /// Autocomplete suggestions.
    suggestions: Vec<Suggestion>,
    /// Selected suggestion index.
    selected_suggestion: Option<usize>,
    /// Security state.
    security: SecurityState,
    /// Is showing suggestions.
    showing_suggestions: bool,
}

impl AddressBar {
    /// Create a new address bar.
    pub fn new() -> Self {
        Self {
            url: String::new(),
            input: String::new(),
            focused: false,
            cursor: 0,
            selection: None,
            suggestions: Vec::new(),
            selected_suggestion: None,
            security: SecurityState::None,
            showing_suggestions: false,
        }
    }

    /// Get the URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Set the URL.
    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
        if !self.focused {
            self.input = url.to_string();
        }
    }

    /// Get the input text.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Set the input text.
    pub fn set_input(&mut self, input: &str) {
        self.input = input.to_string();
        self.cursor = self.input.len();
        self.selection = None;
        self.update_suggestions();
    }

    /// Focus the address bar.
    pub fn focus(&mut self) {
        self.focused = true;
        self.input = self.url.clone();
        self.cursor = self.input.len();
        self.selection = Some((0, self.input.len()));
    }

    /// Blur the address bar.
    pub fn blur(&mut self) {
        self.focused = false;
        self.selection = None;
        self.suggestions.clear();
        self.showing_suggestions = false;
        self.input = self.url.clone();
    }

    /// Check if focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Get cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get selection.
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    /// Get security state.
    pub fn security(&self) -> SecurityState {
        self.security
    }

    /// Set security state.
    pub fn set_security(&mut self, security: SecurityState) {
        self.security = security;
    }

    /// Handle key input.
    pub fn on_key(&mut self, key: Key) -> Option<AddressBarAction> {
        match key {
            Key::Char(c) => {
                // Delete selection first if any
                if let Some((start, end)) = self.selection {
                    self.input.replace_range(start..end, "");
                    self.cursor = start;
                    self.selection = None;
                }

                self.input.insert(self.cursor, c);
                self.cursor += 1;
                self.update_suggestions();
                None
            }
            Key::Backspace => {
                if let Some((start, end)) = self.selection {
                    self.input.replace_range(start..end, "");
                    self.cursor = start;
                    self.selection = None;
                } else if self.cursor > 0 {
                    self.cursor -= 1;
                    self.input.remove(self.cursor);
                }
                self.update_suggestions();
                None
            }
            Key::Delete => {
                if let Some((start, end)) = self.selection {
                    self.input.replace_range(start..end, "");
                    self.cursor = start;
                    self.selection = None;
                } else if self.cursor < self.input.len() {
                    self.input.remove(self.cursor);
                }
                self.update_suggestions();
                None
            }
            Key::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                self.selection = None;
                None
            }
            Key::Right => {
                if self.cursor < self.input.len() {
                    self.cursor += 1;
                }
                self.selection = None;
                None
            }
            Key::Home => {
                self.cursor = 0;
                self.selection = None;
                None
            }
            Key::End => {
                self.cursor = self.input.len();
                self.selection = None;
                None
            }
            Key::Enter => {
                let action = if let Some(index) = self.selected_suggestion {
                    if let Some(suggestion) = self.suggestions.get(index) {
                        AddressBarAction::Navigate(suggestion.url.clone())
                    } else {
                        AddressBarAction::Navigate(self.input.clone())
                    }
                } else {
                    AddressBarAction::Navigate(self.input.clone())
                };
                self.blur();
                Some(action)
            }
            Key::Escape => {
                self.blur();
                Some(AddressBarAction::Cancel)
            }
            Key::Up => {
                if self.showing_suggestions && !self.suggestions.is_empty() {
                    self.selected_suggestion = Some(
                        self.selected_suggestion
                            .map(|i| if i > 0 { i - 1 } else { self.suggestions.len() - 1 })
                            .unwrap_or(self.suggestions.len() - 1),
                    );
                }
                None
            }
            Key::Down => {
                if self.showing_suggestions && !self.suggestions.is_empty() {
                    self.selected_suggestion = Some(
                        self.selected_suggestion
                            .map(|i| (i + 1) % self.suggestions.len())
                            .unwrap_or(0),
                    );
                }
                None
            }
            Key::Tab => {
                if let Some(index) = self.selected_suggestion {
                    if let Some(suggestion) = self.suggestions.get(index) {
                        self.input = suggestion.url.clone();
                        self.cursor = self.input.len();
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Handle select all.
    pub fn select_all(&mut self) {
        self.selection = Some((0, self.input.len()));
    }

    /// Handle copy.
    pub fn copy(&self) -> Option<String> {
        if let Some((start, end)) = self.selection {
            Some(self.input[start..end].to_string())
        } else {
            None
        }
    }

    /// Handle paste.
    pub fn paste(&mut self, text: &str) {
        if let Some((start, end)) = self.selection {
            self.input.replace_range(start..end, text);
            self.cursor = start + text.len();
            self.selection = None;
        } else {
            self.input.insert_str(self.cursor, text);
            self.cursor += text.len();
        }
        self.update_suggestions();
    }

    /// Handle cut.
    pub fn cut(&mut self) -> Option<String> {
        let copied = self.copy();
        if let Some((start, end)) = self.selection {
            self.input.replace_range(start..end, "");
            self.cursor = start;
            self.selection = None;
        }
        self.update_suggestions();
        copied
    }

    /// Update autocomplete suggestions.
    fn update_suggestions(&mut self) {
        if self.input.is_empty() {
            self.suggestions.clear();
            self.showing_suggestions = false;
            return;
        }

        // In a real implementation, this would query history, bookmarks, and search suggestions
        self.suggestions = vec![
            Suggestion {
                title: format!("Search for \"{}\"", self.input),
                url: format!("https://www.google.com/search?q={}", urlencoding::encode(&self.input)),
                suggestion_type: SuggestionType::Search,
            },
        ];

        // Add URL suggestion if it looks like a URL
        if self.input.contains('.') && !self.input.contains(' ') {
            let url = if self.input.starts_with("http://") || self.input.starts_with("https://") {
                self.input.clone()
            } else {
                format!("https://{}", self.input)
            };

            self.suggestions.insert(
                0,
                Suggestion {
                    title: self.input.clone(),
                    url,
                    suggestion_type: SuggestionType::Url,
                },
            );
        }

        self.showing_suggestions = !self.suggestions.is_empty();
        self.selected_suggestion = None;
    }

    /// Get suggestions.
    pub fn suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    /// Get selected suggestion.
    pub fn selected_suggestion(&self) -> Option<usize> {
        self.selected_suggestion
    }

    /// Is showing suggestions.
    pub fn is_showing_suggestions(&self) -> bool {
        self.showing_suggestions
    }

    /// Add history/bookmark suggestions.
    pub fn add_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        // Insert before the search suggestion
        let search_index = self
            .suggestions
            .iter()
            .position(|s| s.suggestion_type == SuggestionType::Search)
            .unwrap_or(self.suggestions.len());

        for (i, suggestion) in suggestions.into_iter().enumerate() {
            self.suggestions.insert(search_index + i, suggestion);
        }
    }
}

impl Default for AddressBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Key input.
#[derive(Clone, Debug)]
pub enum Key {
    Char(char),
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Enter,
    Escape,
    Tab,
}

/// Address bar action.
#[derive(Clone, Debug)]
pub enum AddressBarAction {
    Navigate(String),
    Cancel,
}

/// Autocomplete suggestion.
#[derive(Clone, Debug)]
pub struct Suggestion {
    /// Suggestion title.
    pub title: String,
    /// URL to navigate to.
    pub url: String,
    /// Suggestion type.
    pub suggestion_type: SuggestionType,
}

/// Suggestion type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionType {
    /// History entry.
    History,
    /// Bookmark.
    Bookmark,
    /// URL.
    Url,
    /// Search suggestion.
    Search,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_bar_creation() {
        let bar = AddressBar::new();
        assert!(!bar.is_focused());
        assert!(bar.url().is_empty());
    }

    #[test]
    fn test_focus_selects_all() {
        let mut bar = AddressBar::new();
        bar.set_url("https://example.com");

        bar.focus();
        assert!(bar.is_focused());
        assert_eq!(bar.selection(), Some((0, 19)));
    }

    #[test]
    fn test_typing() {
        let mut bar = AddressBar::new();
        bar.focus();

        bar.on_key(Key::Char('a'));
        bar.on_key(Key::Char('b'));
        bar.on_key(Key::Char('c'));

        assert_eq!(bar.input(), "abc");
        assert_eq!(bar.cursor(), 3);
    }

    #[test]
    fn test_enter_navigates() {
        let mut bar = AddressBar::new();
        bar.focus();
        bar.set_input("https://example.com");

        let action = bar.on_key(Key::Enter);
        assert!(matches!(action, Some(AddressBarAction::Navigate(_))));
        assert!(!bar.is_focused());
    }

    #[test]
    fn test_escape_cancels() {
        let mut bar = AddressBar::new();
        bar.set_url("https://example.com");
        bar.focus();
        bar.set_input("something else");

        let action = bar.on_key(Key::Escape);
        assert!(matches!(action, Some(AddressBarAction::Cancel)));
        assert!(!bar.is_focused());
        assert_eq!(bar.input(), "https://example.com"); // Reverted
    }
}
