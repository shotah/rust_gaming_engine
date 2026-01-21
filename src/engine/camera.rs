//! Camera system module.
//!
//! Provides first-person camera with mouse look and movement,
//! plus frustum culling for efficient rendering.

use glam::{Mat4, Vec3, Vec4};

/// Axis-aligned bounding box for frustum testing.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    /// Minimum corner (lowest x, y, z).
    pub min: Vec3,
    /// Maximum corner (highest x, y, z).
    pub max: Vec3,
}

impl Aabb {
    /// Creates a new AABB from min and max corners.
    #[must_use]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates an AABB for a chunk at the given position.
    /// Chunks are 16x256x16 blocks.
    #[must_use]
    pub fn from_chunk(chunk_x: i32, chunk_z: i32) -> Self {
        let min = Vec3::new((chunk_x * 16) as f32, 0.0, (chunk_z * 16) as f32);
        let max = Vec3::new(
            (chunk_x * 16 + 16) as f32,
            256.0,
            (chunk_z * 16 + 16) as f32,
        );
        Self { min, max }
    }

    /// Returns the center point of the AABB.
    #[must_use]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the positive vertex relative to a plane normal.
    /// (The corner furthest in the direction of the normal.)
    #[must_use]
    fn positive_vertex(&self, normal: Vec3) -> Vec3 {
        Vec3::new(
            if normal.x >= 0.0 {
                self.max.x
            } else {
                self.min.x
            },
            if normal.y >= 0.0 {
                self.max.y
            } else {
                self.min.y
            },
            if normal.z >= 0.0 {
                self.max.z
            } else {
                self.min.z
            },
        )
    }

    /// Returns the negative vertex relative to a plane normal.
    /// (The corner closest in the direction of the normal.)
    #[must_use]
    #[allow(dead_code)] // Part of complete AABB algorithm, may be used for containment tests
    fn negative_vertex(&self, normal: Vec3) -> Vec3 {
        Vec3::new(
            if normal.x >= 0.0 {
                self.min.x
            } else {
                self.max.x
            },
            if normal.y >= 0.0 {
                self.min.y
            } else {
                self.max.y
            },
            if normal.z >= 0.0 {
                self.min.z
            } else {
                self.max.z
            },
        )
    }
}

/// A plane in 3D space, represented as ax + by + cz + d = 0.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector (a, b, c).
    pub normal: Vec3,
    /// Distance from origin (d).
    pub distance: f32,
}

impl Plane {
    /// Creates a plane from a Vec4 (normal.xyz, distance.w).
    #[must_use]
    pub fn from_vec4(v: Vec4) -> Self {
        let normal = Vec3::new(v.x, v.y, v.z);
        let length = normal.length();
        Self {
            normal: normal / length,
            distance: v.w / length,
        }
    }

    /// Returns the signed distance from a point to the plane.
    /// Positive = in front, Negative = behind.
    #[must_use]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// View frustum for culling - 6 planes extracted from view-projection matrix.
#[derive(Debug, Clone)]
pub struct Frustum {
    /// The 6 frustum planes: left, right, bottom, top, near, far.
    planes: [Plane; 6],
}

impl Frustum {
    /// Extracts frustum planes from a view-projection matrix.
    #[must_use]
    pub fn from_view_projection(vp: Mat4) -> Self {
        // Extract planes using Gribb/Hartmann method
        // Each row of the matrix gives us plane coefficients
        let row0 = vp.row(0);
        let row1 = vp.row(1);
        let row2 = vp.row(2);
        let row3 = vp.row(3);

        let planes = [
            Plane::from_vec4(row3 + row0), // Left
            Plane::from_vec4(row3 - row0), // Right
            Plane::from_vec4(row3 + row1), // Bottom
            Plane::from_vec4(row3 - row1), // Top
            Plane::from_vec4(row3 + row2), // Near
            Plane::from_vec4(row3 - row2), // Far
        ];

        Self { planes }
    }

    /// Tests if an AABB intersects or is inside the frustum.
    /// Returns true if visible, false if completely outside.
    #[must_use]
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        for plane in &self.planes {
            // Get the positive vertex (furthest in normal direction)
            let p_vertex = aabb.positive_vertex(plane.normal);

            // If positive vertex is behind the plane, AABB is completely outside
            if plane.distance_to_point(p_vertex) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Tests if a chunk at the given position is visible.
    #[must_use]
    pub fn is_chunk_visible(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let aabb = Aabb::from_chunk(chunk_x, chunk_z);
        self.intersects_aabb(&aabb)
    }
}

/// Camera configuration options.
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// Field of view in degrees.
    pub fov_degrees: f32,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
    /// Mouse sensitivity for looking around.
    pub sensitivity: f32,
    /// Base movement speed (units per second).
    pub move_speed: f32,
    /// Sprint speed multiplier.
    pub sprint_multiplier: f32,
    /// Crouch speed multiplier.
    pub crouch_multiplier: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            fov_degrees: 70.0,
            near: 0.1,
            far: 1000.0,
            sensitivity: 0.1,
            move_speed: 8.0,
            sprint_multiplier: 2.5,
            crouch_multiplier: 0.5,
        }
    }
}

/// First-person camera for navigating the world.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space.
    pub position: Vec3,
    /// Yaw rotation in degrees (horizontal look).
    pub yaw: f32,
    /// Pitch rotation in degrees (vertical look).
    pub pitch: f32,
    /// Camera configuration.
    config: CameraConfig,
    /// Cached aspect ratio.
    aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(CameraConfig::default())
    }
}

impl Camera {
    /// Creates a new camera with the given configuration.
    #[must_use]
    pub fn new(config: CameraConfig) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            yaw: -90.0, // Face negative Z by default
            pitch: 0.0,
            config,
            aspect_ratio: 16.0 / 9.0,
        }
    }

    /// Creates a camera at the specified position.
    #[must_use]
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Sets the aspect ratio (width / height).
    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        if height > 0.0 {
            self.aspect_ratio = width / height;
        }
    }

    /// Updates the camera rotation based on mouse delta movement.
    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += delta_x * self.config.sensitivity;
        self.pitch -= delta_y * self.config.sensitivity;

        // Clamp pitch to prevent camera flipping
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        // Normalize yaw to 0-360 range
        self.yaw = self.yaw.rem_euclid(360.0);
    }

    /// Returns the forward direction vector (where the camera is looking).
    #[must_use]
    pub fn forward(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        )
        .normalize()
    }

    /// Returns the right direction vector.
    #[must_use]
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Returns the up direction vector relative to camera orientation.
    #[must_use]
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    /// Returns the forward direction on the XZ plane (for walking).
    #[must_use]
    pub fn forward_xz(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        Vec3::new(yaw_rad.cos(), 0.0, yaw_rad.sin()).normalize()
    }

    /// Returns the right direction on the XZ plane (for strafing).
    #[must_use]
    pub fn right_xz(&self) -> Vec3 {
        self.forward_xz().cross(Vec3::Y).normalize()
    }

    /// Moves the camera based on input direction and delta time.
    ///
    /// # Arguments
    /// * `direction` - Movement direction (forward, right, up) in local space
    /// * `delta_time` - Time since last frame in seconds
    /// * `sprinting` - Whether the player is sprinting
    /// * `crouching` - Whether the player is crouching
    pub fn move_by(&mut self, direction: Vec3, delta_time: f32, sprinting: bool, crouching: bool) {
        let mut speed = self.config.move_speed;

        if sprinting {
            speed *= self.config.sprint_multiplier;
        } else if crouching {
            speed *= self.config.crouch_multiplier;
        }

        let velocity = direction.normalize_or_zero() * speed * delta_time;

        // Apply movement in world space (fly mode - moves in look direction)
        self.position += self.forward() * velocity.z; // Forward/back (including pitch)
        self.position += self.right() * velocity.x; // Left/right
        self.position += Vec3::Y * velocity.y; // Up/down (Space/Shift)
    }

    /// Returns the view matrix for rendering.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward(), Vec3::Y)
    }

    /// Returns the projection matrix for rendering.
    #[must_use]
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.config.fov_degrees.to_radians(),
            self.aspect_ratio,
            self.config.near,
            self.config.far,
        )
    }

    /// Returns the combined view-projection matrix.
    #[must_use]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Returns a reference to the camera configuration.
    #[must_use]
    pub const fn config(&self) -> &CameraConfig {
        &self.config
    }

    /// Returns a mutable reference to the camera configuration.
    pub fn config_mut(&mut self) -> &mut CameraConfig {
        &mut self.config
    }

    /// Returns the view frustum for culling.
    #[must_use]
    pub fn frustum(&self) -> Frustum {
        Frustum::from_view_projection(self.view_projection_matrix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.0001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec3_approx_eq(a: Vec3, b: Vec3) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z)
    }

    #[test]
    fn camera_default_position() {
        let camera = Camera::default();
        assert!(vec3_approx_eq(camera.position, Vec3::new(0.0, 0.0, 3.0)));
    }

    #[test]
    fn camera_at_position() {
        let camera = Camera::default().at_position(Vec3::new(10.0, 5.0, -20.0));
        assert!(vec3_approx_eq(camera.position, Vec3::new(10.0, 5.0, -20.0)));
    }

    #[test]
    fn camera_forward_default() {
        let camera = Camera::default();
        // With yaw=-90 and pitch=0, should face -Z
        let forward = camera.forward();
        assert!(approx_eq(forward.x, 0.0));
        assert!(approx_eq(forward.y, 0.0));
        assert!(approx_eq(forward.z, -1.0));
    }

    #[test]
    fn camera_rotation_pitch_clamp() {
        let mut camera = Camera::default();

        // Try to rotate pitch beyond limits
        camera.rotate(0.0, 1000.0); // Look way up
        assert!(camera.pitch >= -89.0 && camera.pitch <= 89.0);

        camera.rotate(0.0, -2000.0); // Look way down
        assert!(camera.pitch >= -89.0 && camera.pitch <= 89.0);
    }

    #[test]
    fn camera_right_perpendicular_to_forward() {
        let camera = Camera::default();
        let forward = camera.forward();
        let right = camera.right();

        // Dot product should be ~0 (perpendicular)
        let dot = forward.dot(right);
        assert!(approx_eq(dot, 0.0));
    }

    #[test]
    fn camera_movement() {
        let mut camera = Camera::default();
        let start_pos = camera.position;

        // Move forward for 1 second
        camera.move_by(Vec3::new(0.0, 0.0, 1.0), 1.0, false, false);

        // Should have moved forward by move_speed units
        let expected_offset = camera.forward_xz() * camera.config.move_speed;
        let actual_offset = camera.position - start_pos;

        assert!(vec3_approx_eq(actual_offset, expected_offset));
    }

    #[test]
    fn camera_sprint_faster() {
        let mut camera1 = Camera::default();
        let mut camera2 = Camera::default();

        camera1.move_by(Vec3::new(0.0, 0.0, 1.0), 1.0, false, false); // Normal
        camera2.move_by(Vec3::new(0.0, 0.0, 1.0), 1.0, true, false); // Sprinting

        let dist1 = (camera1.position - Vec3::new(0.0, 0.0, 3.0)).length();
        let dist2 = (camera2.position - Vec3::new(0.0, 0.0, 3.0)).length();

        assert!(dist2 > dist1);
    }

    #[test]
    fn camera_view_matrix_valid() {
        let camera = Camera::default();
        let view = camera.view_matrix();

        // View matrix should be invertible (determinant != 0)
        assert!(view.determinant().abs() > EPSILON);
    }

    #[test]
    fn camera_projection_matrix_valid() {
        let camera = Camera::default();
        let proj = camera.projection_matrix();

        // Projection matrix should be valid
        assert!(proj.determinant().abs() > EPSILON);
    }

    #[test]
    fn camera_aspect_ratio() {
        let mut camera = Camera::default();
        camera.set_aspect_ratio(1920.0, 1080.0);

        assert!(approx_eq(camera.aspect_ratio, 1920.0 / 1080.0));
    }

    #[test]
    fn frustum_chunk_in_front_is_visible() {
        // Camera at origin looking at -Z
        let camera = Camera::default().at_position(Vec3::new(0.0, 64.0, 50.0));
        let frustum = camera.frustum();

        // Chunk at (0, 0) should be visible (directly in front)
        assert!(frustum.is_chunk_visible(0, 0));
    }

    #[test]
    fn frustum_chunk_behind_is_not_visible() {
        // Camera at origin looking at -Z
        let camera = Camera::default().at_position(Vec3::new(0.0, 64.0, 0.0));
        let frustum = camera.frustum();

        // Chunk very far behind (positive Z) should not be visible
        assert!(!frustum.is_chunk_visible(0, 100));
    }

    #[test]
    fn frustum_chunk_to_side_is_not_visible() {
        let camera = Camera::default().at_position(Vec3::new(0.0, 64.0, 0.0));
        let frustum = camera.frustum();

        // Chunk very far to the side should not be visible
        assert!(!frustum.is_chunk_visible(100, -10));
    }

    #[test]
    fn aabb_from_chunk() {
        let aabb = Aabb::from_chunk(1, 2);
        assert!(vec3_approx_eq(aabb.min, Vec3::new(16.0, 0.0, 32.0)));
        assert!(vec3_approx_eq(aabb.max, Vec3::new(32.0, 256.0, 48.0)));
    }

    #[test]
    fn aabb_center() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
        assert!(vec3_approx_eq(aabb.center(), Vec3::new(5.0, 5.0, 5.0)));
    }

    #[test]
    fn plane_distance_to_point() {
        // Plane at Z=5, facing +Z
        let plane = Plane {
            normal: Vec3::Z,
            distance: -5.0,
        };

        // Point at Z=10 is 5 units in front
        let dist = plane.distance_to_point(Vec3::new(0.0, 0.0, 10.0));
        assert!(approx_eq(dist, 5.0));

        // Point at Z=0 is 5 units behind
        let dist = plane.distance_to_point(Vec3::new(0.0, 0.0, 0.0));
        assert!(approx_eq(dist, -5.0));
    }
}
