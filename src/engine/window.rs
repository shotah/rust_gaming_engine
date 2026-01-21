//! Window management module.
//!
//! Handles window creation and event processing using winit.

use anyhow::Result;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

/// Window configuration options.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title.
    pub title: String,
    /// Initial window width.
    pub width: u32,
    /// Initial window height.
    pub height: u32,
    /// Whether the window should be resizable.
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("Voxel Forge"),
            width: 1280,
            height: 720,
            resizable: true,
        }
    }
}

/// Manages the game window.
pub struct GameWindow {
    /// The winit window instance.
    window: Arc<Window>,
}

impl GameWindow {
    /// Creates a new game window with the given event loop and configuration.
    pub fn new(event_loop: &ActiveEventLoop, config: &WindowConfig) -> Result<Self> {
        let window_attributes = Window::default_attributes()
            .with_title(&config.title)
            .with_inner_size(PhysicalSize::new(config.width, config.height))
            .with_resizable(config.resizable);

        let window = event_loop.create_window(window_attributes)?;

        Ok(Self {
            window: Arc::new(window),
        })
    }

    /// Returns the current inner size of the window.
    #[must_use]
    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    /// Returns a reference to the underlying winit window.
    #[must_use]
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    /// Requests a redraw of the window.
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

/// Creates a new event loop for the application.
///
/// # Errors
///
/// Returns an error if the event loop cannot be created.
pub fn create_event_loop() -> Result<EventLoop<()>> {
    let event_loop = EventLoop::new()?;
    Ok(event_loop)
}
