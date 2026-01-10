//! Computed style values.

use common::color::Color;
use common::geometry::EdgeSizes;
use common::units::{Length, LengthContext, LengthPercentage};
use css_parser::properties::PropertyId;
use css_parser::values::CssValue;
use std::collections::HashMap;

/// Computed style for an element.
#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    /// Display type.
    pub display: Display,
    /// Position type.
    pub position: Position,
    /// Box sizing.
    pub box_sizing: BoxSizing,

    /// Dimensions.
    pub width: SizeValue,
    pub height: SizeValue,
    pub min_width: SizeValue,
    pub min_height: SizeValue,
    pub max_width: SizeValue,
    pub max_height: SizeValue,

    /// Margin.
    pub margin: EdgeValues,
    /// Padding.
    pub padding: EdgeValues,
    /// Border width.
    pub border_width: EdgeSizes,
    /// Border style.
    pub border_style: BorderStyles,
    /// Border color.
    pub border_color: BorderColors,
    /// Border radius.
    pub border_radius: CornerValues,

    /// Position offsets.
    pub top: SizeValue,
    pub right: SizeValue,
    pub bottom: SizeValue,
    pub left: SizeValue,

    /// Z-index.
    pub z_index: ZIndex,
    /// Float.
    pub float: Float,
    /// Clear.
    pub clear: Clear,
    /// Overflow.
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    /// Visibility.
    pub visibility: Visibility,
    /// Opacity.
    pub opacity: f32,

    /// Colors.
    pub color: Color,
    pub background_color: Color,

    /// Typography.
    pub font_family: Vec<String>,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub line_height: LineHeight,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub word_break: WordBreak,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub vertical_align: VerticalAlign,

    /// Flexbox.
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub align_self: AlignSelf,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: SizeValue,
    pub order: i32,
    pub gap: f32,
    pub row_gap: f32,
    pub column_gap: f32,

    /// Grid.
    pub grid_template_columns: Vec<GridTrack>,
    pub grid_template_rows: Vec<GridTrack>,
    pub grid_auto_flow: GridAutoFlow,

    /// Transform.
    pub transform: Vec<Transform>,
    pub transform_origin: (f32, f32),

    /// Cursor.
    pub cursor: Cursor,
    /// Pointer events.
    pub pointer_events: PointerEvents,
    /// User select.
    pub user_select: UserSelect,

    /// List style.
    pub list_style_type: ListStyleType,
    pub list_style_position: ListStylePosition,

    /// Table.
    pub border_collapse: BorderCollapse,

    /// Outline.
    pub outline_width: f32,
    pub outline_style: BorderStyle,
    pub outline_color: Color,
    pub outline_offset: f32,

    /// Box shadow.
    pub box_shadow: Vec<BoxShadow>,
    /// Text shadow.
    pub text_shadow: Vec<TextShadow>,

    /// Filter.
    pub filter: Vec<Filter>,
    /// Backdrop filter.
    pub backdrop_filter: Vec<Filter>,
    /// Mix blend mode.
    pub mix_blend_mode: BlendMode,
}

impl ComputedStyle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get default style.
    pub fn default_style() -> Self {
        Self {
            display: Display::Inline,
            position: Position::Static,
            box_sizing: BoxSizing::ContentBox,
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            min_width: SizeValue::Auto,
            min_height: SizeValue::Auto,
            max_width: SizeValue::None,
            max_height: SizeValue::None,
            margin: EdgeValues::zero(),
            padding: EdgeValues::zero(),
            border_width: EdgeSizes::ZERO,
            border_style: BorderStyles::default(),
            border_color: BorderColors::default(),
            border_radius: CornerValues::zero(),
            top: SizeValue::Auto,
            right: SizeValue::Auto,
            bottom: SizeValue::Auto,
            left: SizeValue::Auto,
            z_index: ZIndex::Auto,
            float: Float::None,
            clear: Clear::None,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            visibility: Visibility::Visible,
            opacity: 1.0,
            color: Color::BLACK,
            background_color: Color::TRANSPARENT,
            font_family: vec!["sans-serif".to_string()],
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            line_height: LineHeight::Normal,
            text_align: TextAlign::Start,
            text_decoration: TextDecoration::None,
            text_transform: TextTransform::None,
            white_space: WhiteSpace::Normal,
            word_break: WordBreak::Normal,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            vertical_align: VerticalAlign::Baseline,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            align_self: AlignSelf::Auto,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: SizeValue::Auto,
            order: 0,
            gap: 0.0,
            row_gap: 0.0,
            column_gap: 0.0,
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_auto_flow: GridAutoFlow::Row,
            transform: Vec::new(),
            transform_origin: (50.0, 50.0),
            cursor: Cursor::Auto,
            pointer_events: PointerEvents::Auto,
            user_select: UserSelect::Auto,
            list_style_type: ListStyleType::Disc,
            list_style_position: ListStylePosition::Outside,
            border_collapse: BorderCollapse::Separate,
            outline_width: 0.0,
            outline_style: BorderStyle::None,
            outline_color: Color::BLACK,
            outline_offset: 0.0,
            box_shadow: Vec::new(),
            text_shadow: Vec::new(),
            filter: Vec::new(),
            backdrop_filter: Vec::new(),
            mix_blend_mode: BlendMode::Normal,
        }
    }

    /// Check if element creates a new stacking context.
    pub fn creates_stacking_context(&self) -> bool {
        self.position == Position::Fixed
            || self.position == Position::Sticky
            || (self.position != Position::Static && !matches!(self.z_index, ZIndex::Auto))
            || self.opacity < 1.0
            || !self.transform.is_empty()
            || !self.filter.is_empty()
            || self.mix_blend_mode != BlendMode::Normal
    }

    /// Check if element is block-level.
    pub fn is_block_level(&self) -> bool {
        matches!(
            self.display,
            Display::Block
                | Display::Flex
                | Display::Grid
                | Display::Table
                | Display::ListItem
                | Display::FlowRoot
        )
    }

    /// Check if element is inline-level.
    pub fn is_inline_level(&self) -> bool {
        matches!(
            self.display,
            Display::Inline | Display::InlineBlock | Display::InlineFlex | Display::InlineGrid
        )
    }

    /// Get total horizontal margin.
    pub fn horizontal_margin(&self) -> f32 {
        self.margin.left.resolve(0.0) + self.margin.right.resolve(0.0)
    }

    /// Get total vertical margin.
    pub fn vertical_margin(&self) -> f32 {
        self.margin.top.resolve(0.0) + self.margin.bottom.resolve(0.0)
    }

    /// Get total horizontal padding.
    pub fn horizontal_padding(&self) -> f32 {
        self.padding.left.resolve(0.0) + self.padding.right.resolve(0.0)
    }

    /// Get total vertical padding.
    pub fn vertical_padding(&self) -> f32 {
        self.padding.top.resolve(0.0) + self.padding.bottom.resolve(0.0)
    }

    /// Get total horizontal border.
    pub fn horizontal_border(&self) -> f32 {
        self.border_width.left + self.border_width.right
    }

    /// Get total vertical border.
    pub fn vertical_border(&self) -> f32 {
        self.border_width.top + self.border_width.bottom
    }
}

/// Display value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    None,
    Block,
    #[default]
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Table,
    TableRow,
    TableCell,
    TableColumn,
    TableCaption,
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableColumnGroup,
    ListItem,
    FlowRoot,
    Contents,
}

/// Position value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// Box sizing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

/// Size value (width, height, etc.).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SizeValue {
    #[default]
    Auto,
    Length(f32),
    Percentage(f32),
    MinContent,
    MaxContent,
    FitContent,
    None, // For max-width/height
}

impl SizeValue {
    pub fn resolve(&self, containing: f32) -> f32 {
        match self {
            SizeValue::Auto | SizeValue::None => 0.0,
            SizeValue::Length(l) => *l,
            SizeValue::Percentage(p) => containing * p / 100.0,
            SizeValue::MinContent | SizeValue::MaxContent | SizeValue::FitContent => 0.0,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, SizeValue::Auto)
    }
}

/// Edge values (margin, padding).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EdgeValues {
    pub top: SizeValue,
    pub right: SizeValue,
    pub bottom: SizeValue,
    pub left: SizeValue,
}

impl EdgeValues {
    pub fn zero() -> Self {
        Self {
            top: SizeValue::Length(0.0),
            right: SizeValue::Length(0.0),
            bottom: SizeValue::Length(0.0),
            left: SizeValue::Length(0.0),
        }
    }
}

/// Corner values (border radius).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CornerValues {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerValues {
    pub fn zero() -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        }
    }
}

/// Border styles.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BorderStyles {
    pub top: BorderStyle,
    pub right: BorderStyle,
    pub bottom: BorderStyle,
    pub left: BorderStyle,
}

/// Border colors.
#[derive(Clone, Debug, PartialEq)]
pub struct BorderColors {
    pub top: Color,
    pub right: Color,
    pub bottom: Color,
    pub left: Color,
}

impl Default for BorderColors {
    fn default() -> Self {
        Self {
            top: Color::BLACK,
            right: Color::BLACK,
            bottom: Color::BLACK,
            left: Color::BLACK,
        }
    }
}

/// Border style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BorderStyle {
    #[default]
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// Z-index value.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ZIndex {
    #[default]
    Auto,
    Number(i32),
}

/// Float value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
    InlineStart,
    InlineEnd,
}

/// Clear value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Clear {
    #[default]
    None,
    Left,
    Right,
    Both,
    InlineStart,
    InlineEnd,
}

/// Overflow value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

/// Visibility value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

/// Font weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontWeight {
    Thin,      // 100
    ExtraLight,// 200
    Light,     // 300
    #[default]
    Normal,    // 400
    Medium,    // 500
    SemiBold,  // 600
    Bold,      // 700
    ExtraBold, // 800
    Black,     // 900
    Number(u16),
}

impl FontWeight {
    pub fn to_number(&self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
            FontWeight::Number(n) => *n,
        }
    }
}

/// Font style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Line height.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum LineHeight {
    #[default]
    Normal,
    Number(f32),
    Length(f32),
    Percentage(f32),
}

/// Text align.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

/// Text decoration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Overline,
    LineThrough,
}

/// Text transform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextTransform {
    #[default]
    None,
    Capitalize,
    Uppercase,
    Lowercase,
}

/// White space handling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WhiteSpace {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

/// Word break.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WordBreak {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

/// Vertical align.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Sub,
    Super,
    Top,
    TextTop,
    Middle,
    Bottom,
    TextBottom,
    Length(f32),
    Percentage(f32),
}

/// Flex direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flex wrap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Justify content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    #[default]
    Stretch,
}

/// Align content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    #[default]
    Stretch,
}

/// Align self.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

/// Grid track size.
#[derive(Clone, Debug, PartialEq)]
pub enum GridTrack {
    Length(f32),
    Percentage(f32),
    Fr(f32),
    Auto,
    MinContent,
    MaxContent,
    MinMax(Box<GridTrack>, Box<GridTrack>),
    Repeat(u32, Vec<GridTrack>),
}

/// Grid auto flow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

/// Transform function.
#[derive(Clone, Debug, PartialEq)]
pub enum Transform {
    Translate(f32, f32),
    TranslateX(f32),
    TranslateY(f32),
    TranslateZ(f32),
    Translate3d(f32, f32, f32),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    ScaleZ(f32),
    Scale3d(f32, f32, f32),
    Rotate(f32),
    RotateX(f32),
    RotateY(f32),
    RotateZ(f32),
    Rotate3d(f32, f32, f32, f32),
    Skew(f32, f32),
    SkewX(f32),
    SkewY(f32),
    Matrix(f32, f32, f32, f32, f32, f32),
    Matrix3d([f32; 16]),
    Perspective(f32),
}

/// Cursor type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cursor {
    #[default]
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ColResize,
    RowResize,
    NResize,
    EResize,
    SResize,
    WResize,
    NeResize,
    NwResize,
    SeResize,
    SwResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ZoomIn,
    ZoomOut,
}

/// Pointer events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
    VisiblePainted,
    VisibleFill,
    VisibleStroke,
    Visible,
    Painted,
    Fill,
    Stroke,
    All,
}

/// User select.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UserSelect {
    #[default]
    Auto,
    None,
    Text,
    All,
    Contain,
}

/// List style type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListStyleType {
    None,
    #[default]
    Disc,
    Circle,
    Square,
    Decimal,
    DecimalLeadingZero,
    LowerRoman,
    UpperRoman,
    LowerLatin,
    UpperLatin,
    LowerAlpha,
    UpperAlpha,
    LowerGreek,
}

/// List style position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListStylePosition {
    Inside,
    #[default]
    Outside,
}

/// Border collapse.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BorderCollapse {
    #[default]
    Separate,
    Collapse,
}

/// Box shadow.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: Color,
    pub inset: bool,
}

/// Text shadow.
#[derive(Clone, Debug, PartialEq)]
pub struct TextShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: Color,
}

/// CSS filter.
#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(f32),
    Saturate(f32),
    Sepia(f32),
    DropShadow(f32, f32, f32, Color),
}

/// Blend mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}
