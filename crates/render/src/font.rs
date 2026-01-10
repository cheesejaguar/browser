//! Font handling and text rasterization.

use crate::display_list::{FontKey, FontStyle};
use fontdue::{Font, FontSettings, Metrics};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Rasterized glyph bitmap.
pub struct GlyphBitmap {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Glyph metrics.
    pub metrics: GlyphMetrics,
    /// Grayscale pixel data.
    pub data: Vec<u8>,
}

/// Glyph metrics.
#[derive(Clone, Copy, Debug)]
pub struct GlyphMetrics {
    /// Advance width.
    pub advance_width: f32,
    /// Left side bearing.
    pub xmin: i32,
    /// Bottom of glyph relative to baseline.
    pub ymin: i32,
    /// Width of glyph.
    pub width: u32,
    /// Height of glyph.
    pub height: u32,
}

/// A loaded font with rasterization support.
pub struct LoadedFont {
    /// The fontdue font.
    font: Font,
    /// Font key.
    key: FontKey,
    /// Glyph cache.
    glyph_cache: RwLock<HashMap<(u32, u32), Arc<GlyphBitmap>>>,
}

impl LoadedFont {
    pub fn new(key: FontKey, data: &[u8]) -> Option<Self> {
        let font = Font::from_bytes(data, FontSettings::default()).ok()?;

        Some(Self {
            font,
            key,
            glyph_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Rasterize a glyph at a given size.
    pub fn rasterize(&self, glyph_index: u32, size: f32) -> Arc<GlyphBitmap> {
        let size_key = (size * 10.0) as u32; // Quantize to 0.1px
        let cache_key = (glyph_index, size_key);

        // Check cache
        {
            let cache = self.glyph_cache.read();
            if let Some(bitmap) = cache.get(&cache_key) {
                return bitmap.clone();
            }
        }

        // Rasterize glyph
        // Convert glyph_index to char for fontdue
        let c = char::from_u32(glyph_index).unwrap_or(' ');
        let (metrics, data) = self.font.rasterize(c, size);

        let bitmap = Arc::new(GlyphBitmap {
            width: metrics.width as u32,
            height: metrics.height as u32,
            metrics: GlyphMetrics {
                advance_width: metrics.advance_width,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                width: metrics.width as u32,
                height: metrics.height as u32,
            },
            data,
        });

        // Cache it
        {
            let mut cache = self.glyph_cache.write();
            cache.insert(cache_key, bitmap.clone());
        }

        bitmap
    }

    /// Get glyph metrics without rasterizing.
    pub fn metrics(&self, glyph_index: u32, size: f32) -> GlyphMetrics {
        let c = char::from_u32(glyph_index).unwrap_or(' ');
        let metrics = self.font.metrics(c, size);

        GlyphMetrics {
            advance_width: metrics.advance_width,
            xmin: metrics.xmin,
            ymin: metrics.ymin,
            width: metrics.width as u32,
            height: metrics.height as u32,
        }
    }

    /// Get the font key.
    pub fn key(&self) -> &FontKey {
        &self.key
    }
}

/// Font cache for managing loaded fonts.
pub struct FontCache {
    /// Loaded fonts by key.
    fonts: RwLock<HashMap<FontKey, Arc<LoadedFont>>>,
    /// System font paths.
    system_fonts: RwLock<HashMap<String, Vec<u8>>>,
    /// Default font data.
    default_font: Option<Arc<LoadedFont>>,
}

impl FontCache {
    pub fn new() -> Self {
        let mut cache = Self {
            fonts: RwLock::new(HashMap::new()),
            system_fonts: RwLock::new(HashMap::new()),
            default_font: None,
        };

        // Load default font
        cache.load_default_font();

        cache
    }

    /// Load the default font.
    fn load_default_font(&mut self) {
        // Try to load a system font or use embedded font
        // For now, we'll create a simple default
        let key = FontKey {
            family: "sans-serif".to_string(),
            weight: 400,
            style: FontStyle::Normal,
        };

        // Try loading from common system locations
        let font_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];

        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                if let Some(font) = LoadedFont::new(key.clone(), &data) {
                    self.default_font = Some(Arc::new(font));
                    break;
                }
            }
        }
    }

    /// Get a font by key, loading if necessary.
    pub fn get_font(&self, key: &FontKey) -> Option<Arc<LoadedFont>> {
        // Check cache first
        {
            let fonts = self.fonts.read();
            if let Some(font) = fonts.get(key) {
                return Some(font.clone());
            }
        }

        // Try to load the font
        if let Some(font) = self.load_font(key) {
            let mut fonts = self.fonts.write();
            fonts.insert(key.clone(), font.clone());
            return Some(font);
        }

        // Fall back to default
        self.default_font.clone()
    }

    /// Try to load a font by key.
    fn load_font(&self, key: &FontKey) -> Option<Arc<LoadedFont>> {
        // Map font family to file path
        let family_lower = key.family.to_lowercase();

        let font_name = if family_lower.contains("serif") && !family_lower.contains("sans") {
            "serif"
        } else if family_lower.contains("mono") {
            "monospace"
        } else {
            "sans-serif"
        };

        // Try system font paths
        let paths = self.get_system_font_paths(font_name, key.weight, &key.style);

        for path in paths {
            if let Ok(data) = std::fs::read(&path) {
                if let Some(font) = LoadedFont::new(key.clone(), &data) {
                    return Some(Arc::new(font));
                }
            }
        }

        None
    }

    /// Get system font paths for a generic family.
    fn get_system_font_paths(&self, family: &str, weight: u16, style: &FontStyle) -> Vec<String> {
        let mut paths = Vec::new();

        // Linux paths
        match family {
            "sans-serif" => {
                paths.push("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".to_string());
                paths.push("/usr/share/fonts/TTF/DejaVuSans.ttf".to_string());
                paths.push("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf".to_string());
            }
            "serif" => {
                paths.push("/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf".to_string());
                paths.push("/usr/share/fonts/TTF/DejaVuSerif.ttf".to_string());
                paths.push("/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf".to_string());
            }
            "monospace" => {
                paths.push("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf".to_string());
                paths.push("/usr/share/fonts/TTF/DejaVuSansMono.ttf".to_string());
                paths.push("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf".to_string());
            }
            _ => {}
        }

        // macOS paths
        #[cfg(target_os = "macos")]
        {
            match family {
                "sans-serif" => {
                    paths.push("/System/Library/Fonts/Helvetica.ttc".to_string());
                    paths.push("/Library/Fonts/Arial.ttf".to_string());
                }
                "serif" => {
                    paths.push("/System/Library/Fonts/Times.ttc".to_string());
                    paths.push("/Library/Fonts/Georgia.ttf".to_string());
                }
                "monospace" => {
                    paths.push("/System/Library/Fonts/Menlo.ttc".to_string());
                    paths.push("/System/Library/Fonts/Courier.ttc".to_string());
                }
                _ => {}
            }
        }

        // Windows paths
        #[cfg(target_os = "windows")]
        {
            match family {
                "sans-serif" => {
                    paths.push("C:\\Windows\\Fonts\\arial.ttf".to_string());
                    paths.push("C:\\Windows\\Fonts\\segoeui.ttf".to_string());
                }
                "serif" => {
                    paths.push("C:\\Windows\\Fonts\\times.ttf".to_string());
                    paths.push("C:\\Windows\\Fonts\\georgia.ttf".to_string());
                }
                "monospace" => {
                    paths.push("C:\\Windows\\Fonts\\consola.ttf".to_string());
                    paths.push("C:\\Windows\\Fonts\\cour.ttf".to_string());
                }
                _ => {}
            }
        }

        paths
    }

    /// Add a font from data.
    pub fn add_font(&self, key: FontKey, data: &[u8]) -> bool {
        if let Some(font) = LoadedFont::new(key.clone(), data) {
            let mut fonts = self.fonts.write();
            fonts.insert(key, Arc::new(font));
            true
        } else {
            false
        }
    }

    /// Add a font from a file.
    pub fn add_font_file(&self, key: FontKey, path: &str) -> bool {
        if let Ok(data) = std::fs::read(path) {
            self.add_font(key, &data)
        } else {
            false
        }
    }

    /// Clear the font cache.
    pub fn clear(&self) {
        let mut fonts = self.fonts.write();
        fonts.clear();
    }

    /// Get number of cached fonts.
    pub fn len(&self) -> usize {
        self.fonts.read().len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.fonts.read().is_empty()
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Text layout helper.
pub struct TextLayout {
    /// Font cache reference.
    font_cache: Arc<FontCache>,
}

impl TextLayout {
    pub fn new(font_cache: Arc<FontCache>) -> Self {
        Self { font_cache }
    }

    /// Measure text width.
    pub fn measure_width(&self, text: &str, font_key: &FontKey, size: f32) -> f32 {
        let font = match self.font_cache.get_font(font_key) {
            Some(f) => f,
            None => return text.len() as f32 * size * 0.5,
        };

        text.chars()
            .map(|c| font.metrics(c as u32, size).advance_width)
            .sum()
    }

    /// Measure text height.
    pub fn measure_height(&self, _text: &str, _font_key: &FontKey, size: f32) -> f32 {
        size * 1.2 // Approximate line height
    }

    /// Get line metrics.
    pub fn line_metrics(&self, font_key: &FontKey, size: f32) -> LineMetrics {
        // Approximate metrics
        LineMetrics {
            ascent: size * 0.8,
            descent: size * 0.2,
            line_gap: size * 0.1,
            line_height: size * 1.2,
        }
    }
}

/// Line metrics.
#[derive(Clone, Copy, Debug)]
pub struct LineMetrics {
    /// Distance from baseline to top of line.
    pub ascent: f32,
    /// Distance from baseline to bottom of line.
    pub descent: f32,
    /// Gap between lines.
    pub line_gap: f32,
    /// Total line height.
    pub line_height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_cache_creation() {
        let cache = FontCache::new();
        // May or may not have default font depending on system
    }

    #[test]
    fn test_font_key() {
        let key = FontKey {
            family: "Arial".to_string(),
            weight: 400,
            style: FontStyle::Normal,
        };

        assert_eq!(key.family, "Arial");
        assert_eq!(key.weight, 400);
    }
}
