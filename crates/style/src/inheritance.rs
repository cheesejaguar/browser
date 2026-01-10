//! CSS Inheritance.

use css_parser::properties::PropertyId;

/// Check if a property is inherited by default.
pub fn is_inherited(property: &PropertyId) -> bool {
    property.inherited()
}

/// Get initial value for a property.
pub fn initial_value(property: &PropertyId) -> css_parser::values::CssValue {
    use css_parser::values::CssValue;

    match property {
        // Display
        PropertyId::Display => CssValue::Ident("inline".to_string()),
        PropertyId::Position => CssValue::Ident("static".to_string()),
        PropertyId::Visibility => CssValue::Ident("visible".to_string()),

        // Box Model
        PropertyId::Width | PropertyId::Height => CssValue::Ident("auto".to_string()),
        PropertyId::MinWidth | PropertyId::MinHeight => CssValue::Ident("auto".to_string()),
        PropertyId::MaxWidth | PropertyId::MaxHeight => CssValue::Ident("none".to_string()),
        PropertyId::Margin
        | PropertyId::MarginTop
        | PropertyId::MarginRight
        | PropertyId::MarginBottom
        | PropertyId::MarginLeft => CssValue::Number(0.0),
        PropertyId::Padding
        | PropertyId::PaddingTop
        | PropertyId::PaddingRight
        | PropertyId::PaddingBottom
        | PropertyId::PaddingLeft => CssValue::Number(0.0),
        PropertyId::BorderWidth
        | PropertyId::BorderTopWidth
        | PropertyId::BorderRightWidth
        | PropertyId::BorderBottomWidth
        | PropertyId::BorderLeftWidth => CssValue::Ident("medium".to_string()),
        PropertyId::BorderStyle
        | PropertyId::BorderTopStyle
        | PropertyId::BorderRightStyle
        | PropertyId::BorderBottomStyle
        | PropertyId::BorderLeftStyle => CssValue::Ident("none".to_string()),
        PropertyId::BoxSizing => CssValue::Ident("content-box".to_string()),

        // Position
        PropertyId::Top
        | PropertyId::Right
        | PropertyId::Bottom
        | PropertyId::Left => CssValue::Ident("auto".to_string()),
        PropertyId::ZIndex => CssValue::Ident("auto".to_string()),
        PropertyId::Float => CssValue::Ident("none".to_string()),
        PropertyId::Clear => CssValue::Ident("none".to_string()),

        // Typography
        PropertyId::Color => CssValue::Ident("canvastext".to_string()),
        PropertyId::FontFamily => CssValue::Ident("serif".to_string()),
        PropertyId::FontSize => CssValue::Ident("medium".to_string()),
        PropertyId::FontWeight => CssValue::Ident("normal".to_string()),
        PropertyId::FontStyle => CssValue::Ident("normal".to_string()),
        PropertyId::LineHeight => CssValue::Ident("normal".to_string()),
        PropertyId::TextAlign => CssValue::Ident("start".to_string()),
        PropertyId::TextDecoration => CssValue::Ident("none".to_string()),
        PropertyId::TextTransform => CssValue::Ident("none".to_string()),
        PropertyId::WhiteSpace => CssValue::Ident("normal".to_string()),
        PropertyId::LetterSpacing => CssValue::Ident("normal".to_string()),
        PropertyId::WordSpacing => CssValue::Ident("normal".to_string()),
        PropertyId::VerticalAlign => CssValue::Ident("baseline".to_string()),

        // Background
        PropertyId::BackgroundColor => CssValue::Ident("transparent".to_string()),
        PropertyId::BackgroundImage => CssValue::Ident("none".to_string()),
        PropertyId::BackgroundPosition => CssValue::Percentage(0.0),
        PropertyId::BackgroundSize => CssValue::Ident("auto".to_string()),
        PropertyId::BackgroundRepeat => CssValue::Ident("repeat".to_string()),

        // Flexbox
        PropertyId::FlexDirection => CssValue::Ident("row".to_string()),
        PropertyId::FlexWrap => CssValue::Ident("nowrap".to_string()),
        PropertyId::JustifyContent => CssValue::Ident("flex-start".to_string()),
        PropertyId::AlignItems => CssValue::Ident("stretch".to_string()),
        PropertyId::AlignContent => CssValue::Ident("stretch".to_string()),
        PropertyId::AlignSelf => CssValue::Ident("auto".to_string()),
        PropertyId::FlexGrow => CssValue::Number(0.0),
        PropertyId::FlexShrink => CssValue::Number(1.0),
        PropertyId::FlexBasis => CssValue::Ident("auto".to_string()),
        PropertyId::Order => CssValue::Number(0.0),
        PropertyId::Gap | PropertyId::RowGap | PropertyId::ColumnGap => {
            CssValue::Ident("normal".to_string())
        }

        // Grid
        PropertyId::GridTemplateColumns | PropertyId::GridTemplateRows => {
            CssValue::Ident("none".to_string())
        }
        PropertyId::GridAutoFlow => CssValue::Ident("row".to_string()),

        // Effects
        PropertyId::Opacity => CssValue::Number(1.0),
        PropertyId::Transform => CssValue::Ident("none".to_string()),
        PropertyId::Filter => CssValue::Ident("none".to_string()),
        PropertyId::BoxShadow => CssValue::Ident("none".to_string()),
        PropertyId::TextShadow => CssValue::Ident("none".to_string()),

        // Overflow
        PropertyId::Overflow | PropertyId::OverflowX | PropertyId::OverflowY => {
            CssValue::Ident("visible".to_string())
        }

        // UI
        PropertyId::Cursor => CssValue::Ident("auto".to_string()),
        PropertyId::PointerEvents => CssValue::Ident("auto".to_string()),
        PropertyId::UserSelect => CssValue::Ident("auto".to_string()),

        // Lists
        PropertyId::ListStyleType => CssValue::Ident("disc".to_string()),
        PropertyId::ListStylePosition => CssValue::Ident("outside".to_string()),
        PropertyId::ListStyleImage => CssValue::Ident("none".to_string()),

        // Tables
        PropertyId::BorderCollapse => CssValue::Ident("separate".to_string()),
        PropertyId::TableLayout => CssValue::Ident("auto".to_string()),

        // Outline
        PropertyId::OutlineStyle => CssValue::Ident("none".to_string()),
        PropertyId::OutlineWidth => CssValue::Ident("medium".to_string()),

        // Default
        _ => CssValue::Initial,
    }
}

/// Properties that force inheritance.
pub fn forces_inheritance(property: &PropertyId) -> bool {
    matches!(
        property,
        PropertyId::All
    )
}

/// Get list of all inherited properties.
pub fn inherited_properties() -> Vec<PropertyId> {
    vec![
        PropertyId::Color,
        PropertyId::Font,
        PropertyId::FontFamily,
        PropertyId::FontSize,
        PropertyId::FontWeight,
        PropertyId::FontStyle,
        PropertyId::FontVariant,
        PropertyId::FontStretch,
        PropertyId::LineHeight,
        PropertyId::LetterSpacing,
        PropertyId::WordSpacing,
        PropertyId::TextAlign,
        PropertyId::TextDecoration,
        PropertyId::TextDecorationLine,
        PropertyId::TextDecorationColor,
        PropertyId::TextDecorationStyle,
        PropertyId::TextTransform,
        PropertyId::TextIndent,
        PropertyId::TextShadow,
        PropertyId::WhiteSpace,
        PropertyId::WordBreak,
        PropertyId::WordWrap,
        PropertyId::OverflowWrap,
        PropertyId::Direction,
        PropertyId::UnicodeBidi,
        PropertyId::WritingMode,
        PropertyId::Visibility,
        PropertyId::Cursor,
        PropertyId::ListStyle,
        PropertyId::ListStyleType,
        PropertyId::ListStylePosition,
        PropertyId::ListStyleImage,
        PropertyId::Quotes,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inherited_properties() {
        assert!(is_inherited(&PropertyId::Color));
        assert!(is_inherited(&PropertyId::FontFamily));
        assert!(!is_inherited(&PropertyId::Width));
        assert!(!is_inherited(&PropertyId::Margin));
    }

    #[test]
    fn test_initial_values() {
        let display = initial_value(&PropertyId::Display);
        assert!(matches!(display, css_parser::values::CssValue::Ident(s) if s == "inline"));

        let opacity = initial_value(&PropertyId::Opacity);
        assert!(matches!(opacity, css_parser::values::CssValue::Number(n) if n == 1.0));
    }
}
