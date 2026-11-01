//! Tree builder sink for html5ever.

use dom::document::Document;
use dom::element::{ElementData, TagName};
use dom::node::{DocumentType, NodeData, NodeId};
use html5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::tendril::StrTendril;
use html5ever::{Attribute, ExpandedName, QualName};
use markup5ever::namespace_url;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Handle for nodes in the tree sink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle(pub NodeId);

/// Tree sink implementation for building our DOM.
pub struct DomTreeSink<'a> {
    document: &'a mut Document,
    /// Nodes that have been removed but might be re-parented.
    pending_nodes: HashSet<NodeId>,
    /// Qualified names of elements created by this sink, so that
    /// `elem_name` can hand out references that outlive the call.
    element_names: HashMap<NodeId, QualName>,
    /// Name reported for nodes that are not elements created by this sink.
    fallback_name: QualName,
}

impl<'a> DomTreeSink<'a> {
    pub fn new(document: &'a mut Document) -> Self {
        Self {
            document,
            pending_nodes: HashSet::new(),
            element_names: HashMap::new(),
            fallback_name: QualName::new(None, html5ever::ns!(), html5ever::LocalName::from("")),
        }
    }

    fn make_element(&mut self, name: &QualName, attrs: Vec<Attribute>) -> NodeId {
        let tag = TagName::new(name.local.as_ref());
        let mut data = ElementData::new(tag);

        // Set namespace if not HTML
        if name.ns != html5ever::ns!(html) {
            data.namespace = Some(name.ns.to_string().into());
        }

        // Set attributes
        for attr in attrs {
            data.set_attribute(attr.name.local.as_ref(), &attr.value);
        }

        let id = self.document.tree.create_element(data);
        self.element_names.insert(id, name.clone());
        id
    }
}

impl<'a> TreeSink for DomTreeSink<'a> {
    type Handle = Handle;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        tracing::warn!("HTML parse error: {}", msg);
    }

    fn get_document(&mut self) -> Self::Handle {
        Handle(self.document.tree.root().unwrap())
    }

    fn elem_name<'b>(&'b self, target: &'b Self::Handle) -> ExpandedName<'b> {
        self.element_names
            .get(&target.0)
            .unwrap_or(&self.fallback_name)
            .expanded()
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let id = self.make_element(&name, attrs);
        Handle(id)
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let id = self.document.tree.create_comment(text.to_string());
        Handle(id)
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Self::Handle {
        // Processing instructions - treat as comments in HTML
        let content = format!("{} {}", target, data);
        let id = self.document.tree.create_comment(content);
        Handle(id)
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(handle) => {
                self.pending_nodes.remove(&handle.0);
                self.document.tree.append_child(parent.0, handle.0);
            }
            NodeOrText::AppendText(text) => {
                // Check if last child is text and append to it
                if let Some(last) = self.document.tree.last_child(parent.0) {
                    if let Some(node) = self.document.tree.get(last) {
                        if let NodeData::Text { content } = &node.data {
                            let mut new_content = content.clone();
                            new_content.push_str(&text);
                            self.document.tree.set_text_content(last, &new_content);
                            return;
                        }
                    }
                }

                let id = self.document.tree.create_text(text.to_string());
                self.document.tree.append_child(parent.0, id);
            }
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if self.document.tree.parent(element.0).is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        self.document.set_doctype(&name, &public_id, &system_id);
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        // Return template content (for now, just return the element itself)
        *target
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.0 == y.0
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.document.quirks_mode = match mode {
            QuirksMode::Quirks => dom::document::QuirksMode::Quirks,
            QuirksMode::LimitedQuirks => dom::document::QuirksMode::LimitedQuirks,
            QuirksMode::NoQuirks => dom::document::QuirksMode::NoQuirks,
        };
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        if let Some(parent) = self.document.tree.parent(sibling.0) {
            match new_node {
                NodeOrText::AppendNode(handle) => {
                    self.pending_nodes.remove(&handle.0);
                    self.document
                        .tree
                        .insert_before(parent, handle.0, Some(sibling.0));
                }
                NodeOrText::AppendText(text) => {
                    let id = self.document.tree.create_text(text.to_string());
                    self.document.tree.insert_before(parent, id, Some(sibling.0));
                }
            }
        }
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        if let Some(elem) = self.document.tree.get_element_mut(target.0) {
            for attr in attrs {
                if !elem.has_attribute(attr.name.local.as_ref()) {
                    elem.set_attribute(attr.name.local.as_ref(), &attr.value);
                }
            }
        }
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        self.document.tree.remove_from_parent(target.0);
        self.pending_nodes.insert(target.0);
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        let children: Vec<NodeId> = self.document.tree.children(node.0).collect();
        for child in children {
            self.document.tree.remove_from_parent(child);
            self.document.tree.append_child(new_parent.0, child);
        }
    }

    fn mark_script_already_started(&mut self, _target: &Self::Handle) {
        // Mark script as already started (for document.write handling)
    }

    fn pop(&mut self, _node: &Self::Handle) {
        // Node popped from open elements stack
    }

    fn associate_with_form(
        &mut self,
        _target: &Self::Handle,
        _form: &Self::Handle,
        _nodes: (&Self::Handle, Option<&Self::Handle>),
    ) {
        // Associate form element with form
    }

    fn is_mathml_annotation_xml_integration_point(&self, _target: &Self::Handle) -> bool {
        false
    }

    fn set_current_line(&mut self, _line_number: u64) {
        // Set current line for error reporting
    }

    fn complete_script(&mut self, _node: &Self::Handle) -> html5ever::tree_builder::NextParserState {
        html5ever::tree_builder::NextParserState::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_tree_sink() {
        let mut doc = Document::new(Url::parse("about:blank").unwrap());
        let sink = DomTreeSink::new(&mut doc);

        // Basic functionality is tested through the parser
        assert!(doc.tree.root().is_some());
    }
}
