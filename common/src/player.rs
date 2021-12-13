use crate::inventory::Inventory;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerData {
    pub player_id: u8,
    pub username: String,
    pub inventory: Inventory,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl PlayerData {
    pub fn new(player_id: u8, username: &String) -> PlayerData {
        PlayerData {
            player_id,
            username: username.clone(),
            inventory: Inventory::new(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}
