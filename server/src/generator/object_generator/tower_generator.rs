use common::block::*;
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};

use crate::generator::PregeneratedObject;

use super::ObjectGenerator;

/// Generates a tower with several floors, a door, crenelations and a light
pub struct TowerGenerator {
    random: StdRng,
}

impl TowerGenerator {
    pub fn new(seed: u32) -> Self {
        TowerGenerator {
            random: StdRng::seed_from_u64(seed as u64),
        }
    }
}

impl ObjectGenerator for TowerGenerator {
    fn generate(&mut self) -> PregeneratedObject {
        let size_xy = self.random.gen_range(4, 8) * 2 + 1;
        let floor_count = self.random.gen_range(3, 10);
        let floor_height = self.random.gen_range(4, 6);
        let tower_top_z = floor_count * floor_height + 1;
        let mut tower = PregeneratedObject::new(size_xy, size_xy, tower_top_z + 1);
        tower.foundation_block = Some(Block::rock_block());
        tower.place_on_soil = true;
        tower.overwrite_non_empty = true;

        // Lower walls
        let top_floor_z = tower_top_z - floor_height;
        tower.set_rectangle(
            1,
            1,
            0,
            size_xy - 1,
            size_xy - 1,
            top_floor_z + 1,
            Block::bricks_block(),
            false,
        );
        // Keep existing terrain outside lower walls
        tower.set_rectangle(0, 0, 0, size_xy, size_xy, top_floor_z, IGNORE_BLOCK, false);
        // Upper walls
        tower.set_rectangle(
            0,
            0,
            top_floor_z,
            size_xy,
            size_xy,
            tower_top_z,
            Block::bricks_block(),
            false,
        );
        // Crenelations
        tower.set_rectangle(
            0,
            0,
            tower_top_z,
            size_xy,
            size_xy,
            tower_top_z + 1,
            Block::bricks_block(),
            true,
        );
        // Floors
        for floor in 0..floor_count {
            let floor_z = floor * floor_height;
            tower.set_filled_rectangle(
                2,
                2,
                floor_z,
                size_xy - 2,
                size_xy - 2,
                floor_z + 1,
                Block::wood_block(),
            );
        }
        // Top floor
        let top_floor_z = floor_count * floor_height;
        tower.set_filled_rectangle(
            1,
            1,
            top_floor_z,
            size_xy - 1,
            size_xy - 1,
            top_floor_z + 1,
            Block::bricks_block(),
        );
        // Create a cellar by moving the tower down a bit?
        if self.random.next_u32() % 4 == 0 {
            tower.anchor_z = floor_height + 1;
        } else {
            tower.anchor_z = 1;
        }
        // Cut out a door at the anchor position
        tower.anchor_x = size_xy / 2;
        tower.anchor_y = 0;
        // Add door at position of anchor
        tower.set_filled_rectangle(
            tower.anchor_x - 2,
            tower.anchor_y + 1,
            tower.anchor_z - 1,
            tower.anchor_x + 3,
            tower.anchor_y + 3,
            tower.anchor_z + 4,
            Block::bricks_block(),
        );
        tower.set_filled_rectangle(
            tower.anchor_x - 1,
            tower.anchor_y + 1,
            tower.anchor_z,
            tower.anchor_x + 2,
            tower.anchor_y + 3,
            tower.anchor_z + 3,
            Block::empty_block(),
        );
        // Cut out space for a ladder (which is not implemented yet)
        tower.set_filled_rectangle(
            tower.anchor_x,
            size_xy - 3,
            1,
            tower.anchor_x + 1,
            size_xy - 2,
            tower_top_z,
            Block::empty_block(),
        );
        // Cut out windows on the top floor
        tower.set_rectangle(
            0,
            0,
            top_floor_z - floor_height + 3,
            size_xy,
            size_xy,
            top_floor_z - floor_height + 4,
            Block::empty_block(),
            true,
        );
        // And add a lamp to the top floor
        tower.set(
            size_xy / 2,
            size_xy / 2,
            top_floor_z - 1,
            Block::lamp_block(),
        );
        tower
    }
}
