//! 2D overlay rendering for HUD elements.
//!
//! Renders simple 2D shapes like crosshairs directly to the screen.

use wgpu::util::DeviceExt;

/// Vertex for 2D overlay rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OverlayVertex {
    /// Position in normalized device coordinates (-1 to 1).
    pub position: [f32; 2],
    /// Color (RGBA).
    pub color: [f32; 4],
}

impl OverlayVertex {
    /// Creates a new overlay vertex.
    #[must_use]
    pub const fn new(x: f32, y: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            color,
        }
    }

    /// Returns the vertex buffer layout.
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Renders 2D overlay elements like crosshairs.
pub struct OverlayRenderer {
    /// The render pipeline.
    pipeline: wgpu::RenderPipeline,
    /// Crosshair vertex buffer.
    crosshair_buffer: wgpu::Buffer,
    /// Number of crosshair vertices.
    crosshair_vertex_count: u32,
}

impl OverlayRenderer {
    /// Creates a new overlay renderer.
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Overlay Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/overlay.wgsl").into()),
        });

        // Pipeline layout (no bind groups needed)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Overlay Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overlay Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[OverlayVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for 2D
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for overlay
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create crosshair geometry
        let crosshair_vertices = Self::create_crosshair_vertices();
        let crosshair_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crosshair Buffer"),
            contents: bytemuck::cast_slice(&crosshair_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            crosshair_buffer,
            crosshair_vertex_count: crosshair_vertices.len() as u32,
        }
    }

    /// Creates crosshair vertices (two crossing rectangles).
    fn create_crosshair_vertices() -> Vec<OverlayVertex> {
        let color = [1.0, 1.0, 1.0, 0.8]; // White with slight transparency
        let outline = [0.0, 0.0, 0.0, 0.5]; // Black outline

        // Crosshair dimensions in NDC (screen goes from -1 to 1)
        let size = 0.02; // Length of each arm
        let thickness = 0.003; // Thickness of lines
        let gap = 0.005; // Gap in the center
        let outline_t = 0.001; // Outline thickness

        let mut vertices = Vec::new();

        // Helper to add a rectangle
        let add_rect =
            |verts: &mut Vec<OverlayVertex>, x1: f32, y1: f32, x2: f32, y2: f32, c: [f32; 4]| {
                // Two triangles for a rectangle
                verts.push(OverlayVertex::new(x1, y1, c));
                verts.push(OverlayVertex::new(x2, y1, c));
                verts.push(OverlayVertex::new(x2, y2, c));
                verts.push(OverlayVertex::new(x1, y1, c));
                verts.push(OverlayVertex::new(x2, y2, c));
                verts.push(OverlayVertex::new(x1, y2, c));
            };

        // Horizontal bar (left part) - outline
        add_rect(
            &mut vertices,
            -size - outline_t,
            -thickness - outline_t,
            -gap + outline_t,
            thickness + outline_t,
            outline,
        );
        // Horizontal bar (left part) - fill
        add_rect(&mut vertices, -size, -thickness, -gap, thickness, color);

        // Horizontal bar (right part) - outline
        add_rect(
            &mut vertices,
            gap - outline_t,
            -thickness - outline_t,
            size + outline_t,
            thickness + outline_t,
            outline,
        );
        // Horizontal bar (right part) - fill
        add_rect(&mut vertices, gap, -thickness, size, thickness, color);

        // Vertical bar (top part) - outline
        add_rect(
            &mut vertices,
            -thickness - outline_t,
            gap - outline_t,
            thickness + outline_t,
            size + outline_t,
            outline,
        );
        // Vertical bar (top part) - fill
        add_rect(&mut vertices, -thickness, gap, thickness, size, color);

        // Vertical bar (bottom part) - outline
        add_rect(
            &mut vertices,
            -thickness - outline_t,
            -size - outline_t,
            thickness + outline_t,
            -gap + outline_t,
            outline,
        );
        // Vertical bar (bottom part) - fill
        add_rect(&mut vertices, -thickness, -size, thickness, -gap, color);

        vertices
    }

    /// Renders the crosshair.
    pub fn render_crosshair<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.crosshair_buffer.slice(..));
        render_pass.draw(0..self.crosshair_vertex_count, 0..1);
    }
}
