//! CSS Selector implementation.

use std::cmp::Ordering;

/// List of selectors (comma-separated).
#[derive(Clone, Debug, Default)]
pub struct SelectorList {
    pub selectors: Vec<Selector>,
}

impl SelectorList {
    pub fn new() -> Self {
        Self {
            selectors: Vec::new(),
        }
    }

    pub fn push(&mut self, selector: Selector) {
        self.selectors.push(selector);
    }

    /// Get maximum specificity of all selectors.
    pub fn max_specificity(&self) -> Specificity {
        self.selectors
            .iter()
            .map(|s| s.specificity())
            .max()
            .unwrap_or_default()
    }

    /// Convert to CSS string.
    pub fn to_css_string(&self) -> String {
        self.selectors
            .iter()
            .map(|s| s.to_css_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// CSS selector.
#[derive(Clone, Debug, Default)]
pub struct Selector {
    /// Tag name (e.g., "div").
    pub tag: Option<String>,
    /// ID (e.g., "main").
    pub id: Option<String>,
    /// Classes.
    pub classes: Vec<String>,
    /// Attribute selectors.
    pub attributes: Vec<AttributeSelector>,
    /// Pseudo-classes with optional arguments.
    pub pseudo_classes: Vec<(String, Option<String>)>,
    /// Pseudo-elements.
    pub pseudo_elements: Vec<String>,
    /// Universal selector (*).
    pub universal: bool,
    /// Combinator to next selector.
    pub combinator: Option<Combinator>,
    /// Next selector in chain.
    pub next: Option<Box<Selector>>,
}

impl Selector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create tag selector.
    pub fn tag(name: &str) -> Self {
        Self {
            tag: Some(name.to_ascii_lowercase()),
            ..Default::default()
        }
    }

    /// Create ID selector.
    pub fn id(id: &str) -> Self {
        Self {
            id: Some(id.to_string()),
            ..Default::default()
        }
    }

    /// Create class selector.
    pub fn class(class: &str) -> Self {
        Self {
            classes: vec![class.to_string()],
            ..Default::default()
        }
    }

    /// Create universal selector.
    pub fn universal() -> Self {
        Self {
            universal: true,
            ..Default::default()
        }
    }

    /// Calculate specificity.
    pub fn specificity(&self) -> Specificity {
        let mut spec = Specificity::default();

        // ID selectors
        if self.id.is_some() {
            spec.a += 1;
        }

        // Class selectors, attribute selectors, pseudo-classes
        spec.b += self.classes.len() as u32;
        spec.b += self.attributes.len() as u32;
        spec.b += self.pseudo_classes
            .iter()
            .filter(|(name, _)| !matches!(name.as_str(), "not" | "is" | "where" | "has"))
            .count() as u32;

        // Type selectors, pseudo-elements
        if self.tag.is_some() && !self.universal {
            spec.c += 1;
        }
        spec.c += self.pseudo_elements.len() as u32;

        // Add next selector's specificity
        if let Some(ref next) = self.next {
            let next_spec = next.specificity();
            spec.a += next_spec.a;
            spec.b += next_spec.b;
            spec.c += next_spec.c;
        }

        spec
    }

    /// Check if selector matches element.
    pub fn matches(&self, element: &dom::element::ElementData) -> bool {
        // Check tag
        if let Some(ref tag) = self.tag {
            if element.tag_name.as_str() != tag {
                return false;
            }
        }

        // Check ID
        if let Some(ref id) = self.id {
            match &element.id {
                Some(elem_id) if elem_id.as_ref() == id.as_str() => {}
                _ => return false,
            }
        }

        // Check classes
        for class in &self.classes {
            if !element.has_class(class) {
                return false;
            }
        }

        // Check attributes
        for attr in &self.attributes {
            if !attr.matches(element) {
                return false;
            }
        }

        // Check pseudo-classes (simplified)
        for (pseudo, args) in &self.pseudo_classes {
            if !self.matches_pseudo_class(element, pseudo, args.as_deref()) {
                return false;
            }
        }

        true
    }

    fn matches_pseudo_class(
        &self,
        element: &dom::element::ElementData,
        pseudo: &str,
        args: Option<&str>,
    ) -> bool {
        match pseudo {
            "hover" | "active" | "focus" | "visited" | "link" => {
                // State-dependent - requires additional context
                // For now, return true (would check element state)
                true
            }
            "first-child" | "last-child" | "only-child" => {
                // Requires tree context
                true
            }
            "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                // Requires tree context and parsing args
                true
            }
            "empty" => {
                // Requires checking children
                true
            }
            "enabled" => !element.flags.contains(dom::element::ElementFlags::DISABLED),
            "disabled" => element.flags.contains(dom::element::ElementFlags::DISABLED),
            "checked" => element.flags.contains(dom::element::ElementFlags::CHECKED),
            "required" => element.has_attribute("required"),
            "optional" => !element.has_attribute("required"),
            "read-only" => element.has_attribute("readonly"),
            "read-write" => !element.has_attribute("readonly"),
            "not" => {
                if let Some(inner) = args {
                    // Parse inner selector and check non-match
                    // Simplified - would need full selector parsing
                    true
                } else {
                    true
                }
            }
            "is" | "where" => {
                // Match any of the inner selectors
                true
            }
            "has" => {
                // Relational pseudo-class
                true
            }
            "root" => element.tag_name.as_str() == "html",
            "target" => {
                // Would check if element is URL fragment target
                false
            }
            "lang" => {
                if let Some(lang) = args {
                    element
                        .get_attribute("lang")
                        .map(|l: &str| l.starts_with(lang))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            _ => true, // Unknown pseudo-classes match by default
        }
    }

    /// Convert to CSS string.
    pub fn to_css_string(&self) -> String {
        let mut result = String::new();

        if self.universal && self.tag.is_none() {
            result.push('*');
        }

        if let Some(ref tag) = self.tag {
            result.push_str(tag);
        }

        if let Some(ref id) = self.id {
            result.push('#');
            result.push_str(id);
        }

        for class in &self.classes {
            result.push('.');
            result.push_str(class);
        }

        for attr in &self.attributes {
            result.push_str(&attr.to_css_string());
        }

        for (pseudo, args) in &self.pseudo_classes {
            result.push(':');
            result.push_str(pseudo);
            if let Some(args) = args {
                result.push('(');
                result.push_str(args);
                result.push(')');
            }
        }

        for pseudo in &self.pseudo_elements {
            result.push_str("::");
            result.push_str(pseudo);
        }

        if let Some(ref combinator) = self.combinator {
            result.push_str(combinator.as_str());
        }

        if let Some(ref next) = self.next {
            result.push_str(&next.to_css_string());
        }

        result
    }
}

/// Selector combinator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant (space).
    Descendant,
    /// Child (>).
    Child,
    /// Next sibling (+).
    NextSibling,
    /// Subsequent sibling (~).
    SubsequentSibling,
}

impl Combinator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Combinator::Descendant => " ",
            Combinator::Child => " > ",
            Combinator::NextSibling => " + ",
            Combinator::SubsequentSibling => " ~ ",
        }
    }
}

/// Attribute selector.
#[derive(Clone, Debug)]
pub struct AttributeSelector {
    /// Attribute name.
    pub name: String,
    /// Operator (=, ~=, |=, ^=, $=, *=).
    pub operator: Option<String>,
    /// Expected value.
    pub value: Option<String>,
    /// Case sensitivity.
    pub case_sensitivity: CaseSensitivity,
}

impl AttributeSelector {
    /// Check if selector matches element.
    pub fn matches(&self, element: &dom::element::ElementData) -> bool {
        let attr_value: String = match element.get_attribute(&self.name) {
            Some(v) => v.to_string(),
            None => return false,
        };

        let (attr_value, expected): (String, Option<String>) = match self.case_sensitivity {
            CaseSensitivity::Insensitive => (
                attr_value.to_ascii_lowercase(),
                self.value.as_ref().map(|v| v.to_ascii_lowercase()),
            ),
            _ => (attr_value, self.value.clone()),
        };

        match (self.operator.as_deref(), expected) {
            (None, _) => true, // Just check presence
            (Some("="), Some(ref v)) => attr_value == *v,
            (Some("~="), Some(ref v)) => attr_value.split_whitespace().any(|w| w == v),
            (Some("|="), Some(ref v)) => attr_value == *v || attr_value.starts_with(&format!("{}-", v)),
            (Some("^="), Some(ref v)) => attr_value.starts_with(v.as_str()),
            (Some("$="), Some(ref v)) => attr_value.ends_with(v.as_str()),
            (Some("*="), Some(ref v)) => attr_value.contains(v.as_str()),
            _ => false,
        }
    }

    /// Convert to CSS string.
    pub fn to_css_string(&self) -> String {
        let mut result = format!("[{}", self.name);

        if let (Some(op), Some(value)) = (&self.operator, &self.value) {
            result.push_str(op);
            result.push('"');
            result.push_str(value);
            result.push('"');

            match self.case_sensitivity {
                CaseSensitivity::Insensitive => result.push_str(" i"),
                CaseSensitivity::Sensitive => result.push_str(" s"),
                CaseSensitivity::Default => {}
            }
        }

        result.push(']');
        result
    }
}

/// Attribute case sensitivity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaseSensitivity {
    #[default]
    Default,
    Insensitive,
    Sensitive,
}

/// Selector specificity (a, b, c).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Specificity {
    /// ID selectors.
    pub a: u32,
    /// Class, attribute, pseudo-class selectors.
    pub b: u32,
    /// Type, pseudo-element selectors.
    pub c: u32,
}

impl Specificity {
    pub const fn new(a: u32, b: u32, c: u32) -> Self {
        Self { a, b, c }
    }

    /// Convert to single number for comparison.
    pub fn to_u32(&self) -> u32 {
        self.a * 10000 + self.b * 100 + self.c
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.a
            .cmp(&other.a)
            .then_with(|| self.b.cmp(&other.b))
            .then_with(|| self.c.cmp(&other.c))
    }
}

/// Parse selector from string.
pub fn parse_selector(css: &str) -> Result<Selector, String> {
    use crate::parser::parse_css;
    use url::Url;

    let full_css = format!("{} {{}}", css);
    let stylesheet = parse_css(&full_css, Url::parse("about:blank").unwrap());

    stylesheet
        .rules
        .into_iter()
        .next()
        .and_then(|r| match r {
            crate::stylesheet::CssRule::Style(s) => s.selectors.selectors.into_iter().next(),
            _ => None,
        })
        .ok_or_else(|| "Invalid selector".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity() {
        // ID
        let id = Selector::id("main");
        assert_eq!(id.specificity(), Specificity::new(1, 0, 0));

        // Class
        let class = Selector::class("container");
        assert_eq!(class.specificity(), Specificity::new(0, 1, 0));

        // Tag
        let tag = Selector::tag("div");
        assert_eq!(tag.specificity(), Specificity::new(0, 0, 1));
    }

    #[test]
    fn test_specificity_comparison() {
        let a = Specificity::new(1, 0, 0);
        let b = Specificity::new(0, 10, 0);
        let c = Specificity::new(0, 0, 100);

        assert!(a > b);
        assert!(b > c);
        assert!(a > c);
    }

    #[test]
    fn test_selector_css_string() {
        let mut selector = Selector::new();
        selector.tag = Some("div".to_string());
        selector.classes = vec!["container".to_string(), "main".to_string()];
        selector.id = Some("app".to_string());

        let css = selector.to_css_string();
        assert!(css.contains("div"));
        assert!(css.contains("#app"));
        assert!(css.contains(".container"));
        assert!(css.contains(".main"));
    }
}
