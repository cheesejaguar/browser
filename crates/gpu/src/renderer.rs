//! GPU renderer implementation.

use crate::buffer::{DynamicIndexBuffer, DynamicVertexBuffer, QuadBatch, UniformBuffer};
use crate::context::GpuContext;
use crate::pipeline::{PipelineCache, PipelineType, TexturedVertex, Uniforms, Vertex};
use crate::texture::{GlyphAtlas, GpuTexture, SamplerCache};
use common::color::Color;
use common::geometry::{Point, Rect};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{BindGroup, FilterMode, AddressMode};

/// GPU renderer for hardware-accelerated 2D rendering.
pub struct GpuRenderer {
    context: Arc<GpuContext>,
    pipeline_cache: PipelineCache,
    sampler_cache: SamplerCache,
    uniform_buffer: Option<UniformBuffer<Uniforms>>,
    quad_batch: QuadBatch,
    glyph_atlas: Option<GlyphAtlas>,
    texture_cache: HashMap<u64, GpuTexture>,
    current_size: (u32, u32),
}

impl GpuRenderer {
    /// Create a new GPU renderer.
    pub fn new(context: Arc<GpuContext>) -> Self {
        let pipeline_cache = PipelineCache::new(context.clone());
        let sampler_cache = SamplerCache::new(context.clone());

        Self {
            context,
            pipeline_cache,
            sampler_cache,
            uniform_buffer: None,
            quad_batch: QuadBatch::new(),
            glyph_atlas: None,
            texture_cache: HashMap::new(),
            current_size: (0, 0),
        }
    }

    /// Initialize the renderer with the given surface format.
    pub fn initialize(&mut self, width: u32, height: u32) {
        let format = self.context.surface_format().unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        self.pipeline_cache.initialize(format);

        // Create uniform buffer
        let uniforms = Uniforms::orthographic(width as f32, height as f32);
        self.uniform_buffer = Some(UniformBuffer::new(&self.context, &uniforms));

        // Create glyph atlas
        self.glyph_atlas = Some(GlyphAtlas::new(&self.context, 2048));

        self.current_size = (width, height);
    }

    /// Resize the renderer.
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(uniform_buffer) = &self.uniform_buffer {
            let uniforms = Uniforms::orthographic(width as f32, height as f32);
            uniform_buffer.update(&self.context, &uniforms);
        }
        self.current_size = (width, height);
    }

    /// Begin a new frame.
    pub fn begin_frame(&mut self) -> RenderFrame {
        RenderFrame::new(self)
    }

    /// Get or create a texture from RGBA data.
    pub fn get_or_create_texture(&mut self, id: u64, width: u32, height: u32, data: &[u8]) -> u64 {
        if !self.texture_cache.contains_key(&id) {
            let texture = GpuTexture::from_rgba(&self.context, width, height, data);
            self.texture_cache.insert(id, texture);
        }
        id
    }

    /// Remove a texture.
    pub fn remove_texture(&mut self, id: u64) {
        self.texture_cache.remove(&id);
    }

    /// Clear texture cache.
    pub fn clear_texture_cache(&mut self) {
        self.texture_cache.clear();
    }

    /// Get the GPU context.
    pub fn context(&self) -> &Arc<GpuContext> {
        &self.context
    }
}

/// A single render frame.
pub struct RenderFrame<'a> {
    renderer: &'a mut GpuRenderer,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    textured_vertices: Vec<TexturedVertex>,
    textured_indices: Vec<u32>,
    commands: Vec<RenderCommand>,
}

/// Internal render command.
enum RenderCommand {
    DrawSolidColor {
        vertex_start: u32,
        index_start: u32,
        index_count: u32,
    },
    DrawTextured {
        texture_id: u64,
        vertex_start: u32,
        index_start: u32,
        index_count: u32,
    },
    SetClip {
        rect: Rect,
    },
    ClearClip,
}

impl<'a> RenderFrame<'a> {
    fn new(renderer: &'a mut GpuRenderer) -> Self {
        Self {
            renderer,
            vertices: Vec::new(),
            indices: Vec::new(),
            textured_vertices: Vec::new(),
            textured_indices: Vec::new(),
            commands: Vec::new(),
        }
    }

    /// Draw a solid color rectangle.
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let base_index = self.vertices.len() as u32;
        let index_start = self.indices.len() as u32;

        let color_f = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];

        self.vertices.push(Vertex {
            position: [rect.x, rect.y],
            color: color_f,
        });
        self.vertices.push(Vertex {
            position: [rect.x + rect.width, rect.y],
            color: color_f,
        });
        self.vertices.push(Vertex {
            position: [rect.x + rect.width, rect.y + rect.height],
            color: color_f,
        });
        self.vertices.push(Vertex {
            position: [rect.x, rect.y + rect.height],
            color: color_f,
        });

        self.indices.extend([
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);

        self.commands.push(RenderCommand::DrawSolidColor {
            vertex_start: base_index,
            index_start,
            index_count: 6,
        });
    }

    /// Draw a textured rectangle.
    pub fn draw_texture(
        &mut self,
        texture_id: u64,
        rect: Rect,
        src_rect: Option<Rect>,
        color: Color,
    ) {
        let base_index = self.textured_vertices.len() as u32;
        let index_start = self.textured_indices.len() as u32;

        let (u0, v0, u1, v1) = if let Some(src) = src_rect {
            (src.x, src.y, src.x + src.width, src.y + src.height)
        } else {
            (0.0, 0.0, 1.0, 1.0)
        };

        let color_f = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];

        self.textured_vertices.push(TexturedVertex {
            position: [rect.x, rect.y],
            tex_coord: [u0, v0],
            color: color_f,
        });
        self.textured_vertices.push(TexturedVertex {
            position: [rect.x + rect.width, rect.y],
            tex_coord: [u1, v0],
            color: color_f,
        });
        self.textured_vertices.push(TexturedVertex {
            position: [rect.x + rect.width, rect.y + rect.height],
            tex_coord: [u1, v1],
            color: color_f,
        });
        self.textured_vertices.push(TexturedVertex {
            position: [rect.x, rect.y + rect.height],
            tex_coord: [u0, v1],
            color: color_f,
        });

        self.textured_indices.extend([
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);

        self.commands.push(RenderCommand::DrawTextured {
            texture_id,
            vertex_start: base_index,
            index_start,
            index_count: 6,
        });
    }

    /// Set clip rectangle.
    pub fn set_clip(&mut self, rect: Rect) {
        self.commands.push(RenderCommand::SetClip { rect });
    }

    /// Clear clip rectangle.
    pub fn clear_clip(&mut self) {
        self.commands.push(RenderCommand::ClearClip);
    }

    /// Submit the frame for rendering.
    pub fn submit(self, clear_color: Color) {
        let context = &self.renderer.context;

        // Get surface texture
        let output = match context.get_current_texture() {
            Ok(o) => o,
            Err(_) => return,
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = context.create_command_encoder();

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64 / 255.0,
                            g: clear_color.g as f64 / 255.0,
                            b: clear_color.b as f64 / 255.0,
                            a: clear_color.a as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Execute render commands
            // This is a simplified version - full implementation would batch by pipeline
            // and execute with proper bind groups

            let (width, height) = self.renderer.current_size;
            render_pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        }

        // Submit commands
        context.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/// Render statistics.
#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    /// Number of draw calls.
    pub draw_calls: u32,
    /// Number of vertices.
    pub vertices: u32,
    /// Number of triangles.
    pub triangles: u32,
    /// Frame time in milliseconds.
    pub frame_time_ms: f32,
    /// GPU memory used.
    pub gpu_memory: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // GPU tests require actual hardware
    // These are basic structure tests

    #[test]
    fn test_render_stats_default() {
        let stats = RenderStats::default();
        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.vertices, 0);
    }
}
