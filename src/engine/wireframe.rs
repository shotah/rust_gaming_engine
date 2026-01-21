//! Wireframe rendering for block selection highlights.
//!
//! Renders a wireframe box around the currently targeted block.

use glam::Vec3;
use wgpu::util::DeviceExt;

/// Vertex for wireframe rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WireframeVertex {
    /// Position in world space.
    pub position: [f32; 3],
}

impl WireframeVertex {
    /// Creates a new wireframe vertex.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
        }
    }

    /// Returns the vertex buffer layout.
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

/// Camera uniform for wireframe shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WireframeCameraUniform {
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
}

/// Renders wireframe boxes for block selection.
pub struct WireframeRenderer {
    /// The render pipeline.
    pipeline: wgpu::RenderPipeline,
    /// Camera bind group.
    camera_bind_group: wgpu::BindGroup,
    /// Camera uniform buffer.
    camera_buffer: wgpu::Buffer,
    /// Vertex buffer for the current highlight box.
    vertex_buffer: wgpu::Buffer,
    /// Index buffer for line strips.
    index_buffer: wgpu::Buffer,
    /// Number of indices.
    index_count: u32,
}

impl WireframeRenderer {
    /// Creates a new wireframe renderer.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wireframe.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Camera Buffer"),
            contents: bytemuck::cast_slice(&[WireframeCameraUniform {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Wireframe Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Wireframe Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wireframe Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wireframe Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[WireframeVertex::layout()],
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
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for wireframes
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false, // Don't write to depth
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create unit cube vertices (will be scaled/translated per block)
        let (vertices, indices) = Self::create_box_geometry(Vec3::ZERO, 1.0);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            camera_bind_group,
            camera_buffer,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    /// Creates box geometry for wireframe rendering.
    /// Returns vertices and line indices.
    fn create_box_geometry(pos: Vec3, size: f32) -> (Vec<WireframeVertex>, Vec<u16>) {
        // Slight offset to prevent z-fighting (box is slightly larger than block)
        let offset = 0.002;
        let min = pos - Vec3::splat(offset);
        let max = pos + Vec3::splat(size + offset);

        // 8 corners of the box
        #[rustfmt::skip]
        let vertices = vec![
            WireframeVertex::new(min.x, min.y, min.z), // 0: front-bottom-left
            WireframeVertex::new(max.x, min.y, min.z), // 1: front-bottom-right
            WireframeVertex::new(max.x, max.y, min.z), // 2: front-top-right
            WireframeVertex::new(min.x, max.y, min.z), // 3: front-top-left
            WireframeVertex::new(min.x, min.y, max.z), // 4: back-bottom-left
            WireframeVertex::new(max.x, min.y, max.z), // 5: back-bottom-right
            WireframeVertex::new(max.x, max.y, max.z), // 6: back-top-right
            WireframeVertex::new(min.x, max.y, max.z), // 7: back-top-left
        ];

        // 12 edges of the cube (as line pairs)
        #[rustfmt::skip]
        let indices = vec![
            // Front face
            0, 1,  1, 2,  2, 3,  3, 0,
            // Back face
            4, 5,  5, 6,  6, 7,  7, 4,
            // Connecting edges
            0, 4,  1, 5,  2, 6,  3, 7,
        ];

        (vertices, indices)
    }

    /// Updates the camera uniform.
    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: glam::Mat4) {
        let uniform = WireframeCameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Updates the highlight box position.
    pub fn update_highlight(&self, queue: &wgpu::Queue, block_pos: Vec3) {
        let (vertices, _) = Self::create_box_geometry(block_pos, 1.0);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }

    /// Renders the wireframe highlight.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
