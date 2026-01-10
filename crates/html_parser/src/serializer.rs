//! HTML serialization.

use dom::document::Document;
use dom::element::ElementData;
use dom::node::{NodeData, NodeId, NodeType};
use dom::tree::DomTree;

/// Options for HTML serialization.
#[derive(Clone, Debug, Default)]
pub struct SerializeOptions {
    /// Pretty print with indentation.
    pub pretty: bool,
    /// Indent string.
    pub indent: String,
    /// Include doctype.
    pub include_doctype: bool,
    /// Escape text content.
    pub escape_text: bool,
}

impl SerializeOptions {
    pub fn new() -> Self {
        Self {
            pretty: false,
            indent: "  ".to_string(),
            include_doctype: true,
            escape_text: true,
        }
    }

    pub fn pretty(mut self) -> Self {
        self.pretty = true;
        self
    }
}

/// Serialize a document to HTML string.
pub fn serialize_html(document: &Document) -> String {
    serialize_html_with_options(document, &SerializeOptions::new())
}

/// Serialize a document to HTML string with options.
pub fn serialize_html_with_options(document: &Document, options: &SerializeOptions) -> String {
    let mut output = String::new();

    if options.include_doctype {
        output.push_str("<!DOCTYPE html>\n");
    }

    if let Some(root) = document.tree.root() {
        serialize_children(&document.tree, root, &mut output, options, 0);
    }

    output
}

/// Serialize a node and its subtree.
pub fn serialize_node(tree: &DomTree, node: NodeId) -> String {
    let options = SerializeOptions::new();
    let mut output = String::new();
    serialize_node_internal(tree, node, &mut output, &options, 0);
    output
}

/// Serialize outer HTML of a node.
pub fn serialize_outer_html(tree: &DomTree, node: NodeId) -> String {
    serialize_node(tree, node)
}

/// Serialize inner HTML of a node.
pub fn serialize_inner_html(tree: &DomTree, node: NodeId) -> String {
    let options = SerializeOptions::new();
    let mut output = String::new();
    serialize_children(tree, node, &mut output, &options, 0);
    output
}

fn serialize_node_internal(
    tree: &DomTree,
    node: NodeId,
    output: &mut String,
    options: &SerializeOptions,
    depth: usize,
) {
    let node_data = match tree.get(node) {
        Some(n) => n,
        None => return,
    };

    match &node_data.data {
        NodeData::Document { .. } => {
            serialize_children(tree, node, output, options, depth);
        }
        NodeData::DocumentFragment => {
            serialize_children(tree, node, output, options, depth);
        }
        NodeData::Element(elem) => {
            serialize_element(tree, node, elem, output, options, depth);
        }
        NodeData::Text { content } => {
            if options.escape_text {
                output.push_str(&escape_html_text(content));
            } else {
                output.push_str(content);
            }
        }
        NodeData::Comment { content } => {
            if options.pretty {
                add_indent(output, options, depth);
            }
            output.push_str("<!--");
            output.push_str(content);
            output.push_str("-->");
            if options.pretty {
                output.push('\n');
            }
        }
        NodeData::DocumentType(dt) => {
            output.push_str("<!DOCTYPE ");
            output.push_str(&dt.name);
            if !dt.public_id.is_empty() {
                output.push_str(" PUBLIC \"");
                output.push_str(&dt.public_id);
                output.push('"');
            }
            if !dt.system_id.is_empty() {
                if dt.public_id.is_empty() {
                    output.push_str(" SYSTEM");
                }
                output.push_str(" \"");
                output.push_str(&dt.system_id);
                output.push('"');
            }
            output.push('>');
            if options.pretty {
                output.push('\n');
            }
        }
        NodeData::ProcessingInstruction { target, data } => {
            output.push_str("<?");
            output.push_str(target);
            if !data.is_empty() {
                output.push(' ');
                output.push_str(data);
            }
            output.push_str("?>");
            if options.pretty {
                output.push('\n');
            }
        }
    }
}

fn serialize_element(
    tree: &DomTree,
    node: NodeId,
    elem: &ElementData,
    output: &mut String,
    options: &SerializeOptions,
    depth: usize,
) {
    let tag_name = elem.tag_name.as_str();

    if options.pretty {
        add_indent(output, options, depth);
    }

    // Start tag
    output.push('<');
    output.push_str(tag_name);

    // Attributes
    for (name, value) in elem.attributes.iter() {
        output.push(' ');
        output.push_str(name);
        if !value.is_empty() {
            output.push_str("=\"");
            output.push_str(&escape_html_attribute(value));
            output.push('"');
        }
    }

    // Void elements
    if elem.is_void() {
        output.push('>');
        if options.pretty {
            output.push('\n');
        }
        return;
    }

    output.push('>');

    // Children
    let has_children = tree.get(node).map(|n| !n.children.is_empty()).unwrap_or(false);

    if has_children {
        // Check if children are only text
        let only_text = tree.get(node)
            .map(|n| {
                n.children.len() == 1
                    && tree.get(n.children[0])
                        .map(|c| c.node_type == NodeType::Text)
                        .unwrap_or(false)
            })
            .unwrap_or(false);

        if options.pretty && !only_text {
            output.push('\n');
        }

        // Raw text elements (script, style) don't escape content
        let escape_children = !matches!(tag_name, "script" | "style" | "textarea");

        let child_options = if escape_children {
            options.clone()
        } else {
            let mut opts = options.clone();
            opts.escape_text = false;
            opts
        };

        serialize_children(tree, node, output, &child_options, depth + 1);

        if options.pretty && !only_text {
            add_indent(output, options, depth);
        }
    }

    // End tag
    output.push_str("</");
    output.push_str(tag_name);
    output.push('>');

    if options.pretty {
        output.push('\n');
    }
}

fn serialize_children(
    tree: &DomTree,
    node: NodeId,
    output: &mut String,
    options: &SerializeOptions,
    depth: usize,
) {
    if let Some(node_data) = tree.get(node) {
        for &child in &node_data.children {
            serialize_node_internal(tree, child, output, options, depth);
        }
    }
}

fn add_indent(output: &mut String, options: &SerializeOptions, depth: usize) {
    for _ in 0..depth {
        output.push_str(&options.indent);
    }
}

/// Escape HTML text content.
pub fn escape_html_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape HTML attribute value.
pub fn escape_html_attribute(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
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

/// Unescape HTML entities.
pub fn unescape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            while let Some(&c) = chars.peek() {
                if c == ';' {
                    chars.next();
                    break;
                }
                if !c.is_alphanumeric() && c != '#' {
                    break;
                }
                entity.push(c);
                chars.next();
            }

            let decoded = decode_entity(&entity);
            result.push_str(&decoded);
        } else {
            result.push(c);
        }
    }

    result
}

fn decode_entity(entity: &str) -> String {
    match entity {
        "amp" => "&".to_string(),
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "quot" => "\"".to_string(),
        "apos" => "'".to_string(),
        "nbsp" => "\u{00A0}".to_string(),
        "copy" => "©".to_string(),
        "reg" => "®".to_string(),
        "trade" => "™".to_string(),
        "mdash" => "—".to_string(),
        "ndash" => "–".to_string(),
        "hellip" => "…".to_string(),
        "bull" => "•".to_string(),
        "euro" => "€".to_string(),
        "pound" => "£".to_string(),
        "yen" => "¥".to_string(),
        "cent" => "¢".to_string(),
        e if e.starts_with('#') => {
            // Numeric entity
            let num_str = &e[1..];
            let code = if num_str.starts_with('x') || num_str.starts_with('X') {
                u32::from_str_radix(&num_str[1..], 16).ok()
            } else {
                num_str.parse().ok()
            };
            code.and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_else(|| format!("&{};", entity))
        }
        _ => format!("&{};", entity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_html;
    use url::Url;

    #[test]
    fn test_serialize_simple() {
        let html = "<html><head></head><body><p>Hello</p></body></html>";
        let doc = parse_html(html, Url::parse("about:blank").unwrap());
        let output = serialize_html(&doc);
        assert!(output.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html_text("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html_text("a & b"), "a &amp; b");
    }

    #[test]
    fn test_unescape_html() {
        assert_eq!(unescape_html("&lt;script&gt;"), "<script>");
        assert_eq!(unescape_html("&#60;"), "<");
        assert_eq!(unescape_html("&#x3C;"), "<");
    }
}
