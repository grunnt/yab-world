use crate::generator::Generator;
use common::block::Block;
use common::block::*;

use super::ObjectGrid;

pub struct TreePlacer {
    grid: ObjectGrid,
}

impl TreePlacer {
    pub fn new(seed: u32) -> Self {
        TreePlacer {
            grid: ObjectGrid::new(seed, 8, 3, 7, 0.6),
        }
    }

    pub fn place(
        &mut self,
        x: i16,
        y: i16,
        blocks: &mut Vec<Block>,
        _generator: &mut dyn Generator,
    ) {
        let object_opt = self.grid.try_get_object(x, y);
        if let Some(object) = object_opt {
            if let Some(ground_z) = dry_rock_top_z(&blocks) {
                // Tree
                let height = 6 + (object.density * 6.0) as usize + object.random as usize % 4;
                if x == object.center_x && y == object.center_y {
                    for z in ground_z..ground_z + height {
                        blocks[z] = Block::log_block();
                    }
                    blocks[ground_z + height] = Block::green_concrete_block();
                }
            }
        }
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
