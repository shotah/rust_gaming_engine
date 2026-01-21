//! World module.
//!
//! Contains block definitions, chunk data structures, mesh generation,
//! and chunk management.

pub mod block;
pub mod chunk;
pub mod chunk_manager;
pub mod mesh;
pub mod raycast;
pub mod texture_atlas;

pub use block::{Block, BlockId, BlockProperties};
pub use chunk::{CHUNK_HEIGHT, Chunk, ChunkPos, ChunkSection, SECTION_SIZE};
pub use chunk_manager::{ChunkManager, ChunkManagerConfig, GeneratedChunk};
pub use mesh::{ChunkMesh, ChunkVertex, Face, MeshGenerator};
pub use raycast::{BlockPos, HitFace, RaycastHit, raycast};
pub use texture_atlas::TextureAtlas;
