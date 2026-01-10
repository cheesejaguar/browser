//! Layout engine implementation.
//!
//! This crate handles box layout, inline/block formatting, flexbox, and grid.

pub mod box_model;
pub mod layout_box;
pub mod block;
pub mod inline;
pub mod flex;
pub mod grid;
pub mod text;
pub mod tree;
pub mod engine;

pub use box_model::{BoxDimensions, BoxType};
pub use layout_box::LayoutBox;
pub use tree::LayoutTree;
pub use engine::LayoutEngine;
