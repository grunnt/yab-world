mod tower_generator;
mod tree_generator;

use common::block::*;
use rand::prelude::*;
pub use tower_generator::TowerGenerator;
pub use tree_generator::TreeGenerator;

pub trait ObjectGenerator {
    /// Pre-generate an object to be placed in the world
    fn generate(&mut self) -> PregeneratedObject;
}

/// Container for a pregenerated object, with some helper functions
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
            blocks: vec![AIR_BLOCK; size_x * size_y * size_z],
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

    /// Place a sphere where a random 80% of the blocks are placed
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
