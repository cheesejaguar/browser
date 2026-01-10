//! Oxide Browser - A high-performance web browser written in Rust.
//!
//! This crate integrates all browser components:
//! - HTML/CSS parsing
//! - DOM manipulation
//! - Style computation
//! - Layout engine
//! - GPU-accelerated rendering
//! - JavaScript engine
//! - Networking
//! - Security features
//! - Media playback

pub mod engine;
pub mod page;
pub mod pipeline;
pub mod config;

pub use engine::BrowserEngine;
pub use page::Page;
pub use pipeline::RenderPipeline;
pub use config::BrowserConfig;

/// Browser version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent string.
pub fn user_agent() -> String {
    format!(
        "Mozilla/5.0 (compatible; OxideBrowser/{}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        VERSION
    )
}
