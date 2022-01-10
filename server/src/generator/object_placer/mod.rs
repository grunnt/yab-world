use std::sync::Arc;

use common::block::*;
use noise::*;

use super::{Generator, NoiseSource2D};

#[derive(Clone, Debug)]
pub struct WorldObject {
    center_x: i16,
    center_y: i16,
    size: i16,
    random: f64,
    density: f64,
}

pub struct ObjectGrid {
    grid_size: i16,
    grid_border_size: i16,
    object_density: f64,
    clustered_objects: bool,
    grid_x_noise: NoiseSource2D<Value>,
    grid_y_noise: NoiseSource2D<Value>,
    fbm_density_noise: NoiseSource2D<Perlin>,
    value_density_noise: NoiseSource2D<Value>,
    randomizer_noise: NoiseSource2D<Value>,
    pregenerated: Arc<Vec<PregeneratedObject>>,
}

impl ObjectGrid {
    pub fn new(
        seed: u32,
        pregenerated: Arc<Vec<PregeneratedObject>>,
        grid_border_size: i16,
        object_border_size: i16,
        object_density: f64,
        clustered_objects: bool,
    ) -> Self {
        let mut max_object_size = 1;
        for object in pregenerated.iter() {
            if object.size_x > max_object_size {
                max_object_size = object.size_x;
            }
            if object.size_y > max_object_size {
                max_object_size = object.size_y;
            }
        }
        ObjectGrid {
            grid_size: max_object_size as i16 + grid_border_size * 2 + object_border_size * 2,
            grid_border_size,
            object_density,
            grid_x_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            grid_y_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            fbm_density_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, 1.0),
            value_density_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            randomizer_noise: NoiseSource2D::<Value>::new_value(seed, 1.0, std::i16::MAX as f64),
            pregenerated,
            clustered_objects,
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
        let random = self.randomizer_noise.get(grid_x as f64, grid_y as f64, 1.0);
        let pregenerated = self
            .pregenerated
            .get(random as usize % self.pregenerated.len())
            .unwrap();
        let anchor_world_x = grid_x
            + self.grid_border_size
            + (self
                .grid_x_noise
                .get(grid_x as f64 + 0.123, grid_y as f64 + 50.665, 1.0)
                * (self.grid_size as f64
                    - pregenerated.size_x as f64
                    - self.grid_border_size as f64 * 2.0)) as i16
            + pregenerated.anchor_x as i16;
        let anchor_world_y = grid_y
            + self.grid_border_size
            + (self
                .grid_y_noise
                .get(grid_x as f64 - 102.4, grid_y as f64 + 553.1, 1.0)
                * (self.grid_size as f64
                    - pregenerated.size_y as f64
                    - self.grid_border_size as f64 * 2.0)) as i16
            + pregenerated.anchor_y as i16;
        let x1 = anchor_world_x - pregenerated.anchor_x as i16;
        let y1 = anchor_world_y - pregenerated.anchor_y as i16;
        let x2 = x1 + pregenerated.size_x as i16;
        let y2 = y1 + pregenerated.size_y as i16;
        if x < x1 || x >= x2 || y < y1 || y >= y2 {
            // Outside of this object area
            return;
        }
        // TODO try speedup by basing density check on grid_x, grid_y (which may look weird)
        let density_noise = if self.clustered_objects {
            self.fbm_density_noise
                .get(grid_x as f64, grid_y as f64, 0.01)
        } else {
            self.value_density_noise
                .get(grid_x as f64, grid_y as f64, 1.0)
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
        };
        if anchor_world_z < pregenerated.anchor_z {
            return;
        }
        let z1 = anchor_world_z - pregenerated.anchor_z;
        let z2 = z1 + pregenerated.size_z;
        let foundation_z = anchor_world_z - pregenerated.anchor_z;
        let (rock_top_z, _, top_z) = generator.determine_rock_water_top(x, y);
        let from_z = if pregenerated.place_on_soil {
            top_z
        } else {
            rock_top_z
        };
        let x_rel = (x - x1) as usize;
        let y_rel = (y - y1) as usize;
        for z in from_z..z2 {
            let z_rel = z - z1;
            if z < foundation_z {
                if let Some(foundation_block) = pregenerated.foundation_block {
                    blocks[z] = foundation_block;
                }
            } else {
                blocks[z] = pregenerated.get(x_rel, y_rel, z_rel);
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
    pub overwrite_empty: bool,
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
            overwrite_empty: false,
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

    fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[z + y * self.size_z + x * self.size_z * self.size_y] = block;
    }
}
