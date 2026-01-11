//! DOM Element implementation.

use crate::attributes::AttributeMap;
use bitflags::bitflags;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;

/// Common HTML tag names interned for efficiency.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagName(Arc<str>);

impl TagName {
    pub fn new(name: &str) -> Self {
        // Intern common tag names
        static INTERNED: Lazy<RwLock<HashMap<String, Arc<str>>>> =
            Lazy::new(|| RwLock::new(HashMap::new()));

        let lower = name.to_ascii_lowercase();

        // Check if already interned
        {
            let cache = INTERNED.read();
            if let Some(s) = cache.get(&lower) {
                return TagName(s.clone());
            }
        }

        // Intern new string
        let mut cache = INTERNED.write();
        let s = cache
            .entry(lower.clone())
            .or_insert_with(|| Arc::from(lower.as_str()))
            .clone();
        TagName(s)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Common tag names
    pub fn html() -> Self {
        Self::new("html")
    }
    pub fn head() -> Self {
        Self::new("head")
    }
    pub fn body() -> Self {
        Self::new("body")
    }
    pub fn div() -> Self {
        Self::new("div")
    }
    pub fn span() -> Self {
        Self::new("span")
    }
    pub fn p() -> Self {
        Self::new("p")
    }
    pub fn a() -> Self {
        Self::new("a")
    }
    pub fn img() -> Self {
        Self::new("img")
    }
    pub fn script() -> Self {
        Self::new("script")
    }
    pub fn style() -> Self {
        Self::new("style")
    }
    pub fn link() -> Self {
        Self::new("link")
    }
    pub fn meta() -> Self {
        Self::new("meta")
    }
    pub fn title() -> Self {
        Self::new("title")
    }
    pub fn input() -> Self {
        Self::new("input")
    }
    pub fn button() -> Self {
        Self::new("button")
    }
    pub fn form() -> Self {
        Self::new("form")
    }
    pub fn table() -> Self {
        Self::new("table")
    }
    pub fn tr() -> Self {
        Self::new("tr")
    }
    pub fn td() -> Self {
        Self::new("td")
    }
    pub fn th() -> Self {
        Self::new("th")
    }
    pub fn ul() -> Self {
        Self::new("ul")
    }
    pub fn ol() -> Self {
        Self::new("ol")
    }
    pub fn li() -> Self {
        Self::new("li")
    }
    pub fn h1() -> Self {
        Self::new("h1")
    }
    pub fn h2() -> Self {
        Self::new("h2")
    }
    pub fn h3() -> Self {
        Self::new("h3")
    }
    pub fn h4() -> Self {
        Self::new("h4")
    }
    pub fn h5() -> Self {
        Self::new("h5")
    }
    pub fn h6() -> Self {
        Self::new("h6")
    }
    pub fn br() -> Self {
        Self::new("br")
    }
    pub fn hr() -> Self {
        Self::new("hr")
    }
    pub fn iframe() -> Self {
        Self::new("iframe")
    }
    pub fn canvas() -> Self {
        Self::new("canvas")
    }
    pub fn video() -> Self {
        Self::new("video")
    }
    pub fn audio() -> Self {
        Self::new("audio")
    }
    pub fn svg() -> Self {
        Self::new("svg")
    }
}

impl std::fmt::Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for TagName {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other.to_ascii_lowercase()
    }
}

impl PartialEq<&str> for TagName {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == other.to_ascii_lowercase()
    }
}

bitflags! {
    /// Element flags for quick property checks.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct ElementFlags: u32 {
        const VOID = 1 << 0;
        const RAW_TEXT = 1 << 1;
        const ESCAPABLE_RAW_TEXT = 1 << 2;
        const FOREIGN = 1 << 3;
        const TEMPLATE_CONTENTS = 1 << 4;
        const FORMATTING = 1 << 5;
        const BLOCK = 1 << 6;
        const INLINE = 1 << 7;
        const HIDDEN = 1 << 8;
        const FOCUSABLE = 1 << 9;
        const DISABLED = 1 << 10;
        const CHECKED = 1 << 11;
        const SELECTED = 1 << 12;
        const EXPANDED = 1 << 13;
    }
}

/// Element-specific data.
#[derive(Clone, Debug)]
pub struct ElementData {
    /// Tag name (lowercase).
    pub tag_name: TagName,
    /// Namespace URI.
    pub namespace: Option<Arc<str>>,
    /// Attributes.
    pub attributes: AttributeMap,
    /// ID attribute (cached).
    pub id: Option<Arc<str>>,
    /// Class list (cached).
    pub class_list: SmallVec<[Arc<str>; 4]>,
    /// Element flags.
    pub flags: ElementFlags,
    /// Inline style attribute (parsed).
    pub inline_style: Option<String>,
    /// Shadow root (if any).
    pub shadow_root: Option<ShadowRoot>,
    /// Custom element state.
    pub custom_state: CustomElementState,
}

impl ElementData {
    pub fn new(tag_name: TagName) -> Self {
        let flags = Self::default_flags(&tag_name);
        Self {
            tag_name,
            namespace: None,
            attributes: AttributeMap::new(),
            id: None,
            class_list: SmallVec::new(),
            flags,
            inline_style: None,
            shadow_root: None,
            custom_state: CustomElementState::Undefined,
        }
    }

    pub fn with_namespace(tag_name: TagName, namespace: &str) -> Self {
        let mut elem = Self::new(tag_name);
        elem.namespace = Some(Arc::from(namespace));
        elem
    }

    /// Get default flags for a tag.
    fn default_flags(tag_name: &TagName) -> ElementFlags {
        let mut flags = ElementFlags::empty();
        let name = tag_name.as_str();

        // Void elements (self-closing)
        if matches!(
            name,
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
                | "param" | "source" | "track" | "wbr"
        ) {
            flags |= ElementFlags::VOID;
        }

        // Raw text elements
        if matches!(name, "script" | "style") {
            flags |= ElementFlags::RAW_TEXT;
        }

        // Escapable raw text elements
        if matches!(name, "textarea" | "title") {
            flags |= ElementFlags::ESCAPABLE_RAW_TEXT;
        }

        // Formatting elements
        if matches!(
            name,
            "a" | "b" | "big" | "code" | "em" | "font" | "i" | "nobr" | "s" | "small"
                | "strike" | "strong" | "tt" | "u"
        ) {
            flags |= ElementFlags::FORMATTING;
        }

        // Block elements
        if matches!(
            name,
            "address" | "article" | "aside" | "blockquote" | "details" | "dialog" | "dd"
                | "div" | "dl" | "dt" | "fieldset" | "figcaption" | "figure" | "footer"
                | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | "hgroup"
                | "hr" | "li" | "main" | "nav" | "ol" | "p" | "pre" | "section" | "table"
                | "ul"
        ) {
            flags |= ElementFlags::BLOCK;
        }

        // Focusable elements
        if matches!(
            name,
            "a" | "button" | "input" | "select" | "textarea" | "details" | "summary"
        ) {
            flags |= ElementFlags::FOCUSABLE;
        }

        flags
    }

    /// Set an attribute, updating cached values.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        let name_lower = name.to_ascii_lowercase();

        // Update cached values
        match name_lower.as_str() {
            "id" => {
                self.id = Some(Arc::from(value));
            }
            "class" => {
                self.class_list = value
                    .split_whitespace()
                    .map(|s| Arc::from(s))
                    .collect();
            }
            "style" => {
                self.inline_style = Some(value.to_string());
            }
            "hidden" => {
                self.flags.insert(ElementFlags::HIDDEN);
            }
            "disabled" => {
                self.flags.insert(ElementFlags::DISABLED);
            }
            "checked" => {
                self.flags.insert(ElementFlags::CHECKED);
            }
            "selected" => {
                self.flags.insert(ElementFlags::SELECTED);
            }
            _ => {}
        }

        self.attributes.set(&name_lower, value);
    }

    /// Remove an attribute.
    pub fn remove_attribute(&mut self, name: &str) {
        let name_lower = name.to_ascii_lowercase();

        match name_lower.as_str() {
            "id" => self.id = None,
            "class" => self.class_list.clear(),
            "style" => self.inline_style = None,
            "hidden" => self.flags.remove(ElementFlags::HIDDEN),
            "disabled" => self.flags.remove(ElementFlags::DISABLED),
            "checked" => self.flags.remove(ElementFlags::CHECKED),
            "selected" => self.flags.remove(ElementFlags::SELECTED),
            _ => {}
        }

        self.attributes.remove(&name_lower);
    }

    /// Get an attribute value.
    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(&name.to_ascii_lowercase())
    }

    /// Check if element has an attribute.
    #[inline]
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains(&name.to_ascii_lowercase())
    }

    /// Check if element has a class.
    pub fn has_class(&self, class: &str) -> bool {
        self.class_list.iter().any(|c| c.as_ref() == class)
    }

    /// Add a class.
    pub fn add_class(&mut self, class: &str) {
        if !self.has_class(class) {
            self.class_list.push(Arc::from(class));
            self.update_class_attribute();
        }
    }

    /// Remove a class.
    pub fn remove_class(&mut self, class: &str) {
        if let Some(pos) = self.class_list.iter().position(|c| c.as_ref() == class) {
            self.class_list.remove(pos);
            self.update_class_attribute();
        }
    }

    /// Toggle a class.
    pub fn toggle_class(&mut self, class: &str) -> bool {
        if self.has_class(class) {
            self.remove_class(class);
            false
        } else {
            self.add_class(class);
            true
        }
    }

    fn update_class_attribute(&mut self) {
        let class_str: String = self
            .class_list
            .iter()
            .map(|c| c.as_ref())
            .collect::<Vec<_>>()
            .join(" ");
        self.attributes.set("class", &class_str);
    }

    /// Check if this is a void element.
    #[inline]
    pub fn is_void(&self) -> bool {
        self.flags.contains(ElementFlags::VOID)
    }

    /// Check if this is a block element.
    #[inline]
    pub fn is_block(&self) -> bool {
        self.flags.contains(ElementFlags::BLOCK)
    }

    /// Check if this element is hidden.
    #[inline]
    pub fn is_hidden(&self) -> bool {
        self.flags.contains(ElementFlags::HIDDEN)
    }

    /// Check if this element is focusable.
    #[inline]
    pub fn is_focusable(&self) -> bool {
        self.flags.contains(ElementFlags::FOCUSABLE) && !self.flags.contains(ElementFlags::DISABLED)
    }

    /// Check if this element matches a simple selector.
    pub fn matches_selector(&self, selector: &SimpleSelector) -> bool {
        match selector {
            SimpleSelector::Tag(tag) => self.tag_name.as_str() == tag,
            SimpleSelector::Id(id) => self.id.as_ref().map(|i| i.as_ref()) == Some(id.as_str()),
            SimpleSelector::Class(class) => self.has_class(class),
            SimpleSelector::Attribute { name, op, value } => {
                self.matches_attribute_selector(name, op.as_deref(), value.as_deref())
            }
            SimpleSelector::Universal => true,
        }
    }

    fn matches_attribute_selector(
        &self,
        name: &str,
        op: Option<&str>,
        value: Option<&str>,
    ) -> bool {
        let attr_value = match self.get_attribute(name) {
            Some(v) => v,
            None => return false,
        };

        match (op, value) {
            (None, _) => true, // Just check presence
            (Some("="), Some(v)) => attr_value == v,
            (Some("~="), Some(v)) => attr_value.split_whitespace().any(|w| w == v),
            (Some("|="), Some(v)) => attr_value == v || attr_value.starts_with(&format!("{}-", v)),
            (Some("^="), Some(v)) => attr_value.starts_with(v),
            (Some("$="), Some(v)) => attr_value.ends_with(v),
            (Some("*="), Some(v)) => attr_value.contains(v),
            _ => false,
        }
    }
}

/// Simple CSS selector for matching.
#[derive(Clone, Debug)]
pub enum SimpleSelector {
    Tag(String),
    Id(String),
    Class(String),
    Attribute {
        name: String,
        op: Option<String>,
        value: Option<String>,
    },
    Universal,
}

/// Shadow DOM root.
#[derive(Clone, Debug)]
pub struct ShadowRoot {
    pub mode: ShadowRootMode,
    pub delegates_focus: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowRootMode {
    Open,
    Closed,
}

/// Custom element state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CustomElementState {
    #[default]
    Undefined,
    Failed,
    Uncustomized,
    Precustomized,
    Custom,
}

/// HTML element interface trait.
pub trait Element {
    fn tag_name(&self) -> &str;
    fn id(&self) -> Option<&str>;
    fn class_name(&self) -> String;
    fn get_attribute(&self, name: &str) -> Option<&str>;
    fn set_attribute(&mut self, name: &str, value: &str);
    fn remove_attribute(&mut self, name: &str);
    fn has_attribute(&self, name: &str) -> bool;
    fn inner_html(&self) -> String;
    fn outer_html(&self) -> String;
    fn set_inner_html(&mut self, html: &str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_name() {
        let div = TagName::div();
        assert_eq!(div.as_str(), "div");
        assert!(div == "div");
        assert!(div == "DIV");
    }

    #[test]
    fn test_element_attributes() {
        let mut elem = ElementData::new(TagName::div());
        elem.set_attribute("id", "test");
        elem.set_attribute("class", "foo bar baz");

        assert_eq!(elem.id.as_ref().map(|s| s.as_ref()), Some("test"));
        assert_eq!(elem.class_list.len(), 3);
        assert!(elem.has_class("foo"));
        assert!(elem.has_class("bar"));
        assert!(!elem.has_class("qux"));
    }

    #[test]
    fn test_void_elements() {
        let br = ElementData::new(TagName::br());
        let img = ElementData::new(TagName::img());
        let div = ElementData::new(TagName::div());

        assert!(br.is_void());
        assert!(img.is_void());
        assert!(!div.is_void());
    }
}
