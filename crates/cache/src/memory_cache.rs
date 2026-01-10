//! In-memory LRU cache.

use std::collections::HashMap;
use std::hash::Hash;
use parking_lot::RwLock;

/// LRU memory cache.
pub struct MemoryCache<K, V> {
    entries: RwLock<LruMap<K, V>>,
    max_size: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> MemoryCache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: RwLock::new(LruMap::new(max_size)),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.write().get(key).cloned()
    }

    pub fn put(&self, key: K, value: V) {
        self.entries.write().put(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.entries.write().remove(key)
    }

    pub fn clear(&self) {
        self.entries.write().clear();
    }

    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    pub fn contains(&self, key: &K) -> bool {
        self.entries.read().contains(key)
    }
}

/// Simple LRU map implementation.
struct LruMap<K, V> {
    map: HashMap<K, usize>,
    entries: Vec<Option<(K, V)>>,
    order: Vec<usize>,
    max_size: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> LruMap<K, V> {
    fn new(max_size: usize) -> Self {
        Self {
            map: HashMap::new(),
            entries: Vec::new(),
            order: Vec::new(),
            max_size,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&index) = self.map.get(key) {
            // Move to end (most recently used)
            self.order.retain(|&i| i != index);
            self.order.push(index);

            return self.entries.get(index).and_then(|e| e.as_ref().map(|(_, v)| v));
        }
        None
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&index) = self.map.get(&key) {
            // Update existing
            self.entries[index] = Some((key, value));
            self.order.retain(|&i| i != index);
            self.order.push(index);
            return;
        }

        // Evict if necessary
        while self.order.len() >= self.max_size {
            if let Some(oldest) = self.order.first().copied() {
                self.order.remove(0);
                if let Some(Some((k, _))) = self.entries.get(oldest) {
                    self.map.remove(k);
                }
                self.entries[oldest] = None;
            }
        }

        // Find an empty slot or add new
        let index = self.entries.iter().position(|e| e.is_none()).unwrap_or_else(|| {
            self.entries.push(None);
            self.entries.len() - 1
        });

        self.entries[index] = Some((key.clone(), value));
        self.map.insert(key, index);
        self.order.push(index);
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(index) = self.map.remove(key) {
            self.order.retain(|&i| i != index);
            if let Some(entry) = self.entries.get_mut(index) {
                return entry.take().map(|(_, v)| v);
            }
        }
        None
    }

    fn clear(&mut self) {
        self.map.clear();
        self.entries.clear();
        self.order.clear();
    }

    fn len(&self) -> usize {
        self.order.len()
    }

    fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache() {
        let cache = MemoryCache::new(3);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = MemoryCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3); // Should evict "a"

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_lru_access_updates_order() {
        let cache = MemoryCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);

        // Access "a" to make it recently used
        cache.get(&"a");

        cache.put("c", 3); // Should evict "b", not "a"

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(3));
    }
}
