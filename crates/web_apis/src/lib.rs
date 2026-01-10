//! Web APIs implementation.
//!
//! This crate provides implementations of various Web APIs including:
//! - Fetch API
//! - WebSocket API
//! - Storage API (localStorage, sessionStorage)
//! - History API
//! - Location API
//! - Navigator API
//! - Performance API
//! - Intersection Observer API
//! - Mutation Observer API
//! - Resize Observer API

pub mod console;
pub mod crypto;
pub mod events;
pub mod fetch;
pub mod geolocation;
pub mod history;
pub mod intersection_observer;
pub mod location;
pub mod mutation_observer;
pub mod navigator;
pub mod performance;
pub mod resize_observer;
pub mod storage;
pub mod url;
pub mod websocket;
pub mod workers;

pub use fetch::{Fetch, FetchRequest, FetchResponse};
pub use history::History;
pub use location::Location;
pub use navigator::Navigator;
pub use performance::Performance;
pub use storage::{LocalStorage, SessionStorage, Storage};
pub use websocket::WebSocket;
