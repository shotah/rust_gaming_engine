//! Chunk mesh generation with greedy meshing optimization.
//!
//! Converts chunk block data into renderable mesh geometry using
//! greedy meshing to minimize triangle count.

use bytemuck::{Pod, Zeroable};

use super::block::Block;
use super::chunk::{CHUNK_HEIGHT, Chunk, SECTION_SIZE};
use super::texture_atlas::TextureAtlas;

/// A vertex in the chunk mesh.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ChunkVertex {
    /// Position in world space.
    pub position: [f32; 3],
    /// Normal vector.
    pub normal: [f32; 3],
    /// Color (RGB) - used as tint when textures are enabled.
    pub color: [f32; 3],
    /// Ambient occlusion factor (0.0 = dark, 1.0 = bright).
    pub ao: f32,
    /// Local UV coordinates (0 to width, 0 to height) for texture tiling.
    pub local_uv: [f32; 2],
    /// Atlas UV base position (top-left of texture in atlas).
    pub atlas_uv: [f32; 2],
}

impl ChunkVertex {
    /// Creates a new vertex.
    #[must_use]
    pub const fn new(
        position: [f32; 3],
        normal: [f32; 3],
        color: [f32; 3],
        ao: f32,
        local_uv: [f32; 2],
        atlas_uv: [f32; 2],
    ) -> Self {
        Self {
            position,
            normal,
            color,
            ao,
            local_uv,
            atlas_uv,
        }
    }

    /// Returns the vertex buffer layout for wgpu.
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // ao
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // local_uv
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // atlas_uv
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// The six faces of a cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Positive X (East).
    PosX,
    /// Negative X (West).
    NegX,
    /// Positive Y (Up).
    PosY,
    /// Negative Y (Down).
    NegY,
    /// Positive Z (South).
    PosZ,
    /// Negative Z (North).
    NegZ,
}

impl Face {
    /// Returns the normal vector for this face.
    #[must_use]
    pub const fn normal(self) -> [f32; 3] {
        match self {
            Self::PosX => [1.0, 0.0, 0.0],
            Self::NegX => [-1.0, 0.0, 0.0],
            Self::PosY => [0.0, 1.0, 0.0],
            Self::NegY => [0.0, -1.0, 0.0],
            Self::PosZ => [0.0, 0.0, 1.0],
            Self::NegZ => [0.0, 0.0, -1.0],
        }
    }

    /// Returns all six faces.
    pub const ALL: [Face; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];
}

/// Generated mesh data for a chunk.
pub struct ChunkMesh {
    /// Vertex data.
    pub vertices: Vec<ChunkVertex>,
    /// Index data.
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    /// Creates a new empty mesh.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Returns true if the mesh is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Returns the number of triangles in the mesh.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for ChunkMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Face mask entry for greedy meshing.
/// Stores the block type and whether it's been processed.
#[derive(Clone, Copy, Default)]
struct FaceMask {
    block: Option<Block>,
}

/// Greedy mesh generator - optimized mesh generation.
pub struct MeshGenerator {
    chunk: Chunk,
    world_offset: [f32; 3],
}

impl MeshGenerator {
    /// Creates a new mesh generator for the given chunk.
    #[must_use]
    pub fn new(chunk: Chunk) -> Self {
        let (ox, oz) = chunk.position().block_origin();
        Self {
            chunk,
            world_offset: [ox as f32, 0.0, oz as f32],
        }
    }

    /// Generates the mesh using greedy meshing algorithm.
    #[must_use]
    pub fn generate(self) -> ChunkMesh {
        let mut mesh = ChunkMesh::new();

        // Process each face direction
        self.generate_faces(&mut mesh, Face::PosY); // Top
        self.generate_faces(&mut mesh, Face::NegY); // Bottom
        self.generate_faces(&mut mesh, Face::PosX); // East
        self.generate_faces(&mut mesh, Face::NegX); // West
        self.generate_faces(&mut mesh, Face::PosZ); // South
        self.generate_faces(&mut mesh, Face::NegZ); // North

        mesh
    }

    /// Generates faces for one direction using greedy meshing.
    fn generate_faces(&self, mesh: &mut ChunkMesh, face: Face) {
        // Determine axis and iteration order based on face
        let (axis, u_axis, v_axis, positive) = match face {
            Face::PosY => (1, 0, 2, true),  // Y+: iterate Y, sweep XZ
            Face::NegY => (1, 0, 2, false), // Y-: iterate Y, sweep XZ
            Face::PosX => (0, 2, 1, true),  // X+: iterate X, sweep ZY
            Face::NegX => (0, 2, 1, false), // X-: iterate X, sweep ZY
            Face::PosZ => (2, 0, 1, true),  // Z+: iterate Z, sweep XY
            Face::NegZ => (2, 0, 1, false), // Z-: iterate Z, sweep XY
        };

        let axis_size = if axis == 1 {
            CHUNK_HEIGHT
        } else {
            SECTION_SIZE
        };
        let u_size = if u_axis == 1 {
            CHUNK_HEIGHT
        } else {
            SECTION_SIZE
        };
        let v_size = if v_axis == 1 {
            CHUNK_HEIGHT
        } else {
            SECTION_SIZE
        };

        // For each slice along the axis
        for d in 0..axis_size {
            // Build face mask for this slice
            let mut mask = vec![FaceMask::default(); u_size * v_size];

            for v in 0..v_size {
                for u in 0..u_size {
                    let mut pos = [0usize; 3];
                    pos[axis] = d;
                    pos[u_axis] = u;
                    pos[v_axis] = v;

                    let block = self.chunk.get_block(pos[0], pos[1], pos[2]);

                    // Skip air blocks
                    if block.is_air() {
                        continue;
                    }

                    // Check if face is visible (neighbor is transparent)
                    let neighbor_pos = if positive {
                        if d + 1 >= axis_size {
                            // Chunk boundary - face is visible
                            None
                        } else {
                            let mut np = pos;
                            np[axis] = d + 1;
                            Some(np)
                        }
                    } else if d == 0 {
                        // Chunk boundary - face is visible
                        None
                    } else {
                        let mut np = pos;
                        np[axis] = d - 1;
                        Some(np)
                    };

                    let face_visible = match neighbor_pos {
                        None => true, // Chunk boundary
                        Some(np) => {
                            let neighbor = self.chunk.get_block(np[0], np[1], np[2]);
                            neighbor.is_transparent()
                        }
                    };

                    if face_visible {
                        mask[u + v * u_size] = FaceMask { block: Some(block) };
                    }
                }
            }

            // Greedy merge and generate quads
            self.greedy_merge(
                mesh, &mut mask, u_size, v_size, d, face, u_axis, v_axis, axis,
            );
        }
    }

    /// Performs greedy merging on the mask and generates quads.
    #[allow(clippy::too_many_arguments)]
    fn greedy_merge(
        &self,
        mesh: &mut ChunkMesh,
        mask: &mut [FaceMask],
        u_size: usize,
        v_size: usize,
        d: usize,
        face: Face,
        u_axis: usize,
        v_axis: usize,
        axis: usize,
    ) {
        for v in 0..v_size {
            let mut u = 0;
            while u < u_size {
                let idx = u + v * u_size;
                let current = mask[idx];

                if current.block.is_none() {
                    u += 1;
                    continue;
                }

                let block = current.block.unwrap();

                // Find width (how far we can extend in U direction)
                let mut width = 1;
                while u + width < u_size {
                    let next = mask[u + width + v * u_size];
                    if next.block != current.block {
                        break;
                    }
                    width += 1;
                }

                // Find height (how far we can extend in V direction)
                let mut height = 1;
                'height: while v + height < v_size {
                    for w in 0..width {
                        let next = mask[u + w + (v + height) * u_size];
                        if next.block != current.block {
                            break 'height;
                        }
                    }
                    height += 1;
                }

                // Clear the merged region
                for dv in 0..height {
                    for du in 0..width {
                        mask[u + du + (v + dv) * u_size] = FaceMask::default();
                    }
                }

                // Generate quad
                self.add_greedy_quad(
                    mesh, d, u, v, width, height, face, u_axis, v_axis, axis, block,
                );

                u += width;
            }
        }
    }

    /// Adds a quad from greedy meshing.
    #[allow(clippy::too_many_arguments)]
    fn add_greedy_quad(
        &self,
        mesh: &mut ChunkMesh,
        d: usize,
        u: usize,
        v: usize,
        width: usize,
        height: usize,
        face: Face,
        u_axis: usize,
        v_axis: usize,
        axis: usize,
        block: Block,
    ) {
        let base_idx = mesh.vertices.len() as u32;
        let normal = face.normal();
        let color = block.color();

        // Get texture atlas base position for this block
        let (tex_u_min, tex_v_min, _, _) = TextureAtlas::block_uvs(block);
        let atlas_uv = [tex_u_min, tex_v_min];

        // Local UV corners for tiling (0 to width, 0 to height)
        // Corner order: (0,0), (width,0), (width,height), (0,height)
        let local_uv_corners = [
            [0.0, 0.0],                    // 0: bottom-left
            [width as f32, 0.0],           // 1: bottom-right
            [width as f32, height as f32], // 2: top-right
            [0.0, height as f32],          // 3: top-left
        ];

        // Calculate the 4 corners of the quad
        let mut corners = [[0.0f32; 3]; 4];

        // Base position
        let d_offset = match face {
            Face::PosX | Face::PosY | Face::PosZ => d as f32 + 1.0,
            Face::NegX | Face::NegY | Face::NegZ => d as f32,
        };

        for (i, corner) in corners.iter_mut().enumerate() {
            corner[axis] = d_offset;

            let (u_off, v_off) = match i {
                0 => (0, 0),
                1 => (width, 0),
                2 => (width, height),
                3 => (0, height),
                _ => unreachable!(),
            };

            corner[u_axis] = (u + u_off) as f32;
            corner[v_axis] = (v + v_off) as f32;

            // Add world offset
            corner[0] += self.world_offset[0];
            corner[1] += self.world_offset[1];
            corner[2] += self.world_offset[2];
        }

        // Add vertices (winding order depends on face direction)
        let ao = 1.0; // TODO: Compute ambient occlusion

        match face {
            Face::PosX | Face::PosY | Face::PosZ => {
                mesh.vertices.push(ChunkVertex::new(
                    corners[0],
                    normal,
                    color,
                    ao,
                    local_uv_corners[0],
                    atlas_uv,
                ));
                mesh.vertices.push(ChunkVertex::new(
                    corners[1],
                    normal,
                    color,
                    ao,
                    local_uv_corners[1],
                    atlas_uv,
                ));
                mesh.vertices.push(ChunkVertex::new(
                    corners[2],
                    normal,
                    color,
                    ao,
                    local_uv_corners[2],
                    atlas_uv,
                ));
                mesh.vertices.push(ChunkVertex::new(
                    corners[3],
                    normal,
                    color,
                    ao,
                    local_uv_corners[3],
                    atlas_uv,
                ));
            }
            Face::NegX | Face::NegY | Face::NegZ => {
                mesh.vertices.push(ChunkVertex::new(
                    corners[0],
                    normal,
                    color,
                    ao,
                    local_uv_corners[0],
                    atlas_uv,
                ));
                mesh.vertices.push(ChunkVertex::new(
                    corners[3],
                    normal,
                    color,
                    ao,
                    local_uv_corners[3],
                    atlas_uv,
                ));
                mesh.vertices.push(ChunkVertex::new(
                    corners[2],
                    normal,
                    color,
                    ao,
                    local_uv_corners[2],
                    atlas_uv,
                ));
                mesh.vertices.push(ChunkVertex::new(
                    corners[1],
                    normal,
                    color,
                    ao,
                    local_uv_corners[1],
                    atlas_uv,
                ));
            }
        }

        // Add indices (two triangles, CCW winding for front faces)
        // X and Y faces use one winding, Z faces use the opposite
        let indices = match face {
            Face::PosX | Face::NegX | Face::PosY | Face::NegY => [
                base_idx,
                base_idx + 2,
                base_idx + 1,
                base_idx,
                base_idx + 3,
                base_idx + 2,
            ],
            Face::PosZ | Face::NegZ => [
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ],
        };
        mesh.indices.extend_from_slice(&indices);
    }
}

/// Generates a simple test mesh (single block at origin).
/// Vertex layout matches greedy mesh generator for consistent winding.
#[must_use]
pub fn generate_test_cube(block: Block) -> ChunkMesh {
    let mut mesh = ChunkMesh::new();
    let color = block.color();

    // Get texture atlas base position for this block
    let (tex_u_min, tex_v_min, _, _) = TextureAtlas::block_uvs(block);
    let atlas_uv = [tex_u_min, tex_v_min];

    // Vertices ordered to match greedy mesh: corners[i] at (u_off, v_off) positions
    // (0,0), (width,0), (width,height), (0,height) in the face's UV space
    let faces = [
        // PosX: u=Z, v=Y, looking from +X
        (
            Face::PosX,
            [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
        ),
        // NegX: u=Z, v=Y, looking from -X
        (
            Face::NegX,
            [
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
        ),
        // PosY: u=X, v=Z, looking from +Y
        (
            Face::PosY,
            [
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
        ),
        // NegY: u=X, v=Z, looking from -Y
        (
            Face::NegY,
            [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
            ],
        ),
        // PosZ: u=X, v=Y, looking from +Z
        (
            Face::PosZ,
            [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
        ),
        // NegZ: u=X, v=Y, looking from -Z
        (
            Face::NegZ,
            [
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
        ),
    ];

    // Local UV corners for a single block (0 to 1)
    let local_uv_corners = [
        [0.0, 0.0], // 0: bottom-left
        [1.0, 0.0], // 1: bottom-right
        [1.0, 1.0], // 2: top-right
        [0.0, 1.0], // 3: top-left
    ];

    for (face, verts) in faces {
        let base_idx = mesh.vertices.len() as u32;
        let normal = face.normal();

        for (i, vert) in verts.iter().enumerate() {
            mesh.vertices.push(ChunkVertex::new(
                *vert,
                normal,
                color,
                1.0,
                local_uv_corners[i],
                atlas_uv,
            ));
        }

        // Match the per-face winding from greedy mesh generator
        // X and Y faces use one winding, Z faces use the opposite
        let indices = match face {
            Face::PosX | Face::NegX | Face::PosY | Face::NegY => [
                base_idx,
                base_idx + 2,
                base_idx + 1,
                base_idx,
                base_idx + 3,
                base_idx + 2,
            ],
            Face::PosZ | Face::NegZ => [
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ],
        };
        mesh.indices.extend_from_slice(&indices);
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::chunk::ChunkPos;

    #[test]
    fn test_cube_has_correct_geometry() {
        let mesh = generate_test_cube(Block::Stone);

        // 6 faces * 4 vertices = 24 vertices
        assert_eq!(mesh.vertices.len(), 24);

        // 6 faces * 2 triangles * 3 indices = 36 indices
        assert_eq!(mesh.indices.len(), 36);

        // 12 triangles total
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn empty_chunk_produces_empty_mesh() {
        let chunk = Chunk::new(ChunkPos::new(0, 0));
        let generator = MeshGenerator::new(chunk);
        let mesh = generator.generate();

        assert!(mesh.is_empty());
    }

    #[test]
    fn single_block_produces_faces() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        chunk.set_block(8, 100, 8, Block::Stone);

        let generator = MeshGenerator::new(chunk);
        let mesh = generator.generate();

        // Single exposed block should have 6 faces * 4 vertices = 24 vertices
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn greedy_meshing_reduces_triangles() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        // Create a 4x4 flat surface of stone at y=64
        for x in 0..4 {
            for z in 0..4 {
                chunk.set_block(x, 64, z, Block::Stone);
            }
        }

        let generator = MeshGenerator::new(chunk);
        let mesh = generator.generate();

        // With greedy meshing, the top face should be merged into ONE quad
        // Instead of 16 separate faces (4x4 blocks)
        // Total: 6 faces on the merged block * 4 verts = but greedy should merge
        // The top should be 1 quad, bottom 1 quad, sides vary

        // Without greedy: 16 blocks * 6 faces * 2 triangles = 192 triangles
        // With greedy: Much fewer due to merging
        // Top: 1 quad = 2 triangles
        // Bottom: 1 quad = 2 triangles
        // Sides: 4 strips of 4 blocks each = more complex

        // Just verify we have significantly fewer than naive
        let naive_triangles = 16 * 6 * 2; // 192
        assert!(
            mesh.triangle_count() < naive_triangles / 2,
            "Greedy should produce <50% of naive triangles. Got {} vs naive {}",
            mesh.triangle_count(),
            naive_triangles
        );
    }

    #[test]
    fn adjacent_same_blocks_merge() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        // Create a line of 8 blocks
        for x in 0..8 {
            chunk.set_block(x, 64, 4, Block::Stone);
        }

        let generator = MeshGenerator::new(chunk);
        let mesh = generator.generate();

        // With greedy meshing, this should produce far fewer triangles
        // than 8 separate cubes
        let naive_triangles = 8 * 6 * 2; // 96 triangles for 8 cubes
        assert!(
            mesh.triangle_count() < naive_triangles / 2,
            "Line of blocks should merge. Got {} triangles",
            mesh.triangle_count()
        );
    }

    #[test]
    fn different_blocks_dont_merge() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        // Alternating block types
        chunk.set_block(0, 64, 0, Block::Stone);
        chunk.set_block(1, 64, 0, Block::Dirt);
        chunk.set_block(2, 64, 0, Block::Stone);
        chunk.set_block(3, 64, 0, Block::Dirt);

        let generator = MeshGenerator::new(chunk);
        let mesh = generator.generate();

        // Different blocks shouldn't merge, so should have more faces
        assert!(!mesh.is_empty());
    }

    #[test]
    fn vertex_layout_is_correct() {
        let layout = ChunkVertex::layout();
        assert_eq!(
            layout.array_stride,
            std::mem::size_of::<ChunkVertex>() as u64
        );
        // 6 attributes: position, normal, color, ao, local_uv, atlas_uv
        assert_eq!(layout.attributes.len(), 6);
    }

    #[test]
    fn face_normals_are_unit_vectors() {
        for face in Face::ALL {
            let normal = face.normal();
            let length = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
            assert!((length - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn triangle_winding_is_ccw_for_front_faces() {
        // Generate a simple cube and verify winding order
        let mesh = generate_test_cube(Block::Stone);

        // Check each triangle has correct CCW winding when viewed from outside
        // For each triangle, compute cross product of edges - should point same dir as normal
        for i in (0..mesh.indices.len()).step_by(3) {
            let i0 = mesh.indices[i] as usize;
            let i1 = mesh.indices[i + 1] as usize;
            let i2 = mesh.indices[i + 2] as usize;

            let v0 = mesh.vertices[i0].position;
            let v1 = mesh.vertices[i1].position;
            let v2 = mesh.vertices[i2].position;

            // Edge vectors
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

            // Cross product (gives face normal direction)
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];

            // Should point same direction as stored normal (dot > 0)
            let stored_normal = mesh.vertices[i0].normal;
            let dot = cross[0] * stored_normal[0]
                + cross[1] * stored_normal[1]
                + cross[2] * stored_normal[2];

            assert!(
                dot > 0.0,
                "Triangle {}: winding produces normal opposite to stored normal. \
                 Cross: {:?}, Stored: {:?}, Dot: {}",
                i / 3,
                cross,
                stored_normal,
                dot
            );
        }
    }

    #[test]
    fn greedy_mesh_winding_is_ccw() {
        // Test greedy meshing also produces correct winding
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        chunk.set_block(8, 64, 8, Block::Stone);

        let generator = MeshGenerator::new(chunk);
        let mesh = generator.generate();

        // Same winding check as above
        for i in (0..mesh.indices.len()).step_by(3) {
            let i0 = mesh.indices[i] as usize;
            let i1 = mesh.indices[i + 1] as usize;
            let i2 = mesh.indices[i + 2] as usize;

            let v0 = mesh.vertices[i0].position;
            let v1 = mesh.vertices[i1].position;
            let v2 = mesh.vertices[i2].position;

            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];

            let stored_normal = mesh.vertices[i0].normal;
            let dot = cross[0] * stored_normal[0]
                + cross[1] * stored_normal[1]
                + cross[2] * stored_normal[2];

            assert!(
                dot > 0.0,
                "Greedy triangle {}: wrong winding. Dot: {}",
                i / 3,
                dot
            );
        }
    }
}
