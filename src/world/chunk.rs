//! Chunk data structure.
//!
//! A chunk is a 16x16 column of the world, divided into 16x16x16 sections.

use super::block::Block;

/// Size of a chunk section in each dimension.
pub const SECTION_SIZE: usize = 16;

/// Number of blocks in a section.
pub const SECTION_VOLUME: usize = SECTION_SIZE * SECTION_SIZE * SECTION_SIZE;

/// Number of sections in a chunk (height = 256 blocks = 16 sections).
pub const SECTIONS_PER_CHUNK: usize = 16;

/// Total height of a chunk in blocks.
pub const CHUNK_HEIGHT: usize = SECTION_SIZE * SECTIONS_PER_CHUNK;

/// A 16x16x16 section of blocks within a chunk.
#[derive(Clone)]
pub struct ChunkSection {
    /// Block data stored as a flat array.
    /// Index = x + z * 16 + y * 256
    blocks: Box<[Block; SECTION_VOLUME]>,
    /// Number of non-air blocks in this section.
    solid_count: u32,
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkSection {
    /// Creates a new empty (all air) section.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blocks: Box::new([Block::Air; SECTION_VOLUME]),
            solid_count: 0,
        }
    }

    /// Creates a section filled with the specified block.
    #[must_use]
    pub fn filled(block: Block) -> Self {
        let solid_count = if block.is_air() {
            0
        } else {
            SECTION_VOLUME as u32
        };
        Self {
            blocks: Box::new([block; SECTION_VOLUME]),
            solid_count,
        }
    }

    /// Converts local coordinates to array index.
    #[inline]
    const fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < SECTION_SIZE);
        debug_assert!(y < SECTION_SIZE);
        debug_assert!(z < SECTION_SIZE);
        x + z * SECTION_SIZE + y * SECTION_SIZE * SECTION_SIZE
    }

    /// Gets the block at the given local coordinates.
    #[inline]
    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Self::index(x, y, z)]
    }

    /// Sets the block at the given local coordinates.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let idx = Self::index(x, y, z);
        let old = self.blocks[idx];

        // Update solid count
        if old.is_air() && !block.is_air() {
            self.solid_count += 1;
        } else if !old.is_air() && block.is_air() {
            self.solid_count -= 1;
        }

        self.blocks[idx] = block;
    }

    /// Returns true if this section is empty (all air).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.solid_count == 0
    }

    /// Returns the number of non-air blocks.
    #[must_use]
    pub const fn solid_count(&self) -> u32 {
        self.solid_count
    }

    /// Returns an iterator over all blocks with their local coordinates.
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, usize, Block)> + '_ {
        self.blocks.iter().enumerate().map(|(idx, &block)| {
            let x = idx % SECTION_SIZE;
            let z = (idx / SECTION_SIZE) % SECTION_SIZE;
            let y = idx / (SECTION_SIZE * SECTION_SIZE);
            (x, y, z, block)
        })
    }

    /// Returns a reference to the raw block data.
    #[must_use]
    pub fn blocks(&self) -> &[Block; SECTION_VOLUME] {
        &self.blocks
    }
}

/// Chunk position in the world (chunk coordinates, not block coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    /// X coordinate (chunk units).
    pub x: i32,
    /// Z coordinate (chunk units).
    pub z: i32,
}

impl ChunkPos {
    /// Creates a new chunk position.
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Converts world block coordinates to chunk position.
    #[must_use]
    pub const fn from_block(block_x: i32, block_z: i32) -> Self {
        Self {
            x: block_x.div_euclid(SECTION_SIZE as i32),
            z: block_z.div_euclid(SECTION_SIZE as i32),
        }
    }

    /// Converts floating-point world position to chunk position.
    #[must_use]
    pub fn from_world_pos(x: f32, z: f32) -> Self {
        Self::from_block(x.floor() as i32, z.floor() as i32)
    }

    /// Returns the world block coordinates of the chunk's origin (min corner).
    #[must_use]
    pub const fn block_origin(&self) -> (i32, i32) {
        (self.x * SECTION_SIZE as i32, self.z * SECTION_SIZE as i32)
    }

    /// Returns neighboring chunk positions.
    #[must_use]
    pub const fn neighbors(&self) -> [ChunkPos; 4] {
        [
            Self::new(self.x + 1, self.z),
            Self::new(self.x - 1, self.z),
            Self::new(self.x, self.z + 1),
            Self::new(self.x, self.z - 1),
        ]
    }
}

/// A full chunk column containing multiple sections.
#[derive(Clone)]
pub struct Chunk {
    /// The position of this chunk in the world.
    position: ChunkPos,
    /// The sections in this chunk (bottom to top).
    sections: Vec<Option<ChunkSection>>,
    /// Whether the chunk mesh needs to be rebuilt.
    dirty: bool,
}

impl Chunk {
    /// Creates a new empty chunk at the given position.
    #[must_use]
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            sections: vec![None; SECTIONS_PER_CHUNK],
            dirty: true,
        }
    }

    /// Returns the chunk position.
    #[must_use]
    pub const fn position(&self) -> ChunkPos {
        self.position
    }

    /// Gets the block at world-relative coordinates within this chunk.
    ///
    /// Coordinates are relative to the chunk (0-15 for x/z, 0-255 for y).
    #[must_use]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        debug_assert!(x < SECTION_SIZE);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < SECTION_SIZE);

        let section_y = y / SECTION_SIZE;
        let local_y = y % SECTION_SIZE;

        self.sections
            .get(section_y)
            .and_then(|s| s.as_ref())
            .map_or(Block::Air, |section| section.get(x, local_y, z))
    }

    /// Sets the block at chunk-relative coordinates.
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        debug_assert!(x < SECTION_SIZE);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < SECTION_SIZE);

        let section_y = y / SECTION_SIZE;
        let local_y = y % SECTION_SIZE;

        // Create section if it doesn't exist and we're placing a non-air block
        if self.sections[section_y].is_none() {
            if block.is_air() {
                return; // No need to create section for air
            }
            self.sections[section_y] = Some(ChunkSection::new());
        }

        if let Some(ref mut section) = self.sections[section_y] {
            section.set(x, local_y, z, block);

            // Remove empty sections to save memory
            if section.is_empty() {
                self.sections[section_y] = None;
            }
        }

        self.dirty = true;
    }

    /// Returns true if the chunk mesh needs to be rebuilt.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the chunk as clean (mesh is up to date).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Marks the chunk as dirty (mesh needs rebuild).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Returns the section at the given Y index (0-15).
    #[must_use]
    pub fn get_section(&self, section_y: usize) -> Option<&ChunkSection> {
        self.sections.get(section_y).and_then(|s| s.as_ref())
    }

    /// Returns a mutable reference to the section at the given Y index.
    pub fn get_section_mut(&mut self, section_y: usize) -> Option<&mut ChunkSection> {
        self.sections.get_mut(section_y).and_then(|s| s.as_mut())
    }

    /// Fills the chunk with a simple test pattern.
    pub fn fill_test_pattern(&mut self) {
        // Create a flat grass surface at y=64
        for x in 0..SECTION_SIZE {
            for z in 0..SECTION_SIZE {
                // Bedrock at bottom
                self.set_block(x, 0, z, Block::Bedrock);

                // Stone layers
                for y in 1..60 {
                    self.set_block(x, y, z, Block::Stone);
                }

                // Dirt layers
                for y in 60..64 {
                    self.set_block(x, y, z, Block::Dirt);
                }

                // Grass on top
                self.set_block(x, 64, z, Block::Grass);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_index_calculation() {
        // Test corner cases
        assert_eq!(ChunkSection::index(0, 0, 0), 0);
        assert_eq!(ChunkSection::index(15, 0, 0), 15);
        assert_eq!(ChunkSection::index(0, 0, 15), 15 * 16);
        assert_eq!(ChunkSection::index(0, 15, 0), 15 * 16 * 16);
        assert_eq!(ChunkSection::index(15, 15, 15), SECTION_VOLUME - 1);
    }

    #[test]
    fn section_get_set() {
        let mut section = ChunkSection::new();

        assert_eq!(section.get(5, 5, 5), Block::Air);
        assert!(section.is_empty());

        section.set(5, 5, 5, Block::Stone);
        assert_eq!(section.get(5, 5, 5), Block::Stone);
        assert!(!section.is_empty());
        assert_eq!(section.solid_count(), 1);

        section.set(5, 5, 5, Block::Air);
        assert_eq!(section.get(5, 5, 5), Block::Air);
        assert!(section.is_empty());
    }

    #[test]
    fn section_filled() {
        let section = ChunkSection::filled(Block::Stone);
        assert_eq!(section.solid_count(), SECTION_VOLUME as u32);
        assert_eq!(section.get(0, 0, 0), Block::Stone);
        assert_eq!(section.get(15, 15, 15), Block::Stone);
    }

    #[test]
    fn chunk_pos_from_block() {
        assert_eq!(ChunkPos::from_block(0, 0), ChunkPos::new(0, 0));
        assert_eq!(ChunkPos::from_block(15, 15), ChunkPos::new(0, 0));
        assert_eq!(ChunkPos::from_block(16, 0), ChunkPos::new(1, 0));
        assert_eq!(ChunkPos::from_block(-1, 0), ChunkPos::new(-1, 0));
        assert_eq!(ChunkPos::from_block(-16, 0), ChunkPos::new(-1, 0));
        assert_eq!(ChunkPos::from_block(-17, 0), ChunkPos::new(-2, 0));
    }

    #[test]
    fn chunk_get_set_block() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        assert_eq!(chunk.get_block(5, 100, 5), Block::Air);

        chunk.set_block(5, 100, 5, Block::Stone);
        assert_eq!(chunk.get_block(5, 100, 5), Block::Stone);
        assert!(chunk.is_dirty());

        chunk.mark_clean();
        assert!(!chunk.is_dirty());

        chunk.set_block(5, 100, 5, Block::Air);
        assert!(chunk.is_dirty());
    }

    #[test]
    fn chunk_lazy_section_creation() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        // Setting air shouldn't create a section
        chunk.set_block(5, 100, 5, Block::Air);
        assert!(chunk.get_section(100 / SECTION_SIZE).is_none());

        // Setting stone should create a section
        chunk.set_block(5, 100, 5, Block::Stone);
        assert!(chunk.get_section(100 / SECTION_SIZE).is_some());

        // Removing the block should remove the section
        chunk.set_block(5, 100, 5, Block::Air);
        assert!(chunk.get_section(100 / SECTION_SIZE).is_none());
    }

    #[test]
    fn chunk_block_origin() {
        let pos = ChunkPos::new(3, -2);
        let (bx, bz) = pos.block_origin();
        assert_eq!(bx, 48);
        assert_eq!(bz, -32);
    }

    #[test]
    fn chunk_pos_neighbors() {
        let pos = ChunkPos::new(5, 10);
        let neighbors = pos.neighbors();

        assert!(neighbors.contains(&ChunkPos::new(6, 10))); // +X
        assert!(neighbors.contains(&ChunkPos::new(4, 10))); // -X
        assert!(neighbors.contains(&ChunkPos::new(5, 11))); // +Z
        assert!(neighbors.contains(&ChunkPos::new(5, 9))); // -Z
        assert_eq!(neighbors.len(), 4);
    }

    #[test]
    fn section_iterator() {
        let mut section = ChunkSection::new();
        section.set(1, 2, 3, Block::Stone);
        section.set(5, 5, 5, Block::Dirt);

        let blocks: Vec<_> = section.iter().filter(|(_, _, _, b)| !b.is_air()).collect();
        assert_eq!(blocks.len(), 2);

        // Check specific positions
        assert!(
            blocks
                .iter()
                .any(|(x, y, z, b)| *x == 1 && *y == 2 && *z == 3 && *b == Block::Stone)
        );
        assert!(
            blocks
                .iter()
                .any(|(x, y, z, b)| *x == 5 && *y == 5 && *z == 5 && *b == Block::Dirt)
        );
    }

    #[test]
    fn section_blocks_reference() {
        let section = ChunkSection::filled(Block::Grass);
        let blocks = section.blocks();
        assert_eq!(blocks.len(), SECTION_VOLUME);
        assert_eq!(blocks[0], Block::Grass);
    }

    #[test]
    fn chunk_multiple_sections() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        // Set blocks in different sections
        chunk.set_block(0, 0, 0, Block::Bedrock); // Section 0
        chunk.set_block(0, 16, 0, Block::Stone); // Section 1
        chunk.set_block(0, 64, 0, Block::Grass); // Section 4
        chunk.set_block(0, 255, 0, Block::Glass); // Section 15

        assert_eq!(chunk.get_block(0, 0, 0), Block::Bedrock);
        assert_eq!(chunk.get_block(0, 16, 0), Block::Stone);
        assert_eq!(chunk.get_block(0, 64, 0), Block::Grass);
        assert_eq!(chunk.get_block(0, 255, 0), Block::Glass);

        // Intermediate sections should still be empty
        assert!(chunk.get_section(2).is_none());
        assert!(chunk.get_section(3).is_none());
    }

    #[test]
    fn chunk_test_pattern_creates_terrain() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        chunk.fill_test_pattern();

        // Should have bedrock at bottom
        assert_eq!(chunk.get_block(0, 0, 0), Block::Bedrock);
        assert_eq!(chunk.get_block(8, 0, 8), Block::Bedrock);

        // Should have stone below surface
        assert_eq!(chunk.get_block(0, 30, 0), Block::Stone);

        // Should have grass at surface
        assert_eq!(chunk.get_block(0, 64, 0), Block::Grass);

        // Should have air above surface
        assert_eq!(chunk.get_block(0, 65, 0), Block::Air);
    }

    #[test]
    fn chunk_dirty_flag() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));

        // New chunks start dirty
        assert!(chunk.is_dirty());

        chunk.mark_clean();
        assert!(!chunk.is_dirty());

        chunk.mark_dirty();
        assert!(chunk.is_dirty());
    }

    #[test]
    fn section_solid_count_accuracy() {
        let mut section = ChunkSection::new();

        // Add 10 blocks
        for i in 0..10 {
            section.set(i, 0, 0, Block::Stone);
        }
        assert_eq!(section.solid_count(), 10);

        // Replace one with different block (should stay same count)
        section.set(0, 0, 0, Block::Dirt);
        assert_eq!(section.solid_count(), 10);

        // Remove 5 blocks
        for i in 0..5 {
            section.set(i, 0, 0, Block::Air);
        }
        assert_eq!(section.solid_count(), 5);
    }

    #[test]
    fn chunk_position_equality() {
        let pos1 = ChunkPos::new(5, -3);
        let pos2 = ChunkPos::new(5, -3);
        let pos3 = ChunkPos::new(5, -4);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn section_default_is_empty() {
        let section = ChunkSection::default();
        assert!(section.is_empty());
        assert_eq!(section.solid_count(), 0);
    }
}
