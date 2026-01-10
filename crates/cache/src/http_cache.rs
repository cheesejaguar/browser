//! HTTP cache implementation.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use parking_lot::RwLock;

/// HTTP cache.
pub struct HttpCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    max_size: usize,
    current_size: RwLock<usize>,
}

impl HttpCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_size,
            current_size: RwLock::new(0),
        }
    }

    pub fn get(&self, url: &str) -> Option<CacheEntry> {
        let entries = self.entries.read();
        entries.get(url).cloned()
    }

    pub fn put(&self, url: &str, entry: CacheEntry) {
        let entry_size = entry.data.len();

        // Evict if needed
        while *self.current_size.read() + entry_size > self.max_size {
            if !self.evict_one() {
                break;
            }
        }

        let mut entries = self.entries.write();
        if let Some(old) = entries.insert(url.to_string(), entry) {
            *self.current_size.write() -= old.data.len();
        }
        *self.current_size.write() += entry_size;
    }

    pub fn remove(&self, url: &str) {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.remove(url) {
            *self.current_size.write() -= entry.data.len();
        }
    }

    pub fn clear(&self) {
        self.entries.write().clear();
        *self.current_size.write() = 0;
    }

    pub fn is_fresh(&self, url: &str) -> bool {
        self.entries.read().get(url).map(|e| e.is_fresh()).unwrap_or(false)
    }

    fn evict_one(&self) -> bool {
        let mut entries = self.entries.write();
        let oldest = entries.iter().min_by_key(|(_, e)| e.created_at).map(|(k, _)| k.clone());
        if let Some(key) = oldest {
            if let Some(entry) = entries.remove(&key) {
                *self.current_size.write() -= entry.data.len();
                return true;
            }
        }
        false
    }

    pub fn size(&self) -> usize {
        *self.current_size.read()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.read().len()
    }
}

/// Cache entry.
#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub url: String,
    pub data: Vec<u8>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub cache_control: CacheControl,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
}

impl CacheEntry {
    pub fn new(url: &str, data: Vec<u8>, status_code: u16) -> Self {
        Self {
            url: url.to_string(),
            data,
            content_type: None,
            etag: None,
            last_modified: None,
            cache_control: CacheControl::default(),
            created_at: SystemTime::now(),
            expires_at: None,
            status_code,
            headers: HashMap::new(),
        }
    }

    pub fn is_fresh(&self) -> bool {
        if self.cache_control.no_cache || self.cache_control.no_store {
            return false;
        }

        if let Some(expires) = self.expires_at {
            return SystemTime::now() < expires;
        }

        if let Some(max_age) = self.cache_control.max_age {
            let age = self.created_at.elapsed().unwrap_or(Duration::MAX);
            return age < max_age;
        }

        false
    }

    pub fn is_stale(&self) -> bool {
        !self.is_fresh()
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed().unwrap_or(Duration::ZERO)
    }

    pub fn can_revalidate(&self) -> bool {
        self.etag.is_some() || self.last_modified.is_some()
    }

    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }

    pub fn with_etag(mut self, etag: &str) -> Self {
        self.etag = Some(etag.to_string());
        self
    }

    pub fn with_last_modified(mut self, last_modified: &str) -> Self {
        self.last_modified = Some(last_modified.to_string());
        self
    }

    pub fn with_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = cache_control;
        self
    }

    pub fn with_expires(mut self, expires: SystemTime) -> Self {
        self.expires_at = Some(expires);
        self
    }
}

/// Cache-Control header parsed.
#[derive(Clone, Debug, Default)]
pub struct CacheControl {
    pub max_age: Option<Duration>,
    pub s_maxage: Option<Duration>,
    pub no_cache: bool,
    pub no_store: bool,
    pub private: bool,
    pub public: bool,
    pub must_revalidate: bool,
    pub proxy_revalidate: bool,
    pub immutable: bool,
    pub stale_while_revalidate: Option<Duration>,
    pub stale_if_error: Option<Duration>,
}

impl CacheControl {
    pub fn parse(header: &str) -> Self {
        let mut cc = CacheControl::default();

        for directive in header.split(',').map(|s| s.trim()) {
            if directive.starts_with("max-age=") {
                if let Ok(secs) = directive[8..].parse::<u64>() {
                    cc.max_age = Some(Duration::from_secs(secs));
                }
            } else if directive.starts_with("s-maxage=") {
                if let Ok(secs) = directive[9..].parse::<u64>() {
                    cc.s_maxage = Some(Duration::from_secs(secs));
                }
            } else if directive.starts_with("stale-while-revalidate=") {
                if let Ok(secs) = directive[23..].parse::<u64>() {
                    cc.stale_while_revalidate = Some(Duration::from_secs(secs));
                }
            } else if directive.starts_with("stale-if-error=") {
                if let Ok(secs) = directive[15..].parse::<u64>() {
                    cc.stale_if_error = Some(Duration::from_secs(secs));
                }
            } else {
                match directive {
                    "no-cache" => cc.no_cache = true,
                    "no-store" => cc.no_store = true,
                    "private" => cc.private = true,
                    "public" => cc.public = true,
                    "must-revalidate" => cc.must_revalidate = true,
                    "proxy-revalidate" => cc.proxy_revalidate = true,
                    "immutable" => cc.immutable = true,
                    _ => {}
                }
            }
        }

        cc
    }

    pub fn is_cacheable(&self) -> bool {
        !self.no_store
    }
}

/// Cache validation result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheValidation {
    Fresh,
    Stale,
    MustRevalidate,
    NotCacheable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_parse() {
        let cc = CacheControl::parse("max-age=3600, public");
        assert_eq!(cc.max_age, Some(Duration::from_secs(3600)));
        assert!(cc.public);
        assert!(!cc.private);
    }

    #[test]
    fn test_cache_control_no_store() {
        let cc = CacheControl::parse("no-store");
        assert!(cc.no_store);
        assert!(!cc.is_cacheable());
    }

    #[test]
    fn test_cache_entry_freshness() {
        let mut entry = CacheEntry::new("https://example.com", vec![1, 2, 3], 200);
        entry.cache_control.max_age = Some(Duration::from_secs(3600));

        assert!(entry.is_fresh());

        entry.cache_control.no_cache = true;
        assert!(!entry.is_fresh());
    }

    #[test]
    fn test_http_cache() {
        let cache = HttpCache::new(1024);

        let entry = CacheEntry::new("https://example.com", vec![1, 2, 3, 4, 5], 200);
        cache.put("https://example.com", entry);

        assert!(cache.get("https://example.com").is_some());
        assert_eq!(cache.size(), 5);

        cache.remove("https://example.com");
        assert!(cache.get("https://example.com").is_none());
        assert_eq!(cache.size(), 0);
    }
}
