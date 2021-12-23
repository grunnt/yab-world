use common::{
    block::{Block, BlockTrait},
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};
use noise::{Fbm, Perlin, Value};

use crate::generator::NoiseSource2D;

const OBJECT_GRID_SIZE: f64 = CHUNK_SIZE as f64 / 2.0;

pub trait ObjectPlacer {
    fn place(&mut self, x: i16, y: i16, blocks: &mut Vec<Block>);
}

pub struct TreePlacer {
    grid_x_noise: NoiseSource2D<Perlin>,
    grid_y_noise: NoiseSource2D<Perlin>,
    density_noise: NoiseSource2D<Fbm>,
    type_noise: NoiseSource2D<Value>,
    size_noise: NoiseSource2D<Value>,
}

impl TreePlacer {
    pub fn new(seed: u32) -> Self {
        TreePlacer {
            grid_x_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, OBJECT_GRID_SIZE),
            grid_y_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, OBJECT_GRID_SIZE),
            density_noise: NoiseSource2D::<Fbm>::new_fbm(seed, 0.0, 1.0),
            type_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 10.0),
            size_noise: NoiseSource2D::<Value>::new_value(seed, 3.0, 8.0),
        }
    }
}

impl ObjectPlacer for TreePlacer {
    // fn place(&mut self, x: i16, y: i16, blocks: &mut Vec<Block>) {
    //     // Check the closest objects
    //     let grid_x = x / OBJECT_GRID_SIZE as i16;
    //     let grid_y = y / OBJECT_GRID_SIZE as i16;
    //     for gy in grid_y - 1..grid_y + 2 {
    //         let gy = gy as f64;
    //         for gx in grid_x - 1..grid_x + 2 {
    //             let gx = gx as f64;
    //             let point_x = (gx * OBJECT_GRID_SIZE
    //                 + self.grid_x_noise.get(gx + 0.123, gy + 50.665, 10.0))
    //                 as i16;
    //             let point_y = (gy * OBJECT_GRID_SIZE
    //                 + self.grid_y_noise.get(gx - 102.4, gy + 553.1, 10.0))
    //                 as i16;
    //             let density = self.density_noise.get(point_x as f64, point_y as f64, 0.01);
    //             if density > 0.5 {
    //                 // let tree_type = self.type_noise.get(point_x as f64, point_y as f64, 1.0);
    //                 let size = self.size_noise.get(point_x as f64, point_y as f64, 1.0) as i16;
    //                 // Is this column within this objects space?
    //                 if x > point_x - size / 2
    //                     && x < point_x + size / 2
    //                     && y > point_y - size / 2
    //                     && y < point_y + size / 2
    //                 {
    //                     let mut top_z = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
    //                     for z in (0..top_z).rev() {
    //                         let kind = blocks[z].kind();
    //                         if kind == Block::grass_block() || kind == Block::ice_block() {
    //                             top_z = z;
    //                             break;
    //                         } else if kind != Block::empty_block() {
    //                             return;
    //                         }
    //                     }
    //                     let height = 10;
    //                     if top_z < WORLD_HEIGHT_CHUNKS * CHUNK_SIZE - height {
    //                         for z in top_z..top_z + height {
    //                             blocks[z] = Block::sandstone_block();
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
    fn place(&mut self, x: i16, y: i16, blocks: &mut Vec<Block>) {
        // Determine closest object
        let grid_x = (x as f64 / OBJECT_GRID_SIZE).floor();
        let grid_y = (y as f64 / OBJECT_GRID_SIZE).floor();
        let point_x = (grid_x * OBJECT_GRID_SIZE
            + self.grid_x_noise.get(grid_x + 0.123, grid_y + 50.665, 10.0))
            as i16;
        let point_y = (grid_y * OBJECT_GRID_SIZE
            + self.grid_y_noise.get(grid_x - 102.4, grid_y + 553.1, 10.0))
            as i16;
        let density = self.density_noise.get(point_x as f64, point_y as f64, 0.01);
        if density > 0.5 {
            // Trees
            if x == point_x && y == point_y {
                // Object center
                let tree_type = self.type_noise.get(point_x as f64, point_y as f64, 1.0);
                let block = if tree_type < 1.0 {
                    Block::gold_block()
                } else {
                    Block::log_block()
                };
                let tree_height = if density > 0.75 { 15 } else { 10 };
                let mut top_z = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
                for z in (0..top_z).rev() {
                    let kind = blocks[z].kind();
                    if kind == Block::grass_block() || kind == Block::ice_block() {
                        top_z = z;
                        break;
                    } else if kind != Block::empty_block() {
                        return;
                    }
                }
                if top_z < WORLD_HEIGHT_CHUNKS * CHUNK_SIZE - tree_height {
                    for z in top_z..top_z + tree_height {
                        blocks[z] = block;
                    }
                }
            }
        } else if density < 0.15 {
            // Buildings or other objects in open spots
            if x == point_x && y == point_y {
                let block = Block::lamp_block();
                let tree_height = 40;
                let mut top_z = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
                for z in (0..top_z).rev() {
                    let kind = blocks[z].kind();
                    if kind == Block::grass_block() || kind == Block::ice_block() {
                        top_z = z;
                        break;
                    } else if kind != Block::empty_block() {
                        return;
                    }
                }
                if top_z < WORLD_HEIGHT_CHUNKS * CHUNK_SIZE - tree_height {
                    for z in top_z..top_z + tree_height {
                        blocks[z] = block;
                    }
                }
            }
        }
    }
}
