//! GPU texture management.

use crate::context::GpuContext;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{
    AddressMode, Extent3d, FilterMode, Sampler, SamplerDescriptor, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

/// GPU texture wrapper.
pub struct GpuTexture {
    /// The wgpu texture.
    pub texture: Texture,
    /// Texture view.
    pub view: TextureView,
    /// Texture dimensions.
    pub width: u32,
    pub height: u32,
    /// Texture format.
    pub format: TextureFormat,
}

impl GpuTexture {
    /// Create a new texture.
    pub fn new(context: &GpuContext, width: u32, height: u32, format: TextureFormat) -> Self {
        let texture = context.device.create_texture(&TextureDescriptor {
            label: Some("GPU Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
            format,
        }
    }

    /// Create a texture from RGBA data.
    pub fn from_rgba(context: &GpuContext, width: u32, height: u32, data: &[u8]) -> Self {
        let texture = Self::new(context, width, height, TextureFormat::Rgba8UnormSrgb);

        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    /// Create a render target texture.
    pub fn render_target(context: &GpuContext, width: u32, height: u32, format: TextureFormat) -> Self {
        let texture = context.device.create_texture(&TextureDescriptor {
            label: Some("Render Target"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
            format,
        }
    }

    /// Update texture data.
    pub fn update(&self, context: &GpuContext, data: &[u8]) {
        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Update a region of the texture.
    pub fn update_region(
        &self,
        context: &GpuContext,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// Texture atlas for efficient texture management.
pub struct TextureAtlas {
    /// The atlas texture.
    texture: GpuTexture,
    /// Allocated regions.
    regions: Vec<AtlasRegion>,
    /// Current row y position.
    current_y: u32,
    /// Current row height.
    current_row_height: u32,
    /// Current x position in row.
    current_x: u32,
    /// Padding between regions.
    padding: u32,
}

/// A region within a texture atlas.
#[derive(Clone, Debug)]
pub struct AtlasRegion {
    pub id: u64,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl AtlasRegion {
    /// Get UV coordinates for this region.
    pub fn uv(&self, atlas_width: u32, atlas_height: u32) -> (f32, f32, f32, f32) {
        let u0 = self.x as f32 / atlas_width as f32;
        let v0 = self.y as f32 / atlas_height as f32;
        let u1 = (self.x + self.width) as f32 / atlas_width as f32;
        let v1 = (self.y + self.height) as f32 / atlas_height as f32;
        (u0, v0, u1, v1)
    }
}

impl TextureAtlas {
    /// Create a new texture atlas.
    pub fn new(context: &GpuContext, width: u32, height: u32) -> Self {
        let texture = GpuTexture::new(context, width, height, TextureFormat::Rgba8UnormSrgb);

        Self {
            texture,
            regions: Vec::new(),
            current_y: 0,
            current_row_height: 0,
            current_x: 0,
            padding: 1,
        }
    }

    /// Get the atlas texture.
    pub fn texture(&self) -> &GpuTexture {
        &self.texture
    }

    /// Allocate a region in the atlas.
    pub fn allocate(
        &mut self,
        context: &GpuContext,
        id: u64,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Option<AtlasRegion> {
        let padded_width = width + self.padding * 2;
        let padded_height = height + self.padding * 2;

        // Check if fits in current row
        if self.current_x + padded_width > self.texture.width {
            // Move to next row
            self.current_x = 0;
            self.current_y += self.current_row_height;
            self.current_row_height = 0;
        }

        // Check if fits in atlas
        if self.current_y + padded_height > self.texture.height {
            return None;
        }

        // Allocate region
        let x = self.current_x + self.padding;
        let y = self.current_y + self.padding;

        let region = AtlasRegion {
            id,
            x,
            y,
            width,
            height,
        };

        // Upload data
        self.texture.update_region(context, x, y, width, height, data);

        // Update position
        self.current_x += padded_width;
        self.current_row_height = self.current_row_height.max(padded_height);

        self.regions.push(region.clone());

        Some(region)
    }

    /// Find a region by ID.
    pub fn find(&self, id: u64) -> Option<&AtlasRegion> {
        self.regions.iter().find(|r| r.id == id)
    }

    /// Clear the atlas.
    pub fn clear(&mut self) {
        self.regions.clear();
        self.current_x = 0;
        self.current_y = 0;
        self.current_row_height = 0;
    }
}

/// Sampler cache.
pub struct SamplerCache {
    context: Arc<GpuContext>,
    samplers: HashMap<SamplerKey, Sampler>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SamplerKey {
    mag_filter: FilterMode,
    min_filter: FilterMode,
    address_mode: AddressMode,
}

impl SamplerCache {
    pub fn new(context: Arc<GpuContext>) -> Self {
        Self {
            context,
            samplers: HashMap::new(),
        }
    }

    /// Get a sampler with the given parameters.
    pub fn get(
        &mut self,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        address_mode: AddressMode,
    ) -> &Sampler {
        let key = SamplerKey {
            mag_filter,
            min_filter,
            address_mode,
        };

        self.samplers.entry(key).or_insert_with(|| {
            self.context.device.create_sampler(&SamplerDescriptor {
                label: Some("Cached Sampler"),
                address_mode_u: address_mode,
                address_mode_v: address_mode,
                address_mode_w: address_mode,
                mag_filter,
                min_filter,
                mipmap_filter: FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            })
        })
    }

    /// Get default linear sampler.
    pub fn linear(&mut self) -> &Sampler {
        self.get(FilterMode::Linear, FilterMode::Linear, AddressMode::ClampToEdge)
    }

    /// Get nearest-neighbor sampler.
    pub fn nearest(&mut self) -> &Sampler {
        self.get(FilterMode::Nearest, FilterMode::Nearest, AddressMode::ClampToEdge)
    }
}

/// Glyph atlas for text rendering.
pub struct GlyphAtlas {
    /// The atlas texture.
    atlas: TextureAtlas,
    /// Glyph mappings.
    glyphs: HashMap<GlyphKey, AtlasRegion>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub glyph_id: u32,
    pub size: u32, // Quantized size (size * 10)
}

impl GlyphAtlas {
    pub fn new(context: &GpuContext, size: u32) -> Self {
        Self {
            atlas: TextureAtlas::new(context, size, size),
            glyphs: HashMap::new(),
        }
    }

    /// Get or create a glyph region.
    pub fn get_or_create<F>(
        &mut self,
        context: &GpuContext,
        key: GlyphKey,
        rasterize: F,
    ) -> Option<&AtlasRegion>
    where
        F: FnOnce() -> (u32, u32, Vec<u8>),
    {
        if self.glyphs.contains_key(&key) {
            return self.glyphs.get(&key);
        }

        let (width, height, data) = rasterize();

        // Convert grayscale to RGBA
        let rgba: Vec<u8> = data
            .iter()
            .flat_map(|&a| [255, 255, 255, a])
            .collect();

        let id = (key.glyph_id as u64) << 32 | key.size as u64;

        if let Some(region) = self.atlas.allocate(context, id, width, height, &rgba) {
            self.glyphs.insert(key, region);
        }

        self.glyphs.get(&key)
    }

    /// Get the atlas texture.
    pub fn texture(&self) -> &GpuTexture {
        self.atlas.texture()
    }

    /// Clear the atlas.
    pub fn clear(&mut self) {
        self.atlas.clear();
        self.glyphs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_region_uv() {
        let region = AtlasRegion {
            id: 1,
            x: 100,
            y: 100,
            width: 50,
            height: 50,
        };

        let (u0, v0, u1, v1) = region.uv(1000, 1000);
        assert!((u0 - 0.1).abs() < 0.001);
        assert!((v0 - 0.1).abs() < 0.001);
        assert!((u1 - 0.15).abs() < 0.001);
        assert!((v1 - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_glyph_key() {
        let key1 = GlyphKey { glyph_id: 65, size: 160 };
        let key2 = GlyphKey { glyph_id: 65, size: 160 };
        assert_eq!(key1, key2);
    }
}
