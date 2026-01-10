//! Networking layer for the browser.
//!
//! This crate handles:
//! - HTTP/1.1 and HTTP/2 requests
//! - HTTPS with TLS
//! - Connection pooling
//! - Cookie management
//! - Request/response handling
//! - Content encoding (gzip, brotli)

pub mod client;
pub mod request;
pub mod response;
pub mod headers;
pub mod cookies;
pub mod connection;
pub mod dns;
pub mod loader;

pub use client::HttpClient;
pub use request::{Request, RequestBuilder};
pub use response::Response;
pub use loader::ResourceLoader;
