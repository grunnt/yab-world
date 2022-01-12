use common::block::*;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::generator::PregeneratedObject;

use super::ObjectGenerator;

pub struct TreeGenerator {
    random: StdRng,
}

impl TreeGenerator {
    pub fn new(seed: u32) -> Self {
        TreeGenerator {
            random: StdRng::seed_from_u64(seed as u64),
        }
    }
}

impl ObjectGenerator for TreeGenerator {
    fn generate(&mut self) -> PregeneratedObject {
        let size = 12 + (self.random.next_u32() % 12) as usize;
        let mut tree =
            PregeneratedObject::solid(7, 7, size, Block::empty_block(), Block::log_block());
        tree.anchor_x = 3;
        tree.anchor_y = 3;
        tree.anchor_z = 1;
        tree.overwrite_non_empty = false;
        tree.place_on_soil = true;
        // Create leaves
        tree.spray_sphere(3, 3, size - 4, 3, LEAVES_BLOCK, &mut self.random);
        // Create trunk
        tree.set_filled_rectangle(3, 3, 0, 4, 4, size - 4, Block::log_block());
        tree
    }
}
