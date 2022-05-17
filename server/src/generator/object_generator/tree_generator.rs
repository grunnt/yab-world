use common::block::*;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::generator::PregeneratedObject;

use super::ObjectGenerator;

/// Generates a simple tree with a trunk and a semispherical top of leaves
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
        let radius = 2 + (self.random.next_u32() % 3) as usize;
        let size_xy = radius * 2 + 1;
        let size_z = size_xy + 1 + (self.random.next_u32() % 8) as usize;
        let mut tree = PregeneratedObject::solid(
            size_xy,
            size_xy,
            size_z,
            Block::empty_block(),
            Block::log_block(),
        );
        tree.anchor_x = size_xy / 2;
        tree.anchor_y = size_xy / 2;
        tree.anchor_z = 1;
        tree.overwrite_non_empty = false;
        tree.place_on_soil = true;
        // Create leaves
        tree.spray_sphere(
            tree.anchor_x,
            tree.anchor_y,
            size_z - (radius + 1),
            radius,
            LEAVES_BLOCK,
            &mut self.random,
        );
        // Create trunk
        tree.set_filled_rectangle(
            tree.anchor_x,
            tree.anchor_y,
            0,
            tree.anchor_x + 1,
            tree.anchor_y + 1,
            size_z - (radius + 1),
            Block::log_block(),
        );
        tree
    }
}
