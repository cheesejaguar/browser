//! Common utilities and types used across the browser engine.

pub mod color;
pub mod geometry;
pub mod error;
pub mod units;

pub use color::Color;
pub use geometry::{Point, Size, Rect, EdgeSizes};
pub use error::{BrowserError, BrowserResult};
pub use units::{Length, LengthPercentage, Percentage};
