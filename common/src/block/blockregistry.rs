use super::{block::AIR_BLOCK, Block};
use crate::resource::*;
use gamework::{video::color::ColorRGBu8, Assets};
use log::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};

pub const FACE_XP: usize = 0;
pub const FACE_XM: usize = 1;
pub const FACE_YP: usize = 2;
pub const FACE_YM: usize = 3;
pub const FACE_ZP: usize = 4;
pub const FACE_ZM: usize = 5;

pub const WATER_BLOCK: u32 = 1;
pub const DIRT_BLOCK: u32 = 512;
pub const GRASS_BLOCK: u32 = 513;
pub const ROCK_BLOCK: u32 = 514;
pub const SAND_BLOCK: u32 = 515;
pub const SANDSTONE_BLOCK: u32 = 516;
pub const WOOD_BLOCK: u32 = 517;
pub const BEDROCK_BLOCK: u32 = 518;
pub const LAMP_BLOCK: u32 = 519;
pub const IRON_BLOCK: u32 = 520;
pub const GOLD_BLOCK: u32 = 521;
pub const ICE_BLOCK: u32 = 522;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDef {
    pub name: String,
    pub color: ColorRGBu8,
    pub textures: Vec<String>,
    pub light: u8,
    pub buildable: bool,
    pub resource_yield: Vec<(Resource, u32)>,
    pub resource_cost: Vec<(Resource, u32)>,
}

impl BlockDef {
    pub fn yields(&self, resource: Resource) -> bool {
        for res in &self.resource_yield {
            if res.0 == resource {
                return true;
            }
        }
        false
    }

    pub fn costs(&self, resource: Resource) -> bool {
        for res in &self.resource_cost {
            if res.0 == resource {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRegistry {
    blocks: HashMap<Block, BlockDef>,
    texture_index_map: HashMap<String, usize>,
    block_texture_map: HashMap<Block, [f32; 6]>,
}

impl BlockRegistry {
    pub fn empty() -> BlockRegistry {
        BlockRegistry {
            blocks: HashMap::new(),
            texture_index_map: HashMap::new(),
            block_texture_map: HashMap::new(),
        }
    }

    pub fn default() -> BlockRegistry {
        let mut blocks = HashMap::new();

        blocks.insert(
            AIR_BLOCK,
            BlockDef {
                name: "Air".to_string(),
                color: ColorRGBu8::new(0, 0, 0),
                textures: Vec::new(),
                light: 0,
                buildable: false,
                resource_yield: vec![],
                resource_cost: vec![],
            },
        );
        blocks.insert(
            WATER_BLOCK,
            BlockDef {
                name: "Water".to_string(),
                color: ColorRGBu8::new(79, 190, 255),
                textures: vec![
                    "water".to_string(),
                    "water".to_string(),
                    "water".to_string(),
                    "water".to_string(),
                    "water".to_string(),
                    "water".to_string(),
                ],
                light: 0,
                buildable: false,
                resource_yield: vec![],
                resource_cost: vec![],
            },
        );
        blocks.insert(
            DIRT_BLOCK,
            BlockDef {
                name: "Dirt".to_string(),
                color: ColorRGBu8::new(127, 90, 61),
                textures: vec![
                    "dirt".to_string(),
                    "dirt".to_string(),
                    "dirt".to_string(),
                    "dirt".to_string(),
                    "dirt".to_string(),
                    "dirt".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 2)],
            },
        );
        blocks.insert(
            GRASS_BLOCK,
            BlockDef {
                name: "Grass".to_string(),
                color: ColorRGBu8::new(87, 133, 75),
                textures: vec![
                    "grass_block_side".to_string(),
                    "grass_block_side".to_string(),
                    "grass_block_side".to_string(),
                    "grass_block_side".to_string(),
                    "grass_block_top".to_string(),
                    "dirt".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 2)],
            },
        );
        blocks.insert(
            ROCK_BLOCK,
            BlockDef {
                name: "Rock".to_string(),
                color: ColorRGBu8::new(125, 122, 121),
                textures: vec![
                    "stone".to_string(),
                    "stone".to_string(),
                    "stone".to_string(),
                    "stone".to_string(),
                    "stone".to_string(),
                    "stone".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 2)],
                resource_cost: vec![(0, 3)],
            },
        );
        blocks.insert(
            SAND_BLOCK,
            BlockDef {
                name: "Sand".to_string(),
                color: ColorRGBu8::new(213, 198, 153),
                textures: vec![
                    "sand".to_string(),
                    "sand".to_string(),
                    "sand".to_string(),
                    "sand".to_string(),
                    "sand".to_string(),
                    "sand".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 2)],
            },
        );
        blocks.insert(
            SANDSTONE_BLOCK,
            BlockDef {
                name: "Sandstone".to_string(),
                color: ColorRGBu8::new(173, 162, 126),
                textures: vec![
                    "sandstone".to_string(),
                    "sandstone".to_string(),
                    "sandstone".to_string(),
                    "sandstone".to_string(),
                    "sandstone_top".to_string(),
                    "sandstone_bottom".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 2)],
            },
        );
        blocks.insert(
            WOOD_BLOCK,
            BlockDef {
                name: "Wood".to_string(),
                color: ColorRGBu8::new(106, 82, 48),
                textures: vec![
                    "oak_planks".to_string(),
                    "oak_planks".to_string(),
                    "oak_planks".to_string(),
                    "oak_planks".to_string(),
                    "oak_planks".to_string(),
                    "oak_planks".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 2)],
            },
        );
        blocks.insert(
            BEDROCK_BLOCK,
            BlockDef {
                name: "Bedrock".to_string(),
                color: ColorRGBu8::new(25, 15, 20),
                textures: vec![
                    "bedrock".to_string(),
                    "bedrock".to_string(),
                    "bedrock".to_string(),
                    "bedrock".to_string(),
                    "bedrock".to_string(),
                    "bedrock".to_string(),
                ],
                light: 0,
                buildable: false,
                resource_yield: vec![],
                resource_cost: vec![],
            },
        );
        blocks.insert(
            LAMP_BLOCK,
            BlockDef {
                name: "Lamp".to_string(),
                color: ColorRGBu8::new(255, 255, 255),
                textures: vec![
                    "lamp".to_string(),
                    "lamp".to_string(),
                    "lamp".to_string(),
                    "lamp".to_string(),
                    "lamp".to_string(),
                    "lamp".to_string(),
                ],
                light: 15,
                buildable: true,
                resource_yield: vec![(2, 1)],
                resource_cost: vec![(2, 1)],
            },
        );
        blocks.insert(
            IRON_BLOCK,
            BlockDef {
                name: "Iron".to_string(),
                color: ColorRGBu8::new(250, 220, 25),
                textures: vec![
                    "ore_iron".to_string(),
                    "ore_iron".to_string(),
                    "ore_iron".to_string(),
                    "ore_iron".to_string(),
                    "ore_iron".to_string(),
                    "ore_iron".to_string(),
                ],
                light: 0,
                buildable: false,
                resource_yield: vec![(1, 1)],
                resource_cost: vec![],
            },
        );
        blocks.insert(
            GOLD_BLOCK,
            BlockDef {
                name: "Gold".to_string(),
                color: ColorRGBu8::new(250, 220, 25),
                textures: vec![
                    "ore_gold".to_string(),
                    "ore_gold".to_string(),
                    "ore_gold".to_string(),
                    "ore_gold".to_string(),
                    "ore_gold".to_string(),
                    "ore_gold".to_string(),
                ],
                light: 0,
                buildable: false,
                resource_yield: vec![(2, 1)],
                resource_cost: vec![],
            },
        );
        blocks.insert(
            ICE_BLOCK,
            BlockDef {
                name: "Ice".to_string(),
                color: ColorRGBu8::new(250, 220, 25),
                textures: vec![
                    "ice".to_string(),
                    "ice".to_string(),
                    "ice".to_string(),
                    "ice".to_string(),
                    "ice".to_string(),
                    "ice".to_string(),
                ],
                light: 0,
                buildable: false,
                resource_yield: vec![],
                resource_cost: vec![],
            },
        );
        let mut index = ICE_BLOCK + 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Small Lamp".to_string(),
                color: ColorRGBu8::new(255, 255, 255),
                textures: vec![
                    "small_lamp".to_string(),
                    "small_lamp".to_string(),
                    "small_lamp".to_string(),
                    "small_lamp".to_string(),
                    "small_lamp".to_string(),
                    "small_lamp".to_string(),
                ],
                light: 10,
                buildable: true,
                resource_yield: vec![(1, 1)],
                resource_cost: vec![(1, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Stone bricks".to_string(),
                color: ColorRGBu8::new(218, 224, 224),
                textures: vec![
                    "stone_bricks".to_string(),
                    "stone_bricks".to_string(),
                    "stone_bricks".to_string(),
                    "stone_bricks".to_string(),
                    "stone_bricks".to_string(),
                    "stone_bricks".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "White concrete".to_string(),
                color: ColorRGBu8::new(218, 224, 224),
                textures: vec![
                    "white_concrete".to_string(),
                    "white_concrete".to_string(),
                    "white_concrete".to_string(),
                    "white_concrete".to_string(),
                    "white_concrete".to_string(),
                    "white_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Gray concrete".to_string(),
                color: ColorRGBu8::new(152, 152, 144),
                textures: vec![
                    "gray_concrete".to_string(),
                    "gray_concrete".to_string(),
                    "gray_concrete".to_string(),
                    "gray_concrete".to_string(),
                    "gray_concrete".to_string(),
                    "gray_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Black concrete".to_string(),
                textures: vec![
                    "black_concrete".to_string(),
                    "black_concrete".to_string(),
                    "black_concrete".to_string(),
                    "black_concrete".to_string(),
                    "black_concrete".to_string(),
                    "black_concrete".to_string(),
                ],
                color: ColorRGBu8::new(5, 5, 5),
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Red concrete".to_string(),
                color: ColorRGBu8::new(166, 62, 61),
                textures: vec![
                    "red_concrete".to_string(),
                    "red_concrete".to_string(),
                    "red_concrete".to_string(),
                    "red_concrete".to_string(),
                    "red_concrete".to_string(),
                    "red_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Brown concrete".to_string(),
                color: ColorRGBu8::new(127, 92, 60),
                textures: vec![
                    "brown_concrete".to_string(),
                    "brown_concrete".to_string(),
                    "brown_concrete".to_string(),
                    "brown_concrete".to_string(),
                    "brown_concrete".to_string(),
                    "brown_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Orange concrete".to_string(),
                color: ColorRGBu8::new(233, 128, 3),
                textures: vec![
                    "orange_concrete".to_string(),
                    "orange_concrete".to_string(),
                    "orange_concrete".to_string(),
                    "orange_concrete".to_string(),
                    "orange_concrete".to_string(),
                    "orange_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Yellow concrete".to_string(),
                color: ColorRGBu8::new(245, 193, 45),
                textures: vec![
                    "yellow_concrete".to_string(),
                    "yellow_concrete".to_string(),
                    "yellow_concrete".to_string(),
                    "yellow_concrete".to_string(),
                    "yellow_concrete".to_string(),
                    "yellow_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Green concrete".to_string(),
                color: ColorRGBu8::new(126, 189, 49),
                textures: vec![
                    "green_concrete".to_string(),
                    "green_concrete".to_string(),
                    "green_concrete".to_string(),
                    "green_concrete".to_string(),
                    "green_concrete".to_string(),
                    "green_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Cyan concrete".to_string(),
                color: ColorRGBu8::new(45, 147, 162),
                textures: vec![
                    "cyan_concrete".to_string(),
                    "cyan_concrete".to_string(),
                    "cyan_concrete".to_string(),
                    "cyan_concrete".to_string(),
                    "cyan_concrete".to_string(),
                    "cyan_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Blue concrete".to_string(),
                color: ColorRGBu8::new(66, 162, 212),
                textures: vec![
                    "blue_concrete".to_string(),
                    "blue_concrete".to_string(),
                    "blue_concrete".to_string(),
                    "blue_concrete".to_string(),
                    "blue_concrete".to_string(),
                    "blue_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Purple concrete".to_string(),
                color: ColorRGBu8::new(129, 60, 177),
                textures: vec![
                    "purple_concrete".to_string(),
                    "purple_concrete".to_string(),
                    "purple_concrete".to_string(),
                    "purple_concrete".to_string(),
                    "purple_concrete".to_string(),
                    "purple_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );
        index += 1;
        blocks.insert(
            index,
            BlockDef {
                name: "Pink concrete".to_string(),
                color: ColorRGBu8::new(225, 131, 168),
                textures: vec![
                    "pink_concrete".to_string(),
                    "pink_concrete".to_string(),
                    "pink_concrete".to_string(),
                    "pink_concrete".to_string(),
                    "pink_concrete".to_string(),
                    "pink_concrete".to_string(),
                ],
                light: 0,
                buildable: true,
                resource_yield: vec![(0, 1)],
                resource_cost: vec![(0, 1)],
            },
        );

        let mut registry = BlockRegistry {
            blocks,
            texture_index_map: HashMap::new(),
            block_texture_map: HashMap::new(),
        };
        registry.build_texture_index();
        registry
    }

    fn build_texture_index(&mut self) {
        // Build map from texture name to texture array index (should be loaded in this order later)
        let mut texture_index_map = HashMap::new();
        let mut block_keys: Vec<Block> = self.blocks.keys().map(|k| *k).collect();
        block_keys.sort();
        for key in &block_keys {
            let block_def = self.blocks.get(key).unwrap();
            for texture in &block_def.textures {
                if !texture_index_map.contains_key(texture) {
                    let next = texture_index_map.len();
                    texture_index_map.insert(texture.clone(), next);
                }
            }
        }
        // Build map from block type to an array of texture indices
        let mut block_texture_map = HashMap::new();
        for (block_type, block_def) in &self.blocks {
            let mut texture_indices = [0.0; 6];
            if block_def.textures.len() > 0 {
                assert!(block_def.textures.len() == 6);
                for i in 0..6 {
                    let index = texture_index_map
                        .get(block_def.textures.get(i).unwrap())
                        .unwrap();
                    texture_indices[i] = *index as f32;
                }
                block_texture_map.insert(*block_type, texture_indices);
            }
        }
        self.texture_index_map = texture_index_map;
        self.block_texture_map = block_texture_map;
    }

    pub fn new(folder_path: &Path) -> BlockRegistry {
        let path = folder_path.join("blocks.json");
        match fs::read_to_string(&path) {
            Ok(string) => match serde_json::from_str(&string) {
                Ok(blocks) => {
                    let mut registry = BlockRegistry {
                        blocks,
                        texture_index_map: HashMap::new(),
                        block_texture_map: HashMap::new(),
                    };
                    registry.build_texture_index();
                    return registry;
                }
                Err(e) => {
                    warn!("Error loading file: {}", e);
                }
            },
            Err(e) => {
                warn!("Error loading file: {}", e);
            }
        }
        let mut defaults = BlockRegistry::default();
        defaults.save(&path);
        defaults
    }

    fn save(&mut self, path: &Path) {
        let string = serde_json::to_string_pretty(&self.blocks).unwrap();
        fs::write(path, string).unwrap();
        info!("File {:?} created", path);
    }

    pub fn get(&self, id: Block) -> &BlockDef {
        self.blocks.get(&id).unwrap()
    }

    pub fn all_blocks(&self) -> &HashMap<Block, BlockDef> {
        &self.blocks
    }

    pub fn texture_index_map(&self) -> &HashMap<String, usize> {
        &self.texture_index_map
    }

    pub fn block_texture_map(&self) -> &HashMap<Block, [f32; 6]> {
        &self.block_texture_map
    }

    pub fn texture_paths_sorted(&self, assets: &Assets) -> Vec<PathBuf> {
        let mut texture_paths: Vec<(String, usize)> = self
            .texture_index_map()
            .iter()
            .map(|file_index| (file_index.0.clone(), *file_index.1))
            .collect();
        texture_paths.sort_by(|a, b| (a.1).partial_cmp(&b.1).unwrap());
        texture_paths
            .iter()
            .map(|file_index| assets.assets_path(&format!("block_textures/{}.png", file_index.0)))
            .collect()
    }
}
