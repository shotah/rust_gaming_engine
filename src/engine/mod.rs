//! Core engine module.
//!
//! This module contains the main engine struct and core functionality.

pub mod app;
pub mod camera;
pub mod chunk_renderer;
pub mod fps_counter;
pub mod input;
pub mod overlay;
pub mod renderer;
pub mod window;
pub mod wireframe;

use anyhow::Result;
use tracing::info;

use app::App;
use renderer::RendererConfig;
use window::{WindowConfig, create_event_loop};

/// The main game engine.
///
/// This struct manages the game loop and coordinates all subsystems.
pub struct Engine {
    /// Window configuration.
    window_config: WindowConfig,
    /// Renderer configuration.
    renderer_config: RendererConfig,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            window_config: WindowConfig::default(),
            renderer_config: RendererConfig::default(),
        }
    }
}

impl Engine {
    /// Creates a new engine instance with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if engine initialization fails.
    pub fn new() -> Result<Self> {
        info!("Initializing Voxel Forge engine...");
        Ok(Self::default())
    }

    /// Creates a new engine instance with custom configuration.
    #[must_use]
    pub const fn with_config(window_config: WindowConfig, renderer_config: RendererConfig) -> Self {
        Self {
            window_config,
            renderer_config,
        }
    }

    /// Sets the window title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.window_config.title = title.into();
        self
    }

    /// Sets the initial window size.
    #[must_use]
    pub const fn with_size(mut self, width: u32, height: u32) -> Self {
        self.window_config.width = width;
        self.window_config.height = height;
        self
    }

    /// Sets the clear color (RGBA values from 0.0 to 1.0).
    #[must_use]
    pub fn with_clear_color(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.renderer_config.clear_color = wgpu::Color { r, g, b, a };
        self
    }

    /// Runs the main game loop.
    ///
    /// This method blocks until the game is closed.
    ///
    /// # Errors
    ///
    /// Returns an error if the game loop encounters a fatal error.
    pub fn run(self) -> Result<()> {
        info!("Starting Voxel Forge...");

        let event_loop = create_event_loop()?;
        let mut app = App::new(self.window_config, self.renderer_config);

        event_loop.run_app(&mut app)?;

        info!("Voxel Forge shut down cleanly");
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
    fn engine_builder_pattern() {
        let engine = Engine::new()
            .unwrap()
            .with_title("Test Game")
            .with_size(800, 600);

        assert_eq!(engine.window_config.title, "Test Game");
        assert_eq!(engine.window_config.width, 800);
        assert_eq!(engine.window_config.height, 600);
    }

    #[test]
    fn engine_default() {
        let engine = Engine::default();
        assert_eq!(engine.window_config.title, "Voxel Forge");
        assert_eq!(engine.window_config.width, 1280);
        assert_eq!(engine.window_config.height, 720);
    }

    #[test]
    fn engine_with_clear_color() {
        let engine = Engine::new().unwrap().with_clear_color(1.0, 0.5, 0.25, 1.0);

        assert!((engine.renderer_config.clear_color.r - 1.0).abs() < 0.001);
        assert!((engine.renderer_config.clear_color.g - 0.5).abs() < 0.001);
        assert!((engine.renderer_config.clear_color.b - 0.25).abs() < 0.001);
    }

    #[test]
    fn engine_with_config() {
        let window_config = WindowConfig {
            title: String::from("Custom"),
            width: 1920,
            height: 1080,
            resizable: false,
        };
        let renderer_config = RendererConfig::default();

        let engine = Engine::with_config(window_config, renderer_config);

        assert_eq!(engine.window_config.title, "Custom");
        assert_eq!(engine.window_config.width, 1920);
        assert!(!engine.window_config.resizable);
    }

    #[test]
    fn engine_chained_builder() {
        let engine = Engine::new()
            .unwrap()
            .with_title("Chained")
            .with_size(640, 480)
            .with_clear_color(0.0, 0.0, 0.0, 1.0);

        assert_eq!(engine.window_config.title, "Chained");
        assert_eq!(engine.window_config.width, 640);
        assert!((engine.renderer_config.clear_color.r).abs() < 0.001);
    }
}
