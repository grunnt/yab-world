use std::sync::Arc;

use common::{block::*, chunk::WORLD_HEIGHT_BLOCKS};
use noise::*;
use rand::{prelude::StdRng, RngCore};

use super::{Generator, NoiseSource2D};

#[derive(Clone, Debug)]
pub struct WorldObject {
    center_x: i16,
    center_y: i16,
    size: i16,
    random: f64,
    density: f64,
}

pub struct ObjectPlacer {
    grid_size: i16,
    grid_margin: i16,
    object_density: f64,
    clustered_objects: bool,
    grid_x_noise: NoiseSource2D<Value>,
    grid_y_noise: NoiseSource2D<Value>,
    fbm_density_noise: NoiseSource2D<Fbm>,
    value_density_noise: NoiseSource2D<Value>,
    randomizer_noise: NoiseSource2D<Value>,
    pregenerated: Arc<Vec<PregeneratedObject>>,
    overlapping: bool,
}

impl ObjectPlacer {
    pub fn new(
        seed: u32,
        pregenerated: Arc<Vec<PregeneratedObject>>,
        grid_size: i16,
        grid_margin: i16,
        object_density: f64,
        clustered_objects: bool,
        overlapping: bool,
    ) -> Self {
        ObjectPlacer {
            grid_size,
            grid_margin,
            object_density,
            grid_x_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            grid_y_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            fbm_density_noise: NoiseSource2D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            value_density_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            randomizer_noise: NoiseSource2D::<Value>::new_value(seed, 1.0, std::i16::MAX as f64),
            pregenerated,
            clustered_objects,
            overlapping,
        }
    }

    pub fn place(
        &mut self,
        x: i16,
        y: i16,
        blocks: &mut Vec<Block>,
        generator: &mut dyn Generator,
    ) {
        let grid_x = (x / self.grid_size) * self.grid_size;
        let grid_y = (y / self.grid_size) * self.grid_size;
        if self.overlapping {
            // If the object may cross the grid cell border we need to check its neighbours as well
            for dgx in -1..2 {
                for dgy in -1..2 {
                    self.place_grid_object(
                        grid_x + dgx * self.grid_size,
                        grid_y + dgy * self.grid_size,
                        x,
                        y,
                        generator,
                        blocks,
                    );
                }
            }
        } else {
            self.place_grid_object(grid_x, grid_y, x, y, generator, blocks);
        }
    }

    fn place_grid_object(
        &mut self,
        grid_x: i16,
        grid_y: i16,
        x: i16,
        y: i16,
        generator: &mut dyn Generator,
        blocks: &mut Vec<u32>,
    ) {
        let random = self.randomizer_noise.get(grid_x as f64, grid_y as f64, 1.0);
        let pregenerated = self
            .pregenerated
            .get(random as usize % self.pregenerated.len())
            .unwrap();
        let (grid_start_x, grid_start_y, grid_range_x, grid_range_y) = if self.overlapping {
            (
                grid_x + self.grid_margin,
                grid_y + self.grid_margin,
                self.grid_size as f64 - self.grid_margin as f64 * 2.0,
                self.grid_size as f64 - self.grid_margin as f64 * 2.0,
            )
        } else {
            (
                grid_x + self.grid_margin + pregenerated.anchor_x as i16,
                grid_y + self.grid_margin + pregenerated.anchor_y as i16,
                self.grid_size as f64 - pregenerated.size_x as f64 - self.grid_margin as f64 * 2.0,
                self.grid_size as f64 - pregenerated.size_y as f64 - self.grid_margin as f64 * 2.0,
            )
        };
        let anchor_world_x = grid_start_x
            + (self
                .grid_x_noise
                .get(grid_x as f64 + 0.123, grid_y as f64 + 50.665, 1.0)
                * grid_range_x) as i16;
        let anchor_world_y = grid_start_y
            + (self
                .grid_y_noise
                .get(grid_x as f64 - 102.4, grid_y as f64 + 553.1, 1.0)
                * grid_range_y) as i16;
        let x1 = anchor_world_x - pregenerated.anchor_x as i16;
        let y1 = anchor_world_y - pregenerated.anchor_y as i16;
        let x2 = x1 + pregenerated.size_x as i16;
        let y2 = y1 + pregenerated.size_y as i16;
        if x < x1 || x >= x2 || y < y1 || y >= y2 {
            // Outside of this object area
            return;
        }

        // println!(
        //     "grid {},{} anchor {},{} for {},{}",
        //     grid_x, grid_y, anchor_world_x, anchor_world_y, x, y
        // );

        let density_noise = if self.clustered_objects {
            self.fbm_density_noise
                .get(anchor_world_x as f64, anchor_world_y as f64, 0.01)
        } else {
            self.value_density_noise
                .get(anchor_world_x as f64, anchor_world_y as f64, 1.0)
        };
        if density_noise > self.object_density {
            return;
        }
        let (anchor_rock_top_z, anchor_water_top_z, anchor_top_z) =
            generator.determine_rock_water_top(anchor_world_x, anchor_world_y);
        if anchor_water_top_z > anchor_rock_top_z {
            return;
        }
        let anchor_world_z = if pregenerated.place_on_soil {
            anchor_top_z
        } else {
            anchor_rock_top_z
        } + 1;
        if anchor_world_z < pregenerated.anchor_z {
            return;
        }
        let z1 = anchor_world_z - pregenerated.anchor_z;
        let z2 = z1 + pregenerated.size_z;
        let (rock_top_z, _, top_z) = generator.determine_rock_water_top(x, y);
        let from_z = if pregenerated.place_on_soil {
            top_z
        } else {
            rock_top_z
        }
        .min(z1);
        let x_rel = (x - x1) as usize;
        let y_rel = (y - y1) as usize;
        let bottom_block = pregenerated.get(x_rel, y_rel, 0);
        let place_foundation = bottom_block != Block::empty_block() && bottom_block != IGNORE_BLOCK;
        for z in from_z..z2 {
            if pregenerated.overwrite_non_empty || blocks[z] == AIR_BLOCK {
                let z_rel = z - z1;
                if z < z1 {
                    if place_foundation {
                        if let Some(foundation_block) = pregenerated.foundation_block {
                            blocks[z] = foundation_block;
                        }
                    }
                } else {
                    let object_block = pregenerated.get(x_rel, y_rel, z_rel);
                    if object_block != IGNORE_BLOCK {
                        blocks[z] = pregenerated.get(x_rel, y_rel, z_rel);
                    }
                }
            }
        }
        if pregenerated.overwrite_non_empty {
            for z in z2..WORLD_HEIGHT_BLOCKS as usize {
                blocks[z] = Block::empty_block();
            }
        }
    }
}

#[derive(Clone)]
pub struct PregeneratedObject {
    pub anchor_x: usize,
    pub anchor_y: usize,
    pub anchor_z: usize,
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub foundation_block: Option<Block>,
    pub place_on_soil: bool,
    pub overwrite_non_empty: bool,
    pub blocks: Vec<Block>,
}

impl PregeneratedObject {
    pub fn new(size_x: usize, size_y: usize, size_z: usize) -> Self {
        PregeneratedObject {
            anchor_x: size_x / 2,
            anchor_y: size_y / 2,
            anchor_z: 0,
            size_x,
            size_y,
            size_z,
            foundation_block: None,
            place_on_soil: false,
            overwrite_non_empty: false,
            blocks: vec![Block::empty_block(); size_x * size_y * size_z],
        }
    }

    pub fn solid(
        size_x: usize,
        size_y: usize,
        size_z: usize,
        block: Block,
        foundation_block: Block,
    ) -> Self {
        let mut object = PregeneratedObject::new(size_x, size_y, size_z);
        for x in 0..size_x {
            for y in 0..size_y {
                for z in 0..size_z {
                    object.set(x, y, z, block);
                }
            }
        }
        object.foundation_block = Some(foundation_block);
        object
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[z + y * self.size_z + x * self.size_z * self.size_y]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[z + y * self.size_z + x * self.size_z * self.size_y] = block;
    }

    pub fn set_rectangle(
        &mut self,
        x1: usize,
        y1: usize,
        z1: usize,
        x2: usize,
        y2: usize,
        z2: usize,
        block: Block,
        checkerboard: bool,
    ) {
        for rz in z1..z2 {
            for rx in x1..x2 {
                if !checkerboard || rx % 2 != 0 {
                    self.set(rx, y1, rz, block);
                    self.set(rx, y2 - 1, rz, block);
                }
            }
            for ry in y1..y2 {
                if !checkerboard || ry % 2 != 0 {
                    self.set(x1, ry, rz, block);
                    self.set(x2 - 1, ry, rz, block);
                }
            }
        }
    }

    pub fn set_filled_rectangle(
        &mut self,
        x1: usize,
        y1: usize,
        z1: usize,
        x2: usize,
        y2: usize,
        z2: usize,
        block: Block,
    ) {
        for rx in x1..x2 {
            for ry in y1..y2 {
                for rz in z1..z2 {
                    self.set(rx, ry, rz, block);
                }
            }
        }
    }

    pub fn fill_sphere(&mut self, x: usize, y: usize, z: usize, range: usize, block: Block) {
        let range_sq = (range * range) as isize;
        for rx in x - range..x + range {
            for ry in y - range..y + range {
                for rz in z - range..z + range {
                    let dx = rx as isize - x as isize;
                    let dy = ry as isize - y as isize;
                    let dz = rz as isize - z as isize;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq <= range_sq {
                        self.set(rx, ry, rz, block);
                    }
                }
            }
        }
    }

    pub fn spray_sphere(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        range: usize,
        block: Block,
        random: &mut StdRng,
    ) {
        let range_sq = (range * range) as isize;
        for rx in x - range..x + range {
            for ry in y - range..y + range {
                for rz in z - range..z + range {
                    let dx = rx as isize - x as isize;
                    let dy = ry as isize - y as isize;
                    let dz = rz as isize - z as isize;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq <= range_sq {
                        if random.next_u32() % 10 > 2 {
                            self.set(rx, ry, rz, block);
                        }
                    }
                }
            }
        }
    }
}
