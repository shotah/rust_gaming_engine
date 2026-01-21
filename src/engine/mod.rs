//! Core engine module.
//!
//! This module contains the main engine struct and core functionality.

use anyhow::Result;
use tracing::info;

/// The main game engine.
///
/// This struct manages the game loop and coordinates all subsystems.
#[derive(Debug, Default)]
pub struct Engine {
    /// Whether the engine is currently running.
    running: bool,
}

impl Engine {
    /// Creates a new engine instance.
    ///
    /// # Errors
    ///
    /// Returns an error if engine initialization fails.
    pub fn new() -> Result<Self> {
        info!("Initializing Voxel Forge engine...");

        Ok(Self { running: false })
    }

    /// Runs the main game loop.
    ///
    /// This method blocks until the game is closed.
    ///
    /// # Errors
    ///
    /// Returns an error if the game loop encounters a fatal error.
    pub fn run(mut self) -> Result<()> {
        info!("Starting game loop...");
        self.running = true;

        // TODO: Implement actual game loop
        // For now, just exit immediately
        info!("Engine placeholder - no window yet!");

        self.running = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_creates_successfully() {
        let engine = Engine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn engine_default_not_running() {
        let engine = Engine::default();
        assert!(!engine.running);
    }
}
