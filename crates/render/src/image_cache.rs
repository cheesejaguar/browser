//! Image caching and decoding.

use crate::display_list::ImageKey;
use common::color::Color;
use image::{DynamicImage, GenericImageView, ImageFormat};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Decoded image data.
#[derive(Clone)]
pub struct ImageData {
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// RGBA pixel data.
    pub data: Vec<u8>,
    /// Original format.
    pub format: ImageFormat,
}

impl ImageData {
    /// Create from raw RGBA data.
    pub fn from_rgba(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
            format: ImageFormat::Png,
        }
    }

    /// Decode from bytes.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let format = image::guess_format(bytes).ok()?;
        let img = image::load_from_memory(bytes).ok()?;
        let rgba = img.to_rgba8();

        Some(Self {
            width: rgba.width(),
            height: rgba.height(),
            data: rgba.into_raw(),
            format,
        })
    }

    /// Decode with specific format.
    pub fn decode_with_format(bytes: &[u8], format: ImageFormat) -> Option<Self> {
        let img = image::load_from_memory_with_format(bytes, format).ok()?;
        let rgba = img.to_rgba8();

        Some(Self {
            width: rgba.width(),
            height: rgba.height(),
            data: rgba.into_raw(),
            format,
        })
    }

    /// Get pixel at position.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::transparent();
        }

        let offset = ((y * self.width + x) * 4) as usize;
        Color::new(
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        )
    }

    /// Resize image.
    pub fn resize(&self, width: u32, height: u32) -> Self {
        let img = image::RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .expect("Invalid image data");

        let dynamic = DynamicImage::ImageRgba8(img);
        let resized = dynamic.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();

        Self {
            width: rgba.width(),
            height: rgba.height(),
            data: rgba.into_raw(),
            format: self.format,
        }
    }

    /// Create a thumbnail.
    pub fn thumbnail(&self, max_size: u32) -> Self {
        let (new_width, new_height) = if self.width > self.height {
            let ratio = max_size as f32 / self.width as f32;
            (max_size, (self.height as f32 * ratio) as u32)
        } else {
            let ratio = max_size as f32 / self.height as f32;
            ((self.width as f32 * ratio) as u32, max_size)
        };

        self.resize(new_width, new_height)
    }

    /// Get memory size in bytes.
    pub fn memory_size(&self) -> usize {
        self.data.len()
    }
}

/// Image cache entry.
struct CacheEntry {
    /// The image data.
    data: Arc<ImageData>,
    /// Last access time.
    last_access: std::time::Instant,
    /// Access count.
    access_count: u32,
}

/// Image cache for decoded images.
pub struct ImageCache {
    /// Cached images by key.
    cache: RwLock<HashMap<ImageKey, CacheEntry>>,
    /// Maximum cache size in bytes.
    max_size: usize,
    /// Current cache size in bytes.
    current_size: RwLock<usize>,
}

impl ImageCache {
    /// Create a new image cache with the given maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            current_size: RwLock::new(0),
        }
    }

    /// Create with default 100MB cache.
    pub fn with_default_size() -> Self {
        Self::new(100 * 1024 * 1024)
    }

    /// Get an image from the cache.
    pub fn get(&self, key: &ImageKey) -> Option<Arc<ImageData>> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(key) {
            entry.last_access = std::time::Instant::now();
            entry.access_count += 1;
            return Some(entry.data.clone());
        }

        None
    }

    /// Insert an image into the cache.
    pub fn insert(&self, key: ImageKey, data: ImageData) -> Arc<ImageData> {
        let data = Arc::new(data);
        let size = data.memory_size();

        // Evict if necessary
        self.evict_if_needed(size);

        let entry = CacheEntry {
            data: data.clone(),
            last_access: std::time::Instant::now(),
            access_count: 1,
        };

        {
            let mut cache = self.cache.write();
            cache.insert(key, entry);
        }

        {
            let mut current = self.current_size.write();
            *current += size;
        }

        data
    }

    /// Insert or get an image.
    pub fn get_or_insert<F>(&self, key: ImageKey, f: F) -> Option<Arc<ImageData>>
    where
        F: FnOnce() -> Option<ImageData>,
    {
        // Try to get first
        if let Some(data) = self.get(&key) {
            return Some(data);
        }

        // Decode and insert
        let data = f()?;
        Some(self.insert(key, data))
    }

    /// Decode and cache an image from bytes.
    pub fn decode_and_cache(&self, key: ImageKey, bytes: &[u8]) -> Option<Arc<ImageData>> {
        self.get_or_insert(key, || ImageData::decode(bytes))
    }

    /// Evict entries if the cache is over capacity.
    fn evict_if_needed(&self, new_size: usize) {
        let mut current = self.current_size.write();

        if *current + new_size <= self.max_size {
            return;
        }

        // Collect entries for eviction
        let mut cache = self.cache.write();
        let mut entries: Vec<(ImageKey, std::time::Instant, u32, usize)> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.last_access, v.access_count, v.data.memory_size()))
            .collect();

        // Sort by last access time (oldest first), then by access count
        entries.sort_by(|a, b| {
            a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2))
        });

        // Evict until we have enough space
        for (key, _, _, size) in entries {
            if *current + new_size <= self.max_size {
                break;
            }

            cache.remove(&key);
            *current -= size;
        }
    }

    /// Remove an image from the cache.
    pub fn remove(&self, key: &ImageKey) -> Option<Arc<ImageData>> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.remove(key) {
            let mut current = self.current_size.write();
            *current -= entry.data.memory_size();
            return Some(entry.data);
        }

        None
    }

    /// Clear the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();

        let mut current = self.current_size.write();
        *current = 0;
    }

    /// Get current cache size in bytes.
    pub fn size(&self) -> usize {
        *self.current_size.read()
    }

    /// Get number of cached images.
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();

        let total_access: u32 = cache.values().map(|e| e.access_count).sum();
        let oldest = cache.values().map(|e| e.last_access).min();

        CacheStats {
            entry_count: cache.len(),
            total_size: *self.current_size.read(),
            max_size: self.max_size,
            total_accesses: total_access,
            oldest_entry: oldest,
        }
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::with_default_size()
    }
}

/// Cache statistics.
#[derive(Debug)]
pub struct CacheStats {
    /// Number of cached entries.
    pub entry_count: usize,
    /// Total size of cached data.
    pub total_size: usize,
    /// Maximum cache size.
    pub max_size: usize,
    /// Total number of cache accesses.
    pub total_accesses: u32,
    /// Age of oldest entry.
    pub oldest_entry: Option<std::time::Instant>,
}

/// Placeholder image generator.
pub struct PlaceholderGenerator;

impl PlaceholderGenerator {
    /// Create a placeholder image.
    pub fn create(width: u32, height: u32, color: Color) -> ImageData {
        let mut data = vec![0u8; (width * height * 4) as usize];

        for chunk in data.chunks_exact_mut(4) {
            chunk[0] = color.r;
            chunk[1] = color.g;
            chunk[2] = color.b;
            chunk[3] = color.a;
        }

        ImageData::from_rgba(width, height, data)
    }

    /// Create a broken image placeholder.
    pub fn broken_image(width: u32, height: u32) -> ImageData {
        let mut data = vec![0u8; (width * height * 4) as usize];

        // Fill with light gray
        for chunk in data.chunks_exact_mut(4) {
            chunk[0] = 240;
            chunk[1] = 240;
            chunk[2] = 240;
            chunk[3] = 255;
        }

        // Draw an X
        for i in 0..width.min(height) {
            let offset1 = ((i * width + i) * 4) as usize;
            let offset2 = ((i * width + (width - 1 - i)) * 4) as usize;

            if offset1 + 3 < data.len() {
                data[offset1] = 200;
                data[offset1 + 1] = 50;
                data[offset1 + 2] = 50;
                data[offset1 + 3] = 255;
            }

            if offset2 + 3 < data.len() {
                data[offset2] = 200;
                data[offset2 + 1] = 50;
                data[offset2 + 2] = 50;
                data[offset2 + 3] = 255;
            }
        }

        ImageData::from_rgba(width, height, data)
    }

    /// Create a loading placeholder.
    pub fn loading(width: u32, height: u32) -> ImageData {
        let mut data = vec![0u8; (width * height * 4) as usize];

        // Fill with animated-looking gradient
        for y in 0..height {
            for x in 0..width {
                let offset = ((y * width + x) * 4) as usize;
                let gray = 200 + ((x + y) % 40) as u8;
                data[offset] = gray;
                data[offset + 1] = gray;
                data[offset + 2] = gray;
                data[offset + 3] = 255;
            }
        }

        ImageData::from_rgba(width, height, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_data_creation() {
        let data = ImageData::from_rgba(10, 10, vec![255; 400]);
        assert_eq!(data.width, 10);
        assert_eq!(data.height, 10);
    }

    #[test]
    fn test_image_cache() {
        let cache = ImageCache::new(1024 * 1024);
        let data = ImageData::from_rgba(10, 10, vec![255; 400]);
        let key = ImageKey(12345);

        cache.insert(key.clone(), data);
        assert!(cache.get(&key).is_some());
    }

    #[test]
    fn test_placeholder() {
        let placeholder = PlaceholderGenerator::create(100, 100, Color::rgb(255, 0, 0));
        assert_eq!(placeholder.width, 100);
        assert_eq!(placeholder.height, 100);
    }
}
