//! HTTP caching and resource cache.

pub mod disk_cache;
pub mod http_cache;
pub mod memory_cache;

pub use disk_cache::DiskCache;
pub use http_cache::{CacheControl, CacheEntry, HttpCache};
pub use memory_cache::MemoryCache;
