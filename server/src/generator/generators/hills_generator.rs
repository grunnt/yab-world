use super::Generator;
use crate::generator::{NoiseSource2D, NoiseSource3D};
use common::{
    block::*,
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};
use noise::{Fbm, Perlin, Value};

const OBJECT_GRID_SIZE: f64 = (CHUNK_SIZE / 2) as f64;

pub struct HillsGenerator {
    roughness_noise: NoiseSource2D<Perlin>,
    terrain_noise: NoiseSource3D<Fbm>,
    water_z: usize,
    resource_type_noise: NoiseSource2D<Perlin>,
    resource_density_noise: NoiseSource3D<Perlin>,
    grid_x_noise: NoiseSource2D<Perlin>,
    grid_y_noise: NoiseSource2D<Perlin>,
    density_noise: NoiseSource2D<Fbm>,
    type_noise: NoiseSource2D<Value>,
    size_noise: NoiseSource2D<Value>,
}

impl HillsGenerator {
    pub fn new(seed: u32) -> Self {
        HillsGenerator {
            roughness_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0025, 0.015),
            terrain_noise: NoiseSource3D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            resource_type_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, 2.0),
            resource_density_noise: NoiseSource3D::<Perlin>::new_perlin(seed, 0.0, 1.0),
            water_z: 80,
            grid_x_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, OBJECT_GRID_SIZE as f64),
            grid_y_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, OBJECT_GRID_SIZE as f64),
            density_noise: NoiseSource2D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            type_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 10.0),
            size_noise: NoiseSource2D::<Value>::new_value(seed, 1.0, 5.0),
        }
    }
}

impl Generator for HillsGenerator {
    fn generate(&mut self, x: i16, y: i16, objects: bool) -> Vec<Block> {
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

        if objects {
            // Collect information about the closest objects
            let mut close_objects = Vec::new();
            let this_grid_x = (x as f64 / OBJECT_GRID_SIZE as f64).floor() as i16;
            let this_grid_y = (y as f64 / OBJECT_GRID_SIZE as f64).floor() as i16;
            for dy in -1..2 {
                for dx in -1..2 {
                    let grid_x = (this_grid_x + dx) as f64;
                    let grid_y = (this_grid_y + dy) as f64;
                    let point_x = (grid_x * OBJECT_GRID_SIZE
                        + self.grid_x_noise.get(grid_x + 0.123, grid_y + 50.665, 10.0))
                        as i16;
                    let point_y = (grid_y * OBJECT_GRID_SIZE
                        + self.grid_y_noise.get(grid_x - 102.4, grid_y + 553.1, 10.0))
                        as i16;
                    let density = self.density_noise.get(point_x as f64, point_y as f64, 0.01);
                    if density > 0.5 || density < 0.2 {
                        // 0.15
                        let size =
                            (self.size_noise.get(point_x as f64, point_y as f64, 1.0) * 2.0) as i16;
                        if x >= point_x - size
                            && x <= point_x + size
                            && y >= point_y - size
                            && y <= point_y + size
                        {
                            close_objects.push((point_x, point_y, density, size));
                        }
                    }
                }
            }
            // Place objects on this column
            for (ox, oy, density, size) in close_objects {
                if density > 0.5 {
                    if let Some(ground_z) = dry_rock_top_z(&blocks) {
                        // Tree
                        if x == ox && y == oy {
                            for z in ground_z..ground_z + 16 {
                                blocks[z] = Block::log_block();
                            }
                        }
                    }
                } else {
                    if let Some(center_ground_z) = dry_rock_top_z(&self.generate(ox, oy, false)) {
                        // Tower
                        let height = 16;
                        let roof_height = 14;
                        let ground_z = rock_top_z(&blocks);
                        let edge =
                            x == ox - size || x == ox + size || y == oy - size || y == oy + size;
                        if edge {
                            let height = (height - (x % 2 + y % 2).min(1)) as usize;
                            for z in ground_z..center_ground_z + height {
                                blocks[z] = Block::bricks_block();
                            }
                            if x == ox || y == oy {
                                blocks[center_ground_z + roof_height - 2] = Block::empty_block();
                            }
                        } else {
                            if ground_z <= center_ground_z {
                                for z in ground_z..center_ground_z {
                                    blocks[z] = Block::bricks_block();
                                }
                            }
                            blocks[center_ground_z] = Block::wood_block();
                            for z in center_ground_z + 1..center_ground_z + roof_height {
                                blocks[z] = Block::empty_block();
                            }
                            blocks[center_ground_z + roof_height] = Block::wood_block();
                            if x == ox && y == oy {
                                blocks[center_ground_z + roof_height - 1] = Block::lamp_block();
                            }
                        }
                    }
                }
            }
        }

        blocks
    }
}

pub fn dry_rock_top_z(blocks: &Vec<Block>) -> Option<usize> {
    for z in (0..blocks.len()).rev() {
        if blocks[z] == Block::water_block() {
            return None;
        } else if blocks[z] == Block::rock_block() {
            return Some(z);
        }
    }
    None
}

pub fn rock_top_z(blocks: &Vec<Block>) -> usize {
    for z in (0..blocks.len()).rev() {
        if blocks[z] == Block::rock_block() {
            return z;
        }
    }
    0
}
