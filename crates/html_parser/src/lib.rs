//! HTML5 Parser implementation using html5ever.
//!
//! This crate provides HTML parsing capabilities using Mozilla's html5ever
//! library, converting HTML into our DOM tree structure.

pub mod parser;
pub mod tree_builder;
pub mod serializer;
pub mod tokenizer;

pub use parser::{parse_html, parse_html_fragment, HtmlParser, ParseOptions};
pub use serializer::serialize_html;
