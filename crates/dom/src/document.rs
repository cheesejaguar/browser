//! DOM Document implementation.

use crate::node::{NodeId, NodeData, DocumentType};
use crate::element::{ElementData, TagName};
use crate::tree::DomTree;
use parking_lot::RwLock;
use std::sync::Arc;
use url::Url;

/// Document ready state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyState {
    Loading,
    Interactive,
    Complete,
}

impl ReadyState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadyState::Loading => "loading",
            ReadyState::Interactive => "interactive",
            ReadyState::Complete => "complete",
        }
    }
}

/// Document content type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentType {
    Html,
    Xml,
    Xhtml,
}

/// Quirks mode for rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QuirksMode {
    #[default]
    NoQuirks,
    Quirks,
    LimitedQuirks,
}

/// DOM Document.
pub struct Document {
    /// The DOM tree.
    pub tree: DomTree,
    /// Document URL.
    pub url: Url,
    /// Base URL for resolving relative URLs.
    pub base_url: Url,
    /// Document title.
    pub title: String,
    /// Content type.
    pub content_type: ContentType,
    /// Character encoding.
    pub encoding: String,
    /// Ready state.
    pub ready_state: ReadyState,
    /// Quirks mode.
    pub quirks_mode: QuirksMode,
    /// Document element (<html>).
    pub document_element: Option<NodeId>,
    /// Head element.
    pub head: Option<NodeId>,
    /// Body element.
    pub body: Option<NodeId>,
    /// Active element (focused).
    pub active_element: Option<NodeId>,
    /// Stylesheets.
    pub stylesheets: Vec<StylesheetRef>,
    /// Scripts.
    pub scripts: Vec<ScriptRef>,
    /// Document is loading.
    pub loading: bool,
    /// Referrer.
    pub referrer: String,
    /// Last modified.
    pub last_modified: Option<String>,
    /// Cookie access.
    pub cookie: String,
    /// Domain.
    pub domain: String,
}

impl Document {
    pub fn new(url: Url) -> Self {
        let base_url = url.clone();
        let domain = url.host_str().unwrap_or("").to_string();

        let mut tree = DomTree::new();

        Self {
            tree,
            url,
            base_url,
            title: String::new(),
            content_type: ContentType::Html,
            encoding: "UTF-8".to_string(),
            ready_state: ReadyState::Loading,
            quirks_mode: QuirksMode::NoQuirks,
            document_element: None,
            head: None,
            body: None,
            active_element: None,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            loading: true,
            referrer: String::new(),
            last_modified: None,
            cookie: String::new(),
            domain,
        }
    }

    /// Create a blank document.
    pub fn blank() -> Self {
        Self::new(Url::parse("about:blank").unwrap())
    }

    /// Get document element.
    pub fn document_element(&self) -> Option<NodeId> {
        self.document_element
    }

    /// Get body element.
    pub fn body(&self) -> Option<NodeId> {
        self.body
    }

    /// Get head element.
    pub fn head(&self) -> Option<NodeId> {
        self.head
    }

    /// Create an element.
    pub fn create_element(&mut self, tag_name: &str) -> NodeId {
        let tag = TagName::new(tag_name);
        let data = ElementData::new(tag);
        self.tree.create_element(data)
    }

    /// Create a text node.
    pub fn create_text_node(&mut self, content: &str) -> NodeId {
        self.tree.create_text(content.to_string())
    }

    /// Create a comment node.
    pub fn create_comment(&mut self, content: &str) -> NodeId {
        self.tree.create_comment(content.to_string())
    }

    /// Create a document fragment.
    pub fn create_document_fragment(&mut self) -> NodeId {
        self.tree.create_document_fragment()
    }

    /// Get element by ID.
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.tree.find_element_by_id(id)
    }

    /// Get elements by tag name.
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
        self.tree.find_elements_by_tag_name(tag_name)
    }

    /// Get elements by class name.
    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<NodeId> {
        self.tree.find_elements_by_class_name(class_name)
    }

    /// Query selector.
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        self.tree.query_selector(selector)
    }

    /// Query selector all.
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        self.tree.query_selector_all(selector)
    }

    /// Set document title.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        // Also update <title> element if it exists
        if let Some(head) = self.head {
            if let Some(title_elem) = self.tree.find_child_by_tag(head, "title") {
                if let Some(text_node) = self.tree.first_child(title_elem) {
                    self.tree.set_text_content(text_node, title);
                }
            }
        }
    }

    /// Resolve a URL relative to the document.
    pub fn resolve_url(&self, url: &str) -> Result<Url, url::ParseError> {
        self.base_url.join(url)
    }

    /// Append child to body.
    pub fn append_to_body(&mut self, node: NodeId) {
        if let Some(body) = self.body {
            self.tree.append_child(body, node);
        }
    }

    /// Set the document's doctype.
    pub fn set_doctype(&mut self, name: &str, public_id: &str, system_id: &str) {
        let doctype = DocumentType {
            name: name.to_string(),
            public_id: public_id.to_string(),
            system_id: system_id.to_string(),
        };

        // Determine quirks mode based on doctype
        self.quirks_mode = Self::determine_quirks_mode(&doctype);
    }

    /// Determine quirks mode from doctype.
    fn determine_quirks_mode(doctype: &DocumentType) -> QuirksMode {
        let name = doctype.name.to_ascii_lowercase();
        let public_id = doctype.public_id.to_ascii_lowercase();
        let system_id = doctype.system_id.to_ascii_lowercase();

        // HTML5 doctype
        if name == "html" && public_id.is_empty() && system_id.is_empty() {
            return QuirksMode::NoQuirks;
        }

        // XHTML doctypes
        if public_id.contains("-//w3c//dtd xhtml") {
            return QuirksMode::NoQuirks;
        }

        // HTML 4.01 Strict
        if public_id == "-//w3c//dtd html 4.01//en"
            && (system_id.is_empty() || system_id == "http://www.w3.org/tr/html4/strict.dtd")
        {
            return QuirksMode::NoQuirks;
        }

        // Quirks mode triggers
        if public_id.starts_with("-//w3c//dtd html 4.01 transitional")
            && system_id.is_empty()
        {
            return QuirksMode::Quirks;
        }

        if public_id.is_empty() && system_id.is_empty() && name != "html" {
            return QuirksMode::Quirks;
        }

        // Limited quirks
        if public_id.starts_with("-//w3c//dtd html 4.01 transitional")
            || public_id.starts_with("-//w3c//dtd html 4.01 frameset")
        {
            return QuirksMode::LimitedQuirks;
        }

        QuirksMode::NoQuirks
    }

    /// Mark document as completely loaded.
    pub fn finish_loading(&mut self) {
        self.loading = false;
        self.ready_state = ReadyState::Complete;
    }

    /// Get all forms in document.
    pub fn forms(&self) -> Vec<NodeId> {
        self.get_elements_by_tag_name("form")
    }

    /// Get all images in document.
    pub fn images(&self) -> Vec<NodeId> {
        self.get_elements_by_tag_name("img")
    }

    /// Get all links in document.
    pub fn links(&self) -> Vec<NodeId> {
        let mut links = self.get_elements_by_tag_name("a");
        links.extend(self.get_elements_by_tag_name("area"));
        links
    }

    /// Get all anchors (named <a> elements).
    pub fn anchors(&self) -> Vec<NodeId> {
        self.get_elements_by_tag_name("a")
            .into_iter()
            .filter(|&id| {
                self.tree
                    .get_element(id)
                    .map(|e| e.has_attribute("name"))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Write HTML to document (document.write).
    pub fn write(&mut self, _html: &str) {
        // Implementation requires parser integration
        // TODO: Implement document.write
    }

    /// Open document for writing.
    pub fn open(&mut self) {
        // Clear document
        self.tree = DomTree::new();
        self.document_element = None;
        self.head = None;
        self.body = None;
        self.ready_state = ReadyState::Loading;
        self.loading = true;
    }

    /// Close document after writing.
    pub fn close(&mut self) {
        self.ready_state = ReadyState::Interactive;
    }
}

/// Reference to a stylesheet.
#[derive(Clone, Debug)]
pub struct StylesheetRef {
    pub href: Option<String>,
    pub content: Option<String>,
    pub media: String,
    pub disabled: bool,
}

/// Reference to a script.
#[derive(Clone, Debug)]
pub struct ScriptRef {
    pub src: Option<String>,
    pub content: Option<String>,
    pub script_type: String,
    pub async_: bool,
    pub defer: bool,
    pub module: bool,
}

/// Shared document reference.
pub type DocumentRef = Arc<RwLock<Document>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::blank();
        assert_eq!(doc.url.as_str(), "about:blank");
        assert_eq!(doc.ready_state, ReadyState::Loading);
    }

    #[test]
    fn test_quirks_mode_detection() {
        // HTML5
        let html5 = DocumentType {
            name: "html".to_string(),
            public_id: String::new(),
            system_id: String::new(),
        };
        assert_eq!(Document::determine_quirks_mode(&html5), QuirksMode::NoQuirks);

        // No doctype
        let none = DocumentType {
            name: String::new(),
            public_id: String::new(),
            system_id: String::new(),
        };
        assert_eq!(Document::determine_quirks_mode(&none), QuirksMode::Quirks);
    }
}
