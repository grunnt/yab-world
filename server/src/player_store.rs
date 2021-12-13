use common::player::PlayerData;
use log::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, time::Instant};
use std::{fs, time::Duration};

const MIN_SAVE_INTERVAL: Duration = Duration::from_millis(5000);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMap(HashMap<String, PlayerData>);

pub struct PlayerStore {
    players: PlayerMap,
    tmp_path: PathBuf,
    path: PathBuf,
    last_save: Instant,
}

impl PlayerStore {
    pub fn load(folder_path: &Path) -> PlayerStore {
        let path = folder_path.join("players.json");
        let tmp_path = folder_path.join("players.json.tmp");
        match fs::read_to_string(&path) {
            Ok(data_string) => match serde_json::from_str(&data_string) {
                Ok(data) => {
                    return PlayerStore {
                        players: data,
                        path,
                        tmp_path,
                        last_save: Instant::now(),
                    };
                }
                Err(e) => {
                    warn!("Error parsing player data file: {}", e);
                }
            },
            Err(e) => {
                warn!("Error loading player data file: {}", e);
            }
        }
        PlayerStore {
            players: PlayerMap(HashMap::new()),
            path,
            tmp_path,
            last_save: Instant::now(),
        }
    }

    pub fn new_player(&mut self, player: &PlayerData) {
        self.players
            .0
            .insert(player.username.clone(), player.clone());
    }

    pub fn get_player(&self, username: &str) -> Option<&PlayerData> {
        self.players.0.get(username)
    }

    pub fn get_mut_player(&mut self, username: &str) -> Option<&mut PlayerData> {
        self.players.0.get_mut(username)
    }

    pub fn save_if_needed(&mut self, force: bool) {
        // Is it time to save chunks?
        if !force && Instant::now().duration_since(self.last_save) < MIN_SAVE_INTERVAL {
            return;
        }
        let data_string = serde_json::to_string_pretty(&self.players).unwrap();
        fs::write(&self.tmp_path, data_string).unwrap();
        if let Err(e) = std::fs::rename(&self.tmp_path, &self.path) {
            error!(
                "Error renaming player data file from {} to {}: {}",
                self.tmp_path.to_string_lossy(),
                self.path.to_string_lossy(),
                e
            );
        }
        self.last_save = Instant::now();
    }
}
