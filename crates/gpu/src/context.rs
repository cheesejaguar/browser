//! GPU context and device management.

use parking_lot::RwLock;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;
use thiserror::Error;
use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, TextureFormat,
};

/// Errors that can occur during GPU operations.
#[derive(Error, Debug)]
pub enum GpuError {
    #[error("Failed to create GPU instance")]
    InstanceCreation,
    #[error("No suitable GPU adapter found")]
    NoAdapter,
    #[error("Failed to request device: {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),
    #[error("Failed to create surface")]
    SurfaceCreation,
}

/// GPU context holding all wgpu resources.
pub struct GpuContext {
    /// wgpu instance.
    pub instance: Instance,
    /// GPU adapter.
    pub adapter: Adapter,
    /// GPU device.
    pub device: Device,
    /// Command queue.
    pub queue: Queue,
    /// Surface for rendering (if attached to a window).
    surface: RwLock<Option<Surface<'static>>>,
    /// Surface configuration.
    surface_config: RwLock<Option<SurfaceConfiguration>>,
    /// Current surface size.
    surface_size: RwLock<(u32, u32)>,
}

impl GpuContext {
    /// Create a new GPU context.
    pub async fn new() -> Result<Self, GpuError> {
        // Create instance
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Browser GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: RwLock::new(None),
            surface_config: RwLock::new(None),
            surface_size: RwLock::new((0, 0)),
        })
    }

    /// Create context with a window surface.
    pub async fn with_window<W>(window: Arc<W>) -> Result<Self, GpuError>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // Create surface
        let surface = instance
            .create_surface(window)
            .map_err(|_| GpuError::SurfaceCreation)?;

        // Request adapter compatible with the surface
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        // Request device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Browser GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: RwLock::new(Some(surface)),
            surface_config: RwLock::new(None),
            surface_size: RwLock::new((0, 0)),
        })
    }

    /// Configure the surface for the given size.
    pub fn configure_surface(&self, width: u32, height: u32) {
        let surface = self.surface.read();
        let surface = match surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let caps = surface.get_capabilities(&self.adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config);

        *self.surface_config.write() = Some(config);
        *self.surface_size.write() = (width, height);
    }

    /// Get the surface format.
    pub fn surface_format(&self) -> Option<TextureFormat> {
        self.surface_config.read().as_ref().map(|c| c.format)
    }

    /// Get the current surface texture for rendering.
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, GpuError> {
        let surface = self.surface.read();
        let surface = surface.as_ref().ok_or(GpuError::SurfaceCreation)?;
        surface.get_current_texture().map_err(GpuError::Surface)
    }

    /// Get surface size.
    pub fn surface_size(&self) -> (u32, u32) {
        *self.surface_size.read()
    }

    /// Create a command encoder.
    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Command Encoder"),
        })
    }

    /// Submit commands to the queue.
    pub fn submit(&self, commands: impl IntoIterator<Item = wgpu::CommandBuffer>) {
        self.queue.submit(commands);
    }

    /// Get device limits.
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }

    /// Get device features.
    pub fn features(&self) -> wgpu::Features {
        self.device.features()
    }

    /// Get adapter info.
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Poll the device for completed operations.
    pub fn poll(&self, maintain: wgpu::Maintain) -> bool {
        self.device.poll(maintain).is_queue_empty()
    }
}

/// Builder for GPU context configuration.
pub struct GpuContextBuilder {
    backends: wgpu::Backends,
    power_preference: wgpu::PowerPreference,
    features: wgpu::Features,
    limits: Option<wgpu::Limits>,
}

impl GpuContextBuilder {
    pub fn new() -> Self {
        Self {
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            features: wgpu::Features::empty(),
            limits: None,
        }
    }

    /// Set the backends to use.
    pub fn backends(mut self, backends: wgpu::Backends) -> Self {
        self.backends = backends;
        self
    }

    /// Set power preference.
    pub fn power_preference(mut self, preference: wgpu::PowerPreference) -> Self {
        self.power_preference = preference;
        self
    }

    /// Set required features.
    pub fn features(mut self, features: wgpu::Features) -> Self {
        self.features = features;
        self
    }

    /// Set device limits.
    pub fn limits(mut self, limits: wgpu::Limits) -> Self {
        self.limits = Some(limits);
        self
    }

    /// Build the GPU context.
    pub async fn build(self) -> Result<GpuContext, GpuError> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: self.backends,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        let limits = self.limits.unwrap_or_else(|| wgpu::Limits::default());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Browser GPU Device"),
                    required_features: self.features,
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        Ok(GpuContext {
            instance,
            adapter,
            device,
            queue,
            surface: RwLock::new(None),
            surface_config: RwLock::new(None),
            surface_size: RwLock::new((0, 0)),
        })
    }
}

impl Default for GpuContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: GPU tests require actual GPU hardware
    // These are basic structure tests

    #[test]
    fn test_builder_creation() {
        let builder = GpuContextBuilder::new()
            .backends(wgpu::Backends::VULKAN)
            .power_preference(wgpu::PowerPreference::LowPower);

        assert_eq!(builder.backends, wgpu::Backends::VULKAN);
    }
}
