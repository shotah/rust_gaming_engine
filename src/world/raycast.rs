//! Ray casting for block selection.
//!
//! Uses DDA (Digital Differential Analyzer) algorithm to efficiently
//! step through the voxel grid and find block intersections.

// Allow patterns that are clearer for DDA algorithm
#![allow(
    clippy::while_float,
    clippy::manual_range_contains,
    clippy::cast_precision_loss
)]

use glam::Vec3;

/// Result of a ray cast hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    /// World position of the hit block.
    pub block_pos: BlockPos,
    /// The face of the block that was hit.
    pub face: HitFace,
    /// Distance from ray origin to hit point.
    pub distance: f32,
    /// Exact world position of the hit point.
    pub hit_point: Vec3,
}

/// A block position in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    /// Creates a new block position.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the position of the adjacent block in the given direction.
    #[must_use]
    pub const fn offset(&self, face: HitFace) -> Self {
        match face {
            HitFace::Top => Self::new(self.x, self.y + 1, self.z),
            HitFace::Bottom => Self::new(self.x, self.y - 1, self.z),
            HitFace::North => Self::new(self.x, self.y, self.z - 1),
            HitFace::South => Self::new(self.x, self.y, self.z + 1),
            HitFace::East => Self::new(self.x + 1, self.y, self.z),
            HitFace::West => Self::new(self.x - 1, self.y, self.z),
        }
    }

    /// Converts to chunk position (which chunk contains this block).
    #[must_use]
    pub const fn to_chunk_pos(&self) -> (i32, i32) {
        // Integer division that floors for negative numbers
        let chunk_x = if self.x >= 0 {
            self.x / 16
        } else {
            (self.x - 15) / 16
        };
        let chunk_z = if self.z >= 0 {
            self.z / 16
        } else {
            (self.z - 15) / 16
        };
        (chunk_x, chunk_z)
    }

    /// Converts to local coordinates within a chunk.
    #[must_use]
    pub const fn to_local(&self) -> (usize, usize, usize) {
        let local_x = self.x.rem_euclid(16) as usize;
        let local_z = self.z.rem_euclid(16) as usize;
        (local_x, self.y as usize, local_z)
    }
}

/// The face of a block that was hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitFace {
    /// +Y (top)
    Top,
    /// -Y (bottom)
    Bottom,
    /// -Z (north)
    North,
    /// +Z (south)
    South,
    /// +X (east)
    East,
    /// -X (west)
    West,
}

impl HitFace {
    /// Returns the normal vector for this face.
    #[must_use]
    pub const fn normal(&self) -> [f32; 3] {
        match self {
            Self::Top => [0.0, 1.0, 0.0],
            Self::Bottom => [0.0, -1.0, 0.0],
            Self::North => [0.0, 0.0, -1.0],
            Self::South => [0.0, 0.0, 1.0],
            Self::East => [1.0, 0.0, 0.0],
            Self::West => [-1.0, 0.0, 0.0],
        }
    }
}

/// Casts a ray through the voxel world using DDA algorithm.
///
/// # Arguments
/// * `origin` - Ray start position
/// * `direction` - Ray direction (should be normalized)
/// * `max_distance` - Maximum distance to check
/// * `is_solid` - Function to check if a block at (x, y, z) is solid
///
/// # Returns
/// The first solid block hit, or None if no hit within max_distance.
pub fn raycast<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    is_solid: F,
) -> Option<RaycastHit>
where
    F: Fn(i32, i32, i32) -> bool,
{
    // Handle zero direction components to avoid division by zero
    let dir = direction.normalize();

    // Current block position
    let mut block_x = origin.x.floor() as i32;
    let mut block_y = origin.y.floor() as i32;
    let mut block_z = origin.z.floor() as i32;

    // Step direction (+1 or -1)
    let step_x = if dir.x >= 0.0 { 1 } else { -1 };
    let step_y = if dir.y >= 0.0 { 1 } else { -1 };
    let step_z = if dir.z >= 0.0 { 1 } else { -1 };

    // Distance along ray to next block boundary
    let t_delta_x = if dir.x.abs() < 1e-10 {
        f32::INFINITY
    } else {
        (1.0 / dir.x).abs()
    };
    let t_delta_y = if dir.y.abs() < 1e-10 {
        f32::INFINITY
    } else {
        (1.0 / dir.y).abs()
    };
    let t_delta_z = if dir.z.abs() < 1e-10 {
        f32::INFINITY
    } else {
        (1.0 / dir.z).abs()
    };

    // Distance to first boundary
    let mut t_max_x = if dir.x >= 0.0 {
        ((block_x + 1) as f32 - origin.x) * t_delta_x
    } else {
        (origin.x - block_x as f32) * t_delta_x
    };
    let mut t_max_y = if dir.y >= 0.0 {
        ((block_y + 1) as f32 - origin.y) * t_delta_y
    } else {
        (origin.y - block_y as f32) * t_delta_y
    };
    let mut t_max_z = if dir.z >= 0.0 {
        ((block_z + 1) as f32 - origin.z) * t_delta_z
    } else {
        (origin.z - block_z as f32) * t_delta_z
    };

    let mut distance = 0.0;
    let mut last_face = HitFace::Top;

    // Step through grid
    while distance < max_distance {
        // Check if current block is solid
        if block_y >= 0 && block_y < 256 && is_solid(block_x, block_y, block_z) {
            let hit_point = origin + dir * distance;
            return Some(RaycastHit {
                block_pos: BlockPos::new(block_x, block_y, block_z),
                face: last_face,
                distance,
                hit_point,
            });
        }

        // Move to next block
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                block_x += step_x;
                distance = t_max_x;
                t_max_x += t_delta_x;
                last_face = if step_x > 0 {
                    HitFace::West
                } else {
                    HitFace::East
                };
            } else {
                block_z += step_z;
                distance = t_max_z;
                t_max_z += t_delta_z;
                last_face = if step_z > 0 {
                    HitFace::North
                } else {
                    HitFace::South
                };
            }
        } else if t_max_y < t_max_z {
            block_y += step_y;
            distance = t_max_y;
            t_max_y += t_delta_y;
            last_face = if step_y > 0 {
                HitFace::Bottom
            } else {
                HitFace::Top
            };
        } else {
            block_z += step_z;
            distance = t_max_z;
            t_max_z += t_delta_z;
            last_face = if step_z > 0 {
                HitFace::North
            } else {
                HitFace::South
            };
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raycast_hits_block_ahead() {
        // Block at (5, 0, 0)
        let is_solid = |x, y, z| x == 5 && y == 0 && z == 0;

        let hit = raycast(
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(1.0, 0.0, 0.0),
            10.0,
            is_solid,
        );

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.block_pos, BlockPos::new(5, 0, 0));
        assert_eq!(hit.face, HitFace::West);
    }

    #[test]
    fn raycast_hits_block_below() {
        // Ground at y=0
        let is_solid = |_x, y, _z| y == 0;

        let hit = raycast(
            Vec3::new(0.5, 5.5, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
            10.0,
            is_solid,
        );

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.block_pos.y, 0);
        assert_eq!(hit.face, HitFace::Top);
    }

    #[test]
    fn raycast_misses_when_no_blocks() {
        let is_solid = |_x, _y, _z| false;

        let hit = raycast(
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(1.0, 0.0, 0.0),
            10.0,
            is_solid,
        );

        assert!(hit.is_none());
    }

    #[test]
    fn raycast_respects_max_distance() {
        // Block at (100, 0, 0)
        let is_solid = |x, y, z| x == 100 && y == 0 && z == 0;

        let hit = raycast(
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(1.0, 0.0, 0.0),
            10.0, // Max distance is 10, block is at 100
            is_solid,
        );

        assert!(hit.is_none());
    }

    #[test]
    fn block_pos_to_chunk() {
        assert_eq!(BlockPos::new(0, 0, 0).to_chunk_pos(), (0, 0));
        assert_eq!(BlockPos::new(15, 0, 15).to_chunk_pos(), (0, 0));
        assert_eq!(BlockPos::new(16, 0, 16).to_chunk_pos(), (1, 1));
        assert_eq!(BlockPos::new(-1, 0, -1).to_chunk_pos(), (-1, -1));
        assert_eq!(BlockPos::new(-16, 0, -16).to_chunk_pos(), (-1, -1));
        assert_eq!(BlockPos::new(-17, 0, -17).to_chunk_pos(), (-2, -2));
    }

    #[test]
    fn block_pos_to_local() {
        assert_eq!(BlockPos::new(0, 5, 0).to_local(), (0, 5, 0));
        assert_eq!(BlockPos::new(15, 5, 15).to_local(), (15, 5, 15));
        assert_eq!(BlockPos::new(16, 5, 16).to_local(), (0, 5, 0));
        assert_eq!(BlockPos::new(-1, 5, -1).to_local(), (15, 5, 15));
    }

    #[test]
    fn block_pos_offset() {
        let pos = BlockPos::new(5, 10, 5);
        assert_eq!(pos.offset(HitFace::Top), BlockPos::new(5, 11, 5));
        assert_eq!(pos.offset(HitFace::Bottom), BlockPos::new(5, 9, 5));
        assert_eq!(pos.offset(HitFace::North), BlockPos::new(5, 10, 4));
        assert_eq!(pos.offset(HitFace::South), BlockPos::new(5, 10, 6));
        assert_eq!(pos.offset(HitFace::East), BlockPos::new(6, 10, 5));
        assert_eq!(pos.offset(HitFace::West), BlockPos::new(4, 10, 5));
    }
}
