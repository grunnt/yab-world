use super::Generator;
use crate::generator::NoiseSource2D;
use common::block::*;
use common::chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS};
use noise::*;

pub struct WaterWorldGenerator {
    ocean_floor_noise: NoiseSource2D<Perlin>,
}

impl WaterWorldGenerator {
    pub fn new(seed: u32) -> Self {
        WaterWorldGenerator {
            ocean_floor_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 16.0, 64.0),
        }
    }
}

impl Generator for WaterWorldGenerator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block> {
        let mut blocks = Vec::new();
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            let floor_z = self.ocean_floor_noise.get(x as f64, y as f64, 0.01) as usize;
            let block = if z <= 2 {
                Block::bedrock_block()
            } else if z < floor_z {
                Block::rock_block()
            } else if z < floor_z + 3 {
                Block::sand_block()
            } else if z < 128 {
                Block::water_block()
            } else {
                Block::empty_block()
            };
            blocks.push(block);
        }

        blocks
    }
}
