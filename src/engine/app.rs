//! Application handler module.
//!
//! Implements the winit `ApplicationHandler` trait to manage the game loop.

use std::time::Instant;

use anyhow::Result;
use glam::Vec3;
use tracing::{error, info};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, WindowId},
};

use crate::world::{Block, ChunkManager, ChunkManagerConfig, ChunkPos, RaycastHit, raycast};

use super::camera::{Camera, CameraConfig};
use super::chunk_renderer::{CameraUniform, ChunkBuffers, ChunkRenderer};
use super::fps_counter::FpsCounter;
use super::input::{InputState, MouseButton};
use super::renderer::{Renderer, RendererConfig};
use super::window::{GameWindow, WindowConfig};

use std::collections::HashMap;

/// The main application state.
pub struct App {
    /// Window configuration.
    window_config: WindowConfig,
    /// Renderer configuration.
    renderer_config: RendererConfig,
    /// The game window (created on resume).
    window: Option<GameWindow>,
    /// The renderer (created after window).
    renderer: Option<Renderer>,
    /// The chunk renderer for voxel rendering.
    chunk_renderer: Option<ChunkRenderer>,
    /// Chunk manager for dynamic loading/unloading.
    chunk_manager: ChunkManager,
    /// Loaded chunk GPU buffers, keyed by position.
    chunk_buffers: HashMap<ChunkPos, ChunkBuffers>,
    /// The camera for viewing the world.
    camera: Camera,
    /// Input state tracker.
    input: InputState,
    /// FPS counter.
    fps_counter: FpsCounter,
    /// Last frame time for delta calculation.
    last_frame: Instant,
    /// Whether the app should close.
    should_close: bool,
    /// Frame counter for periodic logging.
    frame_count: u64,
    /// Currently targeted block (if any).
    targeted_block: Option<RaycastHit>,
    /// Block type to place (simple hotbar simulation).
    selected_block: Block,
}

impl App {
    /// Creates a new application instance.
    #[must_use]
    pub fn new(window_config: WindowConfig, renderer_config: RendererConfig) -> Self {
        // Start camera at a good viewing position
        let camera = Camera::new(CameraConfig::default()).at_position(Vec3::new(8.0, 80.0, 24.0));

        // Create chunk manager with default config
        let chunk_manager = ChunkManager::new(ChunkManagerConfig {
            render_distance: 6, // 6 chunk radius = 113 chunks
            max_chunks_per_frame: 8,
            max_unloads_per_frame: 16,
        });

        Self {
            window_config,
            renderer_config,
            window: None,
            renderer: None,
            chunk_renderer: None,
            chunk_manager,
            chunk_buffers: HashMap::new(),
            camera,
            input: InputState::new(),
            fps_counter: FpsCounter::new(),
            last_frame: Instant::now(),
            should_close: false,
            frame_count: 0,
            targeted_block: None,
            selected_block: Block::Stone,
        }
    }

    /// Creates the renderer and chunk renderer.
    fn create_renderer(&mut self) -> Result<()> {
        if let Some(ref window) = self.window {
            let renderer = pollster::block_on(Renderer::new(
                window.window().clone(),
                self.renderer_config.clone(),
            ))?;

            // Set camera aspect ratio
            let size = renderer.size();
            self.camera
                .set_aspect_ratio(size.width as f32, size.height as f32);

            // Create chunk renderer
            let chunk_renderer = ChunkRenderer::new(
                renderer.device(),
                renderer.queue(),
                renderer.surface_format(),
                size.width,
                size.height,
            )?;

            info!(
                "Chunk manager started with render distance {}",
                self.chunk_manager.render_distance()
            );

            self.chunk_renderer = Some(chunk_renderer);
            self.renderer = Some(renderer);
        }
        Ok(())
    }

    /// Captures or releases the mouse cursor.
    fn set_cursor_captured(&mut self, captured: bool) {
        if let Some(ref window) = self.window {
            let win = window.window();
            if captured {
                // Use Confined mode (keeps cursor in window) with manual re-centering
                // Locked mode doesn't work reliably on WSL/X11
                let grab_result = win.set_cursor_grab(CursorGrabMode::Confined);

                match &grab_result {
                    Ok(()) => {
                        info!("Cursor locked for FPS-style mouselook");
                        win.set_cursor_visible(false);
                    }
                    Err(e) => {
                        info!("Cursor grab failed ({}), using position-based fallback", e);
                    }
                }

                // Center the cursor initially
                let size = win.inner_size();
                let center = winit::dpi::PhysicalPosition::new(
                    size.width as f64 / 2.0,
                    size.height as f64 / 2.0,
                );
                let _ = win.set_cursor_position(center);
                self.input.mouse_moved((center.x, center.y));
            } else {
                let _ = win.set_cursor_grab(CursorGrabMode::None);
                win.set_cursor_visible(true);
            }
            self.input.set_cursor_locked(captured);
        }
    }

    /// Re-centers the cursor to allow infinite mouselook.
    fn recenter_cursor(&mut self) {
        if let Some(ref window) = self.window {
            let win = window.window();
            let size = win.inner_size();
            let center_x = size.width as f64 / 2.0;
            let center_y = size.height as f64 / 2.0;

            let pos = self.input.mouse_position();

            // Re-center if cursor is getting close to edge (within 100px of edge)
            let margin = 100.0;
            if pos.0 < margin
                || pos.0 > size.width as f64 - margin
                || pos.1 < margin
                || pos.1 > size.height as f64 - margin
            {
                let center = winit::dpi::PhysicalPosition::new(center_x, center_y);
                let _ = win.set_cursor_position(center);
                self.input.mouse_moved((center_x, center_y));
            }
        }
    }

    /// Updates game logic each frame.
    fn update(&mut self, delta_time: f32) {
        // Handle camera rotation from mouse
        if self.input.is_cursor_locked() {
            let (dx, dy) = self.input.take_mouse_delta();
            if dx != 0.0 || dy != 0.0 {
                self.camera.rotate(dx as f32, dy as f32);
            }
            // Re-center cursor to prevent hitting window edges
            self.recenter_cursor();
        }

        // Handle camera movement from keyboard
        let movement = self.input.movement_direction();
        if movement.length_squared() > 0.0 {
            self.camera.move_by(
                movement,
                delta_time,
                self.input.is_sprinting(),
                self.input.is_crouching(),
            );
        }

        // Raycast to find targeted block
        self.update_targeted_block();

        // Handle block interactions
        self.handle_block_interactions();

        // Update chunk manager - load/unload chunks based on player position
        self.update_chunks();

        // Rebuild dirty chunks (after block modifications)
        self.rebuild_dirty_chunks();
    }

    /// Updates the currently targeted block using raycasting.
    fn update_targeted_block(&mut self) {
        let origin = self.camera.position;
        let direction = self.camera.forward();
        let max_distance = 6.0; // Reach distance

        self.targeted_block = raycast(origin, direction, max_distance, |x, y, z| {
            self.chunk_manager.is_block_solid(x, y, z)
        });
    }

    /// Handles block breaking and placing based on mouse input.
    fn handle_block_interactions(&mut self) {
        // Only handle if cursor is locked (in game mode)
        if !self.input.is_cursor_locked() {
            return;
        }

        // Left click - break block
        if self.input.mouse_just_pressed(MouseButton::Left) {
            if let Some(hit) = &self.targeted_block {
                let pos = hit.block_pos;
                self.chunk_manager
                    .set_block(pos.x, pos.y, pos.z, Block::Air);
            }
        }

        // Right click - place block
        if self.input.mouse_just_pressed(MouseButton::Right) {
            if let Some(hit) = &self.targeted_block {
                // Place on the face we hit (adjacent to the hit block)
                let place_pos = hit.block_pos.offset(hit.face);

                // Don't place if it would intersect the player (simple check)
                let player_block_x = self.camera.position.x.floor() as i32;
                let player_block_y = self.camera.position.y.floor() as i32;
                let player_block_z = self.camera.position.z.floor() as i32;

                // Player occupies 2 blocks vertically
                let would_intersect = place_pos.x == player_block_x
                    && place_pos.z == player_block_z
                    && (place_pos.y == player_block_y || place_pos.y == player_block_y - 1);

                if !would_intersect {
                    self.chunk_manager.set_block(
                        place_pos.x,
                        place_pos.y,
                        place_pos.z,
                        self.selected_block,
                    );
                }
            }
        }

        // Number keys to select block type
        if self.input.is_key_just_pressed(KeyCode::Digit1) {
            self.selected_block = Block::Stone;
        } else if self.input.is_key_just_pressed(KeyCode::Digit2) {
            self.selected_block = Block::Dirt;
        } else if self.input.is_key_just_pressed(KeyCode::Digit3) {
            self.selected_block = Block::Grass;
        } else if self.input.is_key_just_pressed(KeyCode::Digit4) {
            self.selected_block = Block::Log;
        } else if self.input.is_key_just_pressed(KeyCode::Digit5) {
            self.selected_block = Block::Planks;
        } else if self.input.is_key_just_pressed(KeyCode::Digit6) {
            self.selected_block = Block::Bricks;
        } else if self.input.is_key_just_pressed(KeyCode::Digit7) {
            self.selected_block = Block::Glass;
        } else if self.input.is_key_just_pressed(KeyCode::Digit8) {
            self.selected_block = Block::Sand;
        } else if self.input.is_key_just_pressed(KeyCode::Digit9) {
            self.selected_block = Block::Cobblestone;
        }
    }

    /// Rebuilds chunk meshes that were modified.
    fn rebuild_dirty_chunks(&mut self) {
        let Some(renderer) = self.renderer.as_ref() else {
            return;
        };

        let dirty = self.chunk_manager.take_dirty_chunks();
        for pos in dirty {
            if let Some(generated) = self.chunk_manager.rebuild_chunk_mesh(pos) {
                if !generated.mesh.is_empty() {
                    let buffers = ChunkBuffers::from_mesh(renderer.device(), &generated.mesh);
                    self.chunk_buffers.insert(pos, buffers);
                } else {
                    self.chunk_buffers.remove(&pos);
                }
            }
        }
    }

    /// Updates chunk loading/unloading based on player position.
    fn update_chunks(&mut self) {
        let Some(renderer) = self.renderer.as_ref() else {
            return;
        };

        // Get new and unloaded chunks from manager
        let (ready_chunks, unload_chunks) = self.chunk_manager.update(self.camera.position);

        // Create GPU buffers for new chunks
        for generated in ready_chunks {
            if !generated.mesh.is_empty() {
                let buffers = ChunkBuffers::from_mesh(renderer.device(), &generated.mesh);
                self.chunk_buffers.insert(generated.pos, buffers);
            }
        }

        // Remove GPU buffers for unloaded chunks
        for pos in unload_chunks {
            self.chunk_buffers.remove(&pos);
        }

        // Periodic logging
        self.frame_count += 1;
        if self.frame_count % 300 == 0 {
            info!(
                "Chunks: {} loaded, {} generating, {} queued",
                self.chunk_manager.loaded_count(),
                self.chunk_manager.generating_count(),
                self.chunk_manager.queued_count()
            );
        }
    }

    /// Renders the frame.
    fn render_frame(&mut self) -> Result<()> {
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("No renderer"))?;
        let chunk_renderer = self
            .chunk_renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No chunk renderer"))?;

        // Update camera uniform
        let camera_uniform =
            CameraUniform::new(self.camera.view_projection_matrix(), self.camera.position);
        chunk_renderer.update_camera(renderer.queue(), &camera_uniform);

        // Get surface texture
        let output = renderer.surface().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder =
            renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.7,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: chunk_renderer.depth_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Render chunks
            chunk_renderer.render(&mut render_pass, self.chunk_buffers.values());
        }

        // Submit and present
        renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!("Creating window...");

            match GameWindow::new(event_loop, &self.window_config) {
                Ok(window) => {
                    self.window = Some(window);
                    info!("Window created successfully");

                    if let Err(e) = self.create_renderer() {
                        error!("Failed to create renderer: {e}");
                        event_loop.exit();
                    } else {
                        info!("Renderer created successfully");
                        info!("Click in window to capture mouse. ESC to release.");
                        info!(
                            "Controls: WASD move, Mouse look, Space/Shift fly up/down, Ctrl sprint"
                        );
                        info!("Blocks: Left-click break, Right-click place, 1-9 select block type");
                    }
                }
                Err(e) => {
                    error!("Failed to create window: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                self.should_close = true;
                event_loop.exit();
            }

            WindowEvent::Focused(focused) => {
                if !focused {
                    // Release cursor when window loses focus
                    self.set_cursor_captured(false);
                }
                // Don't auto-capture on focus - wait for user click
            }

            WindowEvent::Resized(new_size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(new_size);
                    self.camera
                        .set_aspect_ratio(new_size.width as f32, new_size.height as f32);
                }
                if let Some(ref mut chunk_renderer) = self.chunk_renderer {
                    if let Some(ref renderer) = self.renderer {
                        chunk_renderer.resize(renderer.device(), new_size.width, new_size.height);
                    }
                }
                // Don't auto-capture - let user click to re-capture
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.input.key_pressed(key_code);

                            match key_code {
                                KeyCode::Escape => {
                                    if self.input.is_cursor_locked() {
                                        self.set_cursor_captured(false);
                                    } else {
                                        info!("Escape pressed, closing...");
                                        self.should_close = true;
                                        event_loop.exit();
                                    }
                                }
                                KeyCode::F3 => {
                                    let pos = self.camera.position;
                                    info!(
                                        "Pos: ({:.1}, {:.1}, {:.1}) | Yaw: {:.1}° Pitch: {:.1}° | FPS: {:.1}",
                                        pos.x,
                                        pos.y,
                                        pos.z,
                                        self.camera.yaw,
                                        self.camera.pitch,
                                        self.fps_counter.fps()
                                    );
                                }
                                _ => {}
                            }
                        }
                        ElementState::Released => {
                            self.input.key_released(key_code);
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let btn = MouseButton::from(button);
                match state {
                    ElementState::Pressed => {
                        self.input.mouse_button_pressed(btn);
                        if !self.input.is_cursor_locked() {
                            self.set_cursor_captured(true);
                        }
                    }
                    ElementState::Released => {
                        self.input.mouse_button_released(btn);
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                // Compute delta from cursor position (WSL/X11 fallback - DeviceEvent may not work)
                if self.input.is_cursor_locked() {
                    let old_pos = self.input.mouse_position();
                    // Only count as movement if we have a valid previous position
                    if old_pos.0 > 0.0 || old_pos.1 > 0.0 {
                        let delta = (position.x - old_pos.0, position.y - old_pos.1);
                        // Ignore tiny movements and large jumps (cursor warp)
                        if delta.0.abs() > 0.5 && delta.0.abs() < 100.0 {
                            self.input.mouse_delta((delta.0, 0.0));
                        }
                        if delta.1.abs() > 0.5 && delta.1.abs() < 100.0 {
                            self.input.mouse_delta((0.0, delta.1));
                        }
                    }
                }
                self.input.mouse_moved((position.x, position.y));
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => {
                        (pos.x as f32 / 100.0, pos.y as f32 / 100.0)
                    }
                };
                self.input.scroll(scroll);
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                // Update BEFORE clearing frame state
                self.update(delta_time);

                // Clear per-frame input state AFTER processing
                self.input.begin_frame();

                if let Some(fps) = self.fps_counter.should_log() {
                    info!("FPS: {fps:.1}");
                }
                self.fps_counter.tick();

                if let Err(e) = self.render_frame() {
                    error!("Render error: {e}");
                }

                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: DeviceEvent,
    ) {
        // Note: DeviceEvent::MouseMotion is unreliable on WSL/X11
        // We use CursorMoved in window_event instead for cross-platform support
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}
