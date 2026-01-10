//! HTML tokenizer utilities.
//!
//! This module provides additional tokenizer functionality beyond html5ever.

use std::collections::VecDeque;

/// HTML token types.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Character(char),
    EndOfFile,
}

/// Simple HTML tokenizer for basic operations.
pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
    tokens: VecDeque<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            tokens: VecDeque::new(),
        }
    }

    /// Get next token.
    pub fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.tokens.pop_front() {
            return Some(token);
        }

        self.scan_token()
    }

    fn scan_token(&mut self) -> Option<Token> {
        if self.pos >= self.input.len() {
            return None;
        }

        let remaining = &self.input[self.pos..];

        // Check for tag
        if remaining.starts_with('<') {
            if remaining.starts_with("<!--") {
                return self.scan_comment();
            } else if remaining.starts_with("<!DOCTYPE") || remaining.starts_with("<!doctype") {
                return self.scan_doctype();
            } else if remaining.starts_with("</") {
                return self.scan_end_tag();
            } else if remaining.starts_with("<!")
                || remaining.starts_with("<?")
                || remaining.starts_with("<![CDATA[")
            {
                // Skip special sequences
                if let Some(end) = remaining.find('>') {
                    self.pos += end + 1;
                    return self.next_token();
                }
            } else {
                return self.scan_start_tag();
            }
        }

        // Text content
        self.scan_text()
    }

    fn scan_comment(&mut self) -> Option<Token> {
        let start = self.pos + 4; // Skip "<!--"
        let remaining = &self.input[start..];

        if let Some(end) = remaining.find("-->") {
            let content = &remaining[..end];
            self.pos = start + end + 3;
            Some(Token::Comment(content.to_string()))
        } else {
            // Unclosed comment - take rest of input
            let content = remaining.to_string();
            self.pos = self.input.len();
            Some(Token::Comment(content))
        }
    }

    fn scan_doctype(&mut self) -> Option<Token> {
        let remaining = &self.input[self.pos..];

        if let Some(end) = remaining.find('>') {
            let content = &remaining[9..end]; // Skip "<!DOCTYPE"

            // Parse doctype content
            let parts: Vec<&str> = content.split_whitespace().collect();
            let name = parts.first().map(|s| s.to_string());

            let (public_id, system_id) = if parts.len() >= 4 && parts[1].eq_ignore_ascii_case("PUBLIC") {
                let pub_id = parts.get(2).map(|s| s.trim_matches('"').to_string());
                let sys_id = parts.get(3).map(|s| s.trim_matches('"').to_string());
                (pub_id, sys_id)
            } else if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("SYSTEM") {
                let sys_id = parts.get(2).map(|s| s.trim_matches('"').to_string());
                (None, sys_id)
            } else {
                (None, None)
            };

            self.pos += end + 1;
            Some(Token::Doctype {
                name,
                public_id,
                system_id,
                force_quirks: false,
            })
        } else {
            self.pos = self.input.len();
            None
        }
    }

    fn scan_start_tag(&mut self) -> Option<Token> {
        let start = self.pos + 1; // Skip "<"
        let remaining = &self.input[start..];

        if let Some(end) = remaining.find('>') {
            let tag_content = &remaining[..end];
            let self_closing = tag_content.ends_with('/');
            let tag_content = tag_content.trim_end_matches('/');

            // Split into name and attributes
            let mut parts = tag_content.split_whitespace();
            let name = parts.next().unwrap_or("").to_ascii_lowercase();

            let mut attributes = Vec::new();
            let attr_str = tag_content[name.len()..].trim();
            attributes = self.parse_attributes(attr_str);

            self.pos = start + end + 1;
            Some(Token::StartTag {
                name,
                attributes,
                self_closing,
            })
        } else {
            self.pos = self.input.len();
            None
        }
    }

    fn scan_end_tag(&mut self) -> Option<Token> {
        let start = self.pos + 2; // Skip "</"
        let remaining = &self.input[start..];

        if let Some(end) = remaining.find('>') {
            let name = remaining[..end].trim().to_ascii_lowercase();
            self.pos = start + end + 1;
            Some(Token::EndTag { name })
        } else {
            self.pos = self.input.len();
            None
        }
    }

    fn scan_text(&mut self) -> Option<Token> {
        let remaining = &self.input[self.pos..];

        // Find next tag
        let end = remaining.find('<').unwrap_or(remaining.len());

        if end > 0 {
            let c = remaining.chars().next()?;
            self.pos += c.len_utf8();
            Some(Token::Character(c))
        } else {
            None
        }
    }

    fn parse_attributes(&self, attr_str: &str) -> Vec<(String, String)> {
        let mut attributes = Vec::new();
        let mut chars = attr_str.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                continue;
            }

            // Parse attribute name
            let mut name = String::new();
            name.push(c);
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '=' || c == '/' || c == '>' {
                    break;
                }
                name.push(c);
                chars.next();
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
                            if c.is_whitespace() || c == '/' || c == '>' {
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
                String::new() // Boolean attribute
            };

            if !name.is_empty() {
                attributes.push((name.to_ascii_lowercase(), value));
            }
        }

        attributes
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// Extract text content from HTML.
pub fn extract_text(html: &str) -> String {
    let mut result = String::new();
    let mut in_script = false;
    let mut in_style = false;

    let tokenizer = Tokenizer::new(html);

    for token in tokenizer {
        match token {
            Token::StartTag { name, .. } => {
                if name == "script" {
                    in_script = true;
                } else if name == "style" {
                    in_style = true;
                }
            }
            Token::EndTag { name } => {
                if name == "script" {
                    in_script = false;
                } else if name == "style" {
                    in_style = false;
                }
            }
            Token::Character(c) if !in_script && !in_style => {
                result.push(c);
            }
            _ => {}
        }
    }

    result
}

/// Strip HTML tags from content.
pub fn strip_tags(html: &str) -> String {
    extract_text(html)
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let html = "<div>Hello</div>";
        let mut tokenizer = Tokenizer::new(html);

        let token = tokenizer.next_token().unwrap();
        assert!(matches!(token, Token::StartTag { name, .. } if name == "div"));

        let token = tokenizer.next_token().unwrap();
        assert!(matches!(token, Token::Character('H')));
    }

    #[test]
    fn test_extract_text() {
        let html = "<p>Hello <b>World</b></p>";
        let text = extract_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_strip_tags() {
        let html = "<div>  Hello  <span>World</span>  </div>";
        let text = strip_tags(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }
}
