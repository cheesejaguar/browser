//! JavaScript engine integration using Boa.
//!
//! This crate provides JavaScript execution capabilities for the browser,
//! including DOM bindings, Web APIs, and an event loop.

pub mod bindings;
pub mod console;
pub mod context;
pub mod engine;
pub mod event_loop;
pub mod modules;
pub mod runtime;
pub mod timers;

pub use context::JsContext;
pub use engine::JsEngine;
pub use runtime::Runtime;
