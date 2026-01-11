//! Main layout engine.

use crate::block::BlockFormattingContext;
use crate::box_model::{BoxDimensions, BoxType, ContainingBlock};
use crate::inline::InlineFormattingContext;
use crate::layout_box::{LayoutBox, LayoutBoxId};
use crate::text::TextShaper;
use crate::tree::LayoutTree;
use common::geometry::Rect;
use dom::document::Document;
use dom::node::{NodeId, NodeType};
use slotmap::SlotMap;
use std::sync::Arc;
use style::computed::ComputedStyle;
use style::resolver::StyleResolver;

/// The layout engine.
pub struct LayoutEngine {
    /// Viewport dimensions.
    pub viewport_width: f32,
    pub viewport_height: f32,
    /// Text shaper for measuring text.
    text_shaper: TextShaper,
}

impl LayoutEngine {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            text_shaper: TextShaper::new(),
        }
    }

    /// Set viewport dimensions.
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Perform layout on a document.
    pub fn layout(
        &mut self,
        document: &Document,
        style_resolver: &StyleResolver,
    ) -> LayoutTree {
        let mut tree = LayoutTree::new();

        // Build layout tree from DOM
        self.build_layout_tree(&document.tree, style_resolver, &mut tree);

        // Perform layout
        if let Some(root_id) = tree.root() {
            let containing_block = ContainingBlock::new(self.viewport_width, Some(self.viewport_height));
            self.layout_box(&mut tree, root_id, &containing_block);
        }

        tree
    }

    /// Build layout tree from DOM tree.
    fn build_layout_tree(
        &mut self,
        dom_tree: &dom::tree::DomTree,
        style_resolver: &StyleResolver,
        layout_tree: &mut LayoutTree,
    ) {
        if let Some(root_node) = dom_tree.root() {
            // Find body element
            let body_node = self.find_body(dom_tree, root_node);

            if let Some(body_id) = body_node {
                self.build_subtree(dom_tree, body_id, style_resolver, layout_tree, None);
            }
        }
    }

    fn find_body(&self, tree: &dom::tree::DomTree, from: NodeId) -> Option<NodeId> {
        // Find html element
        for child in tree.children(from) {
            if let Some(elem) = tree.get_element(child) {
                if elem.tag_name.as_str() == "html" {
                    // Find body
                    for html_child in tree.children(child) {
                        if let Some(elem) = tree.get_element(html_child) {
                            if elem.tag_name.as_str() == "body" {
                                return Some(html_child);
                            }
                        }
                    }
                    return Some(child);
                }
            }
        }
        Some(from)
    }

    fn build_subtree(
        &mut self,
        dom_tree: &dom::tree::DomTree,
        node_id: NodeId,
        style_resolver: &StyleResolver,
        layout_tree: &mut LayoutTree,
        parent_box: Option<LayoutBoxId>,
    ) -> Option<LayoutBoxId> {
        let node = dom_tree.get(node_id)?;

        // Get style for element nodes
        let style = if node.node_type == NodeType::Element {
            style_resolver
                .get_style(node_id)
                .unwrap_or_else(|| Arc::new(ComputedStyle::default_style()))
        } else {
            Arc::new(ComputedStyle::default_style())
        };

        // Check for display: none
        if style.display == style::computed::Display::None {
            return None;
        }

        // Create layout box
        let box_type = if node.node_type == NodeType::Text {
            BoxType::Text
        } else {
            BoxType::from_style(&style)
        };

        let layout_box = if node.node_type == NodeType::Text {
            if let Some(text_content) = node.as_text() {
                let text_run = self.text_shaper.shape_text(text_content, &style);
                Some(layout_tree.create_text_box(style.clone(), text_run))
            } else {
                None
            }
        } else {
            Some(layout_tree.create_box(Some(node_id), box_type, style.clone()))
        };

        let layout_box_id = match layout_box {
            Some(id) => id,
            None => return None,
        };

        // Add to parent
        if let Some(parent_id) = parent_box {
            layout_tree.append_child(parent_id, layout_box_id);
        } else {
            layout_tree.set_root(layout_box_id);
        }

        // Build children
        for child_id in dom_tree.children(node_id).collect::<Vec<_>>() {
            self.build_subtree(dom_tree, child_id, style_resolver, layout_tree, Some(layout_box_id));
        }

        Some(layout_box_id)
    }

    /// Perform layout on a single box and its descendants.
    fn layout_box(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let box_type = tree.get(box_id).map(|b| b.box_type).unwrap_or(BoxType::Block);

        match box_type {
            BoxType::Block | BoxType::AnonymousBlock | BoxType::ListItem => {
                self.layout_block(tree, box_id, containing_block);
            }
            BoxType::Inline | BoxType::InlineBlock | BoxType::Text => {
                self.layout_inline(tree, box_id, containing_block);
            }
            BoxType::Flex => {
                self.layout_flex(tree, box_id, containing_block);
            }
            BoxType::Grid => {
                self.layout_grid(tree, box_id, containing_block);
            }
            BoxType::Table => {
                self.layout_table(tree, box_id, containing_block);
            }
            BoxType::Replaced => {
                self.layout_replaced(tree, box_id, containing_block);
            }
            _ => {
                self.layout_block(tree, box_id, containing_block);
            }
        }
    }

    /// Layout a block-level box.
    fn layout_block(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        // Calculate dimensions
        self.calculate_block_width(tree, box_id, containing_block);
        self.calculate_block_position(tree, box_id, containing_block);

        // Layout children
        self.layout_block_children(tree, box_id);

        // Calculate height (after children are laid out)
        self.calculate_block_height(tree, box_id);
    }

    fn calculate_block_width(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let layout_box = match tree.get_mut(box_id) {
            Some(b) => b,
            None => return,
        };

        let style = layout_box.style.clone();
        let cb_width = containing_block.width;

        // Resolve margin, border, padding
        let margin_left = style.margin.left.resolve(cb_width);
        let margin_right = style.margin.right.resolve(cb_width);
        let padding_left = style.padding.left.resolve(cb_width);
        let padding_right = style.padding.right.resolve(cb_width);
        let border_left = style.border_width.left;
        let border_right = style.border_width.right;

        // Calculate width
        let width = match &style.width {
            style::computed::SizeValue::Auto => {
                cb_width
                    - margin_left
                    - margin_right
                    - padding_left
                    - padding_right
                    - border_left
                    - border_right
            }
            style::computed::SizeValue::Length(l) => *l,
            style::computed::SizeValue::Percentage(p) => cb_width * p / 100.0,
            _ => cb_width,
        };

        // Handle auto margins for centering
        let total = width + margin_left + margin_right + padding_left + padding_right + border_left + border_right;

        let (final_margin_left, final_margin_right) = if style.margin.left.is_auto() && style.margin.right.is_auto() {
            let extra = (cb_width - total + margin_left + margin_right) / 2.0;
            (extra.max(0.0), extra.max(0.0))
        } else if style.margin.left.is_auto() {
            let extra = cb_width - total + margin_left;
            (extra.max(0.0), margin_right)
        } else if style.margin.right.is_auto() {
            let extra = cb_width - total + margin_right;
            (margin_left, extra.max(0.0))
        } else {
            (margin_left, margin_right)
        };

        // Set dimensions
        layout_box.dimensions.content.width = width.max(0.0);
        layout_box.dimensions.margin.left = final_margin_left;
        layout_box.dimensions.margin.right = final_margin_right;
        layout_box.dimensions.padding.left = padding_left;
        layout_box.dimensions.padding.right = padding_right;
        layout_box.dimensions.border.left = border_left;
        layout_box.dimensions.border.right = border_right;
    }

    fn calculate_block_position(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let layout_box = match tree.get_mut(box_id) {
            Some(b) => b,
            None => return,
        };

        let style = layout_box.style.clone();
        let cb_width = containing_block.width;

        // Resolve margin, border, padding
        let margin_top = style.margin.top.resolve(cb_width);
        let margin_bottom = style.margin.bottom.resolve(cb_width);
        let padding_top = style.padding.top.resolve(cb_width);
        let padding_bottom = style.padding.bottom.resolve(cb_width);
        let border_top = style.border_width.top;
        let border_bottom = style.border_width.bottom;

        layout_box.dimensions.margin.top = margin_top;
        layout_box.dimensions.margin.bottom = margin_bottom;
        layout_box.dimensions.padding.top = padding_top;
        layout_box.dimensions.padding.bottom = padding_bottom;
        layout_box.dimensions.border.top = border_top;
        layout_box.dimensions.border.bottom = border_bottom;

        // Position
        layout_box.dimensions.content.x = containing_block.x
            + layout_box.dimensions.margin.left
            + layout_box.dimensions.border.left
            + layout_box.dimensions.padding.left;

        layout_box.dimensions.content.y = containing_block.y
            + layout_box.dimensions.margin.top
            + layout_box.dimensions.border.top
            + layout_box.dimensions.padding.top;
    }

    fn layout_block_children(&mut self, tree: &mut LayoutTree, box_id: LayoutBoxId) {
        let (children, content_x, content_y, content_width) = {
            let layout_box = match tree.get(box_id) {
                Some(b) => b,
                None => return,
            };
            let children = layout_box.children.clone();
            (
                children,
                layout_box.dimensions.content.x,
                layout_box.dimensions.content.y,
                layout_box.dimensions.content.width,
            )
        };

        let mut y_offset = content_y;

        for child_id in children {
            let containing = ContainingBlock {
                width: content_width,
                height: None,
                x: content_x,
                y: y_offset,
            };

            self.layout_box(tree, child_id, &containing);

            // Update y_offset
            if let Some(child) = tree.get(child_id) {
                y_offset = child.dimensions.margin_box().bottom();
            }
        }
    }

    fn calculate_block_height(&mut self, tree: &mut LayoutTree, box_id: LayoutBoxId) {
        let (style, children) = match tree.get(box_id) {
            Some(b) => (b.style.clone(), b.children.clone()),
            None => return,
        };

        // Explicit height
        let explicit_height = match &style.height {
            style::computed::SizeValue::Length(l) => Some(*l),
            style::computed::SizeValue::Percentage(p) => {
                // Need containing block height
                None
            }
            _ => None,
        };

        let height = if let Some(h) = explicit_height {
            h
        } else {
            // Auto height - sum of children
            let mut total_height = 0.0;
            for child_id in children {
                if let Some(child) = tree.get(child_id) {
                    let margin_box = child.dimensions.margin_box();
                    total_height = (margin_box.bottom() - tree.get(box_id).unwrap().dimensions.content.y).max(total_height);
                }
            }
            total_height
        };

        if let Some(layout_box) = tree.get_mut(box_id) {
            layout_box.dimensions.content.height = height.max(0.0);
        }
    }

    /// Layout inline boxes.
    fn layout_inline(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        // Simplified inline layout
        let layout_box = match tree.get_mut(box_id) {
            Some(b) => b,
            None => return,
        };

        // For text boxes, calculate size from text
        if let Some(ref text_run) = layout_box.text {
            layout_box.dimensions.content.width = text_run.width;
            layout_box.dimensions.content.height = text_run.height;
        } else {
            // For inline elements, layout children
            let style = layout_box.style.clone();
            let padding_left = style.padding.left.resolve(containing_block.width);
            let padding_right = style.padding.right.resolve(containing_block.width);
            let border_left = style.border_width.left;
            let border_right = style.border_width.right;

            layout_box.dimensions.padding.left = padding_left;
            layout_box.dimensions.padding.right = padding_right;
            layout_box.dimensions.border.left = border_left;
            layout_box.dimensions.border.right = border_right;
        }

        layout_box.dimensions.content.x = containing_block.x;
        layout_box.dimensions.content.y = containing_block.y;
    }

    /// Layout flex container.
    fn layout_flex(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        // Calculate container dimensions first
        self.calculate_block_width(tree, box_id, containing_block);
        self.calculate_block_position(tree, box_id, containing_block);

        // Perform flex layout on children
        crate::flex::layout_flex_container(tree, box_id, containing_block);

        // Calculate height after flex layout
        self.calculate_block_height(tree, box_id);
    }

    /// Layout grid container.
    fn layout_grid(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        // Calculate container dimensions first
        self.calculate_block_width(tree, box_id, containing_block);
        self.calculate_block_position(tree, box_id, containing_block);

        // Perform grid layout on children
        crate::grid::layout_grid_container(tree, box_id, containing_block);

        // Calculate height after grid layout
        self.calculate_block_height(tree, box_id);
    }

    /// Layout table.
    fn layout_table(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        // Simplified table layout - treat as block
        self.layout_block(tree, box_id, containing_block);
    }

    /// Layout replaced element (img, video, etc).
    fn layout_replaced(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let layout_box = match tree.get_mut(box_id) {
            Some(b) => b,
            None => return,
        };

        let style = layout_box.style.clone();

        // Get specified dimensions
        let specified_width = match &style.width {
            style::computed::SizeValue::Length(l) => Some(*l),
            style::computed::SizeValue::Percentage(p) => Some(containing_block.width * p / 100.0),
            _ => None,
        };
        let specified_height = match &style.height {
            style::computed::SizeValue::Length(l) => Some(*l),
            style::computed::SizeValue::Percentage(p) => {
                containing_block.height.map(|h| h * p / 100.0)
            }
            _ => None,
        };

        // Compute used size
        let (width, height) = if let Some(ref replaced) = layout_box.replaced {
            replaced.compute_size(
                specified_width,
                specified_height,
                containing_block.width,
                containing_block.height.unwrap_or(0.0),
            )
        } else {
            // Default size for replaced elements without intrinsic dimensions
            (specified_width.unwrap_or(300.0), specified_height.unwrap_or(150.0))
        };

        layout_box.dimensions.content.width = width;
        layout_box.dimensions.content.height = height;
        layout_box.dimensions.content.x = containing_block.x;
        layout_box.dimensions.content.y = containing_block.y;
    }

    /// Relayout a subtree.
    pub fn relayout_subtree(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
    ) {
        // Get containing block from parent
        let containing_block = if let Some(parent_id) = tree.get(box_id).and_then(|b| b.parent) {
            if let Some(parent) = tree.get(parent_id) {
                ContainingBlock::from_dimensions(&parent.dimensions)
            } else {
                ContainingBlock::new(self.viewport_width, Some(self.viewport_height))
            }
        } else {
            ContainingBlock::new(self.viewport_width, Some(self.viewport_height))
        };

        self.layout_box(tree, box_id, &containing_block);
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = LayoutEngine::new(800.0, 600.0);
        assert_eq!(engine.viewport_width, 800.0);
        assert_eq!(engine.viewport_height, 600.0);
    }
}
