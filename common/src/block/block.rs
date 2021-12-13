use crate::block::blockregistry::*;

// Air is a special case with ID 0
pub const AIR_BLOCK: u16 = 0;
// Solid (not passable) blocks have an ID from 256 or higher (this partially overlaps with translucent block IDs)
pub const SOLID_MIN_ID: u16 = 256;
// Translucent blocks have an ID from 1 to 511
pub const TRANSPARENT_MAX_ID: u16 = 511;

pub const LIGHT_BIT_MASK: u16 = 0xF000;
// pub const FLAGS_BIT_MASK: u32 = 0x0000F000;
pub const KIND_BIT_MASK: u16 = 0x0FFF;

pub type Block = u16;

pub trait BlockTrait: Copy + Clone + PartialEq + Eq {
    fn kind(&self) -> Block;

    /// Special "air" block a.k.a. empty space
    fn is_empty(&self) -> bool;

    /// If true light will not pass through
    fn is_opaque(&self) -> bool;

    /// If true light will pass through
    fn is_transparent(&self) -> bool;

    /// If true is not empty but light will pass through
    fn is_translucent(&self) -> bool;

    /// If true objects cannot pass through
    fn is_solid(&self) -> bool;

    /// If true objects can pass through
    fn is_passable(&self) -> bool;

    fn empty_block() -> Self;

    fn water_block() -> Self;

    fn dirt_block() -> Self;

    fn grass_block() -> Self;

    fn rock_block() -> Self;

    fn sand_block() -> Self;

    fn sandstone_block() -> Self;

    fn wood_block() -> Self;

    fn gold_block() -> Self;

    fn iron_block() -> Self;

    fn ice_block() -> Self;

    fn bedrock_block() -> Self;

    fn lamp_block() -> Self;
}

impl BlockTrait for u16 {
    fn kind(&self) -> Block {
        (self & KIND_BIT_MASK) as Block
    }

    fn is_empty(&self) -> bool {
        self.kind() == AIR_BLOCK
    }

    fn is_opaque(&self) -> bool {
        self.kind() > TRANSPARENT_MAX_ID
    }

    fn is_transparent(&self) -> bool {
        self.kind() <= TRANSPARENT_MAX_ID
    }

    fn is_translucent(&self) -> bool {
        let kind = self.kind();
        kind > AIR_BLOCK && kind <= TRANSPARENT_MAX_ID
    }

    fn is_solid(&self) -> bool {
        self.kind() >= SOLID_MIN_ID
    }

    fn is_passable(&self) -> bool {
        self.kind() < SOLID_MIN_ID
    }

    fn empty_block() -> Self {
        AIR_BLOCK
    }
    fn water_block() -> Self {
        WATER_BLOCK
    }
    fn dirt_block() -> Self {
        DIRT_BLOCK
    }
    fn grass_block() -> Self {
        GRASS_BLOCK
    }
    fn rock_block() -> Self {
        ROCK_BLOCK
    }
    fn sand_block() -> Self {
        SAND_BLOCK
    }
    fn sandstone_block() -> Self {
        SANDSTONE_BLOCK
    }
    fn wood_block() -> Self {
        WOOD_BLOCK
    }

    fn iron_block() -> Self {
        IRON_BLOCK
    }

    fn gold_block() -> Self {
        GOLD_BLOCK
    }

    fn bedrock_block() -> Self {
        BEDROCK_BLOCK
    }

    fn lamp_block() -> Self {
        LAMP_BLOCK
    }

    fn ice_block() -> Self {
        ICE_BLOCK
    }
}

/// Blocks that support getting and setting light values
pub trait LightBlock: Copy + Clone + PartialEq + Eq {
    fn get_light(&self) -> u8;

    fn set_light(&mut self, light: u8);
}

impl LightBlock for Block {
    fn get_light(&self) -> u8 {
        ((self & LIGHT_BIT_MASK) >> 12) as u8
    }

    fn set_light(&mut self, light: u8) {
        *self = (*self & !LIGHT_BIT_MASK) | ((light as Block) << 12);
    }
}

#[cfg(test)]
mod block_light_test {

    use crate::block::*;

    #[test]
    fn get_light() {
        let block: Block = 0xF000;
        assert_eq!(block.get_light(), 15);
    }

    #[test]
    fn set_get_light() {
        let mut block: Block = 1;
        assert_eq!(block.get_light(), 0);
        block.set_light(10);
        assert_eq!(block.get_light(), 10);
        block.set_light(7);
        assert_eq!(block.get_light(), 7);
        block.set_light(15);
        assert_eq!(block.get_light(), 15);
        block.set_light(0);
        assert_eq!(block.get_light(), 0);
    }
}
