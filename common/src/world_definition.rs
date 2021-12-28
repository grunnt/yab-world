use log::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::Path};

use crate::world_type::GeneratorType;

pub const WORLD_DEF_FILE: &str = "world.json";

pub struct WorldList {
    store_path: PathBuf,
}

impl WorldList {
    pub fn new() -> Self {
        WorldList {
            store_path: PathBuf::from("worlds"),
        }
    }

    pub fn list_worlds(&self) -> Vec<WorldDef> {
        let mut worlds: Vec<WorldDef> = Vec::new();
        if !self.store_path.exists() {
            info!(
                "Worlds directory does not exists, creating {}",
                self.store_path.to_str().unwrap()
            );
            std::fs::create_dir(&self.store_path).unwrap();
        }
        let paths = fs::read_dir(&self.store_path).unwrap();
        for path in paths {
            match path {
                Ok(dir_entry) => {
                    let world_info_file = dir_entry.path().join("world.json");
                    if let Some(world_def) = WorldDef::load(&world_info_file) {
                        worlds.push(world_def);
                    }
                }
                _ => {}
            }
        }
        // Sort by newest first
        worlds.sort_by(|a, b| b.timestamp.partial_cmp(&a.timestamp).unwrap());
        worlds
    }

    pub fn get_world_path(&self, seed: u32) -> PathBuf {
        self.store_path
            .join(PathBuf::from(format!("world_{}", seed)))
    }

    pub fn create_new_world(
        &self,
        seed: u32,
        description: &str,
        world_type: GeneratorType,
    ) -> WorldDef {
        let world_path = self.get_world_path(seed);
        if world_path.exists() {
            panic!("World already exists at {}", world_path.to_str().unwrap());
        }
        fs::create_dir_all(&world_path).unwrap();
        const VERSION: &'static str = env!("CARGO_PKG_VERSION");
        let world_info_file = world_path.join(WORLD_DEF_FILE);
        let world = WorldDef {
            seed,
            world_type,
            description: description.to_string(),
            version: VERSION.to_string(),
            timestamp: now_ms(),
            gametime: 0.3, // Early morning
        };
        world.save(&world_info_file);
        world
    }

    pub fn try_load_world(&self, seed: u32) -> Option<WorldDef> {
        let world_path = self.get_world_path(seed);
        if !world_path.exists() {
            warn!("World not found at: {}", world_path.to_str().unwrap());
            return None;
        }
        let world_info_file = world_path.join(WORLD_DEF_FILE);
        let mut world = WorldDef::load(&world_info_file).unwrap();
        world.version = env!("CARGO_PKG_VERSION").to_string();
        world.save(&world_info_file);
        Some(world)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldDef {
    pub seed: u32,
    pub world_type: GeneratorType,
    pub description: String,
    pub version: String,
    pub timestamp: u128,
    pub gametime: f32,
}

impl WorldDef {
    pub fn load(path: &Path) -> Option<WorldDef> {
        match fs::read_to_string(path) {
            Ok(def_string) => match serde_json::from_str(&def_string) {
                Ok(world_def) => {
                    return Some(world_def);
                }
                Err(e) => {
                    warn!("Error parsing world definition file: {}", e);
                }
            },
            Err(e) => {
                warn!("Error loading world definition file: {}", e);
            }
        }
        None
    }

    pub fn save(&self, path: &Path) {
        let def_string = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, def_string).unwrap();
    }
}

pub fn now_ms() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}
