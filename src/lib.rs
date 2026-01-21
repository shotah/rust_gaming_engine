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
//! fn main() {
//!     let engine = Engine::new();
//!     engine.run();
//! }
//! ```

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod engine;

// Re-export commonly used types
pub use engine::Engine;

/// The current version of the engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default chunk size (16x16x16 blocks per section).
pub const CHUNK_SIZE: usize = 16;

/// Default chunk height (256 blocks).
pub const CHUNK_HEIGHT: usize = 256;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_valid() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn chunk_constants_are_valid() {
        assert!(CHUNK_SIZE > 0);
        assert!(CHUNK_HEIGHT > 0);
    }
}
