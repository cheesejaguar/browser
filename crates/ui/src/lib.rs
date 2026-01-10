//! Browser UI shell.
//!
//! This crate provides the browser user interface including:
//! - Window management
//! - Tab bar and tabs
//! - Address bar
//! - Navigation controls
//! - Bookmarks
//! - Settings
//! - DevTools integration

pub mod address_bar;
pub mod bookmarks;
pub mod browser;
pub mod devtools;
pub mod downloads;
pub mod find_bar;
pub mod history_ui;
pub mod menu;
pub mod navigation;
pub mod settings;
pub mod tab;
pub mod tab_bar;
pub mod theme;
pub mod window;

pub use browser::Browser;
pub use tab::Tab;
pub use window::BrowserWindow;
