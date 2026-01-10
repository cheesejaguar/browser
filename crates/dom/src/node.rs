//! DOM Node implementation.

use crate::element::ElementData;
use parking_lot::RwLock;
use slotmap::new_key_type;
use smallvec::SmallVec;
use std::sync::Arc;

new_key_type! {
    /// Unique identifier for a DOM node.
    pub struct NodeId;
}

/// Type of DOM node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Element = 1,
    Text = 3,
    CDataSection = 4,
    ProcessingInstruction = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11,
}

impl NodeType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(NodeType::Element),
            3 => Some(NodeType::Text),
            4 => Some(NodeType::CDataSection),
            7 => Some(NodeType::ProcessingInstruction),
            8 => Some(NodeType::Comment),
            9 => Some(NodeType::Document),
            10 => Some(NodeType::DocumentType),
            11 => Some(NodeType::DocumentFragment),
            _ => None,
        }
    }
}

/// Data specific to each node type.
#[derive(Clone, Debug)]
pub enum NodeData {
    Document {
        doctype: Option<DocumentType>,
    },
    DocumentFragment,
    Element(ElementData),
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
    ProcessingInstruction {
        target: String,
        data: String,
    },
    DocumentType(DocumentType),
}

/// Document type declaration.
#[derive(Clone, Debug, Default)]
pub struct DocumentType {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

/// A DOM node.
#[derive(Debug)]
pub struct Node {
    /// Unique identifier.
    pub id: NodeId,
    /// Node type.
    pub node_type: NodeType,
    /// Node-specific data.
    pub data: NodeData,
    /// Parent node.
    pub parent: Option<NodeId>,
    /// Child nodes.
    pub children: SmallVec<[NodeId; 8]>,
    /// Previous sibling.
    pub prev_sibling: Option<NodeId>,
    /// Next sibling.
    pub next_sibling: Option<NodeId>,
    /// Layout data (used by layout engine).
    pub layout_data: Option<Box<dyn std::any::Any + Send + Sync>>,
    /// Style data (used by style system).
    pub style_data: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            node_type: self.node_type.clone(),
            data: self.data.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
            prev_sibling: self.prev_sibling.clone(),
            next_sibling: self.next_sibling.clone(),
            // These fields are not cloned as they are regenerated
            layout_data: None,
            style_data: None,
        }
    }
}

impl Node {
    pub fn new(id: NodeId, node_type: NodeType, data: NodeData) -> Self {
        Self {
            id,
            node_type,
            data,
            parent: None,
            children: SmallVec::new(),
            prev_sibling: None,
            next_sibling: None,
            layout_data: None,
            style_data: None,
        }
    }

    pub fn new_document(id: NodeId) -> Self {
        Self::new(
            id,
            NodeType::Document,
            NodeData::Document { doctype: None },
        )
    }

    pub fn new_element(id: NodeId, data: ElementData) -> Self {
        Self::new(id, NodeType::Element, NodeData::Element(data))
    }

    pub fn new_text(id: NodeId, content: String) -> Self {
        Self::new(id, NodeType::Text, NodeData::Text { content })
    }

    pub fn new_comment(id: NodeId, content: String) -> Self {
        Self::new(id, NodeType::Comment, NodeData::Comment { content })
    }

    pub fn new_document_fragment(id: NodeId) -> Self {
        Self::new(id, NodeType::DocumentFragment, NodeData::DocumentFragment)
    }

    /// Get node name according to DOM spec.
    pub fn node_name(&self) -> &str {
        match &self.data {
            NodeData::Document { .. } => "#document",
            NodeData::DocumentFragment => "#document-fragment",
            NodeData::Element(elem) => elem.tag_name.as_str(),
            NodeData::Text { .. } => "#text",
            NodeData::Comment { .. } => "#comment",
            NodeData::ProcessingInstruction { target, .. } => target,
            NodeData::DocumentType(dt) => &dt.name,
        }
    }

    /// Get node value according to DOM spec.
    pub fn node_value(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text { content } | NodeData::Comment { content } => Some(content),
            NodeData::ProcessingInstruction { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Get text content of this node and its descendants.
    pub fn text_content(&self) -> Option<String> {
        match &self.data {
            NodeData::Document { .. } | NodeData::DocumentType(_) => None,
            NodeData::Text { content } | NodeData::Comment { content } => {
                Some(content.clone())
            }
            NodeData::ProcessingInstruction { data, .. } => Some(data.clone()),
            NodeData::Element(_) | NodeData::DocumentFragment => {
                // Would need tree traversal - handled at tree level
                None
            }
        }
    }

    /// Check if this is an element node.
    #[inline]
    pub fn is_element(&self) -> bool {
        self.node_type == NodeType::Element
    }

    /// Check if this is a text node.
    #[inline]
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }

    /// Check if this is a document node.
    #[inline]
    pub fn is_document(&self) -> bool {
        self.node_type == NodeType::Document
    }

    /// Get element data if this is an element.
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Get mutable element data if this is an element.
    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Get text content if this is a text node.
    pub fn as_text(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text { content } => Some(content),
            _ => None,
        }
    }

    /// Check if node has children.
    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get first child.
    #[inline]
    pub fn first_child(&self) -> Option<NodeId> {
        self.children.first().copied()
    }

    /// Get last child.
    #[inline]
    pub fn last_child(&self) -> Option<NodeId> {
        self.children.last().copied()
    }

    /// Get number of children.
    #[inline]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

/// Node reference with shared ownership.
pub type NodeRef = Arc<RwLock<Node>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::ElementData;

    #[test]
    fn test_node_types() {
        assert_eq!(NodeType::Element as u8, 1);
        assert_eq!(NodeType::Text as u8, 3);
        assert_eq!(NodeType::Document as u8, 9);
    }
}
