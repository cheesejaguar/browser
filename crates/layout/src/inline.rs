//! Inline formatting context implementation.

use crate::box_model::{BoxDimensions, BoxType, ContainingBlock};
use crate::layout_box::{LayoutBox, LayoutBoxId};
use crate::text::{LineBox, LineFragment, TextRun, TextShaper};
use crate::tree::LayoutTree;
use common::geometry::{EdgeSizes, Rect};
use std::sync::Arc;
use style::computed::{ComputedStyle, TextAlign, VerticalAlign, WhiteSpace};

/// Inline formatting context.
pub struct InlineFormattingContext {
    /// Line boxes.
    lines: Vec<LineBox>,
    /// Current line being built.
    current_line: LineBox,
    /// Available width.
    available_width: f32,
    /// Current X position on line.
    current_x: f32,
    /// Text shaper.
    text_shaper: TextShaper,
}

impl InlineFormattingContext {
    pub fn new(available_width: f32) -> Self {
        Self {
            lines: Vec::new(),
            current_line: LineBox::new(0.0, 0.0, available_width),
            available_width,
            current_x: 0.0,
            text_shaper: TextShaper::new(),
        }
    }

    /// Layout inline children.
    pub fn layout_inline_children(
        &mut self,
        tree: &mut LayoutTree,
        parent_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) -> f32 {
        let children: Vec<LayoutBoxId> = tree.children(parent_id).collect();

        for child_id in children {
            self.layout_inline_box(tree, child_id, containing_block);
        }

        // Finish the last line
        self.finish_line();

        // Position all lines
        self.position_lines(tree, parent_id);

        // Return total height
        self.total_height()
    }

    /// Layout a single inline box.
    fn layout_inline_box(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        let layout_box = match tree.get(box_id) {
            Some(b) => b,
            None => return,
        };

        match layout_box.box_type {
            BoxType::Text => {
                self.layout_text_box(tree, box_id);
            }
            BoxType::Inline => {
                self.layout_inline_element(tree, box_id, containing_block);
            }
            BoxType::InlineBlock => {
                self.layout_inline_block(tree, box_id, containing_block);
            }
            BoxType::Replaced => {
                self.layout_replaced_element(tree, box_id);
            }
            _ => {
                // Block-level elements shouldn't be here
            }
        }
    }

    /// Layout a text box.
    fn layout_text_box(&mut self, tree: &mut LayoutTree, box_id: LayoutBoxId) {
        let layout_box = match tree.get(box_id) {
            Some(b) => b,
            None => return,
        };

        let text_run = match &layout_box.text {
            Some(tr) => tr.clone(),
            None => return,
        };

        let style = layout_box.style.clone();

        // Check if text fits on current line
        if self.current_x + text_run.width > self.available_width && self.current_x > 0.0 {
            // Need to wrap - try to break the text
            self.wrap_text(tree, box_id, &text_run, &style);
        } else {
            // Fits on current line
            self.add_fragment_to_line(box_id, text_run.width, text_run.ascent, text_run.descent, 0, text_run.glyphs.len());
        }
    }

    /// Wrap text across multiple lines.
    fn wrap_text(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        text_run: &TextRun,
        style: &ComputedStyle,
    ) {
        // Check white-space property
        let can_wrap = !matches!(style.white_space, WhiteSpace::NoWrap | WhiteSpace::Pre);

        if !can_wrap {
            // Can't wrap, just overflow
            self.add_fragment_to_line(
                box_id,
                text_run.width,
                text_run.ascent,
                text_run.descent,
                0,
                text_run.glyphs.len(),
            );
            return;
        }

        // Find wrap points (after whitespace)
        let mut glyph_start = 0;
        let mut current_width = 0.0;
        let mut last_break_point = 0;
        let mut last_break_width = 0.0;

        for (i, glyph) in text_run.glyphs.iter().enumerate() {
            // Track potential break points
            if glyph.character.is_whitespace() {
                last_break_point = i + 1;
                last_break_width = current_width + glyph.advance;
            }

            let new_width = current_width + glyph.advance;

            // Check if we need to wrap
            if self.current_x + new_width > self.available_width && glyph_start < i {
                if last_break_point > glyph_start {
                    // Break at last whitespace
                    let fragment_width: f32 = text_run.glyphs[glyph_start..last_break_point]
                        .iter()
                        .map(|g| g.advance)
                        .sum();

                    self.add_fragment_to_line(
                        box_id,
                        fragment_width,
                        text_run.ascent,
                        text_run.descent,
                        glyph_start,
                        last_break_point,
                    );

                    self.finish_line();
                    self.start_new_line();

                    glyph_start = last_break_point;
                    current_width = new_width - last_break_width;
                    last_break_point = glyph_start;
                    last_break_width = 0.0;
                } else {
                    // Force break at current position
                    let fragment_width: f32 = text_run.glyphs[glyph_start..i]
                        .iter()
                        .map(|g| g.advance)
                        .sum();

                    self.add_fragment_to_line(
                        box_id,
                        fragment_width,
                        text_run.ascent,
                        text_run.descent,
                        glyph_start,
                        i,
                    );

                    self.finish_line();
                    self.start_new_line();

                    glyph_start = i;
                    current_width = glyph.advance;
                    last_break_point = i;
                    last_break_width = 0.0;
                }
            } else {
                current_width = new_width;
            }
        }

        // Add remaining text
        if glyph_start < text_run.glyphs.len() {
            let fragment_width: f32 = text_run.glyphs[glyph_start..]
                .iter()
                .map(|g| g.advance)
                .sum();

            self.add_fragment_to_line(
                box_id,
                fragment_width,
                text_run.ascent,
                text_run.descent,
                glyph_start,
                text_run.glyphs.len(),
            );
        }
    }

    /// Layout an inline element.
    fn layout_inline_element(
        &mut self,
        tree: &mut LayoutTree,
        box_id: LayoutBoxId,
        containing_block: &ContainingBlock,
    ) {
        // Layout children first
        let children: Vec<LayoutBoxId> = tree.children(box_id).collect();

        for child_id in children {
            self.layout_inline_box(tree, child_id, containing_block);
        }
    }

    /// Layout an inline-block element.
    fn layout_inline_block(
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

        // Calculate width and height
        let width = self.resolve_inline_block_width(&style, containing_block);
        let height = self.resolve_inline_block_height(&style, containing_block);

        // Check if it fits
        if self.current_x + width > self.available_width && self.current_x > 0.0 {
            self.finish_line();
            self.start_new_line();
        }

        // Set dimensions
        if let Some(layout_box) = tree.get_mut(box_id) {
            layout_box.dimensions.content.width = width;
            layout_box.dimensions.content.height = height;
        }

        // Add to line
        let ascent = height; // Baseline at bottom for inline-block
        let descent = 0.0;

        self.add_fragment_to_line(box_id, width, ascent, descent, 0, 0);
    }

    /// Layout a replaced element (img, video, etc.).
    fn layout_replaced_element(&mut self, tree: &mut LayoutTree, box_id: LayoutBoxId) {
        let layout_box = match tree.get(box_id) {
            Some(b) => b,
            None => return,
        };

        // Get intrinsic dimensions from replaced content
        let (width, height) = if let Some(ref replaced) = layout_box.replaced {
            (replaced.intrinsic_width.unwrap_or(300.0), replaced.intrinsic_height.unwrap_or(150.0))
        } else {
            (300.0, 150.0) // Default replaced element size
        };

        // Check if it fits
        if self.current_x + width > self.available_width && self.current_x > 0.0 {
            self.finish_line();
            self.start_new_line();
        }

        // Set dimensions
        if let Some(layout_box) = tree.get_mut(box_id) {
            layout_box.dimensions.content.width = width;
            layout_box.dimensions.content.height = height;
        }

        // Add to line
        self.add_fragment_to_line(box_id, width, height, 0.0, 0, 0);
    }

    /// Add a fragment to the current line.
    fn add_fragment_to_line(
        &mut self,
        box_id: LayoutBoxId,
        width: f32,
        ascent: f32,
        descent: f32,
        glyph_start: usize,
        glyph_end: usize,
    ) {
        let fragment = LineFragment {
            x: self.current_x,
            width,
            ascent,
            descent,
            layout_box: box_id,
            glyph_start,
            glyph_end,
        };

        self.current_line.add_fragment(fragment);
        self.current_x += width;
    }

    /// Finish the current line.
    fn finish_line(&mut self) {
        if !self.current_line.fragments.is_empty() {
            self.lines.push(std::mem::replace(
                &mut self.current_line,
                LineBox::new(0.0, 0.0, self.available_width),
            ));
        }
    }

    /// Start a new line.
    fn start_new_line(&mut self) {
        let y = self.lines.iter().map(|l| l.rect.y + l.rect.height).sum();
        self.current_line = LineBox::new(0.0, y, self.available_width);
        self.current_x = 0.0;
    }

    /// Position all lines and their fragments.
    fn position_lines(&mut self, tree: &mut LayoutTree, parent_id: LayoutBoxId) {
        let parent_style = tree.get(parent_id).map(|b| b.style.clone());
        let text_align = parent_style
            .as_ref()
            .map(|s| s.text_align.clone())
            .unwrap_or(TextAlign::Left);

        let mut y = 0.0;

        for line in &mut self.lines {
            line.rect.y = y;

            // Apply text alignment
            let used_width = line.used_width();
            let extra_space = (line.rect.width - used_width).max(0.0);

            let offset = match text_align {
                TextAlign::Left | TextAlign::Start => 0.0,
                TextAlign::Right | TextAlign::End => extra_space,
                TextAlign::Center => extra_space / 2.0,
                TextAlign::Justify => 0.0, // TODO: Implement justify
            };

            // Position fragments
            for fragment in &mut line.fragments {
                fragment.x += offset;

                // Apply vertical alignment
                let vertical_offset = Self::calculate_vertical_offset_for(
                    tree,
                    fragment.layout_box,
                    line.baseline,
                    fragment.ascent,
                );

                // Update layout box position
                if let Some(layout_box) = tree.get_mut(fragment.layout_box) {
                    layout_box.dimensions.content.x = fragment.x;
                    layout_box.dimensions.content.y = y + vertical_offset;
                }
            }

            y += line.rect.height;
        }
    }

    /// Calculate vertical offset for alignment (static version).
    fn calculate_vertical_offset_for(
        tree: &LayoutTree,
        box_id: LayoutBoxId,
        line_baseline: f32,
        fragment_ascent: f32,
    ) -> f32 {
        let layout_box = match tree.get(box_id) {
            Some(b) => b,
            None => return line_baseline - fragment_ascent,
        };

        match layout_box.style.vertical_align {
            VerticalAlign::Baseline => line_baseline - fragment_ascent,
            VerticalAlign::Top => 0.0,
            VerticalAlign::Middle => (line_baseline - fragment_ascent) / 2.0,
            VerticalAlign::Bottom => line_baseline - fragment_ascent, // Simplified
            VerticalAlign::TextTop => 0.0,
            VerticalAlign::TextBottom => line_baseline - fragment_ascent,
            VerticalAlign::Sub => line_baseline - fragment_ascent + fragment_ascent * 0.3,
            VerticalAlign::Super => line_baseline - fragment_ascent - fragment_ascent * 0.3,
            VerticalAlign::Length(l) => line_baseline - fragment_ascent - l,
            VerticalAlign::Percentage(p) => {
                let line_height = layout_box.style.font_size * 1.2; // Approximate
                line_baseline - fragment_ascent - line_height * p / 100.0
            }
        }
    }

    /// Get total height of all lines.
    fn total_height(&self) -> f32 {
        self.lines.iter().map(|l| l.rect.height).sum()
    }

    /// Resolve inline-block width.
    fn resolve_inline_block_width(
        &self,
        style: &ComputedStyle,
        containing_block: &ContainingBlock,
    ) -> f32 {
        use style::computed::SizeValue;

        match &style.width {
            SizeValue::Length(l) => *l,
            SizeValue::Percentage(p) => containing_block.width * p / 100.0,
            SizeValue::Auto => {
                // Shrink-to-fit width
                // For now, use a default
                100.0
            }
            _ => 100.0,
        }
    }

    /// Resolve inline-block height.
    fn resolve_inline_block_height(
        &self,
        style: &ComputedStyle,
        containing_block: &ContainingBlock,
    ) -> f32 {
        use style::computed::SizeValue;

        match &style.height {
            SizeValue::Length(l) => *l,
            SizeValue::Percentage(p) => {
                if let Some(h) = containing_block.height {
                    h * p / 100.0
                } else {
                    style.font_size * 1.2
                }
            }
            SizeValue::Auto => style.font_size * 1.2,
            _ => style.font_size * 1.2,
        }
    }

    /// Get line boxes.
    pub fn lines(&self) -> &[LineBox] {
        &self.lines
    }
}

/// Establish an inline formatting context.
pub fn establish_ifc(
    tree: &mut LayoutTree,
    root_id: LayoutBoxId,
    containing_block: &ContainingBlock,
) -> f32 {
    let mut ifc = InlineFormattingContext::new(containing_block.width);
    ifc.layout_inline_children(tree, root_id, containing_block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ifc_creation() {
        let ifc = InlineFormattingContext::new(800.0);
        assert_eq!(ifc.available_width, 800.0);
        assert!(ifc.lines.is_empty());
    }

    #[test]
    fn test_line_box() {
        let mut line = LineBox::new(0.0, 0.0, 800.0);
        assert_eq!(line.used_width(), 0.0);
        assert_eq!(line.remaining_width(), 800.0);
    }
}
