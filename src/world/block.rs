//! Block definitions and registry.
//!
//! Defines all block types and their properties.

/// Unique identifier for a block type.
pub type BlockId = u16;

/// Block type enumeration.
///
/// Each variant represents a different block type in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u16)]
pub enum Block {
    /// Empty space (air).
    #[default]
    Air = 0,
    /// Basic stone block.
    Stone = 1,
    /// Dirt block.
    Dirt = 2,
    /// Grass block (dirt with grass on top).
    Grass = 3,
    /// Sand block.
    Sand = 4,
    /// Gravel block.
    Gravel = 5,
    /// Wood log block.
    Log = 6,
    /// Leaves block (transparent).
    Leaves = 7,
    /// Glass block (transparent).
    Glass = 8,
    /// Water block (transparent, liquid).
    Water = 9,
    /// Cobblestone block.
    Cobblestone = 10,
    /// Wooden planks.
    Planks = 11,
    /// Brick block.
    Bricks = 12,
    /// Coal ore.
    CoalOre = 13,
    /// Iron ore.
    IronOre = 14,
    /// Gold ore.
    GoldOre = 15,
    /// Diamond ore.
    DiamondOre = 16,
    /// Bedrock (unbreakable).
    Bedrock = 17,
}

impl Block {
    /// Returns the block ID as a u16.
    #[must_use]
    pub const fn id(self) -> BlockId {
        self as BlockId
    }

    /// Creates a block from its ID.
    #[must_use]
    pub const fn from_id(id: BlockId) -> Option<Self> {
        match id {
            0 => Some(Self::Air),
            1 => Some(Self::Stone),
            2 => Some(Self::Dirt),
            3 => Some(Self::Grass),
            4 => Some(Self::Sand),
            5 => Some(Self::Gravel),
            6 => Some(Self::Log),
            7 => Some(Self::Leaves),
            8 => Some(Self::Glass),
            9 => Some(Self::Water),
            10 => Some(Self::Cobblestone),
            11 => Some(Self::Planks),
            12 => Some(Self::Bricks),
            13 => Some(Self::CoalOre),
            14 => Some(Self::IronOre),
            15 => Some(Self::GoldOre),
            16 => Some(Self::DiamondOre),
            17 => Some(Self::Bedrock),
            _ => None,
        }
    }

    /// Returns the properties for this block type.
    #[must_use]
    pub const fn properties(self) -> BlockProperties {
        match self {
            Self::Air => BlockProperties::AIR,
            Self::Stone => BlockProperties::SOLID,
            Self::Dirt => BlockProperties::SOLID,
            Self::Grass => BlockProperties::SOLID,
            Self::Sand => BlockProperties::SOLID,
            Self::Gravel => BlockProperties::SOLID,
            Self::Log => BlockProperties::SOLID,
            Self::Leaves => BlockProperties::TRANSPARENT,
            Self::Glass => BlockProperties::TRANSPARENT,
            Self::Water => BlockProperties::LIQUID,
            Self::Cobblestone => BlockProperties::SOLID,
            Self::Planks => BlockProperties::SOLID,
            Self::Bricks => BlockProperties::SOLID,
            Self::CoalOre => BlockProperties::SOLID,
            Self::IronOre => BlockProperties::SOLID,
            Self::GoldOre => BlockProperties::SOLID,
            Self::DiamondOre => BlockProperties::SOLID,
            Self::Bedrock => BlockProperties::UNBREAKABLE,
        }
    }

    /// Returns true if this block is solid (blocks movement and light).
    #[must_use]
    pub const fn is_solid(self) -> bool {
        self.properties().is_solid
    }

    /// Returns true if this block is transparent (light passes through).
    #[must_use]
    pub const fn is_transparent(self) -> bool {
        self.properties().is_transparent
    }

    /// Returns true if this block is air (empty space).
    #[must_use]
    pub const fn is_air(self) -> bool {
        matches!(self, Self::Air)
    }

    /// Returns the color for this block (temporary until textures).
    #[must_use]
    pub const fn color(self) -> [f32; 3] {
        match self {
            Self::Air => [0.0, 0.0, 0.0],
            Self::Stone => [0.5, 0.5, 0.5],
            Self::Dirt => [0.55, 0.35, 0.2],
            Self::Grass => [0.3, 0.6, 0.2],
            Self::Sand => [0.9, 0.85, 0.6],
            Self::Gravel => [0.55, 0.55, 0.55],
            Self::Log => [0.4, 0.25, 0.1],
            Self::Leaves => [0.2, 0.5, 0.15],
            Self::Glass => [0.8, 0.9, 1.0],
            Self::Water => [0.2, 0.4, 0.8],
            Self::Cobblestone => [0.45, 0.45, 0.45],
            Self::Planks => [0.7, 0.5, 0.3],
            Self::Bricks => [0.6, 0.3, 0.25],
            Self::CoalOre => [0.3, 0.3, 0.3],
            Self::IronOre => [0.6, 0.55, 0.5],
            Self::GoldOre => [0.9, 0.8, 0.3],
            Self::DiamondOre => [0.4, 0.8, 0.9],
            Self::Bedrock => [0.2, 0.2, 0.2],
        }
    }
}

/// Properties that define block behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockProperties {
    /// Whether the block is solid (blocks movement).
    pub is_solid: bool,
    /// Whether the block is transparent (light passes through).
    pub is_transparent: bool,
    /// Whether the block emits light (0-15).
    pub light_emission: u8,
    /// Whether the block can be broken.
    pub is_breakable: bool,
    /// Whether the block is a liquid.
    pub is_liquid: bool,
}

impl BlockProperties {
    /// Air properties - not solid, fully transparent.
    pub const AIR: Self = Self {
        is_solid: false,
        is_transparent: true,
        light_emission: 0,
        is_breakable: false,
        is_liquid: false,
    };

    /// Solid opaque block properties.
    pub const SOLID: Self = Self {
        is_solid: true,
        is_transparent: false,
        light_emission: 0,
        is_breakable: true,
        is_liquid: false,
    };

    /// Transparent solid block properties (glass, leaves).
    pub const TRANSPARENT: Self = Self {
        is_solid: true,
        is_transparent: true,
        light_emission: 0,
        is_breakable: true,
        is_liquid: false,
    };

    /// Liquid block properties.
    pub const LIQUID: Self = Self {
        is_solid: false,
        is_transparent: true,
        light_emission: 0,
        is_breakable: false,
        is_liquid: true,
    };

    /// Unbreakable block properties (bedrock).
    pub const UNBREAKABLE: Self = Self {
        is_solid: true,
        is_transparent: false,
        light_emission: 0,
        is_breakable: false,
        is_liquid: false,
    };

    /// Light-emitting block properties.
    #[must_use]
    pub const fn with_light(mut self, level: u8) -> Self {
        self.light_emission = level;
        self
    }
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self::SOLID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_id_roundtrip() {
        for id in 0..=17 {
            let block = Block::from_id(id).unwrap();
            assert_eq!(block.id(), id);
        }
    }

    #[test]
    fn invalid_block_id() {
        assert!(Block::from_id(255).is_none());
        assert!(Block::from_id(1000).is_none());
    }

    #[test]
    fn air_is_transparent() {
        assert!(Block::Air.is_transparent());
        assert!(!Block::Air.is_solid());
        assert!(Block::Air.is_air());
    }

    #[test]
    fn stone_is_solid() {
        assert!(Block::Stone.is_solid());
        assert!(!Block::Stone.is_transparent());
        assert!(!Block::Stone.is_air());
    }

    #[test]
    fn glass_is_transparent_but_solid() {
        assert!(Block::Glass.is_solid());
        assert!(Block::Glass.is_transparent());
    }

    #[test]
    fn water_is_liquid() {
        assert!(Block::Water.properties().is_liquid);
        assert!(!Block::Water.is_solid());
        assert!(Block::Water.is_transparent());
    }

    #[test]
    fn bedrock_is_unbreakable() {
        assert!(!Block::Bedrock.properties().is_breakable);
        assert!(Block::Bedrock.is_solid());
    }

    #[test]
    fn block_colors_are_valid() {
        for id in 0..=17 {
            let block = Block::from_id(id).unwrap();
            let color = block.color();
            for component in color {
                assert!((0.0..=1.0).contains(&component));
            }
        }
    }
}
