//! GPU buffer management.

use crate::context::GpuContext;
use bytemuck::Pod;
use std::marker::PhantomData;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, util::DeviceExt};

/// Dynamic vertex buffer that grows as needed.
pub struct DynamicVertexBuffer<T: Pod> {
    buffer: Option<Buffer>,
    capacity: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T: Pod> DynamicVertexBuffer<T> {
    pub fn new() -> Self {
        Self {
            buffer: None,
            capacity: 0,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Write data to the buffer, growing if necessary.
    pub fn write(&mut self, context: &GpuContext, data: &[T]) {
        let required_capacity = data.len();

        if self.capacity < required_capacity {
            // Grow buffer
            let new_capacity = (required_capacity * 2).max(1024);
            let size = (new_capacity * std::mem::size_of::<T>()) as u64;

            self.buffer = Some(context.device.create_buffer(&BufferDescriptor {
                label: Some("Dynamic Vertex Buffer"),
                size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

            self.capacity = new_capacity;
        }

        if let Some(buffer) = &self.buffer {
            context.queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
        }

        self.len = data.len();
    }

    /// Get the buffer.
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// Get number of vertices.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Pod> Default for DynamicVertexBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic index buffer.
pub struct DynamicIndexBuffer {
    buffer: Option<Buffer>,
    capacity: usize,
    len: usize,
}

impl DynamicIndexBuffer {
    pub fn new() -> Self {
        Self {
            buffer: None,
            capacity: 0,
            len: 0,
        }
    }

    /// Write data to the buffer.
    pub fn write(&mut self, context: &GpuContext, data: &[u32]) {
        let required_capacity = data.len();

        if self.capacity < required_capacity {
            let new_capacity = (required_capacity * 2).max(1024);
            let size = (new_capacity * std::mem::size_of::<u32>()) as u64;

            self.buffer = Some(context.device.create_buffer(&BufferDescriptor {
                label: Some("Dynamic Index Buffer"),
                size,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

            self.capacity = new_capacity;
        }

        if let Some(buffer) = &self.buffer {
            context.queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
        }

        self.len = data.len();
    }

    /// Get the buffer.
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// Get number of indices.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for DynamicIndexBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Uniform buffer.
pub struct UniformBuffer<T: Pod> {
    buffer: Buffer,
    _marker: PhantomData<T>,
}

impl<T: Pod> UniformBuffer<T> {
    /// Create a new uniform buffer.
    pub fn new(context: &GpuContext, data: &T) -> Self {
        let buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            _marker: PhantomData,
        }
    }

    /// Update the buffer contents.
    pub fn update(&self, context: &GpuContext, data: &T) {
        context.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }

    /// Get the buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

/// Staging buffer for CPU-GPU transfers.
pub struct StagingBuffer {
    buffer: Buffer,
    size: u64,
}

impl StagingBuffer {
    /// Create a new staging buffer.
    pub fn new(context: &GpuContext, size: u64) -> Self {
        let buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer, size }
    }

    /// Get the buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get buffer size.
    pub fn size(&self) -> u64 {
        self.size
    }
}

/// Instance buffer for instanced rendering.
pub struct InstanceBuffer<T: Pod> {
    buffer: Option<Buffer>,
    capacity: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T: Pod> InstanceBuffer<T> {
    pub fn new() -> Self {
        Self {
            buffer: None,
            capacity: 0,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Write instance data.
    pub fn write(&mut self, context: &GpuContext, data: &[T]) {
        let required_capacity = data.len();

        if self.capacity < required_capacity {
            let new_capacity = (required_capacity * 2).max(256);
            let size = (new_capacity * std::mem::size_of::<T>()) as u64;

            self.buffer = Some(context.device.create_buffer(&BufferDescriptor {
                label: Some("Instance Buffer"),
                size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

            self.capacity = new_capacity;
        }

        if let Some(buffer) = &self.buffer {
            context.queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
        }

        self.len = data.len();
    }

    /// Get the buffer.
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// Get instance count.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Pod> Default for InstanceBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Quad batch for efficient 2D rendering.
pub struct QuadBatch {
    vertices: Vec<QuadVertex>,
    indices: Vec<u32>,
    vertex_buffer: DynamicVertexBuffer<QuadVertex>,
    index_buffer: DynamicIndexBuffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: [f32; 4],
}

impl QuadBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: DynamicVertexBuffer::new(),
            index_buffer: DynamicIndexBuffer::new(),
        }
    }

    /// Begin a new batch.
    pub fn begin(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Add a quad to the batch.
    pub fn push_quad(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        u0: f32,
        v0: f32,
        u1: f32,
        v1: f32,
        color: [f32; 4],
    ) {
        let base_index = self.vertices.len() as u32;

        // Add vertices
        self.vertices.push(QuadVertex {
            position: [x, y],
            tex_coord: [u0, v0],
            color,
        });
        self.vertices.push(QuadVertex {
            position: [x + width, y],
            tex_coord: [u1, v0],
            color,
        });
        self.vertices.push(QuadVertex {
            position: [x + width, y + height],
            tex_coord: [u1, v1],
            color,
        });
        self.vertices.push(QuadVertex {
            position: [x, y + height],
            tex_coord: [u0, v1],
            color,
        });

        // Add indices (two triangles)
        self.indices.extend([
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    /// Add a colored quad (no texture).
    pub fn push_colored_quad(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        self.push_quad(x, y, width, height, 0.0, 0.0, 1.0, 1.0, color);
    }

    /// Finish the batch and upload to GPU.
    pub fn end(&mut self, context: &GpuContext) {
        self.vertex_buffer.write(context, &self.vertices);
        self.index_buffer.write(context, &self.indices);
    }

    /// Get vertex buffer.
    pub fn vertex_buffer(&self) -> Option<&Buffer> {
        self.vertex_buffer.buffer()
    }

    /// Get index buffer.
    pub fn index_buffer(&self) -> Option<&Buffer> {
        self.index_buffer.buffer()
    }

    /// Get number of indices.
    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    /// Check if batch is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl Default for QuadBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_batch() {
        let mut batch = QuadBatch::new();
        batch.begin();
        batch.push_colored_quad(0.0, 0.0, 100.0, 100.0, [1.0, 0.0, 0.0, 1.0]);
        batch.push_colored_quad(100.0, 0.0, 100.0, 100.0, [0.0, 1.0, 0.0, 1.0]);

        assert_eq!(batch.vertices.len(), 8);
        assert_eq!(batch.indices.len(), 12);
    }
}
