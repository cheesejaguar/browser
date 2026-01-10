//! Layout box representation.

use crate::box_model::{BoxDimensions, BoxType};
use crate::text::TextRun;
use common::geometry::Rect;
use dom::node::NodeId;
use slotmap::new_key_type;
use smallvec::SmallVec;
use std::sync::Arc;
use style::computed::ComputedStyle;

new_key_type! {
    /// Unique identifier for a layout box.
    pub struct LayoutBoxId;
}

/// A layout box in the layout tree.
#[derive(Clone, Debug)]
pub struct LayoutBox {
    /// Unique ID.
    pub id: LayoutBoxId,
    /// Associated DOM node (if any).
    pub node: Option<NodeId>,
    /// Box type.
    pub box_type: BoxType,
    /// Computed style.
    pub style: Arc<ComputedStyle>,
    /// Box dimensions after layout.
    pub dimensions: BoxDimensions,
    /// Parent box.
    pub parent: Option<LayoutBoxId>,
    /// Child boxes.
    pub children: SmallVec<[LayoutBoxId; 8]>,
    /// Text content (for text boxes).
    pub text: Option<TextRun>,
    /// Replaced content info.
    pub replaced: Option<ReplacedContent>,
    /// Whether this box is a stacking context.
    pub is_stacking_context: bool,
    /// Scroll offset (for overflow: scroll/auto).
    pub scroll_offset: (f32, f32),
    /// Clip rect (if different from content rect).
    pub clip_rect: Option<Rect>,
    /// Additional layout data.
    pub data: LayoutData,
}

impl LayoutBox {
    pub fn new(
        id: LayoutBoxId,
        node: Option<NodeId>,
        box_type: BoxType,
        style: Arc<ComputedStyle>,
    ) -> Self {
        Self {
            id,
            node,
            box_type,
            style: style.clone(),
            dimensions: BoxDimensions::new(),
            parent: None,
            children: SmallVec::new(),
            text: None,
            replaced: None,
            is_stacking_context: style.creates_stacking_context(),
            scroll_offset: (0.0, 0.0),
            clip_rect: None,
            data: LayoutData::None,
        }
    }

    /// Create anonymous block box.
    pub fn anonymous_block(id: LayoutBoxId, style: Arc<ComputedStyle>) -> Self {
        Self::new(id, None, BoxType::AnonymousBlock, style)
    }

    /// Create anonymous inline box.
    pub fn anonymous_inline(id: LayoutBoxId, style: Arc<ComputedStyle>) -> Self {
        Self::new(id, None, BoxType::AnonymousInline, style)
    }

    /// Create text box.
    pub fn text_box(id: LayoutBoxId, style: Arc<ComputedStyle>, text: TextRun) -> Self {
        let mut layout_box = Self::new(id, None, BoxType::Text, style);
        layout_box.text = Some(text);
        layout_box
    }

    /// Get content rect.
    pub fn content_rect(&self) -> Rect {
        self.dimensions.content
    }

    /// Get padding rect.
    pub fn padding_rect(&self) -> Rect {
        self.dimensions.padding_box()
    }

    /// Get border rect.
    pub fn border_rect(&self) -> Rect {
        self.dimensions.border_box()
    }

    /// Get margin rect.
    pub fn margin_rect(&self) -> Rect {
        self.dimensions.margin_box()
    }

    /// Check if box is block-level.
    pub fn is_block(&self) -> bool {
        self.box_type.is_block_level()
    }

    /// Check if box is inline-level.
    pub fn is_inline(&self) -> bool {
        self.box_type.is_inline_level()
    }

    /// Check if box has children.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if box is a flex container.
    pub fn is_flex_container(&self) -> bool {
        matches!(self.box_type, BoxType::Flex)
    }

    /// Check if box is a grid container.
    pub fn is_grid_container(&self) -> bool {
        matches!(self.box_type, BoxType::Grid)
    }

    /// Get the baseline of the box.
    pub fn baseline(&self) -> f32 {
        // Simplified - real implementation would calculate from text metrics
        self.dimensions.content.y + self.dimensions.content.height
    }

    /// Check if box establishes a new block formatting context.
    pub fn establishes_bfc(&self) -> bool {
        self.box_type.establishes_bfc()
            || self.style.overflow_x != style::computed::Overflow::Visible
            || self.style.overflow_y != style::computed::Overflow::Visible
            || self.style.position == style::computed::Position::Absolute
            || self.style.position == style::computed::Position::Fixed
            || self.style.float != style::computed::Float::None
    }

    /// Check if box is positioned (not static).
    pub fn is_positioned(&self) -> bool {
        self.style.position != style::computed::Position::Static
    }

    /// Check if box is absolutely positioned.
    pub fn is_absolutely_positioned(&self) -> bool {
        matches!(
            self.style.position,
            style::computed::Position::Absolute | style::computed::Position::Fixed
        )
    }

    /// Check if box is floating.
    pub fn is_floating(&self) -> bool {
        self.style.float != style::computed::Float::None
    }

    /// Get scroll width.
    pub fn scroll_width(&self) -> f32 {
        // Would calculate from children
        self.dimensions.content.width
    }

    /// Get scroll height.
    pub fn scroll_height(&self) -> f32 {
        // Would calculate from children
        self.dimensions.content.height
    }
}

/// Layout-specific data.
#[derive(Clone, Debug)]
pub enum LayoutData {
    None,
    Flex(FlexLayoutData),
    Grid(GridLayoutData),
    Inline(InlineLayoutData),
    Table(TableLayoutData),
}

/// Flex layout data.
#[derive(Clone, Debug, Default)]
pub struct FlexLayoutData {
    /// Main axis size.
    pub main_size: f32,
    /// Cross axis size.
    pub cross_size: f32,
    /// Flex basis.
    pub flex_basis: f32,
    /// Flex grow factor.
    pub flex_grow: f32,
    /// Flex shrink factor.
    pub flex_shrink: f32,
    /// Violated minimum.
    pub violated_min: bool,
    /// Violated maximum.
    pub violated_max: bool,
    /// Frozen.
    pub frozen: bool,
}

/// Grid layout data.
#[derive(Clone, Debug, Default)]
pub struct GridLayoutData {
    /// Row placement.
    pub row: GridPlacement,
    /// Column placement.
    pub column: GridPlacement,
    /// Resolved row start.
    pub row_start: i32,
    /// Resolved row end.
    pub row_end: i32,
    /// Resolved column start.
    pub column_start: i32,
    /// Resolved column end.
    pub column_end: i32,
}

/// Grid placement.
#[derive(Clone, Debug, Default)]
pub struct GridPlacement {
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub span: Option<u32>,
}

/// Inline layout data.
#[derive(Clone, Debug, Default)]
pub struct InlineLayoutData {
    /// Line boxes this inline is split across.
    pub line_boxes: Vec<LineBoxFragment>,
    /// Total width of all fragments.
    pub total_width: f32,
}

/// Fragment of a line box.
#[derive(Clone, Debug)]
pub struct LineBoxFragment {
    pub rect: Rect,
    pub baseline: f32,
    pub line_index: usize,
}

/// Table layout data.
#[derive(Clone, Debug, Default)]
pub struct TableLayoutData {
    /// Row index.
    pub row: usize,
    /// Column index.
    pub column: usize,
    /// Row span.
    pub row_span: usize,
    /// Column span.
    pub col_span: usize,
}

/// Replaced content info.
#[derive(Clone, Debug)]
pub struct ReplacedContent {
    /// Intrinsic width.
    pub intrinsic_width: Option<f32>,
    /// Intrinsic height.
    pub intrinsic_height: Option<f32>,
    /// Intrinsic ratio (width / height).
    pub intrinsic_ratio: Option<f32>,
    /// Content type.
    pub content_type: ReplacedContentType,
}

/// Type of replaced content.
#[derive(Clone, Debug)]
pub enum ReplacedContentType {
    Image,
    Video,
    Canvas,
    Iframe,
    Object,
    Svg,
}

impl ReplacedContent {
    pub fn image(width: u32, height: u32) -> Self {
        let w = width as f32;
        let h = height as f32;
        Self {
            intrinsic_width: Some(w),
            intrinsic_height: Some(h),
            intrinsic_ratio: if h > 0.0 { Some(w / h) } else { None },
            content_type: ReplacedContentType::Image,
        }
    }

    pub fn video(width: u32, height: u32) -> Self {
        let w = width as f32;
        let h = height as f32;
        Self {
            intrinsic_width: Some(w),
            intrinsic_height: Some(h),
            intrinsic_ratio: if h > 0.0 { Some(w / h) } else { None },
            content_type: ReplacedContentType::Video,
        }
    }

    /// Calculate used dimensions given constraints.
    pub fn compute_size(
        &self,
        specified_width: Option<f32>,
        specified_height: Option<f32>,
        containing_width: f32,
        containing_height: f32,
    ) -> (f32, f32) {
        let iw = self.intrinsic_width;
        let ih = self.intrinsic_height;
        let ratio = self.intrinsic_ratio;

        match (specified_width, specified_height, iw, ih, ratio) {
            // Both specified
            (Some(w), Some(h), _, _, _) => (w, h),
            // Width specified, use ratio
            (Some(w), None, _, _, Some(r)) => (w, w / r),
            // Height specified, use ratio
            (None, Some(h), _, _, Some(r)) => (h * r, h),
            // Width specified, no ratio
            (Some(w), None, _, Some(ih), None) => (w, ih),
            // Height specified, no ratio
            (None, Some(h), Some(iw), _, None) => (iw, h),
            // Neither specified, use intrinsics
            (None, None, Some(iw), Some(ih), _) => (iw, ih),
            // Neither specified, use ratio and containing block
            (None, None, _, _, Some(r)) => {
                if containing_width / containing_height > r {
                    (containing_height * r, containing_height)
                } else {
                    (containing_width, containing_width / r)
                }
            }
            // Fallback to 300x150 (default replaced element size)
            _ => (300.0, 150.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replaced_content_size() {
        let content = ReplacedContent::image(800, 600);

        // Intrinsic size
        let (w, h) = content.compute_size(None, None, 1000.0, 800.0);
        assert_eq!(w, 800.0);
        assert_eq!(h, 600.0);

        // Width specified
        let (w, h) = content.compute_size(Some(400.0), None, 1000.0, 800.0);
        assert_eq!(w, 400.0);
        assert_eq!(h, 300.0);

        // Height specified
        let (w, h) = content.compute_size(None, Some(300.0), 1000.0, 800.0);
        assert_eq!(w, 400.0);
        assert_eq!(h, 300.0);
    }
}
