use crate::block::blockregistry::*;

pub const LIGHT_BIT_MASK: u32 = 0xF0000000;
pub const KIND_BIT_MASK: u32 = 0x00000FFF;
pub const SOLID_BIT_MASK: u32 = 0x00001000;
pub const TRANSPARENT_BIT_MASK: u32 = 0x00002000;

pub type Block = u32;

pub trait BlockTrait: Copy + Clone + PartialEq + Eq {
    fn kind(&self) -> Block;

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

    fn toggle_transparency(&mut self);

    fn toggle_solidity(&mut self);
}

impl BlockTrait for u32 {
    fn kind(&self) -> Block {
        (self & KIND_BIT_MASK) as Block
    }

    fn is_opaque(&self) -> bool {
        (self & TRANSPARENT_BIT_MASK) == 0
    }

    fn is_transparent(&self) -> bool {
        (self & TRANSPARENT_BIT_MASK) > 0
    }

    fn is_translucent(&self) -> bool {
        self.kind() != AIR_BLOCK_KIND && self.is_transparent()
    }

    fn is_solid(&self) -> bool {
        (self & SOLID_BIT_MASK) > 0
    }

    fn is_passable(&self) -> bool {
        (self & SOLID_BIT_MASK) == 0
    }

    fn toggle_transparency(&mut self) {
        *self = *self ^ TRANSPARENT_BIT_MASK;
    }

    fn toggle_solidity(&mut self) {
        *self = *self ^ SOLID_BIT_MASK;
    }
}

/// Blocks that support getting and setting light values
pub trait LightBlock: Copy + Clone + PartialEq + Eq {
    fn get_light(&self) -> u8;

    fn set_light(&mut self, light: u8);
}

impl LightBlock for Block {
    fn get_light(&self) -> u8 {
        ((self & LIGHT_BIT_MASK) >> 28) as u8
    }

    fn set_light(&mut self, light: u8) {
        *self = (*self & !LIGHT_BIT_MASK) | ((light as Block) << 28);
    }
}

#[cfg(test)]
mod block_light_test {

    use crate::block::*;

    #[test]
    fn get_light() {
        let block: Block = 0xF0000000;
        assert_eq!(block.get_light(), 15);
    }

    #[test]
    fn set_get_light() {
        let mut block: Block = Block::rock_block();
        assert_eq!(block.get_light(), 0);
        block.set_light(14);
        block.set_sunlight(15);
        assert_eq!(block.get_light(), 14);
        assert_eq!(block.get_sunlight(), 15);
        assert_eq!(block.kind(), Block::rock_block());
        block.set_light(7);
        assert_eq!(block.get_light(), 7);
        assert_eq!(block.get_sunlight(), 15);
        assert_eq!(block.kind(), Block::rock_block());
    }
}
