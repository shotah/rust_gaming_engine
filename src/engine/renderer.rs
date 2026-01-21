//! Rendering module.
//!
//! Handles wgpu initialization and rendering operations.

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;
use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages,
};
use winit::{dpi::PhysicalSize, window::Window};

/// Renderer configuration options.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Preferred present mode (VSync, Immediate, etc.).
    pub present_mode: PresentMode,
    /// The clear color for the screen (RGBA).
    pub clear_color: wgpu::Color,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            present_mode: PresentMode::AutoVsync,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }
}

/// The main renderer handling all GPU operations.
pub struct Renderer {
    /// The wgpu surface for presenting frames.
    surface: Surface<'static>,
    /// The wgpu device for GPU operations.
    device: Device,
    /// The command queue for submitting work.
    queue: Queue,
    /// The surface configuration.
    surface_config: SurfaceConfiguration,
    /// The preferred texture format.
    surface_format: TextureFormat,
    /// Current window size.
    size: PhysicalSize<u32>,
    /// Renderer configuration.
    config: RendererConfig,
}

impl Renderer {
    /// Creates a new renderer for the given window.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU initialization fails.
    pub async fn new(window: Arc<Window>, config: RendererConfig) -> Result<Self> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window)
            .context("Failed to create surface")?;

        // Request adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find a suitable GPU adapter")?;

        info!("Using GPU: {}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Voxel Forge Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .context("Failed to create device")?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: config.present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        info!(
            "Renderer initialized: {}x{}, format: {:?}",
            size.width, size.height, surface_format
        );

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            surface_format,
            size,
            config,
        })
    }

    /// Resizes the renderer to match a new window size.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            info!("Renderer resized to {}x{}", new_size.width, new_size.height);
        }
    }

    /// Returns the current size.
    #[must_use]
    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Returns a reference to the device.
    #[must_use]
    pub const fn device(&self) -> &Device {
        &self.device
    }

    /// Returns a reference to the queue.
    #[must_use]
    pub const fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Returns the surface format.
    #[must_use]
    pub const fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    /// Returns a reference to the surface.
    #[must_use]
    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    /// Renders a frame with the current clear color.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    pub fn render(&mut self) -> Result<()> {
        let output = self
            .surface
            .get_current_texture()
            .context("Failed to get surface texture")?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.config.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Sets the clear color.
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.config.clear_color = color;
    }
}
