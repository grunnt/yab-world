use super::Generator;
use crate::generator::{NoiseSource2D, NoiseSource3D};
use common::{
    block::*,
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};
use noise::{Fbm, Perlin};

pub struct HillsGenerator {
    roughness_noise: NoiseSource2D<Perlin>,
    terrain_noise: NoiseSource3D<Fbm>,
    water_z: usize,
    resource_type_noise: NoiseSource2D<Perlin>,
    resource_density_noise: NoiseSource3D<Perlin>,
}

impl HillsGenerator {
    pub fn new(seed: u32) -> Self {
        HillsGenerator {
            roughness_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0025, 0.015),
            terrain_noise: NoiseSource3D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            resource_type_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, 2.0),
            resource_density_noise: NoiseSource3D::<Perlin>::new_perlin(seed, 0.0, 1.0),
            water_z: 80,
        }
    }
}

impl Generator for HillsGenerator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block> {
        let roughness = self
            .roughness_noise
            .get(434.0 - x as f64, 545.0 + y as f64, 0.0001);
        let mut blocks = Vec::new();
        // Generate rocks and water
        let mut soil_added = 0;
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            let terrain_min_z = 64;
            let terrain_z_range = 128;
            if z < terrain_min_z {
                blocks.push(Block::rock_block());
                continue;
            }
            if z > terrain_min_z + terrain_z_range + 16 {
                blocks.push(Block::empty_block());
                continue;
            }
            let noise = self
                .terrain_noise
                .get(x as f64, y as f64, z as f64, roughness);
            let noise = noise * noise;
            let h_factor = (z - terrain_min_z) as f64 / terrain_z_range as f64;
            let block = if noise + h_factor < 0.5 {
                soil_added = 0;
                let res = self
                    .resource_density_noise
                    .get(x as f64 + 34.434, y as f64 - 995.5, z as f64 + 55.001, 0.1)
                    .powf(3.0);
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
            } else if z < self.water_z {
                if soil_added < 3 {
                    soil_added += 1;
                    if soil_added == 3 {
                        soil_added = 4; // Trick to avoid placing grass on water
                    }
                    Block::sand_block()
                } else {
                    Block::water_block()
                }
            } else {
                if soil_added < 3 {
                    soil_added += 1;
                    if z < self.water_z + 1 {
                        if soil_added == 3 {
                            soil_added = 4; // Trick to avoid placing grass on water
                        }
                        Block::sand_block()
                    } else {
                        Block::dirt_block()
                    }
                } else if soil_added == 3 {
                    soil_added += 1;
                    Block::grass_block()
                } else {
                    Block::empty_block()
                }
            };
            blocks.push(block);
        }

        blocks
    }
}
