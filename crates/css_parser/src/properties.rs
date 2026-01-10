//! CSS Properties.

use crate::values::CssValue;
use std::fmt;

/// CSS Property ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyId {
    // Display and positioning
    Display,
    Position,
    Top,
    Right,
    Bottom,
    Left,
    Float,
    Clear,
    ZIndex,
    Visibility,
    Overflow,
    OverflowX,
    OverflowY,

    // Box model
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    Border,
    BorderWidth,
    BorderStyle,
    BorderColor,
    BorderTop,
    BorderTopWidth,
    BorderTopStyle,
    BorderTopColor,
    BorderRight,
    BorderRightWidth,
    BorderRightStyle,
    BorderRightColor,
    BorderBottom,
    BorderBottomWidth,
    BorderBottomStyle,
    BorderBottomColor,
    BorderLeft,
    BorderLeftWidth,
    BorderLeftStyle,
    BorderLeftColor,
    BorderRadius,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomRightRadius,
    BorderBottomLeftRadius,
    BoxSizing,

    // Background
    Background,
    BackgroundColor,
    BackgroundImage,
    BackgroundPosition,
    BackgroundPositionX,
    BackgroundPositionY,
    BackgroundSize,
    BackgroundRepeat,
    BackgroundAttachment,
    BackgroundOrigin,
    BackgroundClip,

    // Typography
    Color,
    Font,
    FontFamily,
    FontSize,
    FontWeight,
    FontStyle,
    FontVariant,
    FontStretch,
    LineHeight,
    LetterSpacing,
    WordSpacing,
    TextAlign,
    TextDecoration,
    TextDecorationLine,
    TextDecorationColor,
    TextDecorationStyle,
    TextTransform,
    TextIndent,
    TextShadow,
    TextOverflow,
    WhiteSpace,
    WordBreak,
    WordWrap,
    OverflowWrap,

    // Flexbox
    FlexDirection,
    FlexWrap,
    FlexFlow,
    JustifyContent,
    AlignItems,
    AlignContent,
    Order,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    Flex,
    AlignSelf,
    Gap,
    RowGap,
    ColumnGap,

    // Grid
    GridTemplateColumns,
    GridTemplateRows,
    GridTemplateAreas,
    GridTemplate,
    GridAutoColumns,
    GridAutoRows,
    GridAutoFlow,
    Grid,
    GridRowStart,
    GridRowEnd,
    GridColumnStart,
    GridColumnEnd,
    GridRow,
    GridColumn,
    GridArea,
    JustifyItems,
    JustifySelf,
    PlaceContent,
    PlaceItems,
    PlaceSelf,

    // Transforms
    Transform,
    TransformOrigin,
    TransformStyle,
    Perspective,
    PerspectiveOrigin,
    Backface,

    // Transitions
    Transition,
    TransitionProperty,
    TransitionDuration,
    TransitionTimingFunction,
    TransitionDelay,

    // Animations
    Animation,
    AnimationName,
    AnimationDuration,
    AnimationTimingFunction,
    AnimationDelay,
    AnimationIterationCount,
    AnimationDirection,
    AnimationFillMode,
    AnimationPlayState,

    // Filters & Effects
    Filter,
    BackdropFilter,
    Opacity,
    MixBlendMode,
    BoxShadow,

    // Lists
    ListStyle,
    ListStyleType,
    ListStylePosition,
    ListStyleImage,

    // Tables
    BorderCollapse,
    BorderSpacing,
    TableLayout,
    CaptionSide,
    EmptyCells,

    // UI
    Cursor,
    Outline,
    OutlineWidth,
    OutlineStyle,
    OutlineColor,
    OutlineOffset,
    Resize,
    UserSelect,
    PointerEvents,

    // Content
    Content,
    Quotes,
    CounterReset,
    CounterIncrement,

    // Misc
    ObjectFit,
    ObjectPosition,
    VerticalAlign,
    Direction,
    UnicodeBidi,
    WritingMode,
    TextOrientation,
    ImageRendering,

    // SVG
    Fill,
    Stroke,
    StrokeWidth,

    // Print
    PageBreakBefore,
    PageBreakAfter,
    PageBreakInside,

    // Custom/Unknown
    Custom(String),
}

impl PropertyId {
    /// Get property from name string.
    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            // Display and positioning
            "display" => PropertyId::Display,
            "position" => PropertyId::Position,
            "top" => PropertyId::Top,
            "right" => PropertyId::Right,
            "bottom" => PropertyId::Bottom,
            "left" => PropertyId::Left,
            "float" => PropertyId::Float,
            "clear" => PropertyId::Clear,
            "z-index" => PropertyId::ZIndex,
            "visibility" => PropertyId::Visibility,
            "overflow" => PropertyId::Overflow,
            "overflow-x" => PropertyId::OverflowX,
            "overflow-y" => PropertyId::OverflowY,

            // Box model
            "width" => PropertyId::Width,
            "height" => PropertyId::Height,
            "min-width" => PropertyId::MinWidth,
            "min-height" => PropertyId::MinHeight,
            "max-width" => PropertyId::MaxWidth,
            "max-height" => PropertyId::MaxHeight,
            "margin" => PropertyId::Margin,
            "margin-top" => PropertyId::MarginTop,
            "margin-right" => PropertyId::MarginRight,
            "margin-bottom" => PropertyId::MarginBottom,
            "margin-left" => PropertyId::MarginLeft,
            "padding" => PropertyId::Padding,
            "padding-top" => PropertyId::PaddingTop,
            "padding-right" => PropertyId::PaddingRight,
            "padding-bottom" => PropertyId::PaddingBottom,
            "padding-left" => PropertyId::PaddingLeft,
            "border" => PropertyId::Border,
            "border-width" => PropertyId::BorderWidth,
            "border-style" => PropertyId::BorderStyle,
            "border-color" => PropertyId::BorderColor,
            "border-top" => PropertyId::BorderTop,
            "border-top-width" => PropertyId::BorderTopWidth,
            "border-top-style" => PropertyId::BorderTopStyle,
            "border-top-color" => PropertyId::BorderTopColor,
            "border-right" => PropertyId::BorderRight,
            "border-right-width" => PropertyId::BorderRightWidth,
            "border-right-style" => PropertyId::BorderRightStyle,
            "border-right-color" => PropertyId::BorderRightColor,
            "border-bottom" => PropertyId::BorderBottom,
            "border-bottom-width" => PropertyId::BorderBottomWidth,
            "border-bottom-style" => PropertyId::BorderBottomStyle,
            "border-bottom-color" => PropertyId::BorderBottomColor,
            "border-left" => PropertyId::BorderLeft,
            "border-left-width" => PropertyId::BorderLeftWidth,
            "border-left-style" => PropertyId::BorderLeftStyle,
            "border-left-color" => PropertyId::BorderLeftColor,
            "border-radius" => PropertyId::BorderRadius,
            "border-top-left-radius" => PropertyId::BorderTopLeftRadius,
            "border-top-right-radius" => PropertyId::BorderTopRightRadius,
            "border-bottom-right-radius" => PropertyId::BorderBottomRightRadius,
            "border-bottom-left-radius" => PropertyId::BorderBottomLeftRadius,
            "box-sizing" => PropertyId::BoxSizing,

            // Background
            "background" => PropertyId::Background,
            "background-color" => PropertyId::BackgroundColor,
            "background-image" => PropertyId::BackgroundImage,
            "background-position" => PropertyId::BackgroundPosition,
            "background-position-x" => PropertyId::BackgroundPositionX,
            "background-position-y" => PropertyId::BackgroundPositionY,
            "background-size" => PropertyId::BackgroundSize,
            "background-repeat" => PropertyId::BackgroundRepeat,
            "background-attachment" => PropertyId::BackgroundAttachment,
            "background-origin" => PropertyId::BackgroundOrigin,
            "background-clip" => PropertyId::BackgroundClip,

            // Typography
            "color" => PropertyId::Color,
            "font" => PropertyId::Font,
            "font-family" => PropertyId::FontFamily,
            "font-size" => PropertyId::FontSize,
            "font-weight" => PropertyId::FontWeight,
            "font-style" => PropertyId::FontStyle,
            "font-variant" => PropertyId::FontVariant,
            "font-stretch" => PropertyId::FontStretch,
            "line-height" => PropertyId::LineHeight,
            "letter-spacing" => PropertyId::LetterSpacing,
            "word-spacing" => PropertyId::WordSpacing,
            "text-align" => PropertyId::TextAlign,
            "text-decoration" => PropertyId::TextDecoration,
            "text-decoration-line" => PropertyId::TextDecorationLine,
            "text-decoration-color" => PropertyId::TextDecorationColor,
            "text-decoration-style" => PropertyId::TextDecorationStyle,
            "text-transform" => PropertyId::TextTransform,
            "text-indent" => PropertyId::TextIndent,
            "text-shadow" => PropertyId::TextShadow,
            "text-overflow" => PropertyId::TextOverflow,
            "white-space" => PropertyId::WhiteSpace,
            "word-break" => PropertyId::WordBreak,
            "word-wrap" => PropertyId::WordWrap,
            "overflow-wrap" => PropertyId::OverflowWrap,

            // Flexbox
            "flex-direction" => PropertyId::FlexDirection,
            "flex-wrap" => PropertyId::FlexWrap,
            "flex-flow" => PropertyId::FlexFlow,
            "justify-content" => PropertyId::JustifyContent,
            "align-items" => PropertyId::AlignItems,
            "align-content" => PropertyId::AlignContent,
            "order" => PropertyId::Order,
            "flex-grow" => PropertyId::FlexGrow,
            "flex-shrink" => PropertyId::FlexShrink,
            "flex-basis" => PropertyId::FlexBasis,
            "flex" => PropertyId::Flex,
            "align-self" => PropertyId::AlignSelf,
            "gap" => PropertyId::Gap,
            "row-gap" => PropertyId::RowGap,
            "column-gap" => PropertyId::ColumnGap,

            // Grid
            "grid-template-columns" => PropertyId::GridTemplateColumns,
            "grid-template-rows" => PropertyId::GridTemplateRows,
            "grid-template-areas" => PropertyId::GridTemplateAreas,
            "grid-template" => PropertyId::GridTemplate,
            "grid-auto-columns" => PropertyId::GridAutoColumns,
            "grid-auto-rows" => PropertyId::GridAutoRows,
            "grid-auto-flow" => PropertyId::GridAutoFlow,
            "grid" => PropertyId::Grid,
            "grid-row-start" => PropertyId::GridRowStart,
            "grid-row-end" => PropertyId::GridRowEnd,
            "grid-column-start" => PropertyId::GridColumnStart,
            "grid-column-end" => PropertyId::GridColumnEnd,
            "grid-row" => PropertyId::GridRow,
            "grid-column" => PropertyId::GridColumn,
            "grid-area" => PropertyId::GridArea,
            "justify-items" => PropertyId::JustifyItems,
            "justify-self" => PropertyId::JustifySelf,
            "place-content" => PropertyId::PlaceContent,
            "place-items" => PropertyId::PlaceItems,
            "place-self" => PropertyId::PlaceSelf,

            // Transforms
            "transform" => PropertyId::Transform,
            "transform-origin" => PropertyId::TransformOrigin,
            "transform-style" => PropertyId::TransformStyle,
            "perspective" => PropertyId::Perspective,
            "perspective-origin" => PropertyId::PerspectiveOrigin,
            "backface-visibility" => PropertyId::Backface,

            // Transitions
            "transition" => PropertyId::Transition,
            "transition-property" => PropertyId::TransitionProperty,
            "transition-duration" => PropertyId::TransitionDuration,
            "transition-timing-function" => PropertyId::TransitionTimingFunction,
            "transition-delay" => PropertyId::TransitionDelay,

            // Animations
            "animation" => PropertyId::Animation,
            "animation-name" => PropertyId::AnimationName,
            "animation-duration" => PropertyId::AnimationDuration,
            "animation-timing-function" => PropertyId::AnimationTimingFunction,
            "animation-delay" => PropertyId::AnimationDelay,
            "animation-iteration-count" => PropertyId::AnimationIterationCount,
            "animation-direction" => PropertyId::AnimationDirection,
            "animation-fill-mode" => PropertyId::AnimationFillMode,
            "animation-play-state" => PropertyId::AnimationPlayState,

            // Filters & Effects
            "filter" => PropertyId::Filter,
            "backdrop-filter" => PropertyId::BackdropFilter,
            "opacity" => PropertyId::Opacity,
            "mix-blend-mode" => PropertyId::MixBlendMode,
            "box-shadow" => PropertyId::BoxShadow,

            // Lists
            "list-style" => PropertyId::ListStyle,
            "list-style-type" => PropertyId::ListStyleType,
            "list-style-position" => PropertyId::ListStylePosition,
            "list-style-image" => PropertyId::ListStyleImage,

            // Tables
            "border-collapse" => PropertyId::BorderCollapse,
            "border-spacing" => PropertyId::BorderSpacing,
            "table-layout" => PropertyId::TableLayout,
            "caption-side" => PropertyId::CaptionSide,
            "empty-cells" => PropertyId::EmptyCells,

            // UI
            "cursor" => PropertyId::Cursor,
            "outline" => PropertyId::Outline,
            "outline-width" => PropertyId::OutlineWidth,
            "outline-style" => PropertyId::OutlineStyle,
            "outline-color" => PropertyId::OutlineColor,
            "outline-offset" => PropertyId::OutlineOffset,
            "resize" => PropertyId::Resize,
            "user-select" => PropertyId::UserSelect,
            "pointer-events" => PropertyId::PointerEvents,

            // Content
            "content" => PropertyId::Content,
            "quotes" => PropertyId::Quotes,
            "counter-reset" => PropertyId::CounterReset,
            "counter-increment" => PropertyId::CounterIncrement,

            // Misc
            "object-fit" => PropertyId::ObjectFit,
            "object-position" => PropertyId::ObjectPosition,
            "vertical-align" => PropertyId::VerticalAlign,
            "direction" => PropertyId::Direction,
            "unicode-bidi" => PropertyId::UnicodeBidi,
            "writing-mode" => PropertyId::WritingMode,
            "text-orientation" => PropertyId::TextOrientation,
            "image-rendering" => PropertyId::ImageRendering,

            // SVG
            "fill" => PropertyId::Fill,
            "stroke" => PropertyId::Stroke,
            "stroke-width" => PropertyId::StrokeWidth,

            // Print
            "page-break-before" => PropertyId::PageBreakBefore,
            "page-break-after" => PropertyId::PageBreakAfter,
            "page-break-inside" => PropertyId::PageBreakInside,

            other => PropertyId::Custom(other.to_string()),
        }
    }

    /// Get property name.
    pub fn name(&self) -> &str {
        match self {
            PropertyId::Display => "display",
            PropertyId::Position => "position",
            PropertyId::Top => "top",
            PropertyId::Right => "right",
            PropertyId::Bottom => "bottom",
            PropertyId::Left => "left",
            PropertyId::Float => "float",
            PropertyId::Clear => "clear",
            PropertyId::ZIndex => "z-index",
            PropertyId::Visibility => "visibility",
            PropertyId::Overflow => "overflow",
            PropertyId::OverflowX => "overflow-x",
            PropertyId::OverflowY => "overflow-y",
            PropertyId::Width => "width",
            PropertyId::Height => "height",
            PropertyId::MinWidth => "min-width",
            PropertyId::MinHeight => "min-height",
            PropertyId::MaxWidth => "max-width",
            PropertyId::MaxHeight => "max-height",
            PropertyId::Margin => "margin",
            PropertyId::MarginTop => "margin-top",
            PropertyId::MarginRight => "margin-right",
            PropertyId::MarginBottom => "margin-bottom",
            PropertyId::MarginLeft => "margin-left",
            PropertyId::Padding => "padding",
            PropertyId::PaddingTop => "padding-top",
            PropertyId::PaddingRight => "padding-right",
            PropertyId::PaddingBottom => "padding-bottom",
            PropertyId::PaddingLeft => "padding-left",
            PropertyId::Border => "border",
            PropertyId::BorderWidth => "border-width",
            PropertyId::BorderStyle => "border-style",
            PropertyId::BorderColor => "border-color",
            PropertyId::Color => "color",
            PropertyId::BackgroundColor => "background-color",
            PropertyId::FontFamily => "font-family",
            PropertyId::FontSize => "font-size",
            PropertyId::FontWeight => "font-weight",
            PropertyId::Custom(s) => s,
            // ... add all other properties
            _ => "unknown",
        }
    }

    /// Check if this is a shorthand property.
    pub fn is_shorthand(&self) -> bool {
        matches!(
            self,
            PropertyId::Margin
                | PropertyId::Padding
                | PropertyId::Border
                | PropertyId::BorderWidth
                | PropertyId::BorderStyle
                | PropertyId::BorderColor
                | PropertyId::BorderTop
                | PropertyId::BorderRight
                | PropertyId::BorderBottom
                | PropertyId::BorderLeft
                | PropertyId::BorderRadius
                | PropertyId::Background
                | PropertyId::Font
                | PropertyId::Flex
                | PropertyId::FlexFlow
                | PropertyId::Grid
                | PropertyId::GridTemplate
                | PropertyId::GridRow
                | PropertyId::GridColumn
                | PropertyId::GridArea
                | PropertyId::ListStyle
                | PropertyId::Transition
                | PropertyId::Animation
                | PropertyId::Outline
                | PropertyId::TextDecoration
                | PropertyId::PlaceContent
                | PropertyId::PlaceItems
                | PropertyId::PlaceSelf
                | PropertyId::Gap
        )
    }

    /// Check if property is inherited.
    pub fn inherited(&self) -> bool {
        matches!(
            self,
            PropertyId::Color
                | PropertyId::Font
                | PropertyId::FontFamily
                | PropertyId::FontSize
                | PropertyId::FontWeight
                | PropertyId::FontStyle
                | PropertyId::FontVariant
                | PropertyId::FontStretch
                | PropertyId::LineHeight
                | PropertyId::LetterSpacing
                | PropertyId::WordSpacing
                | PropertyId::TextAlign
                | PropertyId::TextDecoration
                | PropertyId::TextTransform
                | PropertyId::TextIndent
                | PropertyId::TextShadow
                | PropertyId::WhiteSpace
                | PropertyId::WordBreak
                | PropertyId::WordWrap
                | PropertyId::OverflowWrap
                | PropertyId::Direction
                | PropertyId::UnicodeBidi
                | PropertyId::WritingMode
                | PropertyId::Visibility
                | PropertyId::Cursor
                | PropertyId::ListStyle
                | PropertyId::ListStyleType
                | PropertyId::ListStylePosition
                | PropertyId::ListStyleImage
                | PropertyId::Quotes
                | PropertyId::Fill
                | PropertyId::Stroke
                | PropertyId::StrokeWidth
        )
    }
}

impl fmt::Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Property declaration.
#[derive(Clone, Debug)]
pub struct PropertyDeclaration {
    /// Property ID.
    pub property: PropertyId,
    /// Value.
    pub value: CssValue,
    /// Important flag.
    pub important: bool,
}

impl PropertyDeclaration {
    pub fn new(property: PropertyId, value: CssValue) -> Self {
        Self {
            property,
            value,
            important: false,
        }
    }

    pub fn important(mut self) -> Self {
        self.important = true;
        self
    }

    /// Convert to CSS string.
    pub fn to_css_string(&self) -> String {
        let important = if self.important { " !important" } else { "" };
        format!("{}: {}{}", self.property.name(), self.value.to_css_string(), important)
    }
}

impl fmt::Display for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css_string())
    }
}

/// Property with shorthand.
pub trait Property {
    fn id(&self) -> PropertyId;
    fn longhand_ids(&self) -> Vec<PropertyId>;
    fn parse(input: &str) -> Result<Self, String>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_from_name() {
        assert_eq!(PropertyId::from_name("color"), PropertyId::Color);
        assert_eq!(PropertyId::from_name("COLOR"), PropertyId::Color);
        assert_eq!(PropertyId::from_name("font-size"), PropertyId::FontSize);
    }

    #[test]
    fn test_property_inheritance() {
        assert!(PropertyId::Color.inherited());
        assert!(PropertyId::FontFamily.inherited());
        assert!(!PropertyId::Width.inherited());
        assert!(!PropertyId::Margin.inherited());
    }

    #[test]
    fn test_shorthand() {
        assert!(PropertyId::Margin.is_shorthand());
        assert!(PropertyId::Background.is_shorthand());
        assert!(!PropertyId::MarginTop.is_shorthand());
        assert!(!PropertyId::Color.is_shorthand());
    }
}
