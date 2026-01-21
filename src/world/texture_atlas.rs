//! Texture atlas generation and management.
//!
//! Generates procedural textures for blocks and combines them into an atlas.

// Allow some patterns that are intentional for procedural generation
#![allow(
    clippy::cast_precision_loss,      // Intentional for noise/color generation
    clippy::cast_possible_truncation, // Pixel values are always in range
    clippy::cast_sign_loss,           // Pixel values are always positive
    clippy::cast_possible_wrap,       // Pixel coordinates are always small
    clippy::manual_is_multiple_of,    // Clearer as modulo for visual patterns
    clippy::cast_lossless,            // Using as for concise conversions
    clippy::suboptimal_flops,         // Readability over micro-optimization
    clippy::imprecise_flops,          // Precision not critical for visuals
    clippy::too_many_lines,           // Procedural texture gen is complex
)]

use super::block::Block;

/// Size of each texture in pixels.
pub const TEXTURE_SIZE: u32 = 16;

/// Number of textures per row in the atlas.
pub const ATLAS_COLUMNS: u32 = 8;

/// Number of texture rows in the atlas.
pub const ATLAS_ROWS: u32 = 4;

/// Total atlas size in pixels.
pub const ATLAS_SIZE: u32 = TEXTURE_SIZE * ATLAS_COLUMNS;

/// A texture atlas containing all block textures.
pub struct TextureAtlas {
    /// RGBA pixel data.
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl TextureAtlas {
    /// Creates a new texture atlas with procedurally generated block textures.
    #[must_use]
    pub fn generate() -> Self {
        let width = ATLAS_SIZE;
        let height = TEXTURE_SIZE * ATLAS_ROWS;
        let mut data = vec![0u8; (width * height * 4) as usize];

        // Generate texture for each block type
        for block_id in 0..=17u16 {
            if let Some(block) = Block::from_id(block_id) {
                let (atlas_x, atlas_y) = Self::block_atlas_position(block);
                Self::generate_block_texture(&mut data, width, atlas_x, atlas_y, block);
            }
        }

        Self {
            data,
            width,
            height,
        }
    }

    /// Returns the (column, row) position in the atlas for a block type.
    #[must_use]
    pub fn block_atlas_position(block: Block) -> (u32, u32) {
        let id = block.id() as u32;
        (id % ATLAS_COLUMNS, id / ATLAS_COLUMNS)
    }

    /// Returns UV coordinates for a block face.
    ///
    /// Returns `(u_min, v_min, u_max, v_max)` normalized to `[0, 1]`.
    #[must_use]
    pub fn block_uvs(block: Block) -> (f32, f32, f32, f32) {
        let (col, row) = Self::block_atlas_position(block);
        let u_min = col as f32 / ATLAS_COLUMNS as f32;
        let v_min = row as f32 / ATLAS_ROWS as f32;
        let u_max = (col + 1) as f32 / ATLAS_COLUMNS as f32;
        let v_max = (row + 1) as f32 / ATLAS_ROWS as f32;
        (u_min, v_min, u_max, v_max)
    }

    /// Generates a procedural texture for a block at the given atlas position.
    fn generate_block_texture(
        data: &mut [u8],
        atlas_width: u32,
        atlas_x: u32,
        atlas_y: u32,
        block: Block,
    ) {
        let base_color = block.color();
        let base_x = atlas_x * TEXTURE_SIZE;
        let base_y = atlas_y * TEXTURE_SIZE;

        for local_y in 0..TEXTURE_SIZE {
            for local_x in 0..TEXTURE_SIZE {
                let px = base_x + local_x;
                let py = base_y + local_y;
                let idx = ((py * atlas_width + px) * 4) as usize;

                let (r, g, b) = Self::generate_pixel(block, local_x, local_y, base_color);

                data[idx] = (r * 255.0) as u8;
                data[idx + 1] = (g * 255.0) as u8;
                data[idx + 2] = (b * 255.0) as u8;

                // Set alpha based on block type
                data[idx + 3] = match block {
                    Block::Air => 0,
                    Block::Leaves => {
                        // Cutout transparency - some pixels fully transparent
                        let noise = Self::hash_noise(local_x, local_y, 5);
                        if noise > 0.65 { 0 } else { 255 }
                    }
                    Block::Glass => 180, // Semi-transparent
                    Block::Water => 160, // Semi-transparent
                    _ => 255,            // Fully opaque
                };
            }
        }
    }

    /// Generates a pixel color for a specific block texture.
    fn generate_pixel(block: Block, x: u32, y: u32, base: [f32; 3]) -> (f32, f32, f32) {
        match block {
            Block::Air => (0.0, 0.0, 0.0),

            Block::Stone | Block::Cobblestone => {
                // Noisy gray stone texture
                let noise = Self::hash_noise(x, y, 0) * 0.15;
                let v = base[0] + noise - 0.075;
                (v, v, v)
            }

            Block::Dirt => {
                // Brown with darker spots
                let noise = Self::hash_noise(x, y, 1) * 0.2;
                (
                    base[0] + noise - 0.1,
                    base[1] + noise - 0.1,
                    base[2] + noise * 0.5 - 0.05,
                )
            }

            Block::Grass => {
                // Green top with variation
                let noise = Self::hash_noise(x, y, 2) * 0.15;
                (
                    base[0] + noise * 0.5 - 0.05,
                    base[1] + noise - 0.075,
                    base[2] + noise * 0.3 - 0.02,
                )
            }

            Block::Sand => {
                // Sandy tan with slight noise
                let noise = Self::hash_noise(x, y, 3) * 0.1;
                (
                    base[0] + noise - 0.05,
                    base[1] + noise - 0.05,
                    base[2] + noise * 0.5 - 0.025,
                )
            }

            Block::Gravel => {
                // Rough gray with larger noise
                let noise = Self::hash_noise(x, y, 4) * 0.25;
                let v = base[0] + noise - 0.125;
                (v, v, v)
            }

            Block::Log => {
                // Wood rings pattern
                let cx = (x as i32 - 8).abs() as f32;
                let cy = (y as i32 - 8).abs() as f32;
                let dist = (cx * cx + cy * cy).sqrt();
                let ring = ((dist * 0.8).sin() * 0.5 + 0.5) * 0.2;
                (base[0] + ring, base[1] + ring * 0.7, base[2] + ring * 0.3)
            }

            Block::Leaves => {
                // Leafy pattern with cutout holes
                let noise = Self::hash_noise(x, y, 5);
                // Return special marker for transparent pixels (will be handled in alpha)
                let v = noise * 0.2;
                (base[0] + v * 0.5, base[1] + v, base[2] + v * 0.3)
            }

            Block::Glass => {
                // Light blue-white with edge highlight
                let edge = (x == 0 || x == 15 || y == 0 || y == 15) as u8 as f32 * 0.2;
                (base[0] + edge, base[1] + edge, base[2])
            }

            Block::Water => {
                // Wavy blue
                let wave = ((x as f32 * 0.5 + y as f32 * 0.3).sin() * 0.5 + 0.5) * 0.2;
                (base[0] + wave * 0.3, base[1] + wave * 0.5, base[2] + wave)
            }

            Block::Planks => {
                // Wood grain pattern
                let grain = if y % 4 == 0 { 0.9 } else { 1.0 };
                let noise = Self::hash_noise(x, y, 6) * 0.1;
                (
                    (base[0] + noise) * grain,
                    (base[1] + noise * 0.7) * grain,
                    (base[2] + noise * 0.3) * grain,
                )
            }

            Block::Bricks => {
                // Brick pattern
                let brick_h = 4;
                let brick_w = 8;
                let offset = if (y / brick_h) % 2 == 0 {
                    0
                } else {
                    brick_w / 2
                };
                let is_mortar = y % brick_h == 0 || (x + offset) % brick_w == 0;
                if is_mortar {
                    (0.7, 0.7, 0.65) // Mortar color
                } else {
                    let noise = Self::hash_noise(x, y, 7) * 0.15;
                    (
                        base[0] + noise,
                        base[1] + noise * 0.5,
                        base[2] + noise * 0.3,
                    )
                }
            }

            Block::CoalOre | Block::IronOre | Block::GoldOre | Block::DiamondOre => {
                // Stone with ore spots
                let ore_noise = Self::hash_noise(x, y, block.id() as u32);
                if ore_noise > 0.75 {
                    // Ore spot
                    match block {
                        Block::CoalOre => (0.1, 0.1, 0.1),
                        Block::IronOre => (0.7, 0.6, 0.5),
                        Block::GoldOre => (1.0, 0.9, 0.2),
                        Block::DiamondOre => (0.3, 0.9, 1.0),
                        _ => (base[0], base[1], base[2]),
                    }
                } else {
                    // Stone background
                    let noise = Self::hash_noise(x, y, 0) * 0.15;
                    let v = 0.5 + noise - 0.075;
                    (v, v, v)
                }
            }

            Block::Bedrock => {
                // Very dark with cracks
                let noise = Self::hash_noise(x, y, 8) * 0.2;
                let v = base[0] + noise - 0.1;
                (v, v, v)
            }
        }
    }

    /// Simple hash-based noise function.
    fn hash_noise(x: u32, y: u32, seed: u32) -> f32 {
        let n = x
            .wrapping_mul(374761393)
            .wrapping_add(y.wrapping_mul(668265263))
            .wrapping_add(seed.wrapping_mul(1013904223));
        let n = n ^ (n >> 13);
        let n = n.wrapping_mul(1274126177);
        let n = n ^ (n >> 16);
        (n & 0xFFFF) as f32 / 65535.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_has_correct_size() {
        let atlas = TextureAtlas::generate();
        assert_eq!(atlas.width, ATLAS_SIZE);
        assert_eq!(atlas.height, TEXTURE_SIZE * ATLAS_ROWS);
        assert_eq!(atlas.data.len(), (atlas.width * atlas.height * 4) as usize);
    }

    #[test]
    fn block_uvs_are_normalized() {
        for id in 0..=17u16 {
            if let Some(block) = Block::from_id(id) {
                let (u_min, v_min, u_max, v_max) = TextureAtlas::block_uvs(block);
                assert!(u_min >= 0.0 && u_min <= 1.0);
                assert!(v_min >= 0.0 && v_min <= 1.0);
                assert!(u_max >= 0.0 && u_max <= 1.0);
                assert!(v_max >= 0.0 && v_max <= 1.0);
                assert!(u_max > u_min);
                assert!(v_max > v_min);
            }
        }
    }

    #[test]
    fn atlas_positions_are_unique() {
        let mut positions = std::collections::HashSet::new();
        for id in 0..=17u16 {
            if let Some(block) = Block::from_id(id) {
                let pos = TextureAtlas::block_atlas_position(block);
                assert!(positions.insert(pos), "Duplicate position for block {id}");
            }
        }
    }

    #[test]
    fn atlas_pixels_are_non_zero_for_solid_blocks() {
        let atlas = TextureAtlas::generate();
        // Check stone block (id=1) has non-zero pixels
        let (col, row) = TextureAtlas::block_atlas_position(Block::Stone);
        let px = col * TEXTURE_SIZE;
        let py = row * TEXTURE_SIZE;
        let idx = ((py * atlas.width + px) * 4) as usize;
        // At least one RGB channel should be non-zero
        assert!(atlas.data[idx] > 0 || atlas.data[idx + 1] > 0 || atlas.data[idx + 2] > 0);
    }
}
