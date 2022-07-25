use super::Generator;
use crate::generator::{NoiseSource2D, NoiseSource3D};
use common::{
    block::*,
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};
use noise::{Fbm, Perlin};

/// Generates rough hills covered in grass, with a fixed water level and resource placement
pub struct HillsGenerator {
    roughness_noise: NoiseSource2D<Perlin>,
    terrain_noise: NoiseSource3D<Fbm>,
    water_z: usize,
    resource_type_noise: NoiseSource2D<Perlin>,
    resource_density_noise: NoiseSource3D<Perlin>,
    soil_thickness: usize,
    terrain_min_z: usize,
    terrain_z_range: usize,
    stone_block: Block,
    dirt_block: Block,
    grass_block: Block,
    sand_block: Block,
    gold_block: Block,
    iron_block: Block,
    water_block: Block,
}

impl HillsGenerator {
    pub fn new(seed: u32, block_registry: &BlockRegistry) -> Self {
        let stone_block = block_registry.block_kind_from_code("stone");
        let dirt_block = block_registry.block_kind_from_code("dirt");
        let grass_block = block_registry.block_kind_from_code("grass");
        let sand_block = block_registry.block_kind_from_code("sand");
        let gold_block = block_registry.block_kind_from_code("gold");
        let iron_block = block_registry.block_kind_from_code("iron");
        let water_block = block_registry.block_kind_from_code("water");

        HillsGenerator {
            roughness_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0025, 0.025),
            terrain_noise: NoiseSource3D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            resource_type_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, 2.0),
            resource_density_noise: NoiseSource3D::<Perlin>::new_perlin(seed, 0.0, 1.0),
            water_z: 80,
            soil_thickness: 3,
            terrain_min_z: 64,
            terrain_z_range: 128,
            stone_block,
            dirt_block,
            grass_block,
            gold_block,
            iron_block,
            sand_block,
            water_block,
        }
    }

    fn determine_block(&mut self, z: usize, x: i16, y: i16, roughness: f64) -> Block {
        let block = if z < self.terrain_min_z {
            self.stone_block
        } else if z > self.terrain_min_z + self.terrain_z_range + 16 {
            AIR_BLOCK
        } else {
            let noise = self
                .terrain_noise
                .get(x as f64, y as f64, z as f64, roughness);
            let noise = noise * noise;
            let h_factor = (z - self.terrain_min_z) as f64 / self.terrain_z_range as f64;
            if noise + h_factor < 0.5 {
                let res = self
                    .resource_density_noise
                    .get(x as f64, y as f64, z as f64, 0.1)
                    .powf(3.0);
                let depth_factor = (1.0 - (z as f64 / 256.0) * 0.1).clamp(0.0, 0.1);
                if res > 0.7 - depth_factor {
                    let type_noise = self.resource_type_noise.get(x as f64, y as f64, 0.002);
                    if type_noise < 1.0 {
                        self.gold_block
                    } else {
                        self.iron_block
                    }
                } else {
                    self.stone_block
                }
            } else if z < self.water_z {
                self.water_block
            } else {
                AIR_BLOCK
            }
        };
        block
    }

    fn get_terrain_roughness(&mut self, x: i16, y: i16) -> f64 {
        self.roughness_noise.get(x as f64, y as f64, 0.0001)
    }

    fn add_soil(&self, blocks: &mut Vec<Block>, soil_thickness: usize, water_z: usize) {
        let mut soil_added = 0;
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            let block_kind = blocks[z].kind();
            if block_kind == AIR_BLOCK_KIND || block_kind == self.water_block {
                if soil_added < soil_thickness {
                    if z <= water_z + 1 {
                        blocks[z] = self.sand_block;
                    } else if soil_added == soil_thickness - 1 {
                        blocks[z] = self.grass_block;
                    } else {
                        blocks[z] = self.dirt_block;
                    }
                    soil_added += 1;
                }
            } else {
                soil_added = 0;
            }
        }
    }
}

impl Generator for HillsGenerator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block> {
        let roughness = self.get_terrain_roughness(x, y);
        let mut blocks = vec![AIR_BLOCK; WORLD_HEIGHT_CHUNKS * CHUNK_SIZE];
        // Generate rocks and water
        for z in 0..WORLD_HEIGHT_CHUNKS * CHUNK_SIZE {
            blocks[z] = self.determine_block(z, x, y, roughness);
        }
        self.add_soil(&mut blocks, self.soil_thickness, self.water_z);
        blocks
    }

    fn determine_rock_water_top(&mut self, x: i16, y: i16) -> (usize, usize, usize) {
        let roughness = self.get_terrain_roughness(x, y);
        let mut rock_top_z = 0;
        let mut water_top_z = 0;
        for z in (self.terrain_min_z..self.terrain_min_z + self.terrain_z_range).rev() {
            let block = self.determine_block(z, x, y, roughness).kind();
            if water_top_z == 0 && block == self.water_block {
                water_top_z = z;
            }
            if rock_top_z == 0 && block == self.stone_block {
                rock_top_z = z;
                break;
            }
        }
        let top_z = rock_top_z + self.soil_thickness;
        (rock_top_z, water_top_z, top_z)
    }
}
