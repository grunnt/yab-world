use common::{
    block::*,
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};
use noise::{Fbm, Perlin};

use crate::generator::{NoiseSource2D, NoiseSource3D};

use super::Generator;

pub struct AlienGenerator {
    terrain_noise: NoiseSource3D<Fbm>,
    resource_type_noise: NoiseSource2D<Perlin>,
    resource_density_noise: NoiseSource3D<Perlin>,
    terrain_min_z: usize,
    terrain_z_range: usize,
}

impl AlienGenerator {
    pub fn new(seed: u32) -> Self {
        AlienGenerator {
            terrain_noise: NoiseSource3D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            resource_type_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, 2.0),
            resource_density_noise: NoiseSource3D::<Perlin>::new_perlin(seed, 0.0, 1.0),
            terrain_min_z: 16,
            terrain_z_range: 230,
        }
    }

    fn determine_block(&mut self, z: usize, x: i16, y: i16) -> u32 {
        let block = if z < self.terrain_min_z {
            Block::rock_block()
        } else if z > self.terrain_min_z + self.terrain_z_range + 16 {
            Block::empty_block()
        } else {
            let noise = self.terrain_noise.get(x as f64, y as f64, z as f64, 0.01);
            let noise = noise * noise;
            let h_factor = (z - self.terrain_min_z) as f64 / self.terrain_z_range as f64;
            if noise + h_factor < 0.5 {
                let res = self
                    .resource_density_noise
                    .get(x as f64 + 34.434, y as f64 - 995.5, z as f64 + 55.001, 0.1)
                    .powf(2.0);
                let depth_factor = (1.0 - (z as f64 / 256.0) * 0.1).clamp(0.0, 0.1);
                if res > 0.7 - depth_factor {
                    let type_noise =
                        self.resource_type_noise
                            .get(x as f64 + 545.545, y as f64 + 55.323, 0.002);
                    if type_noise < 1.0 {
                        Block::gold_block()
                    } else {
                        Block::iron_block()
                    }
                } else {
                    Block::rock_block()
                }
            } else {
                Block::empty_block()
            }
        };
        block
    }
}

impl Generator for AlienGenerator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block> {
        let mut blocks = vec![Block::empty_block(); WORLD_HEIGHT_CHUNKS * CHUNK_SIZE];
        // Generate rocks and water
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            blocks[z] = self.determine_block(z, x, y);
        }
        add_soil(&mut blocks, 2);
        blocks
    }

    fn determine_rock_water_top(&mut self, x: i16, y: i16) -> (usize, usize, usize) {
        let mut rock_top_z = 0;
        let water_top_z = 0;
        let mut top_z = 0;
        for z in (self.terrain_min_z..self.terrain_min_z + self.terrain_z_range).rev() {
            let block = self.determine_block(z, x, y).kind();
            if top_z == 0 && block != Block::empty_block() {
                top_z = z + 2;
            }
            if rock_top_z == 0 && block.is_rocky() {
                rock_top_z = z;
                break;
            }
        }
        (rock_top_z, water_top_z, top_z)
    }
}

fn add_soil(blocks: &mut Vec<Block>, soil_thickness: usize) {
    let mut soil_added = 0;
    for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
        let block = blocks[z].kind();
        if block == Block::empty_block() || block == Block::water_block() {
            if soil_added < soil_thickness {
                blocks[z] = Block::ice_block();
                soil_added += 1;
            }
        } else {
            soil_added = 0;
        }
    }
}
