//! DOM Tree implementation.

use crate::element::{ElementData, TagName};
use crate::node::{Node, NodeData, NodeId, NodeType};
use parking_lot::RwLock;
use slotmap::SlotMap;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;

/// The DOM tree structure.
pub struct DomTree {
    /// All nodes in the tree.
    nodes: SlotMap<NodeId, Node>,
    /// Root node (document).
    root: Option<NodeId>,
    /// ID to node mapping for fast lookups.
    id_map: HashMap<String, NodeId>,
}

impl DomTree {
    pub fn new() -> Self {
        let mut tree = Self {
            nodes: SlotMap::with_key(),
            root: None,
            id_map: HashMap::new(),
        };
        // Create document node
        let root_id = tree.nodes.insert_with_key(|id| Node::new_document(id));
        tree.root = Some(root_id);
        tree
    }

    /// Get the root document node.
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Get a node by ID.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Get element data for a node.
    pub fn get_element(&self, id: NodeId) -> Option<&ElementData> {
        self.nodes.get(id).and_then(|n| n.as_element())
    }

    /// Get mutable element data for a node.
    pub fn get_element_mut(&mut self, id: NodeId) -> Option<&mut ElementData> {
        self.nodes.get_mut(id).and_then(|n| n.as_element_mut())
    }

    /// Create an element node.
    pub fn create_element(&mut self, data: ElementData) -> NodeId {
        let id_value = data.id.clone();
        let id = self
            .nodes
            .insert_with_key(|id| Node::new_element(id, data));

        // Update ID map
        if let Some(elem_id) = id_value {
            self.id_map.insert(elem_id.to_string(), id);
        }

        id
    }

    /// Create a text node.
    pub fn create_text(&mut self, content: String) -> NodeId {
        self.nodes
            .insert_with_key(|id| Node::new_text(id, content))
    }

    /// Create a comment node.
    pub fn create_comment(&mut self, content: String) -> NodeId {
        self.nodes
            .insert_with_key(|id| Node::new_comment(id, content))
    }

    /// Create a document fragment.
    pub fn create_document_fragment(&mut self) -> NodeId {
        self.nodes
            .insert_with_key(|id| Node::new_document_fragment(id))
    }

    /// Append a child to a parent node.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        // Remove from old parent if any
        self.remove_from_parent(child);

        // Update sibling links
        if let Some(parent_node) = self.nodes.get(parent) {
            if let Some(last_child) = parent_node.last_child() {
                if let Some(last) = self.nodes.get_mut(last_child) {
                    last.next_sibling = Some(child);
                }
                if let Some(child_node) = self.nodes.get_mut(child) {
                    child_node.prev_sibling = Some(last_child);
                }
            }
        }

        // Add to parent
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.children.push(child);
        }

        if let Some(child_node) = self.nodes.get_mut(child) {
            child_node.parent = Some(parent);
            child_node.next_sibling = None;
        }

        // Update ID map if needed
        self.update_id_map(child);
    }

    /// Insert a child before a reference node.
    pub fn insert_before(&mut self, parent: NodeId, child: NodeId, reference: Option<NodeId>) {
        match reference {
            None => self.append_child(parent, child),
            Some(ref_id) => {
                self.remove_from_parent(child);

                // Find position in parent's children
                if let Some(parent_node) = self.nodes.get_mut(parent) {
                    if let Some(pos) = parent_node.children.iter().position(|&id| id == ref_id) {
                        parent_node.children.insert(pos, child);
                    }
                }

                // Update sibling links
                if let Some(ref_node) = self.nodes.get(ref_id) {
                    let prev = ref_node.prev_sibling;

                    if let Some(prev_id) = prev {
                        if let Some(prev_node) = self.nodes.get_mut(prev_id) {
                            prev_node.next_sibling = Some(child);
                        }
                    }

                    if let Some(child_node) = self.nodes.get_mut(child) {
                        child_node.parent = Some(parent);
                        child_node.prev_sibling = prev;
                        child_node.next_sibling = Some(ref_id);
                    }

                    if let Some(ref_node) = self.nodes.get_mut(ref_id) {
                        ref_node.prev_sibling = Some(child);
                    }
                }

                self.update_id_map(child);
            }
        }
    }

    /// Remove a node from its parent.
    pub fn remove_from_parent(&mut self, node: NodeId) {
        let (parent, prev, next) = {
            let node_data = match self.nodes.get(node) {
                Some(n) => n,
                None => return,
            };
            (node_data.parent, node_data.prev_sibling, node_data.next_sibling)
        };

        // Update parent's children list
        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_id) {
                parent_node.children.retain(|&id| id != node);
            }
        }

        // Update sibling links
        if let Some(prev_id) = prev {
            if let Some(prev_node) = self.nodes.get_mut(prev_id) {
                prev_node.next_sibling = next;
            }
        }
        if let Some(next_id) = next {
            if let Some(next_node) = self.nodes.get_mut(next_id) {
                next_node.prev_sibling = prev;
            }
        }

        // Clear node's parent/sibling refs
        if let Some(node_data) = self.nodes.get_mut(node) {
            node_data.parent = None;
            node_data.prev_sibling = None;
            node_data.next_sibling = None;
        }
    }

    /// Remove a node and its subtree from the tree.
    pub fn remove(&mut self, node: NodeId) {
        // First remove from parent
        self.remove_from_parent(node);

        // Collect all descendant IDs
        let mut to_remove = vec![node];
        let mut i = 0;
        while i < to_remove.len() {
            if let Some(n) = self.nodes.get(to_remove[i]) {
                to_remove.extend(n.children.iter().copied());
            }
            i += 1;
        }

        // Remove from ID map
        for &id in &to_remove {
            if let Some(node) = self.nodes.get(id) {
                if let Some(elem) = node.as_element() {
                    if let Some(elem_id) = &elem.id {
                        self.id_map.remove(elem_id.as_ref());
                    }
                }
            }
        }

        // Remove nodes
        for id in to_remove {
            self.nodes.remove(id);
        }
    }

    /// Replace a node with another.
    pub fn replace_child(&mut self, parent: NodeId, new_child: NodeId, old_child: NodeId) {
        self.insert_before(parent, new_child, Some(old_child));
        self.remove(old_child);
    }

    /// Clone a node (optionally deep).
    pub fn clone_node(&mut self, node: NodeId, deep: bool) -> Option<NodeId> {
        let node_data = self.nodes.get(node)?.clone();

        let new_id = self.nodes.insert_with_key(|id| {
            let mut new_node = node_data.clone();
            new_node.id = id;
            new_node.parent = None;
            new_node.prev_sibling = None;
            new_node.next_sibling = None;
            new_node.children = SmallVec::new();
            new_node
        });

        if deep {
            let children: SmallVec<[NodeId; 8]> = self.nodes.get(node)
                .map(|n| n.children.clone())
                .unwrap_or_default();

            for child in children {
                if let Some(cloned_child) = self.clone_node(child, true) {
                    self.append_child(new_id, cloned_child);
                }
            }
        }

        Some(new_id)
    }

    /// Get parent node.
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.parent)
    }

    /// Get first child.
    pub fn first_child(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.first_child())
    }

    /// Get last child.
    pub fn last_child(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.last_child())
    }

    /// Get previous sibling.
    pub fn prev_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.prev_sibling)
    }

    /// Get next sibling.
    pub fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.next_sibling)
    }

    /// Get all children.
    pub fn children(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes
            .get(node)
            .into_iter()
            .flat_map(|n| n.children.iter().copied())
    }

    /// Get ancestors.
    pub fn ancestors(&self, node: NodeId) -> AncestorIterator<'_> {
        AncestorIterator {
            tree: self,
            current: self.parent(node),
        }
    }

    /// Get descendants (pre-order).
    pub fn descendants(&self, node: NodeId) -> DescendantIterator<'_> {
        let mut stack = Vec::new();
        if let Some(n) = self.nodes.get(node) {
            for &child in n.children.iter().rev() {
                stack.push(child);
            }
        }
        DescendantIterator { tree: self, stack }
    }

    /// Find element by ID.
    pub fn find_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.id_map.get(id).copied()
    }

    /// Find elements by tag name.
    pub fn find_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
        let tag_lower = tag_name.to_ascii_lowercase();
        let is_all = tag_name == "*";

        let root = match self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        self.descendants(root)
            .filter(|&id| {
                if let Some(elem) = self.get_element(id) {
                    is_all || elem.tag_name.as_str() == tag_lower
                } else {
                    false
                }
            })
            .collect()
    }

    /// Find elements by class name.
    pub fn find_elements_by_class_name(&self, class_name: &str) -> Vec<NodeId> {
        let classes: Vec<&str> = class_name.split_whitespace().collect();
        if classes.is_empty() {
            return Vec::new();
        }

        let root = match self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        self.descendants(root)
            .filter(|&id| {
                if let Some(elem) = self.get_element(id) {
                    classes.iter().all(|c| elem.has_class(c))
                } else {
                    false
                }
            })
            .collect()
    }

    /// Find child by tag name.
    pub fn find_child_by_tag(&self, parent: NodeId, tag_name: &str) -> Option<NodeId> {
        let tag_lower = tag_name.to_ascii_lowercase();
        self.children(parent).find(|&id| {
            self.get_element(id)
                .map(|e| e.tag_name.as_str() == tag_lower)
                .unwrap_or(false)
        })
    }

    /// Query selector (basic implementation).
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        self.query_selector_all(selector).into_iter().next()
    }

    /// Query selector all (basic implementation).
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        let root = match self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        // Simple selector parsing
        let selector = selector.trim();

        if selector.starts_with('#') {
            // ID selector
            let id = &selector[1..];
            self.find_element_by_id(id).into_iter().collect()
        } else if selector.starts_with('.') {
            // Class selector
            let class = &selector[1..];
            self.find_elements_by_class_name(class)
        } else if selector.starts_with('[') && selector.ends_with(']') {
            // Attribute selector
            let attr = &selector[1..selector.len()-1];
            self.find_elements_by_attribute(attr)
        } else {
            // Tag selector (might include class/id)
            if let Some(dot_pos) = selector.find('.') {
                let tag = &selector[..dot_pos];
                let class = &selector[dot_pos+1..];
                self.find_elements_by_tag_name(tag)
                    .into_iter()
                    .filter(|&id| {
                        self.get_element(id)
                            .map(|e| e.has_class(class))
                            .unwrap_or(false)
                    })
                    .collect()
            } else if let Some(hash_pos) = selector.find('#') {
                let tag = &selector[..hash_pos];
                let id = &selector[hash_pos+1..];
                self.find_element_by_id(id)
                    .filter(|&node_id| {
                        self.get_element(node_id)
                            .map(|e| tag.is_empty() || e.tag_name.as_str() == tag)
                            .unwrap_or(false)
                    })
                    .into_iter()
                    .collect()
            } else {
                self.find_elements_by_tag_name(selector)
            }
        }
    }

    /// Find elements by attribute.
    fn find_elements_by_attribute(&self, attr_selector: &str) -> Vec<NodeId> {
        let root = match self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        // Parse attribute selector: name, name=value, name^=value, etc.
        let (name, op, value) = if let Some(eq_pos) = attr_selector.find('=') {
            let (name_part, value_part) = attr_selector.split_at(eq_pos);
            let value = value_part[1..].trim_matches(|c| c == '"' || c == '\'');

            if name_part.ends_with('^') {
                (&name_part[..name_part.len()-1], Some("^="), Some(value))
            } else if name_part.ends_with('$') {
                (&name_part[..name_part.len()-1], Some("$="), Some(value))
            } else if name_part.ends_with('*') {
                (&name_part[..name_part.len()-1], Some("*="), Some(value))
            } else if name_part.ends_with('~') {
                (&name_part[..name_part.len()-1], Some("~="), Some(value))
            } else if name_part.ends_with('|') {
                (&name_part[..name_part.len()-1], Some("|="), Some(value))
            } else {
                (name_part, Some("="), Some(value))
            }
        } else {
            (attr_selector, None, None)
        };

        self.descendants(root)
            .filter(|&id| {
                if let Some(elem) = self.get_element(id) {
                    match (op, value) {
                        (None, _) => elem.has_attribute(name),
                        (Some("="), Some(v)) => elem.get_attribute(name) == Some(v),
                        (Some("^="), Some(v)) => {
                            elem.get_attribute(name)
                                .map(|a| a.starts_with(v))
                                .unwrap_or(false)
                        }
                        (Some("$="), Some(v)) => {
                            elem.get_attribute(name)
                                .map(|a| a.ends_with(v))
                                .unwrap_or(false)
                        }
                        (Some("*="), Some(v)) => {
                            elem.get_attribute(name)
                                .map(|a| a.contains(v))
                                .unwrap_or(false)
                        }
                        (Some("~="), Some(v)) => {
                            elem.get_attribute(name)
                                .map(|a| a.split_whitespace().any(|w| w == v))
                                .unwrap_or(false)
                        }
                        (Some("|="), Some(v)) => {
                            elem.get_attribute(name)
                                .map(|a| a == v || a.starts_with(&format!("{}-", v)))
                                .unwrap_or(false)
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    /// Set text content of a node.
    pub fn set_text_content(&mut self, node: NodeId, text: &str) {
        if let Some(node_data) = self.nodes.get_mut(node) {
            match &mut node_data.data {
                NodeData::Text { content } => {
                    *content = text.to_string();
                }
                NodeData::Element(_) => {
                    // Remove all children and add text node
                    let children: Vec<_> = node_data.children.drain(..).collect();
                    for child in children {
                        self.remove(child);
                    }
                    let text_node = self.create_text(text.to_string());
                    self.append_child(node, text_node);
                }
                _ => {}
            }
        }
    }

    /// Get text content of a node and its descendants.
    pub fn get_text_content(&self, node: NodeId) -> String {
        let mut result = String::new();
        self.collect_text_content(node, &mut result);
        result
    }

    fn collect_text_content(&self, node: NodeId, result: &mut String) {
        if let Some(node_data) = self.nodes.get(node) {
            match &node_data.data {
                NodeData::Text { content } => {
                    result.push_str(content);
                }
                NodeData::Element(_) | NodeData::DocumentFragment | NodeData::Document { .. } => {
                    for &child in &node_data.children {
                        self.collect_text_content(child, result);
                    }
                }
                _ => {}
            }
        }
    }

    /// Update ID map after node insertion.
    fn update_id_map(&mut self, node: NodeId) {
        if let Some(n) = self.nodes.get(node) {
            if let Some(elem) = n.as_element() {
                if let Some(id) = &elem.id {
                    self.id_map.insert(id.to_string(), node);
                }
            }
        }
    }

    /// Get total number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if tree is empty (only root).
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    /// Iterate over all nodes.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes.iter()
    }
}

impl Default for DomTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over ancestor nodes.
pub struct AncestorIterator<'a> {
    tree: &'a DomTree,
    current: Option<NodeId>,
}

impl<'a> Iterator for AncestorIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.tree.parent(current);
        Some(current)
    }
}

/// Iterator over descendant nodes (pre-order traversal).
pub struct DescendantIterator<'a> {
    tree: &'a DomTree,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for DescendantIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;

        // Add children in reverse order so first child is processed first
        if let Some(node) = self.tree.nodes.get(current) {
            for &child in node.children.iter().rev() {
                self.stack.push(child);
            }
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let tree = DomTree::new();
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_append_child() {
        let mut tree = DomTree::new();
        let root = tree.root().unwrap();

        let div = tree.create_element(ElementData::new(TagName::div()));
        tree.append_child(root, div);

        assert_eq!(tree.parent(div), Some(root));
        assert_eq!(tree.first_child(root), Some(div));
    }

    #[test]
    fn test_find_by_id() {
        let mut tree = DomTree::new();
        let root = tree.root().unwrap();

        let mut data = ElementData::new(TagName::div());
        data.set_attribute("id", "test");
        let div = tree.create_element(data);
        tree.append_child(root, div);

        assert_eq!(tree.find_element_by_id("test"), Some(div));
        assert_eq!(tree.find_element_by_id("other"), None);
    }

    #[test]
    fn test_remove_node() {
        let mut tree = DomTree::new();
        let root = tree.root().unwrap();

        let div = tree.create_element(ElementData::new(TagName::div()));
        let span = tree.create_element(ElementData::new(TagName::span()));

        tree.append_child(root, div);
        tree.append_child(div, span);

        tree.remove(div);

        assert!(tree.get(div).is_none());
        assert!(tree.get(span).is_none());
    }
}
