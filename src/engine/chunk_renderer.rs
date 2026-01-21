//! Chunk rendering module.
//!
//! Handles GPU resources and rendering for voxel chunks.

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

use crate::world::{ChunkMesh, ChunkVertex, TextureAtlas};

/// Camera uniform data sent to the GPU.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    /// Combined view-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Camera position in world space.
    pub view_pos: [f32; 3],
    /// Padding for alignment.
    pub _padding: f32,
}

impl CameraUniform {
    /// Creates a new camera uniform from matrices and position.
    #[must_use]
    pub fn new(view_proj: Mat4, view_pos: glam::Vec3) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            view_pos: view_pos.to_array(),
            _padding: 0.0,
        }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new(Mat4::IDENTITY, glam::Vec3::ZERO)
    }
}

/// GPU buffers for a chunk mesh.
pub struct ChunkBuffers {
    /// Vertex buffer.
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer.
    pub index_buffer: wgpu::Buffer,
    /// Number of indices.
    pub index_count: u32,
}

impl ChunkBuffers {
    /// Creates GPU buffers from a chunk mesh.
    #[must_use]
    pub fn from_mesh(device: &wgpu::Device, mesh: &ChunkMesh) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }
}

/// Handles chunk rendering with a dedicated pipeline.
pub struct ChunkRenderer {
    /// The render pipeline.
    pipeline: wgpu::RenderPipeline,
    /// Camera uniform buffer.
    camera_buffer: wgpu::Buffer,
    /// Camera bind group.
    camera_bind_group: wgpu::BindGroup,
    /// Texture bind group.
    texture_bind_group: wgpu::BindGroup,
    /// Depth texture.
    depth_texture: wgpu::Texture,
    /// Depth texture view.
    depth_view: wgpu::TextureView,
    /// Current size.
    size: (u32, u32),
}

impl ChunkRenderer {
    /// Creates a new chunk renderer.
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline creation fails.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Block Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/block.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Generate texture atlas
        let atlas = TextureAtlas::generate();

        // Create GPU texture for atlas
        let texture_size = wgpu::Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        };

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Block Atlas Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload atlas data to GPU
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas.width),
                rows_per_image: Some(atlas.height),
            },
            texture_size,
        );

        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler with nearest-neighbor filtering (pixel art style)
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Block Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Texture bind group
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        // Pipeline layout with both bind groups
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Block Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(device, width, height);

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Block Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ChunkVertex::layout()],
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
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_group,
            depth_texture,
            depth_view,
            size: (width, height),
        })
    }

    /// Creates a depth texture.
    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    /// Resizes the depth buffer.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width > 0 && height > 0 && (width, height) != self.size {
            let (texture, view) = Self::create_depth_texture(device, width, height);
            self.depth_texture = texture;
            self.depth_view = view;
            self.size = (width, height);
        }
    }

    /// Updates the camera uniform.
    pub fn update_camera(&self, queue: &wgpu::Queue, uniform: &CameraUniform) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*uniform]));
    }

    /// Returns the depth texture view.
    #[must_use]
    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    /// Renders chunks using the given render pass.
    pub fn render<'a, I>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, chunks: I)
    where
        I: Iterator<Item = &'a ChunkBuffers>,
    {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);

        for chunk in chunks {
            render_pass.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
            render_pass.set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..chunk.index_count, 0, 0..1);
        }
    }
}
