//! # Voxel Forge
//!
//! A high-performance voxel game engine inspired by Minecraft and Hytale.
//!
//! ## Features
//!
//! - Modern rendering with wgpu
//! - Efficient chunk-based world management
//! - Entity Component System architecture
//! - Multiplayer networking support
//! - Procedural world generation
//!
//! ## Quick Start
//!
//! ```no_run
//! use voxel_forge::Engine;
//!
//! fn main() -> anyhow::Result<()> {
//!     let engine = Engine::new()?;
//!     engine.run()?;
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod engine;
pub mod world;

// Re-export commonly used types
pub use engine::Engine;
pub use world::{Block, Chunk, ChunkPos};

/// The current version of the engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_valid() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn chunk_constants_are_valid() {
        assert!(world::SECTION_SIZE > 0);
        assert!(world::CHUNK_HEIGHT > 0);
    }
}
