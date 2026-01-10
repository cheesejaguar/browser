//! Style computation system.
//!
//! This crate handles CSS cascade, inheritance, and style resolution.

pub mod cascade;
pub mod computed;
pub mod matching;
pub mod stylist;
pub mod inheritance;
pub mod resolver;

pub use cascade::{cascade_styles, Origin, CascadeLevel};
pub use computed::ComputedStyle;
pub use matching::match_selectors;
pub use stylist::Stylist;
pub use resolver::StyleResolver;
