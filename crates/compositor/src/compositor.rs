//! Main compositor implementation.

use crate::layer::{FilterEffect, Layer, LayerId, LayerTree};
use crate::scene::Scene;
use common::color::Color;
use common::geometry::{Rect, Transform};
use gpu::{GpuContext, GpuRenderer};
use parking_lot::RwLock;
use render::display_list::{BlendMode, DisplayItem, DisplayList};
use std::collections::HashMap;
use std::sync::Arc;

/// The compositor handles layer compositing and rendering.
pub struct Compositor {
    /// GPU context.
    context: Arc<GpuContext>,
    /// GPU renderer.
    renderer: GpuRenderer,
    /// Layer textures.
    layer_textures: HashMap<LayerId, LayerTexture>,
    /// Compositor settings.
    settings: CompositorSettings,
    /// Frame statistics.
    stats: CompositorStats,
}

/// Texture for a composited layer.
struct LayerTexture {
    texture_id: u64,
    width: u32,
    height: u32,
    dirty: bool,
}

/// Compositor settings.
#[derive(Clone, Debug)]
pub struct CompositorSettings {
    /// Enable GPU compositing.
    pub gpu_compositing: bool,
    /// Enable layer caching.
    pub layer_caching: bool,
    /// Maximum cached layers.
    pub max_cached_layers: usize,
    /// Enable damage tracking.
    pub damage_tracking: bool,
    /// Texture atlas size.
    pub atlas_size: u32,
}

impl Default for CompositorSettings {
    fn default() -> Self {
        Self {
            gpu_compositing: true,
            layer_caching: true,
            max_cached_layers: 100,
            damage_tracking: true,
            atlas_size: 4096,
        }
    }
}

/// Compositor statistics.
#[derive(Clone, Debug, Default)]
pub struct CompositorStats {
    /// Number of layers composited.
    pub layers_composited: u32,
    /// Number of cached layers used.
    pub cached_layers_used: u32,
    /// Total texture memory used.
    pub texture_memory: u64,
    /// Composition time in milliseconds.
    pub composition_time_ms: f32,
}

impl Compositor {
    /// Create a new compositor.
    pub fn new(context: Arc<GpuContext>) -> Self {
        let renderer = GpuRenderer::new(context.clone());

        Self {
            context,
            renderer,
            layer_textures: HashMap::new(),
            settings: CompositorSettings::default(),
            stats: CompositorStats::default(),
        }
    }

    /// Initialize the compositor.
    pub fn initialize(&mut self, width: u32, height: u32) {
        self.renderer.initialize(width, height);
    }

    /// Resize the compositor.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        // Invalidate all layer textures
        for (_, texture) in self.layer_textures.iter_mut() {
            texture.dirty = true;
        }
    }

    /// Composite and render a scene.
    pub fn composite(&mut self, scene: &Scene) {
        let start = std::time::Instant::now();

        self.stats = CompositorStats::default();

        // Get layers in paint order
        let paint_order = scene.layer_tree().paint_order();

        // Composite each layer
        for layer_id in &paint_order {
            if let Some(layer) = scene.get_layer(*layer_id) {
                self.composite_layer(scene, layer);
            }
        }

        // Final composition to screen
        self.composite_to_screen(scene, &paint_order);

        self.stats.composition_time_ms = start.elapsed().as_secs_f32() * 1000.0;
    }

    /// Composite a single layer.
    fn composite_layer(&mut self, scene: &Scene, layer: &Layer) {
        if !layer.visible {
            return;
        }

        self.stats.layers_composited += 1;

        // Check if layer needs its own texture
        if layer.requires_compositing() {
            self.ensure_layer_texture(layer);

            // Check if we can use cached texture
            if self.settings.layer_caching {
                if let Some(texture) = self.layer_textures.get(&layer.id) {
                    if !texture.dirty && !layer.dirty {
                        self.stats.cached_layers_used += 1;
                        return;
                    }
                }
            }

            // Render layer to texture
            self.render_layer_to_texture(scene, layer);
        }
    }

    /// Ensure a layer has a texture.
    fn ensure_layer_texture(&mut self, layer: &Layer) {
        let width = layer.bounds.width.ceil() as u32;
        let height = layer.bounds.height.ceil() as u32;

        if let Some(texture) = self.layer_textures.get_mut(&layer.id) {
            if texture.width != width || texture.height != height {
                texture.width = width;
                texture.height = height;
                texture.dirty = true;
            }
        } else {
            let texture_id = layer.id.0.as_ffi() as u64;
            self.layer_textures.insert(
                layer.id,
                LayerTexture {
                    texture_id,
                    width,
                    height,
                    dirty: true,
                },
            );
        }
    }

    /// Render a layer to its texture.
    fn render_layer_to_texture(&mut self, scene: &Scene, layer: &Layer) {
        // In a real implementation, this would:
        // 1. Create a render target texture
        // 2. Render the layer's display list to it
        // 3. Apply any filters

        if let Some(texture) = self.layer_textures.get_mut(&layer.id) {
            texture.dirty = false;
            self.stats.texture_memory += (texture.width * texture.height * 4) as u64;
        }
    }

    /// Composite all layers to the screen.
    fn composite_to_screen(&mut self, scene: &Scene, paint_order: &[LayerId]) {
        let mut frame = self.renderer.begin_frame();

        // Render each layer
        for layer_id in paint_order {
            if let Some(layer) = scene.get_layer(*layer_id) {
                self.render_layer(&mut frame, scene, layer);
            }
        }

        // Submit the frame
        frame.submit(scene.background());
    }

    /// Render a layer to the current frame.
    fn render_layer(
        &self,
        frame: &mut gpu::renderer::RenderFrame,
        scene: &Scene,
        layer: &Layer,
    ) {
        if !layer.visible {
            return;
        }

        // Apply clipping if needed
        if layer.clips_to_bounds {
            frame.set_clip(layer.bounds.clone());
        }

        // If layer has its own texture, composite it
        if layer.requires_compositing() {
            if let Some(texture) = self.layer_textures.get(&layer.id) {
                // Draw the layer texture with opacity and transform
                let color = Color::new(255, 255, 255, (layer.opacity * 255.0) as u8);
                frame.draw_texture(texture.texture_id, layer.bounds.clone(), None, color);
            }
        } else {
            // Render display list directly
            for item in layer.display_list.items() {
                self.render_display_item(frame, item, layer);
            }
        }

        if layer.clips_to_bounds {
            frame.clear_clip();
        }
    }

    /// Render a display item.
    fn render_display_item(
        &self,
        frame: &mut gpu::renderer::RenderFrame,
        item: &DisplayItem,
        layer: &Layer,
    ) {
        use render::display_list::DisplayItemType;

        match &item.item_type {
            DisplayItemType::SolidColor(solid) => {
                let color = solid.color.clone();
                frame.draw_rect(item.bounds.clone(), color);
            }
            DisplayItemType::Image(image) => {
                frame.draw_texture(
                    image.image_key.0,
                    item.bounds.clone(),
                    image.src_rect.clone(),
                    Color::white(),
                );
            }
            _ => {
                // Handle other item types
            }
        }
    }

    /// Get compositor statistics.
    pub fn stats(&self) -> &CompositorStats {
        &self.stats
    }

    /// Get compositor settings.
    pub fn settings(&self) -> &CompositorSettings {
        &self.settings
    }

    /// Set compositor settings.
    pub fn set_settings(&mut self, settings: CompositorSettings) {
        self.settings = settings;
    }

    /// Clear layer texture cache.
    pub fn clear_texture_cache(&mut self) {
        self.layer_textures.clear();
        self.renderer.clear_texture_cache();
    }

    /// Invalidate a layer's texture.
    pub fn invalidate_layer(&mut self, layer_id: LayerId) {
        if let Some(texture) = self.layer_textures.get_mut(&layer_id) {
            texture.dirty = true;
        }
    }
}

/// Damage region for incremental compositing.
#[derive(Clone, Debug)]
pub struct DamageRegion {
    pub rect: Rect,
    pub layer_id: Option<LayerId>,
}

/// Damage tracker for incremental compositing.
pub struct DamageTracker {
    /// Current frame damage.
    current_damage: Vec<DamageRegion>,
    /// Previous frame damage.
    previous_damage: Vec<DamageRegion>,
}

impl DamageTracker {
    pub fn new() -> Self {
        Self {
            current_damage: Vec::new(),
            previous_damage: Vec::new(),
        }
    }

    /// Add a damage region.
    pub fn add_damage(&mut self, rect: Rect, layer_id: Option<LayerId>) {
        self.current_damage.push(DamageRegion { rect, layer_id });
    }

    /// Get the combined damage region.
    pub fn combined_damage(&self) -> Option<Rect> {
        let all_damage: Vec<_> = self
            .current_damage
            .iter()
            .chain(self.previous_damage.iter())
            .collect();

        if all_damage.is_empty() {
            return None;
        }

        let mut combined = all_damage[0].rect.clone();
        for damage in &all_damage[1..] {
            combined = combined.union(&damage.rect);
        }

        Some(combined)
    }

    /// Advance to the next frame.
    pub fn next_frame(&mut self) {
        self.previous_damage = std::mem::take(&mut self.current_damage);
    }

    /// Clear all damage.
    pub fn clear(&mut self) {
        self.current_damage.clear();
        self.previous_damage.clear();
    }
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_settings() {
        let settings = CompositorSettings::default();
        assert!(settings.gpu_compositing);
        assert!(settings.layer_caching);
    }

    #[test]
    fn test_damage_tracker() {
        let mut tracker = DamageTracker::new();

        tracker.add_damage(Rect::new(0.0, 0.0, 100.0, 100.0), None);
        tracker.add_damage(Rect::new(50.0, 50.0, 100.0, 100.0), None);

        let combined = tracker.combined_damage().unwrap();
        assert_eq!(combined.x, 0.0);
        assert_eq!(combined.y, 0.0);
        assert_eq!(combined.width, 150.0);
        assert_eq!(combined.height, 150.0);
    }
}
