use common::block::*;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::generator::PregeneratedObject;

use super::ObjectGenerator;

/// Generates a simple tree with a trunk and a semispherical top of leaves
pub struct TreeGenerator {
    random: StdRng,
    log_block: Block,
    leaves_block: Block,
}

impl TreeGenerator {
    pub fn new(seed: u32, block_registry: &BlockRegistry) -> Self {
        let log_block = block_registry.block_kind_from_code("log");
        let leaves_block = block_registry.block_kind_from_code("leaves");
        TreeGenerator {
            random: StdRng::seed_from_u64(seed as u64),
            log_block,
            leaves_block,
        }
    }
}

impl ObjectGenerator for TreeGenerator {
    fn generate(&mut self) -> PregeneratedObject {
        let radius = 2 + (self.random.next_u32() % 3) as usize;
        let size_xy = radius * 2 + 1;
        let size_z = size_xy + 1 + (self.random.next_u32() % 8) as usize;
        let mut tree =
            PregeneratedObject::solid(size_xy, size_xy, size_z, AIR_BLOCK, self.log_block);
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
            self.leaves_block,
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
            self.log_block,
        );
        tree
    }
}
