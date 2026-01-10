//! DOM Attribute handling.

use indexmap::IndexMap;
use std::sync::Arc;

/// A single DOM attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: Arc<str>,
    pub value: String,
    pub namespace: Option<Arc<str>>,
}

impl Attribute {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: Arc::from(name),
            value: value.to_string(),
            namespace: None,
        }
    }

    pub fn with_namespace(name: &str, value: &str, namespace: &str) -> Self {
        Self {
            name: Arc::from(name),
            value: value.to_string(),
            namespace: Some(Arc::from(namespace)),
        }
    }
}

/// Map of element attributes preserving insertion order.
#[derive(Clone, Debug, Default)]
pub struct AttributeMap {
    attrs: IndexMap<Arc<str>, String>,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self {
            attrs: IndexMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            attrs: IndexMap::with_capacity(capacity),
        }
    }

    /// Set an attribute value.
    pub fn set(&mut self, name: &str, value: &str) {
        self.attrs.insert(Arc::from(name), value.to_string());
    }

    /// Get an attribute value.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(|s| s.as_str())
    }

    /// Remove an attribute.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.attrs.swap_remove(name)
    }

    /// Check if attribute exists.
    pub fn contains(&self, name: &str) -> bool {
        self.attrs.contains_key(name)
    }

    /// Get number of attributes.
    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }

    /// Iterate over attributes.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.attrs.iter().map(|(k, v)| (k.as_ref(), v.as_str()))
    }

    /// Get attribute names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.attrs.keys().map(|k| k.as_ref())
    }

    /// Clear all attributes.
    pub fn clear(&mut self) {
        self.attrs.clear();
    }

    /// Parse from HTML attribute string.
    pub fn parse(html: &str) -> Self {
        let mut map = Self::new();
        let mut chars = html.chars().peekable();

        while let Some(&c) = chars.peek() {
            // Skip whitespace
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            // Parse attribute name
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '=' || c == '>' || c == '/' {
                    break;
                }
                name.push(c);
                chars.next();
            }

            if name.is_empty() {
                break;
            }

            // Skip whitespace
            while let Some(&c) = chars.peek() {
                if !c.is_whitespace() {
                    break;
                }
                chars.next();
            }

            // Check for =
            let value = if chars.peek() == Some(&'=') {
                chars.next(); // consume =

                // Skip whitespace
                while let Some(&c) = chars.peek() {
                    if !c.is_whitespace() {
                        break;
                    }
                    chars.next();
                }

                // Parse value
                match chars.peek() {
                    Some(&'"') => {
                        chars.next();
                        let mut value = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '"' {
                                chars.next();
                                break;
                            }
                            value.push(c);
                            chars.next();
                        }
                        value
                    }
                    Some(&'\'') => {
                        chars.next();
                        let mut value = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '\'' {
                                chars.next();
                                break;
                            }
                            value.push(c);
                            chars.next();
                        }
                        value
                    }
                    Some(_) => {
                        let mut value = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_whitespace() || c == '>' || c == '/' {
                                break;
                            }
                            value.push(c);
                            chars.next();
                        }
                        value
                    }
                    None => String::new(),
                }
            } else {
                // Boolean attribute
                String::new()
            };

            map.set(&name.to_ascii_lowercase(), &value);
        }

        map
    }

    /// Convert to HTML attribute string.
    pub fn to_html(&self) -> String {
        let mut result = String::new();
        for (name, value) in &self.attrs {
            if !result.is_empty() {
                result.push(' ');
            }
            if value.is_empty() {
                result.push_str(name);
            } else if value.contains('"') && !value.contains('\'') {
                result.push_str(&format!("{}='{}'", name, value));
            } else {
                result.push_str(&format!("{}=\"{}\"", name, html_escape(value)));
            }
        }
        result
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

/// Common data attributes.
pub struct DataAttributes<'a> {
    attrs: &'a AttributeMap,
}

impl<'a> DataAttributes<'a> {
    pub fn new(attrs: &'a AttributeMap) -> Self {
        Self { attrs }
    }

    /// Get a data attribute value.
    pub fn get(&self, name: &str) -> Option<&str> {
        let key = format!("data-{}", name.to_ascii_lowercase());
        self.attrs.get(&key)
    }

    /// Iterate over data attributes.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.attrs
            .iter()
            .filter(|(k, _)| k.starts_with("data-"))
            .map(|(k, v)| (&k[5..], v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_map() {
        let mut map = AttributeMap::new();
        map.set("id", "test");
        map.set("class", "foo bar");

        assert_eq!(map.get("id"), Some("test"));
        assert_eq!(map.get("class"), Some("foo bar"));
        assert!(map.contains("id"));
        assert!(!map.contains("style"));
    }

    #[test]
    fn test_parse_attributes() {
        let map = AttributeMap::parse(r#"id="test" class='foo bar' disabled data-value=123"#);
        assert_eq!(map.get("id"), Some("test"));
        assert_eq!(map.get("class"), Some("foo bar"));
        assert_eq!(map.get("disabled"), Some(""));
        assert_eq!(map.get("data-value"), Some("123"));
    }

    #[test]
    fn test_to_html() {
        let mut map = AttributeMap::new();
        map.set("id", "test");
        map.set("class", "foo");
        let html = map.to_html();
        assert!(html.contains("id=\"test\""));
        assert!(html.contains("class=\"foo\""));
    }
}
