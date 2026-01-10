//! Find bar component.

/// Find bar for in-page search.
pub struct FindBar {
    /// Search query.
    query: String,
    /// Current match index.
    current_match: usize,
    /// Total matches.
    total_matches: usize,
    /// Case sensitive.
    case_sensitive: bool,
    /// Whole word.
    whole_word: bool,
    /// Use regex.
    use_regex: bool,
    /// Is focused.
    focused: bool,
    /// Cursor position.
    cursor: usize,
}

impl FindBar {
    /// Create a new find bar.
    pub fn new() -> Self {
        Self {
            query: String::new(),
            current_match: 0,
            total_matches: 0,
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            focused: false,
            cursor: 0,
        }
    }

    /// Get the search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Set the search query.
    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.cursor = self.query.len();
        self.current_match = 0;
    }

    /// Get current match index.
    pub fn current_match(&self) -> usize {
        self.current_match
    }

    /// Get total matches.
    pub fn total_matches(&self) -> usize {
        self.total_matches
    }

    /// Set match info.
    pub fn set_matches(&mut self, current: usize, total: usize) {
        self.current_match = current;
        self.total_matches = total;
    }

    /// Go to next match.
    pub fn next_match(&mut self) {
        if self.total_matches > 0 {
            self.current_match = (self.current_match + 1) % self.total_matches;
        }
    }

    /// Go to previous match.
    pub fn previous_match(&mut self) {
        if self.total_matches > 0 {
            if self.current_match == 0 {
                self.current_match = self.total_matches - 1;
            } else {
                self.current_match -= 1;
            }
        }
    }

    /// Is case sensitive.
    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    /// Toggle case sensitivity.
    pub fn toggle_case_sensitive(&mut self) {
        self.case_sensitive = !self.case_sensitive;
    }

    /// Is whole word.
    pub fn is_whole_word(&self) -> bool {
        self.whole_word
    }

    /// Toggle whole word.
    pub fn toggle_whole_word(&mut self) {
        self.whole_word = !self.whole_word;
    }

    /// Is using regex.
    pub fn is_regex(&self) -> bool {
        self.use_regex
    }

    /// Toggle regex.
    pub fn toggle_regex(&mut self) {
        self.use_regex = !self.use_regex;
    }

    /// Focus the find bar.
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Blur the find bar.
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Clear the find bar.
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.current_match = 0;
        self.total_matches = 0;
    }

    /// Handle key input.
    pub fn on_key(&mut self, key: FindBarKey) -> Option<FindBarAction> {
        match key {
            FindBarKey::Char(c) => {
                self.query.insert(self.cursor, c);
                self.cursor += 1;
                Some(FindBarAction::Search(self.query.clone()))
            }
            FindBarKey::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.query.remove(self.cursor);
                    Some(FindBarAction::Search(self.query.clone()))
                } else {
                    None
                }
            }
            FindBarKey::Enter => {
                self.next_match();
                Some(FindBarAction::NextMatch)
            }
            FindBarKey::ShiftEnter => {
                self.previous_match();
                Some(FindBarAction::PreviousMatch)
            }
            FindBarKey::Escape => {
                self.blur();
                Some(FindBarAction::Close)
            }
            FindBarKey::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                None
            }
            FindBarKey::Right => {
                if self.cursor < self.query.len() {
                    self.cursor += 1;
                }
                None
            }
        }
    }

    /// Get match status string.
    pub fn match_status(&self) -> String {
        if self.query.is_empty() {
            String::new()
        } else if self.total_matches == 0 {
            "No matches".to_string()
        } else {
            format!("{} of {}", self.current_match + 1, self.total_matches)
        }
    }
}

impl Default for FindBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Find bar key input.
#[derive(Clone, Debug)]
pub enum FindBarKey {
    Char(char),
    Backspace,
    Enter,
    ShiftEnter,
    Escape,
    Left,
    Right,
}

/// Find bar action.
#[derive(Clone, Debug)]
pub enum FindBarAction {
    Search(String),
    NextMatch,
    PreviousMatch,
    Close,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_bar() {
        let mut bar = FindBar::new();

        bar.set_query("test");
        assert_eq!(bar.query(), "test");

        bar.set_matches(0, 5);
        assert_eq!(bar.total_matches(), 5);
        assert_eq!(bar.current_match(), 0);

        bar.next_match();
        assert_eq!(bar.current_match(), 1);

        bar.previous_match();
        assert_eq!(bar.current_match(), 0);
    }

    #[test]
    fn test_match_wrapping() {
        let mut bar = FindBar::new();
        bar.set_matches(0, 3);

        bar.previous_match();
        assert_eq!(bar.current_match(), 2);

        bar.next_match();
        assert_eq!(bar.current_match(), 0);
    }

    #[test]
    fn test_match_status() {
        let mut bar = FindBar::new();

        assert_eq!(bar.match_status(), "");

        bar.set_query("test");
        bar.set_matches(0, 0);
        assert_eq!(bar.match_status(), "No matches");

        bar.set_matches(2, 10);
        assert_eq!(bar.match_status(), "3 of 10");
    }
}
