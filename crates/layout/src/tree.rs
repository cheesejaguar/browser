//! Layout tree structure.

use crate::box_model::BoxType;
use crate::layout_box::{LayoutBox, LayoutBoxId};
use crate::text::TextRun;
use dom::node::NodeId;
use slotmap::SlotMap;
use smallvec::SmallVec;
use std::sync::Arc;
use style::computed::ComputedStyle;

/// The layout tree.
pub struct LayoutTree {
    /// All layout boxes.
    boxes: SlotMap<LayoutBoxId, LayoutBox>,
    /// Root box.
    root: Option<LayoutBoxId>,
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            boxes: SlotMap::with_key(),
            root: None,
        }
    }

    /// Get root box.
    pub fn root(&self) -> Option<LayoutBoxId> {
        self.root
    }

    /// Set root box.
    pub fn set_root(&mut self, box_id: LayoutBoxId) {
        self.root = Some(box_id);
    }

    /// Create a layout box.
    pub fn create_box(
        &mut self,
        node: Option<NodeId>,
        box_type: BoxType,
        style: Arc<ComputedStyle>,
    ) -> LayoutBoxId {
        self.boxes
            .insert_with_key(|id| LayoutBox::new(id, node, box_type, style))
    }

    /// Create a text box.
    pub fn create_text_box(
        &mut self,
        style: Arc<ComputedStyle>,
        text: TextRun,
    ) -> LayoutBoxId {
        self.boxes
            .insert_with_key(|id| LayoutBox::text_box(id, style, text))
    }

    /// Create anonymous block box.
    pub fn create_anonymous_block(&mut self, style: Arc<ComputedStyle>) -> LayoutBoxId {
        self.boxes
            .insert_with_key(|id| LayoutBox::anonymous_block(id, style))
    }

    /// Create anonymous inline box.
    pub fn create_anonymous_inline(&mut self, style: Arc<ComputedStyle>) -> LayoutBoxId {
        self.boxes
            .insert_with_key(|id| LayoutBox::anonymous_inline(id, style))
    }

    /// Get a box by ID.
    pub fn get(&self, id: LayoutBoxId) -> Option<&LayoutBox> {
        self.boxes.get(id)
    }

    /// Get a mutable box by ID.
    pub fn get_mut(&mut self, id: LayoutBoxId) -> Option<&mut LayoutBox> {
        self.boxes.get_mut(id)
    }

    /// Append child to parent.
    pub fn append_child(&mut self, parent: LayoutBoxId, child: LayoutBoxId) {
        if let Some(child_box) = self.boxes.get_mut(child) {
            child_box.parent = Some(parent);
        }
        if let Some(parent_box) = self.boxes.get_mut(parent) {
            parent_box.children.push(child);
        }
    }

    /// Insert child before reference.
    pub fn insert_before(
        &mut self,
        parent: LayoutBoxId,
        child: LayoutBoxId,
        reference: Option<LayoutBoxId>,
    ) {
        if let Some(child_box) = self.boxes.get_mut(child) {
            child_box.parent = Some(parent);
        }

        if let Some(parent_box) = self.boxes.get_mut(parent) {
            match reference {
                Some(ref_id) => {
                    if let Some(pos) = parent_box.children.iter().position(|&id| id == ref_id) {
                        parent_box.children.insert(pos, child);
                    } else {
                        parent_box.children.push(child);
                    }
                }
                None => {
                    parent_box.children.push(child);
                }
            }
        }
    }

    /// Remove box from tree.
    pub fn remove(&mut self, box_id: LayoutBoxId) {
        // Remove from parent
        if let Some(parent_id) = self.boxes.get(box_id).and_then(|b| b.parent) {
            if let Some(parent) = self.boxes.get_mut(parent_id) {
                parent.children.retain(|&id| id != box_id);
            }
        }

        // Remove box and children
        let mut to_remove = vec![box_id];
        let mut i = 0;
        while i < to_remove.len() {
            if let Some(b) = self.boxes.get(to_remove[i]) {
                to_remove.extend(b.children.iter().copied());
            }
            i += 1;
        }

        for id in to_remove {
            self.boxes.remove(id);
        }

        if self.root == Some(box_id) {
            self.root = None;
        }
    }

    /// Get parent box.
    pub fn parent(&self, box_id: LayoutBoxId) -> Option<LayoutBoxId> {
        self.boxes.get(box_id).and_then(|b| b.parent)
    }

    /// Get children.
    pub fn children(&self, box_id: LayoutBoxId) -> impl Iterator<Item = LayoutBoxId> + '_ {
        self.boxes
            .get(box_id)
            .into_iter()
            .flat_map(|b| b.children.iter().copied())
    }

    /// Find box by DOM node.
    pub fn find_by_node(&self, node: NodeId) -> Option<LayoutBoxId> {
        self.boxes
            .iter()
            .find(|(_, b)| b.node == Some(node))
            .map(|(id, _)| id)
    }

    /// Get all boxes.
    pub fn iter(&self) -> impl Iterator<Item = (LayoutBoxId, &LayoutBox)> {
        self.boxes.iter()
    }

    /// Get number of boxes.
    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    /// Check if tree is empty.
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// Get box at point.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<LayoutBoxId> {
        self.root.and_then(|root| self.hit_test_box(root, x, y))
    }

    fn hit_test_box(&self, box_id: LayoutBoxId, x: f32, y: f32) -> Option<LayoutBoxId> {
        let layout_box = self.boxes.get(box_id)?;
        let border_box = layout_box.border_rect();

        if !border_box.contains_point(common::geometry::Point::new(x, y)) {
            return None;
        }

        // Check children in reverse order (last child is on top)
        for &child_id in layout_box.children.iter().rev() {
            if let Some(hit) = self.hit_test_box(child_id, x, y) {
                return Some(hit);
            }
        }

        // Return this box if it has a DOM node
        if layout_box.node.is_some() {
            Some(box_id)
        } else {
            None
        }
    }

    /// Clear the tree.
    pub fn clear(&mut self) {
        self.boxes.clear();
        self.root = None;
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree traversal order.
pub struct PreOrderIterator<'a> {
    tree: &'a LayoutTree,
    stack: Vec<LayoutBoxId>,
}

impl<'a> PreOrderIterator<'a> {
    pub fn new(tree: &'a LayoutTree) -> Self {
        let stack = tree.root().into_iter().collect();
        Self { tree, stack }
    }
}

impl<'a> Iterator for PreOrderIterator<'a> {
    type Item = LayoutBoxId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.stack.pop()?;
        if let Some(layout_box) = self.tree.get(id) {
            // Add children in reverse order
            for &child in layout_box.children.iter().rev() {
                self.stack.push(child);
            }
        }
        Some(id)
    }
}

/// Post-order iterator.
pub struct PostOrderIterator<'a> {
    tree: &'a LayoutTree,
    stack: Vec<(LayoutBoxId, bool)>,
}

impl<'a> PostOrderIterator<'a> {
    pub fn new(tree: &'a LayoutTree) -> Self {
        let stack = tree.root().map(|id| (id, false)).into_iter().collect();
        Self { tree, stack }
    }
}

impl<'a> Iterator for PostOrderIterator<'a> {
    type Item = LayoutBoxId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (id, visited) = self.stack.pop()?;

            if visited {
                return Some(id);
            }

            self.stack.push((id, true));

            if let Some(layout_box) = self.tree.get(id) {
                for &child in layout_box.children.iter().rev() {
                    self.stack.push((child, false));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_tree() {
        let mut tree = LayoutTree::new();
        let style = Arc::new(ComputedStyle::default_style());

        let root = tree.create_box(None, BoxType::Block, style.clone());
        tree.set_root(root);

        let child = tree.create_box(None, BoxType::Block, style.clone());
        tree.append_child(root, child);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.parent(child), Some(root));
    }
}
