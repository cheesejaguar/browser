//! HTML Parser implementation.

use crate::tree_builder::DomTreeSink;
use dom::document::Document;
use dom::element::{ElementData, TagName};
use dom::node::NodeId;
use dom::tree::DomTree;
use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, parse_fragment, QualName};
use std::default::Default;
use url::Url;

/// Parser options.
#[derive(Clone, Debug)]
pub struct ParseOptions {
    /// Document URL.
    pub url: Url,
    /// Whether to run scripts.
    pub scripting_enabled: bool,
    /// Whether this is a fragment parse.
    pub fragment: bool,
    /// Context element for fragment parsing.
    pub context_tag: Option<String>,
    /// Whether to preserve whitespace.
    pub preserve_whitespace: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            url: Url::parse("about:blank").unwrap(),
            scripting_enabled: true,
            fragment: false,
            context_tag: None,
            preserve_whitespace: false,
        }
    }
}

impl ParseOptions {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    pub fn fragment(mut self) -> Self {
        self.fragment = true;
        self
    }

    pub fn with_context(mut self, tag: &str) -> Self {
        self.context_tag = Some(tag.to_string());
        self
    }

    pub fn scripting(mut self, enabled: bool) -> Self {
        self.scripting_enabled = enabled;
        self
    }
}

/// HTML Parser.
pub struct HtmlParser {
    options: ParseOptions,
}

impl HtmlParser {
    pub fn new(options: ParseOptions) -> Self {
        Self { options }
    }

    /// Parse HTML string into a Document.
    pub fn parse(&self, html: &str) -> Document {
        let mut document = Document::new(self.options.url.clone());

        let sink = DomTreeSink::new(&mut document);

        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                scripting_enabled: self.options.scripting_enabled,
                ..Default::default()
            },
            ..Default::default()
        };

        parse_document(sink, opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        // Find and set special elements
        Self::find_special_elements(&mut document);

        document
    }

    /// Parse HTML fragment.
    pub fn parse_fragment(&self, html: &str, context_tag: &str) -> Vec<NodeId> {
        let mut document = Document::new(self.options.url.clone());
        let sink = DomTreeSink::new(&mut document);

        let context = QualName::new(
            None,
            html5ever::ns!(html),
            html5ever::LocalName::from(context_tag),
        );

        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                scripting_enabled: self.options.scripting_enabled,
                ..Default::default()
            },
            ..Default::default()
        };

        parse_fragment(sink, opts, context, vec![])
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        // Return children of root
        document
            .tree
            .root()
            .map(|root| document.tree.children(root).collect())
            .unwrap_or_default()
    }

    fn find_special_elements(document: &mut Document) {
        // Find <html>, <head>, <body> elements
        if let Some(root) = document.tree.root() {
            for child in document.tree.children(root).collect::<Vec<_>>() {
                if let Some(elem) = document.tree.get_element(child) {
                    if elem.tag_name == "html" {
                        document.document_element = Some(child);

                        // Find head and body
                        for html_child in document.tree.children(child).collect::<Vec<_>>() {
                            if let Some(elem) = document.tree.get_element(html_child) {
                                if elem.tag_name == "head" {
                                    document.head = Some(html_child);

                                    // Find title
                                    for head_child in
                                        document.tree.children(html_child).collect::<Vec<_>>()
                                    {
                                        if let Some(elem) = document.tree.get_element(head_child) {
                                            if elem.tag_name == "title" {
                                                document.title =
                                                    document.tree.get_text_content(head_child);
                                            }
                                        }
                                    }
                                } else if elem.tag_name == "body" {
                                    document.body = Some(html_child);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Parse HTML string into a Document.
pub fn parse_html(html: &str, url: Url) -> Document {
    let parser = HtmlParser::new(ParseOptions::new(url));
    parser.parse(html)
}

/// Parse HTML fragment.
pub fn parse_html_fragment(html: &str, context_tag: &str) -> Vec<NodeId> {
    let parser = HtmlParser::new(ParseOptions::default().fragment());
    parser.parse_fragment(html, context_tag)
}

/// Incremental parser for streaming HTML.
pub struct IncrementalParser {
    buffer: String,
    document: Document,
    complete: bool,
}

impl IncrementalParser {
    pub fn new(url: Url) -> Self {
        Self {
            buffer: String::new(),
            document: Document::new(url),
            complete: false,
        }
    }

    /// Add chunk of HTML.
    pub fn write(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
    }

    /// Finish parsing.
    pub fn finish(mut self) -> Document {
        let parser = HtmlParser::new(ParseOptions::new(self.document.url.clone()));
        parser.parse(&self.buffer)
    }

    /// Check if parser is complete.
    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><p>Hello World</p></body>
</html>"#;

        let doc = parse_html(html, Url::parse("about:blank").unwrap());
        assert!(doc.document_element.is_some());
        assert!(doc.head.is_some());
        assert!(doc.body.is_some());
        assert_eq!(doc.title, "Test");
    }

    #[test]
    fn test_parse_fragment() {
        let html = "<div><span>Test</span></div>";
        let nodes = parse_html_fragment(html, "body");
        assert!(!nodes.is_empty());
    }
}
