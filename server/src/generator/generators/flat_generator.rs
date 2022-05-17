use common::{
    block::*,
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};

use super::Generator;

/// Generates a completely flat world covered with grass with no water or resources
pub struct FlatGenerator {
    dirt_bottom_z: usize,
    terrain_top_z: usize,
}

impl FlatGenerator {
    pub fn new(dirt_bottom_z: usize, terrain_top_z: usize) -> Self {
        FlatGenerator {
            dirt_bottom_z,
            terrain_top_z,
        }
    }
}

impl Generator for FlatGenerator {
    fn generate(&mut self, _x: i16, _y: i16) -> Vec<Block> {
        let mut blocks = Vec::new();
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            let block = if z <= 2 {
                Block::bedrock_block()
            } else if z <= self.dirt_bottom_z {
                Block::rock_block()
            } else if z < self.terrain_top_z {
                Block::dirt_block()
            } else if z == self.terrain_top_z {
                Block::grass_block()
            } else {
                Block::empty_block()
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
