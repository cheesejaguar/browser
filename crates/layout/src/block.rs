//! Block formatting context implementation.

use crate::box_model::{BoxDimensions, BoxType, ContainingBlock};
use crate::layout_box::{LayoutBox, LayoutBoxId};
use crate::tree::LayoutTree;
use common::geometry::Rect;

/// Block formatting context.
pub struct BlockFormattingContext {
    /// Current Y position.
    current_y: f32,
    /// Available width.
    available_width: f32,
    /// Margin collapsing state.
    margin_state: MarginCollapseState,
}

/// Margin collapsing state.
#[derive(Default)]
struct MarginCollapseState {
    /// Accumulated positive margin.
    positive_margin: f32,
    /// Accumulated negative margin (stored as positive value).
    negative_margin: f32,
    /// Whether we're at the start of the BFC.
    at_bfc_start: bool,
}

impl MarginCollapseState {
    fn new() -> Self {
        Self {
            positive_margin: 0.0,
            negative_margin: 0.0,
            at_bfc_start: true,
        }
    }

    /// Add a margin to the collapse state.
    fn add_margin(&mut self, margin: f32) {
        if margin >= 0.0 {
            self.positive_margin = self.positive_margin.max(margin);
        } else {
            self.negative_margin = self.negative_margin.max(-margin);
        }
    }

    /// Get the collapsed margin value.
    fn collapsed_margin(&self) -> f32 {
        self.positive_margin - self.negative_margin
    }

    /// Reset the margin state.
    fn reset(&mut self) {
        self.positive_margin = 0.0;
        self.negative_margin = 0.0;
    }
}

impl BlockFormattingContext {
    pub fn new(available_width: f32) -> Self {
        Self {
            current_y: 0.0,
            available_width,
            margin_state: MarginCollapseState::new(),
        }
    }

    /// Layout children in block flow.
    pub fn layout_block_children(
        &mut self,
        tree: &mut LayoutTree,
        parent_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let children: Vec<LayoutBoxId> = tree
            .children(parent_id)
            .collect();

        self.current_y = 0.0;
        self.margin_state = MarginCollapseState::new();

        for child_id in children {
            self.layout_block_child(tree, child_id, containing_block);
        }
    }

    /// Layout a single block child.
    fn layout_block_child(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let layout_box = match tree.get(box_id) {
            Some(b) => b,
            None => return,
        };

        let style = layout_box.style.clone();
        let box_type = layout_box.box_type;

        // Skip non-block boxes (they should be in anonymous blocks)
        if !matches!(box_type, BoxType::Block | BoxType::ListItem) {
            return;
        }

        // Calculate used values
        let margin = self.resolve_margins(&style, containing_block);
        let border = self.resolve_borders(&style);
        let padding = self.resolve_padding(&style, containing_block);

        // Handle margin collapsing
        self.margin_state.add_margin(margin.top);
        let collapsed_top_margin = self.margin_state.collapsed_margin();
        self.current_y += collapsed_top_margin;
        self.margin_state.reset();

        // Calculate width
        let width = self.calculate_block_width(&style, containing_block, &margin, &border, &padding);

        // Calculate horizontal position (auto margins for centering)
        let x = self.calculate_block_x(&style, containing_block, width, &margin);

        // Set position
        if let Some(layout_box) = tree.get_mut(box_id) {
            layout_box.dimensions.content.x = x + margin.left + border.left + padding.left;
            layout_box.dimensions.content.y = self.current_y + border.top + padding.top;
            layout_box.dimensions.content.width = width;
            layout_box.dimensions.margin = margin;
            layout_box.dimensions.border = border;
            layout_box.dimensions.padding = padding;
        }

        // Layout children
        let child_containing_block = ContainingBlock::new(width, None);
        self.layout_children_recursive(tree, box_id, &child_containing_block);

        // Calculate height after children are laid out
        let height = self.calculate_block_height(tree, box_id, &style, containing_block);

        if let Some(layout_box) = tree.get_mut(box_id) {
            layout_box.dimensions.content.height = height;
        }

        // Update current_y
        let layout_box = tree.get(box_id).unwrap();
        self.current_y = layout_box.dimensions.content.y
            + layout_box.dimensions.content.height
            + layout_box.dimensions.padding.bottom
            + layout_box.dimensions.border.bottom;

        // Add bottom margin to collapse state
        self.margin_state.add_margin(margin.bottom);
    }

    /// Layout children recursively.
    fn layout_children_recursive(
        &mut self,
        tree: &mut LayoutTree,
        parent_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let children: Vec<LayoutBoxId> = tree.children(parent_id).collect();

        if children.is_empty() {
            return;
        }

        // Check if children are all block or all inline
        let has_block = children.iter().any(|&id| {
            tree.get(id).map(|b| matches!(b.box_type, BoxType::Block | BoxType::ListItem)).unwrap_or(false)
        });

        if has_block {
            // Block formatting
            let mut bfc = BlockFormattingContext::new(containing_block.width);
            bfc.layout_block_children(tree, parent_id, containing_block);

            // Update parent height if auto
            let total_height = bfc.current_y + bfc.margin_state.collapsed_margin();
            if let Some(parent) = tree.get_mut(parent_id) {
                if parent.dimensions.content.height == 0.0 {
                    parent.dimensions.content.height = total_height;
                }
            }
        } else {
            // Inline formatting - handled by InlineFormattingContext
            // For now, simple inline layout
            self.simple_inline_layout(tree, parent_id, containing_block);
        }
    }

    /// Simple inline layout (placeholder for full IFC).
    fn simple_inline_layout(
        &mut self,
        tree: &mut LayoutTree,
        parent_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let children: Vec<LayoutBoxId> = tree.children(parent_id).collect();
        let mut x = 0.0;
        let mut line_height = 0.0f32;

        for child_id in children {
            if let Some(layout_box) = tree.get_mut(child_id) {
                // For text boxes, use the text run dimensions
                let width = if let Some(ref text) = layout_box.text {
                    text.width
                } else {
                    0.0
                };

                let height = if let Some(ref text) = layout_box.text {
                    text.height
                } else {
                    layout_box.style.font_size
                };

                layout_box.dimensions.content.x = x;
                layout_box.dimensions.content.y = 0.0;
                layout_box.dimensions.content.width = width;
                layout_box.dimensions.content.height = height;

                x += width;
                line_height = line_height.max(height);
            }
        }

        // Update parent height
        if let Some(parent) = tree.get_mut(parent_id) {
            if parent.dimensions.content.height == 0.0 {
                parent.dimensions.content.height = line_height;
            }
        }
    }

    /// Resolve margin values.
    fn resolve_margins(
        &self,
        style: &style::computed::ComputedStyle,
        containing_block: &ContainingBlock,
    ) -> common::geometry::EdgeSizes {
        use style::computed::SizeValue;

        let resolve = |value: &SizeValue| -> f32 {
            match value {
                SizeValue::Length(l) => *l,
                SizeValue::Percentage(p) => containing_block.width * p / 100.0,
                SizeValue::Auto => 0.0,
                _ => 0.0,
            }
        };

        common::geometry::EdgeSizes {
            top: resolve(&style.margin.top),
            right: resolve(&style.margin.right),
            bottom: resolve(&style.margin.bottom),
            left: resolve(&style.margin.left),
        }
    }

    /// Resolve border values.
    fn resolve_borders(&self, style: &style::computed::ComputedStyle) -> common::geometry::EdgeSizes {
        common::geometry::EdgeSizes {
            top: style.border_width.top,
            right: style.border_width.right,
            bottom: style.border_width.bottom,
            left: style.border_width.left,
        }
    }

    /// Resolve padding values.
    fn resolve_padding(
        &self,
        style: &style::computed::ComputedStyle,
        containing_block: &ContainingBlock,
    ) -> common::geometry::EdgeSizes {
        use style::computed::SizeValue;

        let resolve = |value: &SizeValue| -> f32 {
            match value {
                SizeValue::Length(l) => *l,
                SizeValue::Percentage(p) => containing_block.width * p / 100.0,
                _ => 0.0,
            }
        };

        common::geometry::EdgeSizes {
            top: resolve(&style.padding.top),
            right: resolve(&style.padding.right),
            bottom: resolve(&style.padding.bottom),
            left: resolve(&style.padding.left),
        }
    }

    /// Calculate block width.
    fn calculate_block_width(
        &self,
        style: &style::computed::ComputedStyle,
        containing_block: &ContainingBlock,
        margin: &common::geometry::EdgeSizes,
        border: &common::geometry::EdgeSizes,
        padding: &common::geometry::EdgeSizes,
    ) -> f32 {
        use style::computed::SizeValue;

        let horizontal_space = margin.left + border.left + padding.left
            + padding.right + border.right + margin.right;

        match &style.width {
            SizeValue::Length(l) => *l,
            SizeValue::Percentage(p) => containing_block.width * p / 100.0,
            SizeValue::Auto => {
                // Fill available space
                (containing_block.width - horizontal_space).max(0.0)
            }
            _ => (containing_block.width - horizontal_space).max(0.0),
        }
    }

    /// Calculate block X position.
    fn calculate_block_x(
        &self,
        style: &style::computed::ComputedStyle,
        containing_block: &ContainingBlock,
        width: f32,
        margin: &common::geometry::EdgeSizes,
    ) -> f32 {
        use style::computed::SizeValue;

        // Handle auto margins for centering
        let left_auto = matches!(style.margin.left, SizeValue::Auto);
        let right_auto = matches!(style.margin.right, SizeValue::Auto);

        if left_auto && right_auto {
            // Center the box
            (containing_block.width - width) / 2.0
        } else if left_auto {
            // Push to the right
            containing_block.width - width - margin.right
        } else {
            // Normal flow
            0.0
        }
    }

    /// Calculate block height.
    fn calculate_block_height(
        &self,
        tree: &LayoutTree,
        box_id: LayoutBoxId,
        style: &style::computed::ComputedStyle,
        containing_block: &ContainingBlock,
    ) -> f32 {
        use style::computed::SizeValue;

        match &style.height {
            SizeValue::Length(l) => *l,
            SizeValue::Percentage(p) => {
                if let Some(cb_height) = containing_block.height {
                    cb_height * p / 100.0
                } else {
                    self.calculate_content_height(tree, box_id)
                }
            }
            SizeValue::Auto => self.calculate_content_height(tree, box_id),
            _ => self.calculate_content_height(tree, box_id),
        }
    }

    /// Calculate content height from children.
    fn calculate_content_height(&self, tree: &LayoutTree, box_id: LayoutBoxId) -> f32 {
        let children: Vec<LayoutBoxId> = tree.children(box_id).collect();

        if children.is_empty() {
            // Check for text content
            if let Some(layout_box) = tree.get(box_id) {
                if let Some(ref text) = layout_box.text {
                    return text.height;
                }
            }
            return 0.0;
        }

        let mut max_bottom = 0.0f32;
        for child_id in children {
            if let Some(child) = tree.get(child_id) {
                let child_bottom = child.dimensions.content.y
                    + child.dimensions.content.height
                    + child.dimensions.padding.bottom
                    + child.dimensions.border.bottom
                    + child.dimensions.margin.bottom;
                max_bottom = max_bottom.max(child_bottom);
            }
        }

        max_bottom
    }
}

/// Establishes a new block formatting context.
pub fn establish_bfc(
    tree: &mut LayoutTree,
    root_id: LayoutBoxId,
    containing_block: &ContainingBlock,
) {
    let mut bfc = BlockFormattingContext::new(containing_block.width);
    bfc.layout_block_children(tree, root_id, containing_block);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_margin_collapse() {
        let mut state = MarginCollapseState::new();
        state.add_margin(20.0);
        state.add_margin(30.0);
        assert_eq!(state.collapsed_margin(), 30.0);

        state.reset();
        state.add_margin(20.0);
        state.add_margin(-10.0);
        assert_eq!(state.collapsed_margin(), 10.0);
    }

    #[test]
    fn test_bfc_creation() {
        let bfc = BlockFormattingContext::new(800.0);
        assert_eq!(bfc.available_width, 800.0);
        assert_eq!(bfc.current_y, 0.0);
    }
}
