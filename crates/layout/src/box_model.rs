//! CSS Box Model implementation.

use common::geometry::{EdgeSizes, Point, Rect, Size};
use style::computed::ComputedStyle;

/// Box type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxType {
    /// Block-level box.
    Block,
    /// Inline-level box.
    Inline,
    /// Inline-block box.
    InlineBlock,
    /// Flex container.
    Flex,
    /// Flex item.
    FlexItem,
    /// Grid container.
    Grid,
    /// Grid item.
    GridItem,
    /// Anonymous block box.
    AnonymousBlock,
    /// Anonymous inline box.
    AnonymousInline,
    /// Table box.
    Table,
    /// Table row.
    TableRow,
    /// Table cell.
    TableCell,
    /// List item.
    ListItem,
    /// Text run.
    Text,
    /// Replaced element (img, video, etc).
    Replaced,
    /// No box (display: none).
    None,
}

impl BoxType {
    pub fn from_style(style: &ComputedStyle) -> Self {
        use style::computed::Display;

        match style.display {
            Display::None => BoxType::None,
            Display::Block => BoxType::Block,
            Display::Inline => BoxType::Inline,
            Display::InlineBlock => BoxType::InlineBlock,
            Display::Flex => BoxType::Flex,
            Display::InlineFlex => BoxType::Flex,
            Display::Grid => BoxType::Grid,
            Display::InlineGrid => BoxType::Grid,
            Display::Table => BoxType::Table,
            Display::TableRow => BoxType::TableRow,
            Display::TableCell => BoxType::TableCell,
            Display::ListItem => BoxType::ListItem,
            Display::FlowRoot => BoxType::Block,
            Display::Contents => BoxType::None,
            _ => BoxType::Block,
        }
    }

    pub fn is_block_level(&self) -> bool {
        matches!(
            self,
            BoxType::Block
                | BoxType::Flex
                | BoxType::Grid
                | BoxType::Table
                | BoxType::ListItem
                | BoxType::AnonymousBlock
        )
    }

    pub fn is_inline_level(&self) -> bool {
        matches!(
            self,
            BoxType::Inline
                | BoxType::InlineBlock
                | BoxType::AnonymousInline
                | BoxType::Text
        )
    }

    pub fn is_replaced(&self) -> bool {
        matches!(self, BoxType::Replaced)
    }

    pub fn establishes_bfc(&self) -> bool {
        matches!(
            self,
            BoxType::Block | BoxType::Flex | BoxType::Grid | BoxType::Table
        )
    }
}

/// Box dimensions.
#[derive(Clone, Debug, Default)]
pub struct BoxDimensions {
    /// Content area.
    pub content: Rect,
    /// Padding.
    pub padding: EdgeSizes,
    /// Border.
    pub border: EdgeSizes,
    /// Margin.
    pub margin: EdgeSizes,
}

impl BoxDimensions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get padding box (content + padding).
    pub fn padding_box(&self) -> Rect {
        Rect::new(
            self.content.x - self.padding.left,
            self.content.y - self.padding.top,
            self.content.width + self.padding.left + self.padding.right,
            self.content.height + self.padding.top + self.padding.bottom,
        )
    }

    /// Get border box (content + padding + border).
    pub fn border_box(&self) -> Rect {
        let padding_box = self.padding_box();
        Rect::new(
            padding_box.x - self.border.left,
            padding_box.y - self.border.top,
            padding_box.width + self.border.left + self.border.right,
            padding_box.height + self.border.top + self.border.bottom,
        )
    }

    /// Get margin box (content + padding + border + margin).
    pub fn margin_box(&self) -> Rect {
        let border_box = self.border_box();
        Rect::new(
            border_box.x - self.margin.left,
            border_box.y - self.margin.top,
            border_box.width + self.margin.left + self.margin.right,
            border_box.height + self.margin.top + self.margin.bottom,
        )
    }

    /// Get total horizontal space (padding + border + margin).
    pub fn horizontal_space(&self) -> f32 {
        self.padding.left
            + self.padding.right
            + self.border.left
            + self.border.right
            + self.margin.left
            + self.margin.right
    }

    /// Get total vertical space (padding + border + margin).
    pub fn vertical_space(&self) -> f32 {
        self.padding.top
            + self.padding.bottom
            + self.border.top
            + self.border.bottom
            + self.margin.top
            + self.margin.bottom
    }

    /// Set content size.
    pub fn set_content_size(&mut self, width: f32, height: f32) {
        self.content.width = width;
        self.content.height = height;
    }

    /// Set content position.
    pub fn set_content_position(&mut self, x: f32, y: f32) {
        self.content.x = x;
        self.content.y = y;
    }

    /// Expand margins (for auto margins).
    pub fn expand_margin_left(&mut self, extra: f32) {
        self.margin.left += extra;
        self.content.x += extra;
    }

    /// Expand margins (for auto margins).
    pub fn expand_margin_right(&mut self, extra: f32) {
        self.margin.right += extra;
    }
}

/// Containing block for layout.
#[derive(Clone, Debug)]
pub struct ContainingBlock {
    pub width: f32,
    pub height: Option<f32>,
    pub x: f32,
    pub y: f32,
}

impl ContainingBlock {
    pub fn new(width: f32, height: Option<f32>) -> Self {
        Self {
            width,
            height,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn from_dimensions(dims: &BoxDimensions) -> Self {
        Self {
            width: dims.content.width,
            height: Some(dims.content.height),
            x: dims.content.x,
            y: dims.content.y,
        }
    }
}

/// Used values after layout.
#[derive(Clone, Debug, Default)]
pub struct UsedValues {
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub border_top: f32,
    pub border_right: f32,
    pub border_bottom: f32,
    pub border_left: f32,
}

impl UsedValues {
    pub fn from_style(style: &ComputedStyle, containing_width: f32) -> Self {
        use style::computed::SizeValue;

        let resolve_size = |value: &SizeValue| -> f32 {
            match value {
                SizeValue::Length(l) => *l,
                SizeValue::Percentage(p) => containing_width * p / 100.0,
                _ => 0.0,
            }
        };

        Self {
            width: match &style.width {
                SizeValue::Auto => 0.0,
                SizeValue::Length(l) => *l,
                SizeValue::Percentage(p) => containing_width * p / 100.0,
                _ => 0.0,
            },
            height: match &style.height {
                SizeValue::Auto => 0.0,
                SizeValue::Length(l) => *l,
                SizeValue::Percentage(p) => containing_width * p / 100.0,
                _ => 0.0,
            },
            margin_top: resolve_size(&style.margin.top),
            margin_right: resolve_size(&style.margin.right),
            margin_bottom: resolve_size(&style.margin.bottom),
            margin_left: resolve_size(&style.margin.left),
            padding_top: resolve_size(&style.padding.top),
            padding_right: resolve_size(&style.padding.right),
            padding_bottom: resolve_size(&style.padding.bottom),
            padding_left: resolve_size(&style.padding.left),
            border_top: style.border_width.top,
            border_right: style.border_width.right,
            border_bottom: style.border_width.bottom,
            border_left: style.border_width.left,
        }
    }
}

/// Margin collapsing state.
#[derive(Clone, Debug, Default)]
pub struct MarginCollapseState {
    /// Positive collapsed margin.
    pub positive: f32,
    /// Negative collapsed margin.
    pub negative: f32,
}

impl MarginCollapseState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a margin to the collapse state.
    pub fn add(&mut self, margin: f32) {
        if margin >= 0.0 {
            self.positive = self.positive.max(margin);
        } else {
            self.negative = self.negative.min(margin);
        }
    }

    /// Get the collapsed margin value.
    pub fn collapsed(&self) -> f32 {
        self.positive + self.negative
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        self.positive = 0.0;
        self.negative = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_dimensions() {
        let mut dims = BoxDimensions::new();
        dims.content = Rect::new(10.0, 10.0, 100.0, 50.0);
        dims.padding = EdgeSizes::all(5.0);
        dims.border = EdgeSizes::all(1.0);
        dims.margin = EdgeSizes::all(10.0);

        let padding_box = dims.padding_box();
        assert_eq!(padding_box.width, 110.0); // 100 + 5 + 5
        assert_eq!(padding_box.height, 60.0); // 50 + 5 + 5

        let border_box = dims.border_box();
        assert_eq!(border_box.width, 112.0); // 110 + 1 + 1
        assert_eq!(border_box.height, 62.0); // 60 + 1 + 1

        let margin_box = dims.margin_box();
        assert_eq!(margin_box.width, 132.0); // 112 + 10 + 10
        assert_eq!(margin_box.height, 82.0); // 62 + 10 + 10
    }

    #[test]
    fn test_margin_collapse() {
        let mut state = MarginCollapseState::new();
        state.add(20.0);
        state.add(30.0);
        assert_eq!(state.collapsed(), 30.0);

        state.add(-10.0);
        assert_eq!(state.collapsed(), 20.0); // 30 + (-10)
    }
}
