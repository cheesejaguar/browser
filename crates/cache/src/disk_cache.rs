//! Disk-based cache.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::SystemTime;

/// Disk cache for persistent storage.
pub struct DiskCache {
    directory: PathBuf,
    index: HashMap<String, DiskCacheEntry>,
    max_size: u64,
    current_size: u64,
}

impl DiskCache {
    pub fn new(directory: PathBuf, max_size: u64) -> std::io::Result<Self> {
        fs::create_dir_all(&directory)?;

        let mut cache = Self {
            directory,
            index: HashMap::new(),
            max_size,
            current_size: 0,
        };

        cache.load_index()?;
        Ok(cache)
    }

    fn load_index(&mut self) -> std::io::Result<()> {
        let index_path = self.directory.join("index.json");
        if index_path.exists() {
            let data = fs::read_to_string(&index_path)?;
            if let Ok(index) = serde_json::from_str(&data) {
                self.index = index;
                self.current_size = self.index.values().map(|e| e.size).sum();
            }
        }
        Ok(())
    }

    fn save_index(&self) -> std::io::Result<()> {
        let index_path = self.directory.join("index.json");
        let data = serde_json::to_string(&self.index)?;
        fs::write(index_path, data)?;
        Ok(())
    }

    fn key_to_filename(&self, key: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    pub fn get(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
        let entry = match self.index.get(key) {
            Some(e) => e,
            None => return Ok(None),
        };

        let path = self.directory.join(&entry.filename);
        if !path.exists() {
            return Ok(None);
        }

        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(Some(data))
    }

    pub fn put(&mut self, key: &str, data: &[u8]) -> std::io::Result<()> {
        let size = data.len() as u64;

        // Evict if necessary
        while self.current_size + size > self.max_size {
            if !self.evict_one()? {
                break;
            }
        }

        let filename = self.key_to_filename(key);
        let path = self.directory.join(&filename);

        let mut file = File::create(path)?;
        file.write_all(data)?;

        // Update index
        if let Some(old) = self.index.get(key) {
            self.current_size -= old.size;
        }

        self.index.insert(
            key.to_string(),
            DiskCacheEntry {
                filename,
                size,
                created: SystemTime::now(),
            },
        );

        self.current_size += size;
        self.save_index()?;

        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> std::io::Result<bool> {
        let entry = match self.index.remove(key) {
            Some(e) => e,
            None => return Ok(false),
        };

        let path = self.directory.join(&entry.filename);
        if path.exists() {
            fs::remove_file(path)?;
        }

        self.current_size -= entry.size;
        self.save_index()?;

        Ok(true)
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        for entry in self.index.values() {
            let path = self.directory.join(&entry.filename);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }

        self.index.clear();
        self.current_size = 0;
        self.save_index()?;

        Ok(())
    }

    fn evict_one(&mut self) -> std::io::Result<bool> {
        let oldest = self
            .index
            .iter()
            .min_by_key(|(_, e)| e.created)
            .map(|(k, _)| k.clone());

        if let Some(key) = oldest {
            return self.remove(&key);
        }

        Ok(false)
    }

    pub fn size(&self) -> u64 {
        self.current_size
    }

    pub fn entry_count(&self) -> usize {
        self.index.len()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.index.contains_key(key)
    }
}

/// Disk cache entry metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DiskCacheEntry {
    filename: String,
    size: u64,
    #[serde(with = "system_time_serde")]
    created: SystemTime,
}

mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_disk_cache() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = DiskCache::new(temp_dir.path().to_path_buf(), 1024).unwrap();

        cache.put("key1", b"value1").unwrap();
        assert!(cache.contains("key1"));

        let data = cache.get("key1").unwrap().unwrap();
        assert_eq!(data, b"value1");

        cache.remove("key1").unwrap();
        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_disk_cache_eviction() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = DiskCache::new(temp_dir.path().to_path_buf(), 10).unwrap();

        cache.put("key1", b"12345").unwrap();
        cache.put("key2", b"67890").unwrap();

        // This should evict key1
        cache.put("key3", b"abcde").unwrap();

        assert!(!cache.contains("key1"));
        assert!(cache.contains("key2"));
        assert!(cache.contains("key3"));
    }
}
