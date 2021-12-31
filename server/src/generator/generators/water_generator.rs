use super::Generator;
use crate::generator::NoiseSource2D;
use common::block::*;
use common::chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS};
use noise::*;

pub struct WaterWorldGenerator {
    soil_thickness: usize,
    water_z: usize,
    ocean_floor_noise: NoiseSource2D<Perlin>,
}

impl WaterWorldGenerator {
    pub fn new(seed: u32) -> Self {
        let floor_min_z = 16;
        let floor_max_z = 64;
        WaterWorldGenerator {
            soil_thickness: 3,
            water_z: 128,
            ocean_floor_noise: NoiseSource2D::<Perlin>::new_perlin(
                seed,
                floor_min_z as f64,
                floor_max_z as f64,
            ),
        }
    }

    fn get_ocean_floor_z(&mut self, x: i16, y: i16) -> usize {
        self.ocean_floor_noise.get(x as f64, y as f64, 0.01) as usize
    }
}

impl Generator for WaterWorldGenerator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block> {
        let mut blocks = Vec::new();
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            let floor_z = self.get_ocean_floor_z(x, y);
            let block = if z <= 2 {
                Block::bedrock_block()
            } else if z < floor_z {
                Block::rock_block()
            } else if z < floor_z + self.soil_thickness {
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

    fn determine_rock_water_top(&mut self, x: i16, y: i16) -> (usize, usize, usize) {
        let floor_z = self.get_ocean_floor_z(x, y);
        let rock_top_z = floor_z;
        let water_top_z = self.water_z;
        let top_z = floor_z + self.soil_thickness;
        (rock_top_z, water_top_z, top_z)
    }
}
