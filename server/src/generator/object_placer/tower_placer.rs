use crate::generator::Generator;
use common::block::Block;
use common::block::*;

use super::ObjectGrid;

const STAIRWELL_BLOCKS: [(i16, i16); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
];

pub struct PoiPlacer {
    grid: ObjectGrid,
}

impl PoiPlacer {
    pub fn new(seed: u32) -> Self {
        PoiPlacer {
            grid: ObjectGrid::new(seed, 64, 5, 15, 0.3),
        }
    }

    pub fn place(
        &mut self,
        x: i16,
        y: i16,
        blocks: &mut Vec<Block>,
        generator: &mut dyn Generator,
    ) {
        let object_opt = self.grid.try_get_object(x, y);
        if let Some(object) = object_opt {
            let (center_rock_top_z, center_water_top_z, _) =
                generator.determine_rock_water_top(object.center_x, object.center_y);
            if center_rock_top_z > center_water_top_z {
                // Tower
                let height = 8 + object.random as i16 % 32;
                let roof_height = (height - 2) as usize;
                let half_size = object.size / 2;
                let ground_z = rock_top_z(&blocks);
                let edge = x == object.center_x - half_size
                    || x == object.center_x + half_size
                    || y == object.center_y - half_size
                    || y == object.center_y + half_size;
                let top;
                if edge {
                    let height = (height - ((x + (y % 2)) % 2).min(1)) as usize;
                    top = center_rock_top_z + height;
                    // Wall
                    if ground_z < center_rock_top_z {
                        for z in ground_z..center_rock_top_z {
                            blocks[z] = Block::rock_block();
                        }
                    }
                    for z in center_rock_top_z..top {
                        blocks[z] = Block::bricks_block();
                    }
                    // Window
                    if x == object.center_x || y == object.center_y {
                        blocks[center_rock_top_z + roof_height - 2] = Block::empty_block();
                    }
                    // Cut out a door
                    let cut_door = match object.random as i16 % 4 {
                        0 => {
                            if x == object.center_x && y < object.center_y {
                                Some((x, y - 1))
                            } else {
                                None
                            }
                        }
                        1 => {
                            if x == object.center_x && y > object.center_y {
                                Some((x, y + 1))
                            } else {
                                None
                            }
                        }
                        2 => {
                            if y == object.center_y && x < object.center_x {
                                Some((x - 1, y))
                            } else {
                                None
                            }
                        }
                        3 => {
                            if y == object.center_y && x > object.center_x {
                                Some((x + 1, y))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some((door_ground_x, door_ground_y)) = cut_door {
                        let (_, _, top_z) =
                            generator.determine_rock_water_top(door_ground_x, door_ground_y);
                        for z in top_z..top_z + 3 {
                            blocks[z] = Block::empty_block();
                        }
                        blocks[top_z + 4] = Block::lamp_block();
                    }
                } else {
                    if center_rock_top_z <= center_rock_top_z {
                        for z in center_rock_top_z..center_rock_top_z {
                            blocks[z] = Block::bricks_block();
                        }
                    }
                    blocks[center_rock_top_z] = Block::wood_block();
                    blocks[center_rock_top_z + roof_height] =
                        if (object.center_x - x).abs() > 1 || (object.center_y - y).abs() > 1 {
                            Block::bricks_block()
                        } else if x == object.center_x && y == object.center_y {
                            Block::lamp_block()
                        } else {
                            Block::empty_block()
                        };

                    if object.center_x == x && object.center_y == y {
                        for z in center_rock_top_z + 1..center_rock_top_z + roof_height + 1 {
                            blocks[z] = Block::bricks_block();
                        }
                    } else {
                        for z in center_rock_top_z + 1..center_rock_top_z + roof_height + 1 {
                            let stair_step = z % 8;
                            let (sx, sy) = STAIRWELL_BLOCKS[stair_step];
                            if x == object.center_x + sx && y == object.center_y + sy {
                                blocks[z] = Block::rock_block();
                            } else if (object.center_x - x).abs() < 2
                                && (object.center_y - y).abs() < 2
                            {
                                blocks[z] = Block::empty_block();
                            } else if (z - center_rock_top_z) % 4 == 0 {
                                blocks[z] = Block::wood_block();
                            } else {
                                blocks[z] = Block::empty_block();
                            }
                        }
                    };

                    top = center_rock_top_z + roof_height + 1;
                }
                for z in top..top + 32 {
                    // Clear any trees that were here
                    blocks[z] = Block::empty_block();
                }
            }
        }
    }
}

pub fn rock_top_z(blocks: &Vec<Block>) -> usize {
    for z in (0..blocks.len()).rev() {
        if blocks[z] == Block::rock_block() {
            return z;
        }
    }
    0
}

pub fn top_z(blocks: &Vec<Block>) -> usize {
    for z in (0..blocks.len()).rev() {
        if blocks[z] != Block::empty_block() {
            return z;
        }
    }
    0
}
