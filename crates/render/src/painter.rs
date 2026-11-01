//! Painter - generates display lists from layout trees.

use crate::display_list::{
    BlendMode, BorderItem, BorderStyle, BoxShadowItem, ClipRegion, DisplayItem, DisplayItemType,
    DisplayList, FontKey, FontStyle, GlyphInstance, ImageItem, ImageKey,
    ImageRendering, LineItem, LineStyle, SolidColorItem,
    StackingContext, TextItem,
};
use common::color::Color;
use common::geometry::{CornerRadii, Point, Rect, Transform};
use dom::node::NodeId;
use layout::layout_box::{LayoutBox, LayoutBoxId};
use layout::tree::LayoutTree;
use layout::BoxType;
use style::computed::{ComputedStyle, Display, TextDecoration, Visibility, ZIndex};

/// Painter for generating display lists.
pub struct Painter {
    /// Current stacking context ID.
    current_stacking_context: usize,
    /// Stacking context counter.
    stacking_context_counter: usize,
    /// Current clip stack.
    clip_stack: Vec<ClipRegion>,
}

impl Painter {
    pub fn new() -> Self {
        Self {
            current_stacking_context: 0,
            stacking_context_counter: 0,
            clip_stack: Vec::new(),
        }
    }

    /// Paint the layout tree and generate a display list.
    pub fn paint(&mut self, tree: &LayoutTree) -> DisplayList {
        let mut display_list = DisplayList::new();

        if let Some(root_id) = tree.root() {
            self.paint_box(tree, root_id, &mut display_list);
        }

        display_list.sort_for_painting();
        display_list
    }

    /// Paint a single layout box and its children.
    fn paint_box(
        &mut self,
        tree: &LayoutTree,
        box_id: LayoutBoxId,
        display_list: &mut DisplayList,
    ) {
        let layout_box = match tree.get(box_id) {
            Some(b) => b,
            None => return,
        };

        let style = &layout_box.style;

        // Skip invisible elements
        if matches!(style.visibility, Visibility::Hidden | Visibility::Collapse) {
            return;
        }

        if matches!(style.display, Display::None) {
            return;
        }

        // Check if this creates a new stacking context
        let creates_stacking_context = self.creates_stacking_context(layout_box);

        if creates_stacking_context {
            self.push_stacking_context(layout_box, display_list);
        }

        // Get the border box for painting
        let border_rect = layout_box.border_rect();
        let padding_rect = layout_box.padding_rect();
        let content_rect = layout_box.content_rect();

        // 1. Paint background
        self.paint_background(layout_box, &border_rect, &padding_rect, &content_rect, display_list);

        // 2. Paint borders
        self.paint_borders(layout_box, &border_rect, display_list);

        // 3. Paint box shadows
        self.paint_box_shadows(layout_box, &border_rect, display_list);

        // 4. Paint content (text or replaced content)
        self.paint_content(layout_box, &content_rect, display_list);

        // 5. Paint children
        self.paint_children(tree, layout_box, display_list);

        // 6. Paint outline (after children)
        self.paint_outline(layout_box, &border_rect, display_list);

        if creates_stacking_context {
            self.pop_stacking_context();
        }
    }

    /// Check if a box creates a new stacking context.
    fn creates_stacking_context(&self, layout_box: &LayoutBox) -> bool {
        let style = &layout_box.style;

        // Root element always creates a stacking context
        if layout_box.parent.is_none() {
            return true;
        }

        // Positioned elements with z-index != auto
        if !matches!(style.position, style::computed::Position::Static) && style.z_index != ZIndex::Auto {
            return true;
        }

        // Elements with opacity < 1
        if style.opacity < 1.0 {
            return true;
        }

        // Elements with transform
        if !style.transform.is_empty() {
            return true;
        }

        // Flex/grid items with z-index
        if style.z_index != ZIndex::Auto {
            return true;
        }

        // Elements with filter
        // Elements with isolation
        // And other cases...

        false
    }

    /// Push a new stacking context.
    fn push_stacking_context(&mut self, layout_box: &LayoutBox, display_list: &mut DisplayList) {
        self.stacking_context_counter += 1;
        let id = self.stacking_context_counter;

        let style = &layout_box.style;
        let border_rect = layout_box.border_rect();

        let transform = to_geometry_transform(&style.transform);

        let context = StackingContext {
            id,
            parent: Some(self.current_stacking_context),
            z_index: match style.z_index {
                ZIndex::Number(z) => z,
                ZIndex::Auto => 0,
            },
            transform,
            opacity: style.opacity,
            blend_mode: BlendMode::Normal,
            isolation: false,
            bounds: border_rect,
        };

        display_list.push_stacking_context(context);
        self.current_stacking_context = id;
    }

    /// Pop the current stacking context.
    fn pop_stacking_context(&mut self) {
        // In a real implementation, we'd track the parent
        self.current_stacking_context = self.current_stacking_context.saturating_sub(1);
    }

    /// Paint the background of a box.
    fn paint_background(
        &self,
        layout_box: &LayoutBox,
        border_rect: &Rect,
        _padding_rect: &Rect,
        _content_rect: &Rect,
        display_list: &mut DisplayList,
    ) {
        let style = &layout_box.style;

        // Background painting area: the computed style has no background-clip
        // yet, so use its initial value (border-box).
        let bg_rect = border_rect.clone();

        // Paint background color
        if style.background_color.a > 0 {
            let color = Color::rgba(
                style.background_color.r,
                style.background_color.g,
                style.background_color.b,
                style.background_color.a,
            );

            let radii = self.get_border_radii(style);

            let item = DisplayItem::new(
                DisplayItemType::SolidColor(SolidColorItem {
                    color,
                    radii,
                }),
                bg_rect.clone(),
            )
            .with_stacking_context(self.current_stacking_context)
            .with_opacity(style.opacity);

            display_list.push(item);
        }
    }

    /// Paint borders.
    fn paint_borders(&self, layout_box: &LayoutBox, border_rect: &Rect, display_list: &mut DisplayList) {
        let style = &layout_box.style;
        let border_width = &style.border_width;

        // Skip if no borders
        if border_width.top == 0.0
            && border_width.right == 0.0
            && border_width.bottom == 0.0
            && border_width.left == 0.0
        {
            return;
        }

        let widths = [
            border_width.top,
            border_width.right,
            border_width.bottom,
            border_width.left,
        ];

        let colors = [
            Color::rgba(
                style.border_color.top.r,
                style.border_color.top.g,
                style.border_color.top.b,
                style.border_color.top.a,
            ),
            Color::rgba(
                style.border_color.right.r,
                style.border_color.right.g,
                style.border_color.right.b,
                style.border_color.right.a,
            ),
            Color::rgba(
                style.border_color.bottom.r,
                style.border_color.bottom.g,
                style.border_color.bottom.b,
                style.border_color.bottom.a,
            ),
            Color::rgba(
                style.border_color.left.r,
                style.border_color.left.g,
                style.border_color.left.b,
                style.border_color.left.a,
            ),
        ];

        let styles = [
            BorderStyle::from(style.border_style.top.clone()),
            BorderStyle::from(style.border_style.right.clone()),
            BorderStyle::from(style.border_style.bottom.clone()),
            BorderStyle::from(style.border_style.left.clone()),
        ];

        let radii = self.get_border_radii(style);

        let item = DisplayItem::new(
            DisplayItemType::Border(BorderItem {
                widths,
                colors,
                styles,
                radii,
            }),
            border_rect.clone(),
        )
        .with_stacking_context(self.current_stacking_context);

        display_list.push(item);
    }

    /// Get border radii from style.
    fn get_border_radii(&self, style: &ComputedStyle) -> Option<CornerRadii> {
        let radii = &style.border_radius;

        if radii.top_left == 0.0
            && radii.top_right == 0.0
            && radii.bottom_right == 0.0
            && radii.bottom_left == 0.0
        {
            return None;
        }

        Some(CornerRadii {
            top_left: radii.top_left,
            top_right: radii.top_right,
            bottom_right: radii.bottom_right,
            bottom_left: radii.bottom_left,
        })
    }

    /// Paint box shadows.
    fn paint_box_shadows(
        &self,
        layout_box: &LayoutBox,
        border_rect: &Rect,
        display_list: &mut DisplayList,
    ) {
        let style = &layout_box.style;

        for shadow in &style.box_shadow {
            let color = Color::rgba(
                shadow.color.r,
                shadow.color.g,
                shadow.color.b,
                shadow.color.a,
            );

            let radii = self.get_border_radii(style);

            // Calculate shadow bounds
            let spread = shadow.spread_radius;
            let blur = shadow.blur_radius;
            let shadow_rect = if shadow.inset {
                border_rect.clone()
            } else {
                Rect::new(
                    border_rect.x + shadow.offset_x - spread - blur,
                    border_rect.y + shadow.offset_y - spread - blur,
                    border_rect.width + spread * 2.0 + blur * 2.0,
                    border_rect.height + spread * 2.0 + blur * 2.0,
                )
            };

            let item = DisplayItem::new(
                DisplayItemType::BoxShadow(BoxShadowItem {
                    color,
                    offset_x: shadow.offset_x,
                    offset_y: shadow.offset_y,
                    blur_radius: shadow.blur_radius,
                    spread_radius: shadow.spread_radius,
                    inset: shadow.inset,
                    radii,
                }),
                shadow_rect,
            )
            .with_stacking_context(self.current_stacking_context);

            // Inset shadows go after background, outer shadows go before
            display_list.push(item);
        }
    }

    /// Paint content (text or replaced elements).
    fn paint_content(
        &self,
        layout_box: &LayoutBox,
        content_rect: &Rect,
        display_list: &mut DisplayList,
    ) {
        let style = &layout_box.style;

        match layout_box.box_type {
            BoxType::Text => {
                self.paint_text(layout_box, content_rect, display_list);
            }
            BoxType::Replaced => {
                self.paint_replaced(layout_box, content_rect, display_list);
            }
            _ => {}
        }
    }

    /// Paint text content.
    fn paint_text(
        &self,
        layout_box: &LayoutBox,
        content_rect: &Rect,
        display_list: &mut DisplayList,
    ) {
        let style = &layout_box.style;

        let text_run = match &layout_box.text {
            Some(tr) => tr,
            None => return,
        };

        let color = Color::rgba(
            style.color.r,
            style.color.g,
            style.color.b,
            style.color.a,
        );

        // Create glyph instances
        let mut glyphs = Vec::with_capacity(text_run.glyphs.len());
        let mut x = content_rect.x;

        for glyph in &text_run.glyphs {
            glyphs.push(GlyphInstance {
                glyph_index: glyph.glyph_id,
                point: Point::new(x + glyph.x_offset, content_rect.y + text_run.baseline() + glyph.y_offset),
            });
            x += glyph.advance;
        }

        let font_key = FontKey {
            family: style.font_family.first().cloned().unwrap_or_else(|| "sans-serif".to_string()),
            weight: style.font_weight.to_number(),
            style: if style.font_style == style::computed::FontStyle::Italic {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            },
        };

        let item = DisplayItem::new(
            DisplayItemType::Text(TextItem {
                text: text_run.text.clone(),
                glyphs,
                font_key,
                font_size: style.font_size,
                color,
                baseline: text_run.baseline(),
            }),
            *content_rect,
        )
        .with_stacking_context(self.current_stacking_context);

        display_list.push(item);

        // Paint text decorations
        self.paint_text_decorations(layout_box, content_rect, text_run, display_list);
    }

    /// Paint text decorations.
    fn paint_text_decorations(
        &self,
        layout_box: &LayoutBox,
        content_rect: &Rect,
        text_run: &layout::text::TextRun,
        display_list: &mut DisplayList,
    ) {
        let style = &layout_box.style;

        let decoration_color =
            Color::rgba(style.color.r, style.color.g, style.color.b, style.color.a);

        let line_thickness = style.font_size * 0.05; // Approximate

        {
            match style.text_decoration {
                TextDecoration::Underline => {
                    let y = content_rect.y + text_run.baseline() + text_run.descent * 0.3;
                    self.paint_decoration_line(
                        content_rect.x,
                        y,
                        text_run.width,
                        line_thickness,
                        decoration_color.clone(),
                        LineStyle::Solid,
                        display_list,
                    );
                }
                TextDecoration::Overline => {
                    let y = content_rect.y;
                    self.paint_decoration_line(
                        content_rect.x,
                        y,
                        text_run.width,
                        line_thickness,
                        decoration_color.clone(),
                        LineStyle::Solid,
                        display_list,
                    );
                }
                TextDecoration::LineThrough => {
                    let y = content_rect.y + text_run.ascent * 0.5;
                    self.paint_decoration_line(
                        content_rect.x,
                        y,
                        text_run.width,
                        line_thickness,
                        decoration_color.clone(),
                        LineStyle::Solid,
                        display_list,
                    );
                }
                TextDecoration::None => {}
            }
        }
    }

    /// Paint a decoration line.
    fn paint_decoration_line(
        &self,
        x: f32,
        y: f32,
        width: f32,
        thickness: f32,
        color: Color,
        line_style: LineStyle,
        display_list: &mut DisplayList,
    ) {

        let item = DisplayItem::new(
            DisplayItemType::Line(LineItem {
                start: Point::new(x, y),
                end: Point::new(x + width, y),
                width: thickness,
                color,
                style: line_style,
            }),
            Rect::new(x, y - thickness / 2.0, width, thickness),
        )
        .with_stacking_context(self.current_stacking_context);

        display_list.push(item);
    }

    /// Paint replaced content (images, etc.).
    fn paint_replaced(
        &self,
        layout_box: &LayoutBox,
        content_rect: &Rect,
        display_list: &mut DisplayList,
    ) {
        if layout_box.replaced.is_none() {
            return;
        }

        // Replaced content carries no image identifier of its own; key the
        // image by the DOM node that generated this box.
        let Some(node) = layout_box.node else {
            return;
        };
        let image_key = ImageKey(hash_string(&format!("{:?}", node)));

        let item = DisplayItem::new(
            DisplayItemType::Image(ImageItem {
                image_key,
                src_rect: None,
                rendering: ImageRendering::Auto,
            }),
            *content_rect,
        )
        .with_stacking_context(self.current_stacking_context);

        display_list.push(item);
    }

    /// Paint children.
    fn paint_children(
        &mut self,
        tree: &LayoutTree,
        layout_box: &LayoutBox,
        display_list: &mut DisplayList,
    ) {
        // Handle overflow clipping
        let style = &layout_box.style;
        let needs_clip = matches!(
            style.overflow_x,
            style::computed::Overflow::Hidden | style::computed::Overflow::Scroll | style::computed::Overflow::Auto
        ) || matches!(
            style.overflow_y,
            style::computed::Overflow::Hidden | style::computed::Overflow::Scroll | style::computed::Overflow::Auto
        );

        if needs_clip {
            let clip = ClipRegion::rect(layout_box.padding_rect());
            display_list.push(DisplayItem::new(
                DisplayItemType::PushClip(clip.clone()),
                layout_box.padding_rect(),
            ));
            self.clip_stack.push(clip);
        }

        // Paint children in order
        for &child_id in &layout_box.children {
            self.paint_box(tree, child_id, display_list);
        }

        if needs_clip {
            display_list.push(DisplayItem::new(
                DisplayItemType::PopClip,
                Rect::default(),
            ));
            self.clip_stack.pop();
        }
    }

    /// Paint outline.
    fn paint_outline(
        &self,
        layout_box: &LayoutBox,
        border_rect: &Rect,
        display_list: &mut DisplayList,
    ) {
        let style = &layout_box.style;

        if style.outline_width == 0.0 {
            return;
        }

        let outline_rect = Rect::new(
            border_rect.x - style.outline_offset - style.outline_width,
            border_rect.y - style.outline_offset - style.outline_width,
            border_rect.width + 2.0 * (style.outline_offset + style.outline_width),
            border_rect.height + 2.0 * (style.outline_offset + style.outline_width),
        );

        let color = Color::rgba(
            style.outline_color.r,
            style.outline_color.g,
            style.outline_color.b,
            style.outline_color.a,
        );

        let item = DisplayItem::new(
            DisplayItemType::Border(BorderItem {
                widths: [style.outline_width; 4],
                colors: [color.clone(), color.clone(), color.clone(), color],
                styles: [BorderStyle::from(style.outline_style.clone()); 4],
                radii: None,
            }),
            outline_rect,
        )
        .with_stacking_context(self.current_stacking_context);

        display_list.push(item);
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple hash function for strings (used for image keys).
/// Convert a computed `transform` list to a 2D affine transform.
///
/// Functions are applied right-to-left as in CSS. Returns `None` when there is
/// no transform or when it contains functions without a 2D affine mapping here.
fn to_geometry_transform(list: &[style::computed::Transform]) -> Option<Transform> {
    use style::computed::Transform as T;
    if list.is_empty() {
        return None;
    }
    let mut acc = Transform::identity();
    for t in list.iter().rev() {
        let m = match *t {
            T::Translate(x, y) => Transform::translation(x, y),
            T::TranslateX(x) => Transform::translation(x, 0.0),
            T::TranslateY(y) => Transform::translation(0.0, y),
            T::Scale(x, y) => Transform::scale(x, y),
            T::ScaleX(x) => Transform::scale(x, 1.0),
            T::ScaleY(y) => Transform::scale(1.0, y),
            T::Matrix(a, b, c, d, e, f) => Transform {
                m11: a,
                m12: b,
                m21: c,
                m22: d,
                m31: e,
                m32: f,
            },
            _ => return None,
        };
        acc = acc.then(&m);
    }
    Some(acc)
}

fn hash_string(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_painter_creation() {
        let painter = Painter::new();
        assert_eq!(painter.current_stacking_context, 0);
    }

    #[test]
    fn test_empty_tree_painting() {
        let mut painter = Painter::new();
        let tree = LayoutTree::new();
        let display_list = painter.paint(&tree);
        assert!(display_list.is_empty());
    }
}
