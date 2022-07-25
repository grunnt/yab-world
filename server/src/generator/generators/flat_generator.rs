use common::{
    block::*,
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};

use super::Generator;

/// Generates a completely flat world covered with grass with no water or resources
pub struct FlatGenerator {
    dirt_bottom_z: usize,
    terrain_top_z: usize,
    stone_block: Block,
    dirt_block: Block,
    grass_block: Block,
}

impl FlatGenerator {
    pub fn new(dirt_bottom_z: usize, terrain_top_z: usize, block_registry: &BlockRegistry) -> Self {
        let stone_block = block_registry.block_kind_from_code("stone");
        let dirt_block = block_registry.block_kind_from_code("dirt");
        let grass_block = block_registry.block_kind_from_code("grass");
        FlatGenerator {
            dirt_bottom_z,
            terrain_top_z,
            stone_block,
            dirt_block,
            grass_block,
        }
    }
}

impl Generator for FlatGenerator {
    fn generate(&mut self, _x: i16, _y: i16) -> Vec<Block> {
        let mut blocks = Vec::new();
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            let block = if z <= self.dirt_bottom_z {
                self.stone_block
            } else if z < self.terrain_top_z {
                self.dirt_block
            } else if z == self.terrain_top_z {
                self.grass_block
            } else {
                AIR_BLOCK
            };
            blocks.push(block);
        }
        blocks
    }

    fn determine_rock_water_top(&mut self, _x: i16, _y: i16) -> (usize, usize, usize) {
        let rock_top_z = self.dirt_bottom_z - 1;
        let water_top_z = 0;
        let top_z = self.terrain_top_z;
        (rock_top_z, water_top_z, top_z)
    }
}
