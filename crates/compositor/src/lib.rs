//! Compositor for layer-based rendering.
//!
//! The compositor manages paint layers and handles compositing for effects like:
//! - opacity
//! - transforms
//! - filters
//! - masks
//! - blend modes

pub mod layer;
pub mod scene;
pub mod compositor;
pub mod animation;

pub use self::compositor::Compositor;
pub use layer::{Layer, LayerId, LayerTree};
pub use scene::Scene;
