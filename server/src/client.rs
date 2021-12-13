use common::comms::*;
use common::{chunk::*, player::PlayerData};
use std::collections::HashSet;
use std::time::Instant;

pub struct Client {
    pub player_id: u8,
    pub connection: CommChannel,
    _connect_time: Instant,
    pub authenticated: bool,
    pub data: PlayerData,
    chunk_subscriptions: HashSet<ChunkColumnPos>,
}

impl Client {
    pub fn new(connection: CommChannel, player_id: u8) -> Client {
        Client {
            player_id,
            connection,
            _connect_time: Instant::now(),
            authenticated: false,
            data: PlayerData::new(player_id, &format!("player-{}", player_id)),
            chunk_subscriptions: HashSet::new(),
        }
    }

    pub fn is_signed_in(&self) -> bool {
        self.authenticated && self.connection.connected
    }

    pub fn sign_in(&mut self, username: String) {
        self.data.username = username.clone();
        self.authenticated = true;
        self.chunk_subscriptions.clear();
    }

    /// Update client position
    pub fn update_position(&mut self, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) {
        self.data.x = x;
        self.data.y = y;
        self.data.z = z;
        self.data.yaw = yaw;
        self.data.pitch = pitch;
    }

    pub fn subscribe_to(&mut self, col: ChunkColumnPos) {
        self.chunk_subscriptions.insert(col);
    }

    pub fn unsubscribe_from_set(&mut self, remove_columns: &HashSet<ChunkColumnPos>) {
        // Note: used retain() here previously, this apparantly doesnt work correctly (removes too many sometimes?)
        for col in remove_columns {
            self.chunk_subscriptions.remove(col);
        }
    }

    pub fn is_subscribed_to(&self, col: ChunkColumnPos) -> bool {
        self.chunk_subscriptions.contains(&col)
    }
}
