//! Style computation and rule matching.

use crate::cascade::{cascade_styles, CascadedDeclaration, Origin};
use crate::computed::ComputedStyle;
use crate::matching::{match_selectors, MatchContext};
use css_parser::media::MediaContext;
use css_parser::properties::{PropertyDeclaration, PropertyId};
use css_parser::selector::SelectorList;
use css_parser::stylesheet::{CssRule, MediaRule, StyleRule, Stylesheet, SupportsRule};
use css_parser::values::CssValue;
use dom::element::ElementData;
use dom::node::NodeId;
use dom::tree::DomTree;
use indexmap::IndexMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// The Stylist handles style matching and computation.
pub struct Stylist {
    /// User agent stylesheets.
    ua_sheets: Vec<Arc<Stylesheet>>,
    /// User stylesheets.
    user_sheets: Vec<Arc<Stylesheet>>,
    /// Author stylesheets.
    author_sheets: Vec<Arc<Stylesheet>>,
    /// Media context.
    media_context: MediaContext,
    /// Style cache.
    cache: RwLock<HashMap<CacheKey, Arc<ComputedStyle>>>,
    /// Rule source order counter.
    source_order: u32,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    node_id: NodeId,
    // Add more fields for invalidation
}

impl Stylist {
    pub fn new() -> Self {
        Self {
            ua_sheets: Vec::new(),
            user_sheets: Vec::new(),
            author_sheets: Vec::new(),
            media_context: MediaContext::default(),
            cache: RwLock::new(HashMap::new()),
            source_order: 0,
        }
    }

    /// Add user agent stylesheet.
    pub fn add_ua_stylesheet(&mut self, sheet: Stylesheet) {
        self.ua_sheets.push(Arc::new(sheet));
        self.invalidate_cache();
    }

    /// Add user stylesheet.
    pub fn add_user_stylesheet(&mut self, sheet: Stylesheet) {
        self.user_sheets.push(Arc::new(sheet));
        self.invalidate_cache();
    }

    /// Add author stylesheet.
    pub fn add_author_stylesheet(&mut self, sheet: Stylesheet) {
        self.author_sheets.push(Arc::new(sheet));
        self.invalidate_cache();
    }

    /// Clear all stylesheets.
    pub fn clear_stylesheets(&mut self) {
        self.ua_sheets.clear();
        self.user_sheets.clear();
        self.author_sheets.clear();
        self.invalidate_cache();
    }

    /// Set media context.
    pub fn set_media_context(&mut self, context: MediaContext) {
        self.media_context = context;
        self.invalidate_cache();
    }

    /// Invalidate the cache.
    pub fn invalidate_cache(&self) {
        self.cache.write().clear();
    }

    /// Compute style for an element.
    pub fn compute_style(
        &mut self,
        tree: &DomTree,
        node_id: NodeId,
        parent_style: Option<&ComputedStyle>,
    ) -> ComputedStyle {
        let node = match tree.get(node_id) {
            Some(n) => n,
            None => return ComputedStyle::default_style(),
        };

        let element = match node.as_element() {
            Some(e) => e,
            None => return ComputedStyle::default_style(),
        };

        // Collect matching declarations
        let mut declarations = Vec::new();

        // Match UA rules
        for sheet in &self.ua_sheets {
            self.collect_matching_rules(
                &sheet.rules,
                element,
                tree,
                node_id,
                Origin::UserAgent,
                &mut declarations,
            );
        }

        // Match user rules
        for sheet in &self.user_sheets {
            self.collect_matching_rules(
                &sheet.rules,
                element,
                tree,
                node_id,
                Origin::User,
                &mut declarations,
            );
        }

        // Match author rules
        for sheet in &self.author_sheets {
            self.collect_matching_rules(
                &sheet.rules,
                element,
                tree,
                node_id,
                Origin::Author,
                &mut declarations,
            );
        }

        // Add inline styles (highest specificity)
        if let Some(style) = &element.inline_style {
            let inline_decls = css_parser::parse_style_attribute(style);
            for decl in inline_decls {
                declarations.push(CascadedDeclaration::new(
                    decl,
                    css_parser::selector::Specificity::new(1, 0, 0),
                    Origin::Author,
                    self.source_order,
                ));
                self.source_order += 1;
            }
        }

        // Cascade
        let cascaded = cascade_styles(declarations);

        // Compute final values
        self.compute_values(cascaded, parent_style)
    }

    fn collect_matching_rules(
        &mut self,
        rules: &[CssRule],
        element: &ElementData,
        tree: &DomTree,
        node_id: NodeId,
        origin: Origin,
        declarations: &mut Vec<CascadedDeclaration>,
    ) {
        for rule in rules {
            match rule {
                CssRule::Style(style_rule) => {
                    if let Some(specificity) =
                        match_selectors(&style_rule.selectors, element, tree, node_id)
                    {
                        for decl in &style_rule.declarations {
                            declarations.push(CascadedDeclaration::new(
                                decl.clone(),
                                specificity,
                                origin,
                                self.source_order,
                            ));
                            self.source_order += 1;
                        }
                    }
                }
                CssRule::Media(media_rule) => {
                    if media_rule.media.matches(&self.media_context) {
                        self.collect_matching_rules(
                            &media_rule.rules,
                            element,
                            tree,
                            node_id,
                            origin,
                            declarations,
                        );
                    }
                }
                CssRule::Supports(supports_rule) => {
                    // For now, assume @supports conditions are met
                    self.collect_matching_rules(
                        &supports_rule.rules,
                        element,
                        tree,
                        node_id,
                        origin,
                        declarations,
                    );
                }
                _ => {}
            }
        }
    }

    fn compute_values(
        &self,
        cascaded: HashMap<PropertyId, CssValue>,
        parent: Option<&ComputedStyle>,
    ) -> ComputedStyle {
        let mut style = match parent {
            Some(p) => self.inherit_style(p),
            None => ComputedStyle::default_style(),
        };

        // Apply cascaded values
        for (property, value) in cascaded {
            self.apply_property(&mut style, &property, &value, parent);
        }

        style
    }

    fn inherit_style(&self, parent: &ComputedStyle) -> ComputedStyle {
        let mut style = ComputedStyle::default_style();

        // Inherit inherited properties
        style.color = parent.color;
        style.font_family = parent.font_family.clone();
        style.font_size = parent.font_size;
        style.font_weight = parent.font_weight;
        style.font_style = parent.font_style;
        style.line_height = parent.line_height.clone();
        style.letter_spacing = parent.letter_spacing;
        style.word_spacing = parent.word_spacing;
        style.text_align = parent.text_align;
        style.text_transform = parent.text_transform;
        style.text_decoration = parent.text_decoration;
        style.white_space = parent.white_space;
        style.word_break = parent.word_break;
        style.visibility = parent.visibility;
        style.cursor = parent.cursor;
        style.list_style_type = parent.list_style_type;
        style.list_style_position = parent.list_style_position;

        style
    }

    fn apply_property(
        &self,
        style: &mut ComputedStyle,
        property: &PropertyId,
        value: &CssValue,
        parent: Option<&ComputedStyle>,
    ) {
        use crate::computed::*;

        // Handle CSS-wide keywords
        if let CssValue::Ident(ident) = value {
            match ident.as_str() {
                "inherit" => {
                    if let Some(p) = parent {
                        self.inherit_property(style, property, p);
                    }
                    return;
                }
                "initial" | "unset" | "revert" => {
                    // Reset to initial value (handled by default style)
                    return;
                }
                _ => {}
            }
        }

        match property {
            PropertyId::Display => {
                style.display = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "none" => Display::None,
                        "block" => Display::Block,
                        "inline" => Display::Inline,
                        "inline-block" => Display::InlineBlock,
                        "flex" => Display::Flex,
                        "inline-flex" => Display::InlineFlex,
                        "grid" => Display::Grid,
                        "inline-grid" => Display::InlineGrid,
                        "table" => Display::Table,
                        "table-row" => Display::TableRow,
                        "table-cell" => Display::TableCell,
                        "list-item" => Display::ListItem,
                        "flow-root" => Display::FlowRoot,
                        "contents" => Display::Contents,
                        _ => Display::Inline,
                    },
                    _ => Display::Inline,
                };
            }
            PropertyId::Position => {
                style.position = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "static" => Position::Static,
                        "relative" => Position::Relative,
                        "absolute" => Position::Absolute,
                        "fixed" => Position::Fixed,
                        "sticky" => Position::Sticky,
                        _ => Position::Static,
                    },
                    _ => Position::Static,
                };
            }
            PropertyId::Width => {
                style.width = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::Height => {
                style.height = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MinWidth => {
                style.min_width = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MinHeight => {
                style.min_height = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MaxWidth => {
                style.max_width = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MaxHeight => {
                style.max_height = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MarginTop => {
                style.margin.top = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MarginRight => {
                style.margin.right = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MarginBottom => {
                style.margin.bottom = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::MarginLeft => {
                style.margin.left = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::PaddingTop => {
                style.padding.top = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::PaddingRight => {
                style.padding.right = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::PaddingBottom => {
                style.padding.bottom = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::PaddingLeft => {
                style.padding.left = self.compute_size_value(value, parent.map(|p| p.font_size));
            }
            PropertyId::Color => {
                if let Some(color) = value.as_color() {
                    style.color = color;
                }
            }
            PropertyId::BackgroundColor => {
                if let Some(color) = value.as_color() {
                    style.background_color = color;
                }
            }
            PropertyId::FontSize => {
                style.font_size = self.compute_font_size(value, parent.map(|p| p.font_size));
            }
            PropertyId::FontWeight => {
                style.font_weight = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "normal" => FontWeight::Normal,
                        "bold" => FontWeight::Bold,
                        "lighter" => FontWeight::Light,
                        "bolder" => FontWeight::Bold,
                        _ => FontWeight::Normal,
                    },
                    CssValue::Number(n) => FontWeight::Number(*n as u16),
                    _ => FontWeight::Normal,
                };
            }
            PropertyId::FontFamily => {
                style.font_family = self.parse_font_family(value);
            }
            PropertyId::FlexDirection => {
                style.flex_direction = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "row" => FlexDirection::Row,
                        "row-reverse" => FlexDirection::RowReverse,
                        "column" => FlexDirection::Column,
                        "column-reverse" => FlexDirection::ColumnReverse,
                        _ => FlexDirection::Row,
                    },
                    _ => FlexDirection::Row,
                };
            }
            PropertyId::FlexWrap => {
                style.flex_wrap = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "nowrap" => FlexWrap::NoWrap,
                        "wrap" => FlexWrap::Wrap,
                        "wrap-reverse" => FlexWrap::WrapReverse,
                        _ => FlexWrap::NoWrap,
                    },
                    _ => FlexWrap::NoWrap,
                };
            }
            PropertyId::JustifyContent => {
                style.justify_content = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "flex-start" | "start" => JustifyContent::FlexStart,
                        "flex-end" | "end" => JustifyContent::FlexEnd,
                        "center" => JustifyContent::Center,
                        "space-between" => JustifyContent::SpaceBetween,
                        "space-around" => JustifyContent::SpaceAround,
                        "space-evenly" => JustifyContent::SpaceEvenly,
                        _ => JustifyContent::FlexStart,
                    },
                    _ => JustifyContent::FlexStart,
                };
            }
            PropertyId::AlignItems => {
                style.align_items = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "flex-start" | "start" => AlignItems::FlexStart,
                        "flex-end" | "end" => AlignItems::FlexEnd,
                        "center" => AlignItems::Center,
                        "baseline" => AlignItems::Baseline,
                        "stretch" => AlignItems::Stretch,
                        _ => AlignItems::Stretch,
                    },
                    _ => AlignItems::Stretch,
                };
            }
            PropertyId::FlexGrow => {
                if let Some(n) = value.as_number() {
                    style.flex_grow = n;
                }
            }
            PropertyId::FlexShrink => {
                if let Some(n) = value.as_number() {
                    style.flex_shrink = n;
                }
            }
            PropertyId::Opacity => {
                if let Some(n) = value.as_number() {
                    style.opacity = n.clamp(0.0, 1.0);
                }
            }
            PropertyId::ZIndex => {
                style.z_index = match value {
                    CssValue::Number(n) => ZIndex::Number(*n as i32),
                    CssValue::Ident(s) if s == "auto" => ZIndex::Auto,
                    _ => ZIndex::Auto,
                };
            }
            PropertyId::Overflow => {
                let overflow = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "visible" => Overflow::Visible,
                        "hidden" => Overflow::Hidden,
                        "scroll" => Overflow::Scroll,
                        "auto" => Overflow::Auto,
                        "clip" => Overflow::Clip,
                        _ => Overflow::Visible,
                    },
                    _ => Overflow::Visible,
                };
                style.overflow_x = overflow;
                style.overflow_y = overflow;
            }
            PropertyId::Visibility => {
                style.visibility = match value {
                    CssValue::Ident(s) => match s.as_str() {
                        "visible" => Visibility::Visible,
                        "hidden" => Visibility::Hidden,
                        "collapse" => Visibility::Collapse,
                        _ => Visibility::Visible,
                    },
                    _ => Visibility::Visible,
                };
            }
            // Add more properties as needed
            _ => {}
        }
    }

    fn inherit_property(&self, style: &mut ComputedStyle, property: &PropertyId, parent: &ComputedStyle) {
        match property {
            PropertyId::Color => style.color = parent.color,
            PropertyId::FontFamily => style.font_family = parent.font_family.clone(),
            PropertyId::FontSize => style.font_size = parent.font_size,
            PropertyId::FontWeight => style.font_weight = parent.font_weight,
            PropertyId::FontStyle => style.font_style = parent.font_style,
            PropertyId::LineHeight => style.line_height = parent.line_height.clone(),
            PropertyId::TextAlign => style.text_align = parent.text_align,
            PropertyId::Visibility => style.visibility = parent.visibility,
            _ => {}
        }
    }

    fn compute_size_value(&self, value: &CssValue, font_size: Option<f32>) -> crate::computed::SizeValue {
        use crate::computed::SizeValue;

        match value {
            CssValue::Ident(s) => match s.as_str() {
                "auto" => SizeValue::Auto,
                "none" => SizeValue::None,
                "min-content" => SizeValue::MinContent,
                "max-content" => SizeValue::MaxContent,
                "fit-content" => SizeValue::FitContent,
                _ => SizeValue::Auto,
            },
            CssValue::Number(n) if *n == 0.0 => SizeValue::Length(0.0),
            CssValue::Dimension(n, unit) => {
                let px = self.unit_to_px(*n, unit, font_size);
                SizeValue::Length(px)
            }
            CssValue::Percentage(p) => SizeValue::Percentage(*p),
            _ => SizeValue::Auto,
        }
    }

    fn compute_font_size(&self, value: &CssValue, parent_size: Option<f32>) -> f32 {
        let base_size = parent_size.unwrap_or(16.0);

        match value {
            CssValue::Ident(s) => match s.as_str() {
                "xx-small" => 9.0,
                "x-small" => 10.0,
                "small" => 13.0,
                "medium" => 16.0,
                "large" => 18.0,
                "x-large" => 24.0,
                "xx-large" => 32.0,
                "xxx-large" => 48.0,
                "smaller" => base_size * 0.833,
                "larger" => base_size * 1.2,
                _ => base_size,
            },
            CssValue::Number(n) => *n,
            CssValue::Dimension(n, unit) => self.unit_to_px(*n, unit, Some(base_size)),
            CssValue::Percentage(p) => base_size * p / 100.0,
            _ => base_size,
        }
    }

    fn unit_to_px(&self, value: f32, unit: &str, font_size: Option<f32>) -> f32 {
        let font_size = font_size.unwrap_or(16.0);

        match unit {
            "px" => value,
            "em" => value * font_size,
            "rem" => value * 16.0, // Root font size
            "%" => value * font_size / 100.0,
            "pt" => value * 96.0 / 72.0,
            "pc" => value * 96.0 / 6.0,
            "in" => value * 96.0,
            "cm" => value * 96.0 / 2.54,
            "mm" => value * 96.0 / 25.4,
            "vw" => value * self.media_context.width / 100.0,
            "vh" => value * self.media_context.height / 100.0,
            "vmin" => value * self.media_context.width.min(self.media_context.height) / 100.0,
            "vmax" => value * self.media_context.width.max(self.media_context.height) / 100.0,
            "ch" => value * font_size * 0.5, // Approximate
            "ex" => value * font_size * 0.5, // Approximate
            _ => value,
        }
    }

    fn parse_font_family(&self, value: &CssValue) -> Vec<String> {
        match value {
            CssValue::String(s) => vec![s.clone()],
            CssValue::Ident(s) => vec![s.clone()],
            CssValue::List(list) => list
                .iter()
                .filter_map(|v| match v {
                    CssValue::String(s) | CssValue::Ident(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => vec!["sans-serif".to_string()],
        }
    }
}

impl Default for Stylist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stylist_creation() {
        let stylist = Stylist::new();
        assert!(stylist.ua_sheets.is_empty());
    }
}
