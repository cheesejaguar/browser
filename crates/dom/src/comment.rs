//! DOM Comment node implementation.

/// Comment node data.
#[derive(Clone, Debug)]
pub struct Comment {
    /// Comment content (without <!-- and -->).
    pub data: String,
}

impl Comment {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    /// Get length of comment.
    pub fn length(&self) -> usize {
        self.data.len()
    }

    /// Append data.
    pub fn append_data(&mut self, data: &str) {
        self.data.push_str(data);
    }

    /// Insert data at position.
    pub fn insert_data(&mut self, offset: usize, data: &str) {
        let offset = offset.min(self.data.len());
        self.data.insert_str(offset, data);
    }

    /// Delete data from position.
    pub fn delete_data(&mut self, offset: usize, count: usize) {
        let start = offset.min(self.data.len());
        let end = (offset + count).min(self.data.len());
        self.data.replace_range(start..end, "");
    }

    /// Replace data at position.
    pub fn replace_data(&mut self, offset: usize, count: usize, data: &str) {
        let start = offset.min(self.data.len());
        let end = (offset + count).min(self.data.len());
        self.data.replace_range(start..end, data);
    }

    /// Get substring.
    pub fn substring(&self, start: usize, end: usize) -> &str {
        let start = start.min(self.data.len());
        let end = end.min(self.data.len());
        &self.data[start..end]
    }

    /// Convert to HTML string.
    pub fn to_html(&self) -> String {
        format!("<!--{}-->", self.data)
    }
}

impl Default for Comment {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!--{}-->", self.data)
    }
}

impl From<&str> for Comment {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl From<String> for Comment {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        let comment = Comment::new("This is a comment".to_string());
        assert_eq!(comment.length(), 17);
        assert_eq!(comment.to_html(), "<!--This is a comment-->");
    }

    #[test]
    fn test_comment_operations() {
        let mut comment = Comment::new("Hello".to_string());
        comment.append_data(" World");
        assert_eq!(comment.data, "Hello World");
    }
}
