//! CSS Stylesheet structure.

use crate::media::MediaQueryList;
use crate::properties::PropertyDeclaration;
use crate::selector::SelectorList;
use std::sync::Arc;
use url::Url;

/// A CSS stylesheet.
#[derive(Clone, Debug)]
pub struct Stylesheet {
    /// Base URL for resolving relative URLs.
    pub url: Url,
    /// CSS rules.
    pub rules: Vec<CssRule>,
    /// Whether this is a user agent stylesheet.
    pub is_user_agent: bool,
    /// Source map URL if any.
    pub source_map_url: Option<String>,
}

impl Stylesheet {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            rules: Vec::new(),
            is_user_agent: false,
            source_map_url: None,
        }
    }

    /// Get all style rules (flattened, including from media/supports).
    pub fn style_rules(&self) -> Vec<&StyleRule> {
        let mut rules = Vec::new();
        self.collect_style_rules(&self.rules, &mut rules);
        rules
    }

    fn collect_style_rules<'a>(&'a self, rules: &'a [CssRule], out: &mut Vec<&'a StyleRule>) {
        for rule in rules {
            match rule {
                CssRule::Style(style) => out.push(style),
                CssRule::Media(media) => self.collect_style_rules(&media.rules, out),
                CssRule::Supports(supports) => self.collect_style_rules(&supports.rules, out),
                _ => {}
            }
        }
    }

    /// Get all @import rules.
    pub fn imports(&self) -> Vec<&ImportRule> {
        self.rules
            .iter()
            .filter_map(|r| match r {
                CssRule::Import(import) => Some(import),
                _ => None,
            })
            .collect()
    }

    /// Get all @font-face rules.
    pub fn font_faces(&self) -> Vec<&FontFaceRule> {
        self.rules
            .iter()
            .filter_map(|r| match r {
                CssRule::FontFace(ff) => Some(ff),
                _ => None,
            })
            .collect()
    }

    /// Get all @keyframes rules.
    pub fn keyframes(&self) -> Vec<&KeyframesRule> {
        self.rules
            .iter()
            .filter_map(|r| match r {
                CssRule::Keyframes(kf) => Some(kf),
                _ => None,
            })
            .collect()
    }
}

/// CSS rule types.
#[derive(Clone, Debug)]
pub enum CssRule {
    /// Style rule (selector { declarations }).
    Style(StyleRule),
    /// @import rule.
    Import(ImportRule),
    /// @media rule.
    Media(MediaRule),
    /// @font-face rule.
    FontFace(FontFaceRule),
    /// @keyframes rule.
    Keyframes(KeyframesRule),
    /// @supports rule.
    Supports(SupportsRule),
    /// @charset rule.
    Charset,
    /// @namespace rule.
    Namespace { prefix: Option<String>, url: String },
    /// @page rule.
    Page {
        selector: Option<String>,
        declarations: Vec<PropertyDeclaration>,
    },
}

/// Style rule (most common).
#[derive(Clone, Debug)]
pub struct StyleRule {
    /// Selector list.
    pub selectors: SelectorList,
    /// Property declarations.
    pub declarations: Vec<PropertyDeclaration>,
}

impl StyleRule {
    /// Get max specificity among all selectors.
    pub fn specificity(&self) -> crate::selector::Specificity {
        self.selectors
            .selectors
            .iter()
            .map(|s| s.specificity())
            .max()
            .unwrap_or_default()
    }
}

/// @import rule.
#[derive(Clone, Debug)]
pub struct ImportRule {
    /// URL string as written.
    pub url: String,
    /// Resolved absolute URL.
    pub resolved_url: Option<Url>,
    /// Media query list.
    pub media: MediaQueryList,
    /// Imported stylesheet (loaded later).
    pub stylesheet: Option<Arc<Stylesheet>>,
}

/// @media rule.
#[derive(Clone, Debug)]
pub struct MediaRule {
    /// Media query list.
    pub media: MediaQueryList,
    /// Nested rules.
    pub rules: Vec<CssRule>,
}

impl MediaRule {
    /// Check if media query matches.
    pub fn matches(&self, context: &crate::media::MediaContext) -> bool {
        self.media.matches(context)
    }
}

/// @font-face rule.
#[derive(Clone, Debug)]
pub struct FontFaceRule {
    /// Font descriptors.
    pub declarations: Vec<PropertyDeclaration>,
}

impl FontFaceRule {
    /// Get font family name.
    pub fn family(&self) -> Option<&str> {
        self.declarations
            .iter()
            .find(|d| d.property.name() == "font-family")
            .and_then(|d| d.value.as_string())
    }

    /// Get font source URLs.
    pub fn sources(&self) -> Vec<FontSource> {
        self.declarations
            .iter()
            .find(|d| d.property.name() == "src")
            .map(|d| self.parse_font_sources(&d.value))
            .unwrap_or_default()
    }

    fn parse_font_sources(&self, value: &crate::values::CssValue) -> Vec<FontSource> {
        // Parse src: url(...) format(...), local(...), etc.
        // Simplified implementation
        Vec::new()
    }

    /// Get font weight.
    pub fn weight(&self) -> Option<&crate::values::CssValue> {
        self.declarations
            .iter()
            .find(|d| d.property.name() == "font-weight")
            .map(|d| &d.value)
    }

    /// Get font style.
    pub fn style(&self) -> Option<&crate::values::CssValue> {
        self.declarations
            .iter()
            .find(|d| d.property.name() == "font-style")
            .map(|d| &d.value)
    }
}

/// Font source.
#[derive(Clone, Debug)]
pub struct FontSource {
    /// URL or local name.
    pub source: FontSourceType,
    /// Format hint.
    pub format: Option<String>,
}

/// Font source type.
#[derive(Clone, Debug)]
pub enum FontSourceType {
    Url(String),
    Local(String),
}

/// @keyframes rule.
#[derive(Clone, Debug)]
pub struct KeyframesRule {
    /// Animation name.
    pub name: String,
    /// Keyframe rules.
    pub keyframes: Vec<KeyframeRule>,
}

impl KeyframesRule {
    /// Get keyframe at specific percentage.
    pub fn at(&self, percentage: f32) -> Option<&KeyframeRule> {
        self.keyframes
            .iter()
            .find(|k| k.contains_percentage(percentage))
    }

    /// Get interpolated declarations at percentage.
    pub fn interpolate(&self, percentage: f32) -> Vec<PropertyDeclaration> {
        // Find surrounding keyframes and interpolate
        let mut before = None;
        let mut after = None;
        let mut before_pct = 0.0;
        let mut after_pct = 100.0;

        for keyframe in &self.keyframes {
            for pct in keyframe.percentages() {
                if pct <= percentage && pct >= before_pct {
                    before = Some(keyframe);
                    before_pct = pct;
                }
                if pct >= percentage && pct <= after_pct {
                    after = Some(keyframe);
                    after_pct = pct;
                }
            }
        }

        // Return declarations from closest keyframe (real impl would interpolate)
        before
            .or(after)
            .map(|k| k.declarations.clone())
            .unwrap_or_default()
    }
}

/// Single keyframe rule.
#[derive(Clone, Debug)]
pub struct KeyframeRule {
    /// Selectors (e.g., ["from"], ["50%"], ["to"]).
    pub selectors: Vec<String>,
    /// Declarations.
    pub declarations: Vec<PropertyDeclaration>,
}

impl KeyframeRule {
    /// Get percentages for this keyframe.
    pub fn percentages(&self) -> Vec<f32> {
        self.selectors
            .iter()
            .filter_map(|s| {
                match s.as_str() {
                    "from" => Some(0.0),
                    "to" => Some(100.0),
                    s if s.ends_with('%') => {
                        s[..s.len()-1].parse().ok()
                    }
                    _ => None,
                }
            })
            .collect()
    }

    /// Check if keyframe contains a percentage.
    pub fn contains_percentage(&self, pct: f32) -> bool {
        self.percentages().iter().any(|&p| (p - pct).abs() < 0.001)
    }
}

/// @supports rule.
#[derive(Clone, Debug)]
pub struct SupportsRule {
    /// Condition string.
    pub condition: String,
    /// Nested rules.
    pub rules: Vec<CssRule>,
}

impl SupportsRule {
    /// Check if supports condition is met.
    pub fn matches(&self) -> bool {
        // Parse and evaluate condition
        // Simplified - always return true
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stylesheet_creation() {
        let ss = Stylesheet::new(Url::parse("https://example.com/style.css").unwrap());
        assert!(ss.rules.is_empty());
        assert!(!ss.is_user_agent);
    }

    #[test]
    fn test_keyframe_percentages() {
        let kf = KeyframeRule {
            selectors: vec!["from".to_string(), "50%".to_string()],
            declarations: vec![],
        };
        let pcts = kf.percentages();
        assert_eq!(pcts, vec![0.0, 50.0]);
    }
}
