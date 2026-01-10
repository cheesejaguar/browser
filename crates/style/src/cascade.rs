//! CSS Cascade implementation.

use css_parser::properties::{PropertyDeclaration, PropertyId};
use css_parser::selector::Specificity;
use std::cmp::Ordering;

/// Style origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    UserAgent,
    User,
    Author,
}

/// Cascade level for sorting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CascadeLevel {
    pub origin: Origin,
    pub important: bool,
}

impl CascadeLevel {
    /// Get cascade priority (higher = wins).
    pub fn priority(&self) -> u32 {
        match (self.origin, self.important) {
            (Origin::UserAgent, false) => 1,
            (Origin::User, false) => 2,
            (Origin::Author, false) => 3,
            (Origin::Author, true) => 4,
            (Origin::User, true) => 5,
            (Origin::UserAgent, true) => 6,
        }
    }
}

impl PartialOrd for CascadeLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CascadeLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

/// A declaration with cascade info.
#[derive(Clone, Debug)]
pub struct CascadedDeclaration {
    pub declaration: PropertyDeclaration,
    pub specificity: Specificity,
    pub level: CascadeLevel,
    pub source_order: u32,
}

impl CascadedDeclaration {
    pub fn new(
        declaration: PropertyDeclaration,
        specificity: Specificity,
        origin: Origin,
        source_order: u32,
    ) -> Self {
        Self {
            level: CascadeLevel {
                origin,
                important: declaration.important,
            },
            declaration,
            specificity,
            source_order,
        }
    }
}

impl PartialOrd for CascadedDeclaration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CascadedDeclaration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level
            .cmp(&other.level)
            .then_with(|| self.specificity.cmp(&other.specificity))
            .then_with(|| self.source_order.cmp(&other.source_order))
    }
}

impl PartialEq for CascadedDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level
            && self.specificity == other.specificity
            && self.source_order == other.source_order
    }
}

impl Eq for CascadedDeclaration {}

/// Cascade styles from multiple sources.
pub fn cascade_styles(
    declarations: Vec<CascadedDeclaration>,
) -> std::collections::HashMap<PropertyId, css_parser::values::CssValue> {
    use std::collections::HashMap;

    let mut by_property: HashMap<PropertyId, Vec<CascadedDeclaration>> = HashMap::new();

    // Group by property
    for decl in declarations {
        by_property
            .entry(decl.declaration.property.clone())
            .or_default()
            .push(decl);
    }

    // Pick winner for each property
    let mut result = HashMap::new();
    for (property, mut decls) in by_property {
        decls.sort();
        if let Some(winner) = decls.pop() {
            result.insert(property, winner.declaration.value);
        }
    }

    result
}

/// Expand shorthand property into longhands.
pub fn expand_shorthand(
    property: &PropertyId,
    value: &css_parser::values::CssValue,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    match property {
        PropertyId::Margin => expand_four_sides(
            value,
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
        ),
        PropertyId::Padding => expand_four_sides(
            value,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
        ),
        PropertyId::BorderWidth => expand_four_sides(
            value,
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ),
        PropertyId::BorderStyle => expand_four_sides(
            value,
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ),
        PropertyId::BorderColor => expand_four_sides(
            value,
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
        ),
        PropertyId::BorderRadius => expand_four_corners(
            value,
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ),
        PropertyId::Background => expand_background(value),
        PropertyId::Font => expand_font(value),
        PropertyId::Flex => expand_flex(value),
        PropertyId::Gap => expand_gap(value),
        _ => vec![(property.clone(), value.clone())],
    }
}

fn expand_four_sides(
    value: &css_parser::values::CssValue,
    top: PropertyId,
    right: PropertyId,
    bottom: PropertyId,
    left: PropertyId,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    let values = match value {
        css_parser::values::CssValue::List(list) => list.clone(),
        _ => vec![value.clone()],
    };

    let (t, r, b, l) = match values.len() {
        1 => (
            values[0].clone(),
            values[0].clone(),
            values[0].clone(),
            values[0].clone(),
        ),
        2 => (
            values[0].clone(),
            values[1].clone(),
            values[0].clone(),
            values[1].clone(),
        ),
        3 => (
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[1].clone(),
        ),
        _ => (
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[3].clone(),
        ),
    };

    vec![(top, t), (right, r), (bottom, b), (left, l)]
}

fn expand_four_corners(
    value: &css_parser::values::CssValue,
    top_left: PropertyId,
    top_right: PropertyId,
    bottom_right: PropertyId,
    bottom_left: PropertyId,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    let values = match value {
        css_parser::values::CssValue::List(list) => list.clone(),
        _ => vec![value.clone()],
    };

    let (tl, tr, br, bl) = match values.len() {
        1 => (
            values[0].clone(),
            values[0].clone(),
            values[0].clone(),
            values[0].clone(),
        ),
        2 => (
            values[0].clone(),
            values[1].clone(),
            values[0].clone(),
            values[1].clone(),
        ),
        3 => (
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[1].clone(),
        ),
        _ => (
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[3].clone(),
        ),
    };

    vec![
        (top_left, tl),
        (top_right, tr),
        (bottom_right, br),
        (bottom_left, bl),
    ]
}

fn expand_background(
    value: &css_parser::values::CssValue,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    // Simplified - real implementation would parse all background components
    vec![(PropertyId::BackgroundColor, value.clone())]
}

fn expand_font(
    value: &css_parser::values::CssValue,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    // Simplified
    vec![(PropertyId::FontFamily, value.clone())]
}

fn expand_flex(
    value: &css_parser::values::CssValue,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    use css_parser::values::CssValue;

    match value {
        CssValue::Ident(s) if s == "none" => vec![
            (PropertyId::FlexGrow, CssValue::Number(0.0)),
            (PropertyId::FlexShrink, CssValue::Number(0.0)),
            (PropertyId::FlexBasis, CssValue::Ident("auto".to_string())),
        ],
        CssValue::Ident(s) if s == "auto" => vec![
            (PropertyId::FlexGrow, CssValue::Number(1.0)),
            (PropertyId::FlexShrink, CssValue::Number(1.0)),
            (PropertyId::FlexBasis, CssValue::Ident("auto".to_string())),
        ],
        CssValue::Number(n) => vec![
            (PropertyId::FlexGrow, CssValue::Number(*n)),
            (PropertyId::FlexShrink, CssValue::Number(1.0)),
            (PropertyId::FlexBasis, CssValue::Number(0.0)),
        ],
        CssValue::List(list) => {
            let mut result = vec![];
            if let Some(grow) = list.first() {
                result.push((PropertyId::FlexGrow, grow.clone()));
            }
            if let Some(shrink) = list.get(1) {
                result.push((PropertyId::FlexShrink, shrink.clone()));
            }
            if let Some(basis) = list.get(2) {
                result.push((PropertyId::FlexBasis, basis.clone()));
            }
            result
        }
        _ => vec![
            (PropertyId::FlexGrow, CssValue::Number(0.0)),
            (PropertyId::FlexShrink, CssValue::Number(1.0)),
            (PropertyId::FlexBasis, CssValue::Ident("auto".to_string())),
        ],
    }
}

fn expand_gap(
    value: &css_parser::values::CssValue,
) -> Vec<(PropertyId, css_parser::values::CssValue)> {
    match value {
        css_parser::values::CssValue::List(list) if list.len() >= 2 => vec![
            (PropertyId::RowGap, list[0].clone()),
            (PropertyId::ColumnGap, list[1].clone()),
        ],
        _ => vec![
            (PropertyId::RowGap, value.clone()),
            (PropertyId::ColumnGap, value.clone()),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_level_priority() {
        let ua = CascadeLevel {
            origin: Origin::UserAgent,
            important: false,
        };
        let author = CascadeLevel {
            origin: Origin::Author,
            important: false,
        };
        let author_important = CascadeLevel {
            origin: Origin::Author,
            important: true,
        };

        assert!(author > ua);
        assert!(author_important > author);
    }

    #[test]
    fn test_expand_margin() {
        use css_parser::values::CssValue;

        let value = CssValue::List(vec![
            CssValue::Dimension(10.0, "px".to_string()),
            CssValue::Dimension(20.0, "px".to_string()),
        ]);

        let expanded = expand_shorthand(&PropertyId::Margin, &value);
        assert_eq!(expanded.len(), 4);
    }
}
