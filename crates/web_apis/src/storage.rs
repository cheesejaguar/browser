//! Web Storage API (localStorage, sessionStorage).

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Storage trait for localStorage and sessionStorage.
pub trait Storage: Send + Sync {
    /// Get the number of items in storage.
    fn length(&self) -> usize;

    /// Get the key at the given index.
    fn key(&self, index: usize) -> Option<String>;

    /// Get the value for the given key.
    fn get_item(&self, key: &str) -> Option<String>;

    /// Set a value for the given key.
    fn set_item(&mut self, key: &str, value: &str) -> Result<(), StorageError>;

    /// Remove the value for the given key.
    fn remove_item(&mut self, key: &str);

    /// Clear all items from storage.
    fn clear(&mut self);
}

/// Memory-backed storage implementation.
#[derive(Clone, Debug, Default)]
pub struct MemoryStorage {
    data: HashMap<String, String>,
    keys: Vec<String>,
    quota: usize,
}

impl MemoryStorage {
    /// Create a new memory storage with default quota (5MB).
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            keys: Vec::new(),
            quota: 5 * 1024 * 1024, // 5MB
        }
    }

    /// Create with custom quota.
    pub fn with_quota(quota: usize) -> Self {
        Self {
            data: HashMap::new(),
            keys: Vec::new(),
            quota,
        }
    }

    /// Get current usage in bytes.
    fn usage(&self) -> usize {
        self.data.iter().map(|(k, v)| k.len() + v.len()).sum()
    }
}

impl Storage for MemoryStorage {
    fn length(&self) -> usize {
        self.data.len()
    }

    fn key(&self, index: usize) -> Option<String> {
        self.keys.get(index).cloned()
    }

    fn get_item(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set_item(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        // Check quota
        let new_size = self.usage() + key.len() + value.len();
        if new_size > self.quota {
            return Err(StorageError::QuotaExceeded);
        }

        // Update keys list if new key
        if !self.data.contains_key(key) {
            self.keys.push(key.to_string());
        }

        self.data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove_item(&mut self, key: &str) {
        self.data.remove(key);
        self.keys.retain(|k| k != key);
    }

    fn clear(&mut self) {
        self.data.clear();
        self.keys.clear();
    }
}

/// localStorage implementation.
pub struct LocalStorage {
    storage: Arc<RwLock<MemoryStorage>>,
    origin: String,
}

impl LocalStorage {
    /// Create a new localStorage for an origin.
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            origin: origin.into(),
        }
    }

    /// Get the origin.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Register localStorage on the global object.
    pub fn register(&self, context: &mut Context) {
        let storage = self.storage.clone();

        let local_storage = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(storage_length), js_string!("length"), 0)
            .function(NativeFunction::from_fn_ptr(storage_key), js_string!("key"), 1)
            .function(NativeFunction::from_fn_ptr(storage_get_item), js_string!("getItem"), 1)
            .function(NativeFunction::from_fn_ptr(storage_set_item), js_string!("setItem"), 2)
            .function(NativeFunction::from_fn_ptr(storage_remove_item), js_string!("removeItem"), 1)
            .function(NativeFunction::from_fn_ptr(storage_clear), js_string!("clear"), 0)
            .build();

        context
            .register_global_property(js_string!("localStorage"), local_storage, Attribute::all())
            .expect("Failed to register localStorage");
    }
}

impl Storage for LocalStorage {
    fn length(&self) -> usize {
        self.storage.read().length()
    }

    fn key(&self, index: usize) -> Option<String> {
        self.storage.read().key(index)
    }

    fn get_item(&self, key: &str) -> Option<String> {
        self.storage.read().get_item(key)
    }

    fn set_item(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.storage.write().set_item(key, value)
    }

    fn remove_item(&mut self, key: &str) {
        self.storage.write().remove_item(key)
    }

    fn clear(&mut self) {
        self.storage.write().clear()
    }
}

/// sessionStorage implementation.
pub struct SessionStorage {
    storage: Arc<RwLock<MemoryStorage>>,
    origin: String,
}

impl SessionStorage {
    /// Create a new sessionStorage for an origin.
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            origin: origin.into(),
        }
    }

    /// Get the origin.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Register sessionStorage on the global object.
    pub fn register(&self, context: &mut Context) {
        let session_storage = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(storage_length), js_string!("length"), 0)
            .function(NativeFunction::from_fn_ptr(storage_key), js_string!("key"), 1)
            .function(NativeFunction::from_fn_ptr(storage_get_item), js_string!("getItem"), 1)
            .function(NativeFunction::from_fn_ptr(storage_set_item), js_string!("setItem"), 2)
            .function(NativeFunction::from_fn_ptr(storage_remove_item), js_string!("removeItem"), 1)
            .function(NativeFunction::from_fn_ptr(storage_clear), js_string!("clear"), 0)
            .build();

        context
            .register_global_property(js_string!("sessionStorage"), session_storage, Attribute::all())
            .expect("Failed to register sessionStorage");
    }
}

impl Storage for SessionStorage {
    fn length(&self) -> usize {
        self.storage.read().length()
    }

    fn key(&self, index: usize) -> Option<String> {
        self.storage.read().key(index)
    }

    fn get_item(&self, key: &str) -> Option<String> {
        self.storage.read().get_item(key)
    }

    fn set_item(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.storage.write().set_item(key, value)
    }

    fn remove_item(&mut self, key: &str) {
        self.storage.write().remove_item(key)
    }

    fn clear(&mut self) {
        self.storage.write().clear()
    }
}

/// Storage error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum StorageError {
    #[error("Quota exceeded")]
    QuotaExceeded,
    #[error("Storage is disabled")]
    Disabled,
    #[error("Security error")]
    SecurityError,
}

// Native function implementations
fn storage_length(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    // Would return the length from the actual storage
    Ok(JsValue::from(0))
}

fn storage_key(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _index = args.get_or_undefined(0).to_u32(context)?;
    Ok(JsValue::null())
}

fn storage_get_item(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _key = args.get_or_undefined(0).to_string(context)?;
    Ok(JsValue::null())
}

fn storage_set_item(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _key = args.get_or_undefined(0).to_string(context)?;
    let _value = args.get_or_undefined(1).to_string(context)?;
    Ok(JsValue::undefined())
}

fn storage_remove_item(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _key = args.get_or_undefined(0).to_string(context)?;
    Ok(JsValue::undefined())
}

fn storage_clear(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

/// Storage event for storage changes.
#[derive(Clone, Debug)]
pub struct StorageEvent {
    /// The key that was changed.
    pub key: Option<String>,
    /// The old value.
    pub old_value: Option<String>,
    /// The new value.
    pub new_value: Option<String>,
    /// The URL of the document that changed the storage.
    pub url: String,
    /// The storage area that was changed.
    pub storage_area: StorageArea,
}

/// Storage area identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageArea {
    Local,
    Session,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage() {
        let mut storage = MemoryStorage::new();

        storage.set_item("key1", "value1").unwrap();
        assert_eq!(storage.get_item("key1"), Some("value1".to_string()));
        assert_eq!(storage.length(), 1);

        storage.set_item("key2", "value2").unwrap();
        assert_eq!(storage.length(), 2);

        storage.remove_item("key1");
        assert_eq!(storage.get_item("key1"), None);
        assert_eq!(storage.length(), 1);

        storage.clear();
        assert_eq!(storage.length(), 0);
    }

    #[test]
    fn test_storage_quota() {
        let mut storage = MemoryStorage::with_quota(10);

        storage.set_item("a", "12345").unwrap();
        let result = storage.set_item("b", "123456");

        assert!(matches!(result, Err(StorageError::QuotaExceeded)));
    }

    #[test]
    fn test_storage_key_index() {
        let mut storage = MemoryStorage::new();

        storage.set_item("first", "1").unwrap();
        storage.set_item("second", "2").unwrap();
        storage.set_item("third", "3").unwrap();

        assert_eq!(storage.key(0), Some("first".to_string()));
        assert_eq!(storage.key(1), Some("second".to_string()));
        assert_eq!(storage.key(2), Some("third".to_string()));
        assert_eq!(storage.key(3), None);
    }
}
