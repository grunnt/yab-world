use super::block::{SOLID_BIT_MASK, TRANSPARENT_BIT_MASK};
use super::{Block, BlockTrait};
use gamework::Assets;
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

// Air is a special case with ID 0
pub const AIR_BLOCK_KIND: u32 = 0;
pub const AIR_BLOCK: u32 = AIR_BLOCK_KIND | TRANSPARENT_BIT_MASK;
pub const BEDROCK_BLOCK_KIND: u32 = 1;
pub const BEDROCK_BLOCK: u32 = BEDROCK_BLOCK_KIND | SOLID_BIT_MASK;

// Special block ID for world generation that should never be used in the world itself
pub const IGNORE_BLOCK: u32 = std::u32::MAX;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDef {
    pub code: String,
    pub name: String,
    pub textures: Vec<String>,
    pub solid: bool,
    pub transparent: bool,
    pub buildable: bool,
    pub light: u8,
}

#[derive(Clone, Debug)]
pub struct BlockRegistry {
    blocks: Vec<BlockDef>,
    texture_index_map: HashMap<String, usize>,
    block_texture_map: Vec<Option<[f32; 6]>>,
}

impl BlockRegistry {
    pub fn empty() -> BlockRegistry {
        BlockRegistry {
            blocks: Vec::new(),
            texture_index_map: HashMap::new(),
            block_texture_map: Vec::new(),
        }
    }

    pub fn from_blocks(blocks: Vec<BlockDef>) -> Self {
        let mut registry = BlockRegistry::empty();
        registry.blocks = blocks;
        registry.build_indexes();
        registry
    }

    pub fn default() -> BlockRegistry {
        let mut registry = BlockRegistry::empty();

        // First blocks (index 0 and 1) is always air and bedrock
        registry.add("air", "Air", Vec::new(), false, true, 0, false);
        registry.add(
            "bedrock",
            "Bedrock",
            vec![
                "bedrock".to_string(),
                "bedrock".to_string(),
                "bedrock".to_string(),
                "bedrock".to_string(),
                "bedrock".to_string(),
                "bedrock".to_string(),
            ],
            true,
            false,
            0,
            false,
        );
        registry.add(
            "water",
            "Water",
            vec![
                "water".to_string(),
                "water".to_string(),
                "water".to_string(),
                "water".to_string(),
                "water".to_string(),
                "water".to_string(),
            ],
            false,
            true,
            0,
            false,
        );
        registry.add(
            "stone",
            "Stone",
            vec![
                "stone".to_string(),
                "stone".to_string(),
                "stone".to_string(),
                "stone".to_string(),
                "stone".to_string(),
                "stone".to_string(),
            ],
            true,
            false,
            0,
            true,
        );
        registry.add(
            "dirt",
            "Dirt",
            vec![
                "dirt".to_string(),
                "dirt".to_string(),
                "dirt".to_string(),
                "dirt".to_string(),
                "dirt".to_string(),
                "dirt".to_string(),
            ],
            true,
            false,
            0,
            true,
        );
        registry.add(
            "sand",
            "Sand",
            vec![
                "sand".to_string(),
                "sand".to_string(),
                "sand".to_string(),
                "sand".to_string(),
                "sand".to_string(),
                "sand".to_string(),
            ],
            true,
            false,
            0,
            true,
        );
        registry.add(
            "grass",
            "Grass",
            vec![
                "grass_block_side".to_string(),
                "grass_block_side".to_string(),
                "grass_block_side".to_string(),
                "grass_block_side".to_string(),
                "grass_block_top".to_string(),
                "dirt".to_string(),
            ],
            true,
            false,
            0,
            true,
        );
        registry.build_indexes();
        registry
    }

    fn add(
        &mut self,
        code: &str,
        name: &str,
        textures: Vec<String>,
        solid: bool,
        transparent: bool,
        light: u8,
        buildable: bool,
    ) -> Block {
        let index = self.blocks.len() as u32;
        self.blocks.push(BlockDef {
            code: code.to_string(),
            name: name.to_string(),
            textures,
            solid,
            transparent,
            light,
            buildable,
        });
        return index as Block;
    }

    pub fn block_from_code(&self, code: &str) -> Block {
        let mut block = 0;
        while block < self.blocks.len() {
            let block_def = &self.blocks[block];
            if block_def.code == code {
                let mut block = block as Block;
                if block_def.transparent {
                    block.toggle_transparency();
                }
                if block_def.solid {
                    block.toggle_solidity();
                }
                return block as Block;
            }
            block += 1;
        }
        return AIR_BLOCK;
    }

    pub fn block_kind_from_code(&self, code: &str) -> Block {
        let mut block = 0;
        while block < self.blocks.len() {
            let block_def = &self.blocks[block];
            if block_def.code == code {
                return block as Block;
            }
            block += 1;
        }
        return AIR_BLOCK_KIND;
    }

    pub fn set_block_flags(&self, block: Block) -> Block {
        let mut block = block.kind();
        let block_def = &self.blocks[block as usize];
        if block_def.transparent {
            block.toggle_transparency();
        }
        if block_def.solid {
            block.toggle_solidity();
        }
        return block;
    }

    /// Load a block definition file from the given path or create a default one if the file cannot be loaded.
    pub fn load_or_create(folder_path: &Path) -> BlockRegistry {
        let path = folder_path.join("blocks.json");
        match fs::read_to_string(&path) {
            Ok(string) => match serde_json::from_str(&string) {
                Ok(blocks) => {
                    let mut registry = BlockRegistry::empty();
                    registry.blocks = blocks;
                    registry.build_indexes();
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

    fn build_indexes(&mut self) {
        // Build map from texture name to texture array index (should be loaded in this order later)
        let mut texture_index_map = HashMap::new();
        let mut block = 0;
        while block < self.blocks.len() {
            let block_def = &self.blocks[block];
            for texture in &block_def.textures {
                if !texture_index_map.contains_key(texture) {
                    let next = texture_index_map.len();
                    texture_index_map.insert(texture.clone(), next);
                }
            }
            block += 1;
        }
        // Build map from block type to an array of texture indices
        let mut block_texture_map = Vec::new();
        let mut block = 0;
        while block < self.blocks.len() {
            let block_def = &self.blocks[block];
            let mut texture_indices = [0.0; 6];
            if block_def.textures.len() > 0 {
                assert!(block_def.textures.len() == 6);
                for i in 0..6 {
                    let index = texture_index_map
                        .get(block_def.textures.get(i).unwrap())
                        .unwrap();
                    texture_indices[i] = *index as f32;
                }
                block_texture_map.push(Some(texture_indices));
            } else {
                block_texture_map.push(None);
            }
            block += 1;
        }

        self.texture_index_map = texture_index_map;
        self.block_texture_map = block_texture_map;
    }

    pub fn get(&self, block: Block) -> &BlockDef {
        self.blocks.get(block.kind() as usize).unwrap()
    }

    pub fn all_blocks(&self) -> &Vec<BlockDef> {
        &self.blocks
    }

    pub fn texture_index_map(&self) -> &HashMap<String, usize> {
        &self.texture_index_map
    }

    pub fn block_texture_map(&self) -> &Vec<Option<[f32; 6]>> {
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
            .map(|file_index| assets.path(&format!("block_textures/{}.png", file_index.0)))
            .collect()
    }
}
