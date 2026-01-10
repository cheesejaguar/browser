//! DOM Text node implementation.

use crate::node::NodeId;

/// Text node data.
#[derive(Clone, Debug)]
pub struct Text {
    /// Text content.
    pub data: String,
    /// Whitespace-only flag.
    pub is_whitespace_only: bool,
}

impl Text {
    pub fn new(data: String) -> Self {
        let is_whitespace_only = data.chars().all(|c| c.is_whitespace());
        Self {
            data,
            is_whitespace_only,
        }
    }

    /// Get length of text.
    pub fn length(&self) -> usize {
        self.data.len()
    }

    /// Get substring.
    pub fn substring(&self, start: usize, end: usize) -> &str {
        let start = start.min(self.data.len());
        let end = end.min(self.data.len());
        &self.data[start..end]
    }

    /// Append data.
    pub fn append_data(&mut self, data: &str) {
        self.data.push_str(data);
        self.is_whitespace_only = self.data.chars().all(|c| c.is_whitespace());
    }

    /// Insert data at position.
    pub fn insert_data(&mut self, offset: usize, data: &str) {
        let offset = offset.min(self.data.len());
        self.data.insert_str(offset, data);
        self.is_whitespace_only = self.data.chars().all(|c| c.is_whitespace());
    }

    /// Delete data from position.
    pub fn delete_data(&mut self, offset: usize, count: usize) {
        let start = offset.min(self.data.len());
        let end = (offset + count).min(self.data.len());
        self.data.replace_range(start..end, "");
        self.is_whitespace_only = self.data.chars().all(|c| c.is_whitespace());
    }

    /// Replace data at position.
    pub fn replace_data(&mut self, offset: usize, count: usize, data: &str) {
        let start = offset.min(self.data.len());
        let end = (offset + count).min(self.data.len());
        self.data.replace_range(start..end, data);
        self.is_whitespace_only = self.data.chars().all(|c| c.is_whitespace());
    }

    /// Split text at position.
    pub fn split_text(&mut self, offset: usize) -> Text {
        let offset = offset.min(self.data.len());
        let new_data = self.data.split_off(offset);
        self.is_whitespace_only = self.data.chars().all(|c| c.is_whitespace());
        Text::new(new_data)
    }

    /// Get whole text (for text node normalization).
    pub fn whole_text(&self) -> &str {
        &self.data
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Normalize whitespace for display.
    pub fn normalize_whitespace(&self) -> String {
        let mut result = String::with_capacity(self.data.len());
        let mut prev_whitespace = false;

        for c in self.data.chars() {
            if c.is_whitespace() {
                if !prev_whitespace {
                    result.push(' ');
                    prev_whitespace = true;
                }
            } else {
                result.push(c);
                prev_whitespace = false;
            }
        }

        result
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_operations() {
        let mut text = Text::new("Hello World".to_string());
        assert_eq!(text.length(), 11);

        text.insert_data(5, ",");
        assert_eq!(text.data, "Hello, World");

        text.delete_data(5, 1);
        assert_eq!(text.data, "Hello World");

        text.replace_data(6, 5, "Rust");
        assert_eq!(text.data, "Hello Rust");
    }

    #[test]
    fn test_split_text() {
        let mut text = Text::new("Hello World".to_string());
        let right = text.split_text(6);

        assert_eq!(text.data, "Hello ");
        assert_eq!(right.data, "World");
    }

    #[test]
    fn test_whitespace_normalization() {
        let text = Text::new("Hello   \n\t World".to_string());
        assert_eq!(text.normalize_whitespace(), "Hello World");
    }

    #[test]
    fn test_whitespace_only() {
        let text = Text::new("   \n\t  ".to_string());
        assert!(text.is_whitespace_only);

        let text = Text::new("Hello".to_string());
        assert!(!text.is_whitespace_only);
    }
}
