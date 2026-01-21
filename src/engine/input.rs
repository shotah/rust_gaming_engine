//! Input handling module.
//!
//! Tracks keyboard and mouse state with support for querying
//! pressed, just_pressed, and just_released states.

use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Other(id) => Self::Other(id),
            winit::event::MouseButton::Back => Self::Other(100),
            winit::event::MouseButton::Forward => Self::Other(101),
        }
    }
}

/// Tracks all input state for keyboard and mouse.
#[derive(Debug, Default)]
pub struct InputState {
    /// Currently held keys.
    keys_held: HashSet<KeyCode>,
    /// Keys pressed this frame.
    keys_pressed: HashSet<KeyCode>,
    /// Keys released this frame.
    keys_released: HashSet<KeyCode>,

    /// Currently held mouse buttons.
    mouse_held: HashSet<MouseButton>,
    /// Mouse buttons pressed this frame.
    mouse_pressed: HashSet<MouseButton>,
    /// Mouse buttons released this frame.
    mouse_released: HashSet<MouseButton>,

    /// Mouse position in screen coordinates.
    mouse_position: (f64, f64),
    /// Mouse delta movement this frame.
    mouse_delta: (f64, f64),
    /// Accumulated mouse delta (for when cursor is locked).
    mouse_delta_accumulated: (f64, f64),
    /// Scroll wheel delta this frame.
    scroll_delta: (f32, f32),

    /// Whether the cursor is locked (for FPS controls).
    cursor_locked: bool,
}

impl InputState {
    /// Creates a new input state tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Called at the start of each frame to reset per-frame state.
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Records a key press event.
    pub fn key_pressed(&mut self, key: KeyCode) {
        if !self.keys_held.contains(&key) {
            self.keys_pressed.insert(key);
        }
        self.keys_held.insert(key);
    }

    /// Records a key release event.
    pub fn key_released(&mut self, key: KeyCode) {
        self.keys_held.remove(&key);
        self.keys_released.insert(key);
    }

    /// Records a mouse button press event.
    pub fn mouse_button_pressed(&mut self, button: MouseButton) {
        if !self.mouse_held.contains(&button) {
            self.mouse_pressed.insert(button);
        }
        self.mouse_held.insert(button);
    }

    /// Records a mouse button release event.
    pub fn mouse_button_released(&mut self, button: MouseButton) {
        self.mouse_held.remove(&button);
        self.mouse_released.insert(button);
    }

    /// Records mouse movement.
    pub fn mouse_moved(&mut self, position: (f64, f64)) {
        self.mouse_position = position;
    }

    /// Records raw mouse delta movement (for locked cursor mode).
    pub fn mouse_delta(&mut self, delta: (f64, f64)) {
        self.mouse_delta.0 += delta.0;
        self.mouse_delta.1 += delta.1;
        self.mouse_delta_accumulated.0 += delta.0;
        self.mouse_delta_accumulated.1 += delta.1;
    }

    /// Records scroll wheel movement.
    pub fn scroll(&mut self, delta: (f32, f32)) {
        self.scroll_delta.0 += delta.0;
        self.scroll_delta.1 += delta.1;
    }

    /// Sets whether the cursor is locked.
    pub fn set_cursor_locked(&mut self, locked: bool) {
        self.cursor_locked = locked;
        if locked {
            self.mouse_delta_accumulated = (0.0, 0.0);
        }
    }

    // --- Query methods ---

    /// Returns true if the key is currently held down.
    #[must_use]
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// Returns true if the key was just pressed this frame.
    #[must_use]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Returns true if the key was just released this frame.
    #[must_use]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    /// Returns true if the mouse button is currently held down.
    #[must_use]
    pub fn is_mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }

    /// Returns true if the mouse button was just pressed this frame.
    #[must_use]
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    /// Alias for `is_mouse_just_pressed` for consistency with `just_pressed`.
    #[must_use]
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.is_mouse_just_pressed(button)
    }

    /// Returns true if the mouse button was just released this frame.
    #[must_use]
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_released.contains(&button)
    }

    /// Returns the current mouse position in screen coordinates.
    #[must_use]
    pub const fn mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    /// Returns the mouse delta movement this frame.
    #[must_use]
    pub const fn get_mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    /// Takes and resets the accumulated mouse delta.
    pub fn take_mouse_delta(&mut self) -> (f64, f64) {
        let delta = self.mouse_delta_accumulated;
        self.mouse_delta_accumulated = (0.0, 0.0);
        delta
    }

    /// Returns the scroll wheel delta this frame.
    #[must_use]
    pub const fn get_scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }

    /// Returns whether the cursor is locked.
    #[must_use]
    pub const fn is_cursor_locked(&self) -> bool {
        self.cursor_locked
    }

    // --- Convenience methods for common game inputs ---

    /// Returns the movement direction based on WASD keys.
    /// Returns (right, up, forward) where each component is -1, 0, or 1.
    #[must_use]
    pub fn movement_direction(&self) -> glam::Vec3 {
        let mut direction = glam::Vec3::ZERO;

        if self.is_key_held(KeyCode::KeyW) {
            direction.z += 1.0;
        }
        if self.is_key_held(KeyCode::KeyS) {
            direction.z -= 1.0;
        }
        if self.is_key_held(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if self.is_key_held(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if self.is_key_held(KeyCode::Space) {
            direction.y += 1.0; // Fly up
        }
        if self.is_key_held(KeyCode::ShiftLeft) || self.is_key_held(KeyCode::ShiftRight) {
            direction.y -= 1.0; // Fly down (Shift - like Minecraft creative)
        }
        // TODO: Add is_flying flag - when walking, Shift = crouch instead

        direction
    }

    /// Returns true if the sprint key is held (Ctrl in fly mode).
    #[must_use]
    pub fn is_sprinting(&self) -> bool {
        self.is_key_held(KeyCode::ControlLeft) || self.is_key_held(KeyCode::ControlRight)
    }

    /// Returns true if the crouch key is held.
    #[must_use]
    pub fn is_crouching(&self) -> bool {
        self.is_key_held(KeyCode::ControlLeft) || self.is_key_held(KeyCode::ControlRight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_state_default() {
        let input = InputState::new();
        assert!(!input.is_key_held(KeyCode::KeyW));
        assert!(!input.is_cursor_locked());
    }

    #[test]
    fn key_press_and_hold() {
        let mut input = InputState::new();

        input.key_pressed(KeyCode::KeyW);

        assert!(input.is_key_held(KeyCode::KeyW));
        assert!(input.is_key_just_pressed(KeyCode::KeyW));
        assert!(!input.is_key_just_released(KeyCode::KeyW));
    }

    #[test]
    fn key_release() {
        let mut input = InputState::new();

        input.key_pressed(KeyCode::KeyW);
        input.begin_frame();
        input.key_released(KeyCode::KeyW);

        assert!(!input.is_key_held(KeyCode::KeyW));
        assert!(!input.is_key_just_pressed(KeyCode::KeyW));
        assert!(input.is_key_just_released(KeyCode::KeyW));
    }

    #[test]
    fn just_pressed_only_once() {
        let mut input = InputState::new();

        input.key_pressed(KeyCode::KeyW);
        assert!(input.is_key_just_pressed(KeyCode::KeyW));

        input.begin_frame();
        // Key still held but not "just pressed"
        assert!(input.is_key_held(KeyCode::KeyW));
        assert!(!input.is_key_just_pressed(KeyCode::KeyW));
    }

    #[test]
    fn mouse_button_tracking() {
        let mut input = InputState::new();

        input.mouse_button_pressed(MouseButton::Left);
        assert!(input.is_mouse_held(MouseButton::Left));
        assert!(input.is_mouse_just_pressed(MouseButton::Left));

        input.begin_frame();
        input.mouse_button_released(MouseButton::Left);
        assert!(!input.is_mouse_held(MouseButton::Left));
        assert!(input.is_mouse_just_released(MouseButton::Left));
    }

    #[test]
    fn mouse_delta_accumulation() {
        let mut input = InputState::new();

        input.mouse_delta((10.0, 5.0));
        input.mouse_delta((3.0, -2.0));

        let delta = input.take_mouse_delta();
        assert!((delta.0 - 13.0).abs() < 0.001);
        assert!((delta.1 - 3.0).abs() < 0.001);

        // Should be reset after take
        let delta2 = input.take_mouse_delta();
        assert!((delta2.0).abs() < 0.001);
        assert!((delta2.1).abs() < 0.001);
    }

    #[test]
    fn movement_direction_wasd() {
        let mut input = InputState::new();

        input.key_pressed(KeyCode::KeyW);
        input.key_pressed(KeyCode::KeyD);

        let dir = input.movement_direction();
        assert!((dir.z - 1.0).abs() < 0.001); // Forward
        assert!((dir.x - 1.0).abs() < 0.001); // Right
        assert!((dir.y).abs() < 0.001); // No vertical
    }

    #[test]
    fn movement_direction_cancel() {
        let mut input = InputState::new();

        // Pressing W and S should cancel out
        input.key_pressed(KeyCode::KeyW);
        input.key_pressed(KeyCode::KeyS);

        let dir = input.movement_direction();
        assert!((dir.z).abs() < 0.001); // Cancelled
    }

    #[test]
    fn sprint_with_ctrl() {
        let mut input = InputState::new();

        assert!(!input.is_sprinting());

        input.key_pressed(KeyCode::ControlLeft);
        assert!(input.is_sprinting());

        input.key_released(KeyCode::ControlLeft);
        assert!(!input.is_sprinting());
    }

    #[test]
    fn scroll_delta() {
        let mut input = InputState::new();

        input.scroll((0.0, 1.0));
        input.scroll((0.0, 0.5));

        let scroll = input.get_scroll_delta();
        assert!((scroll.1 - 1.5).abs() < 0.001);

        input.begin_frame();
        let scroll2 = input.get_scroll_delta();
        assert!((scroll2.1).abs() < 0.001); // Reset
    }
}
