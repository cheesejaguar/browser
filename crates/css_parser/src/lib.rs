//! CSS Parser implementation.
//!
//! This crate provides CSS parsing capabilities using cssparser.

pub mod parser;
pub mod stylesheet;
pub mod selector;
pub mod values;
pub mod properties;
pub mod media;
pub mod color;

pub use parser::{parse_css, parse_style_attribute, CssParser};
pub use stylesheet::{Stylesheet, StyleRule, CssRule};
pub use selector::{Selector, SelectorList, Specificity};
pub use values::{CssValue, CssValueList};
pub use properties::{Property, PropertyId, PropertyDeclaration};
pub use media::{MediaQuery, MediaType, MediaFeature};
