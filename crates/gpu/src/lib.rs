//! GPU abstraction layer using wgpu.
//!
//! This crate provides a hardware-accelerated rendering backend.

pub mod context;
pub mod pipeline;
pub mod texture;
pub mod buffer;
pub mod renderer;
pub mod shaders;

pub use context::GpuContext;
pub use renderer::GpuRenderer;
pub use texture::GpuTexture;
