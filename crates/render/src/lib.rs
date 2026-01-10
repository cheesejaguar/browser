//! Rendering engine for the browser.
//!
//! This crate handles:
//! - Display list generation from the layout tree
//! - Text rasterization
//! - Image decoding and caching
//! - Paint operations

pub mod display_list;
pub mod painter;
pub mod rasterizer;
pub mod font;
pub mod image_cache;
pub mod color;
pub mod commands;

pub use display_list::{DisplayList, DisplayItem};
pub use painter::Painter;
pub use rasterizer::Rasterizer;
pub use font::FontCache;
