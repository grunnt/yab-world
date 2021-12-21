use common::{
    block::{Block, BlockTrait},
    chunk::{CHUNK_SIZE, WORLD_HEIGHT_CHUNKS},
};
use noise::{Fbm, Perlin};

use crate::generator::NoiseSource2D;

const OBJECT_GRID_SIZE: f64 = CHUNK_SIZE as f64 / 2.0;

pub trait ObjectPlacer {
    fn place(&mut self, x: i16, y: i16, blocks: &mut Vec<Block>);
}

pub struct TreePlacer {
    grid_x_noise: NoiseSource2D<Perlin>,
    grid_y_noise: NoiseSource2D<Perlin>,
    density_noise: NoiseSource2D<Fbm>,
}

impl TreePlacer {
    pub fn new(seed: u32) -> Self {
        TreePlacer {
            grid_x_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, OBJECT_GRID_SIZE),
            grid_y_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, OBJECT_GRID_SIZE),
            density_noise: NoiseSource2D::<Fbm>::new_fbm(seed, 0.0, 1.0),
        }
    }
}

impl ObjectPlacer for TreePlacer {
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
            if x == point_x && y == point_y {
                // Object center
                let tree_height = if density > 0.75 { 15 } else { 10 };
                let mut top_z = WORLD_HEIGHT_CHUNKS * CHUNK_SIZE;
                for z in (0..top_z).rev() {
                    let kind = blocks[z].kind();
                    if kind == Block::grass_block() {
                        top_z = z;
                        break;
                    }
                }
                if top_z < WORLD_HEIGHT_CHUNKS * CHUNK_SIZE - tree_height {
                    for z in top_z..top_z + tree_height {
                        blocks[z] = Block::log_block();
                    }
                }
            }
        }
    }
}
