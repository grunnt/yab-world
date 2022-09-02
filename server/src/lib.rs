mod client;
pub mod generator;
mod player_store;
mod server_world_handler;
pub mod superchunk;
pub mod world_store;

extern crate nalgebra_glm as glm;

use client::*;
use common::world_definition::WorldsStore;
use common::world_type::GeneratorType;
use common::{block::BlockRegistry, comms::*};
use common::{block::*, daynight::DayNight};
use common::{chunk::*, player::PlayerData};
use crossbeam::channel::*;
use crossbeam::unbounded;
use floating_duration::TimeAsFloat;
use gamework::profile::Profile;
use glm::Vec3;
use log::*;
use player_store::PlayerStore;
use rand::Rng;
use std::path::PathBuf;
use std::thread::{sleep, Builder};
use std::time::{Duration, Instant};

use crate::server_world_handler::ServerWorldHandler;

const SLEEP_DURATION: Duration = Duration::from_millis(10);
const MAX_PLAYERS: usize = 16;
const UPDATE_TIME_STEP: f32 = 0.5;

pub struct YabServer {
    address: String,
    shutdown_sender: Option<Sender<String>>,
}

impl YabServer {
    pub fn new(address: &String) -> YabServer {
        YabServer {
            address: address.clone(),
            shutdown_sender: None,
        }
    }

    pub fn run(&mut self, wait: bool, seed: u32, description: String, world_type: GeneratorType) {
        let (shutdown_tx, shutdown_rx) = unbounded();
        self.shutdown_sender = Some(shutdown_tx);
        let mut server_comms: CommsServer = CommsServer::new(self.address.as_str());
        let handle = Builder::new()
            .name("yab-world-server".to_string())
            .spawn(move || {
                let world_list = WorldsStore::new();
                let world_folder = world_list.get_world_path(seed);
                // TODO fix dynamic loading of blocks json (from world folder if present, otherwise default)
                // Also make sure its copied to assets folder (better yet, place it there?)
                let data_path = if world_folder.exists() {
                    world_folder.clone()
                } else { 
                    PathBuf::from("server_data") 
                };
                let block_registry = BlockRegistry::load_or_create(&PathBuf::from("server_data") );
                debug!("Server block registry contains {} blocks", block_registry.all_blocks().len());

                let mut world = if world_folder.exists() {
                    ServerWorldHandler::load(seed, &block_registry)
                } else {
                    ServerWorldHandler::new(seed, description.as_str(), world_type, &block_registry)
                };
                let mut player_store = PlayerStore::load(&world_folder);
                let mut clients = Vec::new();
                let mut broadcast_to_all = Vec::new();
                let mut loop_profile = Profile::new(1);
                let mut client_profile = Profile::new(1);
                let mut generator_profile = Profile::new(1);
                let mut update_profile = Profile::new(1);
                let mut last_message = Instant::now();
                let mut time = Instant::now();
                let mut delta_accumulator = 0.0;
                let mut daynight = DayNight::new(10.0 * 60.0);
                daynight.set_time(world.time_on_start());
                let mut rng = rand::thread_rng();
                debug!("World time is {}", daynight.get_time());

                // Preload a starting area of the world
                let starting_chunk_col = ChunkColumnPos::new(REGION_SIZE_BLOCKS / CHUNK_SIZE as i16 / 2, REGION_SIZE_BLOCKS / CHUNK_SIZE as i16 / 2);
                let startup_chunk_range = 4;
                info!(
                    "Preparing spawn area with {} radius around {:?}",
                    startup_chunk_range, starting_chunk_col
                );
                world.prepare_spawn_area(starting_chunk_col, startup_chunk_range);
                info!("Spawn area prepared, starting main server loop");
                loop {
                    loop_profile.start();

                    let delta = time.elapsed().as_fractional_secs() as f32;
                    time = Instant::now();

                    // Receive incoming clients
                    client_profile.start();
                    if let Some(mut connection) = server_comms.try_get_channel() {
                        if clients.len() < MAX_PLAYERS {
                            let player_id = get_free_player_id(&clients);
                            clients.push(Client::new(connection, player_id));
                            info!("Player {} connected", player_id);
                        } else {
                            info!("Max clients reached, connection denied");
                            connection.disconnect();
                        }
                    }

                    // Handle client messages
                    let mut signed_in_player_ids = Vec::new();
                    for client in &mut clients {
                        if let Some(message) = client.connection.try_receive() {
                            match message {
                                ClientMessage::SignIn { username } => {
                                    client.sign_in(username.clone());
                                    if let Some(player) =
                                        player_store.get_player(&client.data.username)
                                    {
                                        // Existing player
                                        client.data = player.clone();
                                        info!(
                                            "Existing player {} ({}) signed in with data {:?}",
                                            client.data.username, client.player_id, client.data
                                        );
                                    } else {
                                        // New player
                                        client.data = PlayerData::new(
                                            client.player_id,
                                            &client.data.username,
                                        );
                                        // Give some starting resources 
                                        // TODO fix
                                        // client.data.inventory.add(0, 200);
                                        // client.data.inventory.add(1, 50);
                                        // client.data.inventory.add(2, 25);
                                        let spawn_range = CHUNK_SIZE as f32 * 0.25;
                                        client.data.x = REGION_SIZE_BLOCKS as f32 / 2.0 + rng.gen_range(-spawn_range, spawn_range);
                                        client.data.y = REGION_SIZE_BLOCKS as f32 / 2.0 + rng.gen_range(-spawn_range, spawn_range);
                                        client.data.z = world
                                            .get_top_z(client.data.x as i16, client.data.y as i16)
                                            as f32
                                            + 3.0;
                                        player_store.new_player(&client.data);
                                        info!(
                                            "New player {} ({}) signed in with data {:?}",
                                            client.data.username, client.player_id, client.data
                                        );
                                    }
                                    client.connection.send(ServerMessage::SignInConfirm {
                                        player_id: client.player_id,
                                        x: client.data.x,
                                        y: client.data.y,
                                        z: client.data.z,
                                        yaw: client.data.yaw,
                                        pitch: client.data.pitch,
                                        inventory: client.data.inventory.clone(),
                                        gametime: daynight.get_time(),
                                        block_registry: serde_json::to_string(&block_registry.all_blocks()).unwrap(),
                                    });
                                    broadcast_to_all.push(ServerMessage::PlayerSpawn {
                                        x: client.data.x,
                                        y: client.data.y,
                                        z: client.data.z,
                                        yaw: client.data.yaw,
                                        pitch: client.data.pitch,
                                        player_id: client.player_id,
                                        username: username,
                                    });
                                    signed_in_player_ids.push(client.player_id);
                                }
                                ClientMessage::SignOut {} => {
                                    info!(
                                        "User {} ({}) signed out",
                                        client.data.username, client.player_id
                                    );
                                    client.connection.disconnect();
                                }
                                ClientMessage::PositionUpdate {
                                    x,
                                    y,
                                    z,
                                    yaw,
                                    pitch,
                                } => {
                                    if !client.is_signed_in() {
                                        continue;
                                    }
                                    client.update_position(x, y, z, yaw, pitch);
                                    let player =
                                        player_store.get_mut_player(&client.data.username).unwrap();
                                    player.x = x;
                                    player.y = y;
                                    player.z = z;
                                    player.yaw = yaw;
                                    player.pitch = pitch;
                                    broadcast_to_all.push(ServerMessage::PositionUpdate {
                                        player_id: client.player_id,
                                        x: client.data.x,
                                        y: client.data.y,
                                        z: client.data.z,
                                        yaw: client.data.yaw,
                                        pitch: client.data.pitch,
                                    });
                                }
                                ClientMessage::Subscribe { columns } => {
                                    if !client.is_signed_in() {
                                        continue;
                                    }
                                    for col in columns {
                                        // Subscribe to column to receive future changes
                                        client.subscribe_to(col);
                                        if let Some(block_data) = world.try_clone_existing_column(col) {
                                            // If it is available, send immediately
                                            client.connection.send(
                                                ServerMessage::ChunkColumn { col, block_data },
                                            );
                                        } else {
                                            world.place_generate_request(col);
                                        }
                                    }
                                }
                                ClientMessage::Unsubscribe { columns } => {
                                    if !client.is_signed_in() {
                                        continue;
                                    }
                                    client.unsubscribe_from_set(&columns);
                                    for col in columns {
                                        world.retract_generate_request(col);
                                    }
                                }
                                ClientMessage::SetBlock {
                                    wbx,
                                    wby,
                                    wbz,
                                    block,
                                } => {
                                    if !client.is_signed_in() {
                                        continue;
                                    }
                                    // Add or remove resources from inventory
                                    let block = block.kind();
                                    let mut allowed = true;
                                    let store_inventory = &mut player_store.get_mut_player(&client.data.username).unwrap().inventory;
                                    if block == AIR_BLOCK_KIND {
                                        // A block was removed
                                        let old_block = world.get_block(wbx, wby, wbz).kind();
                                        client.data.inventory.add(old_block, 1);
                                        store_inventory.add(old_block, 1);
                                    } else {
                                        // A block was placed
                                        if client.data.inventory.count(block) < 1 {
                                            warn!(
                                                "Player {} ({}) tried to build without sufficient resources",
                                                client.data.username, client.player_id
                                            );
                                            client.connection.disconnect();
                                            allowed = false;
                                        } else {
                                            client.data.inventory.remove(block, 1);
                                            store_inventory.remove(block, 1);
                                        }
                                    }
                                    if allowed {
                                        world.set_block(wbx, wby, wbz, block);
                                        broadcast_to_all.push(ServerMessage::SetBlock {
                                            wbx,
                                            wby,
                                            wbz,
                                            block,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Sign in new players by sending them existing spawns
                    for player_id in signed_in_player_ids {
                        let mut spawns = Vec::new();
                        for client in &clients {
                            if client.is_signed_in() && client.player_id != player_id {
                                spawns.push(ServerMessage::PlayerSpawn {
                                    x: client.data.x,
                                    y: client.data.y,
                                    z: client.data.z,
                                    yaw: client.data.yaw,
                                    pitch: client.data.pitch,
                                    player_id: client.player_id,
                                    username: client.data.username.clone(),
                                })
                            }
                        }
                        for client in &mut clients {
                            if client.player_id == player_id {
                                for spawn in &spawns {
                                    client.connection.send(spawn.clone());
                                }
                            }
                        }
                    }

                    // Handle world updates
                    delta_accumulator += delta;
                    while delta_accumulator >= UPDATE_TIME_STEP {
                        delta_accumulator -= UPDATE_TIME_STEP;
                        daynight.update(UPDATE_TIME_STEP);
                    }


                    // Broadcast messages
                    for message in &broadcast_to_all {
                        match message {
                            ServerMessage::SetBlock {
                                wbx,
                                wby,
                                wbz,
                                block,
                            } => {
                                let cp = ChunkPos::from_world_pos(Vec3::new(
                                    *wbx as f32,
                                    *wby as f32,
                                    *wbz as f32,
                                ));
                                for broadcast_target in &mut clients {
                                    if !broadcast_target.is_signed_in() {
                                        continue;
                                    }
                                      if broadcast_target
                                        .is_subscribed_to(ChunkColumnPos::from_chunk_pos(cp))
                                    {
                                        broadcast_target.connection.send(ServerMessage::SetBlock {
                                            wbx: *wbx,
                                            wby: *wby,
                                            wbz: *wbz,
                                            block: *block,
                                        });
                                    }
                                }
                            }
                            ServerMessage::PlayerSpawn {
                                x,
                                y,
                                z,
                                yaw,
                                pitch,
                                player_id,
                                username,
                            } => {
                                for broadcast_target in &mut clients {
                                    if !broadcast_target.is_signed_in() {
                                        continue;
                                    }
                                    if broadcast_target.player_id == *player_id {
                                        continue;
                                    }
                                    broadcast_target
                                        .connection
                                        .send(ServerMessage::PlayerSpawn {
                                            x: *x,
                                            y: *y,
                                            z: *z,
                                            yaw: *yaw,
                                            pitch: *pitch,
                                            player_id: *player_id,
                                            username: username.clone(),
                                        });
                                }
                            }
                            ServerMessage::PlayerDespawn { player_id } => {
                                for broadcast_target in &mut clients {
                                    if !broadcast_target.is_signed_in() {
                                        continue;
                                    }
                                    broadcast_target.connection.send(
                                        ServerMessage::PlayerDespawn {
                                            player_id: *player_id,
                                        },
                                    );
                                }
                            }
                            ServerMessage::PositionUpdate {
                                x,
                                y,
                                z,
                                yaw,
                                pitch,
                                player_id,
                            } => {
                                for broadcast_target in &mut clients {
                                    if !broadcast_target.is_signed_in() {
                                        continue;
                                    }
                                    if broadcast_target.player_id == *player_id {
                                        continue;
                                    }
                                    broadcast_target.connection.send(
                                        ServerMessage::PositionUpdate {
                                            x: *x,
                                            y: *y,
                                            z: *z,
                                            yaw: *yaw,
                                            pitch: *pitch,
                                            player_id: *player_id,
                                        },
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    broadcast_to_all.clear();
                    client_profile.end();

                    // Handle world generator output
                    generator_profile.start();
                    if let Some((col, block_data)) = world.try_get_generated_column(true){
                        for client in &mut clients {
                            if !client.is_signed_in() {
                                continue;
                            }
                            if client.is_subscribed_to(col) {
                                client
                                    .connection
                                    .send(ServerMessage::ChunkColumn { col, block_data: block_data.clone() });
                            }
                        }
                    }
                    generator_profile.end();

                    // Filter out closed clients
                    clients.retain(|c| {
                        if !c.connection.connected {
                            info!("Player {} despawned", c.player_id);
                            broadcast_to_all.push(ServerMessage::PlayerDespawn {
                                player_id: c.player_id,
                            })
                        }
                        c.connection.connected
                    });

                    update_profile.start();
                    world.update( daynight.get_time());
                    player_store.save_if_needed(false);
                    update_profile.end();

                    // Sleep for a little if there is nothing to do to reduce CPU usage
                    if clients.is_empty() {
                        sleep(SLEEP_DURATION);
                    }
                    loop_profile.end();

                    let time_since_last = Instant::now() - last_message;
                    if time_since_last.as_secs() >= 1 {
                        loop_profile.frame();
                        client_profile.frame();
                        generator_profile.frame();
                        update_profile.frame();
                        debug!(
                            "Loop {}/{} client {}/{} generator {}/{} update {}/{}",
                            loop_profile.avg_ms,
                            loop_profile.max_ms,
                            client_profile.avg_ms,
                            client_profile.max_ms,
                            generator_profile.avg_ms,
                            generator_profile.max_ms,
                            update_profile.avg_ms,
                            update_profile.max_ms
                        );
                        last_message = Instant::now();
                    }

                    match shutdown_rx.try_recv() {
                        Ok(message) => {
                            info!("Shutting down: {}", message);
                            for client in &mut clients {
                                client.connection.disconnect();
                            }
                            server_comms.shutdown();
                            world.save(daynight.get_time());
                            player_store.save_if_needed(true);
                            break;
                        }
                        _ => {}
                    }
                }
            })
            .unwrap();
        if wait {
            handle.join().unwrap();
        }
    }

    pub fn shutdown(&mut self, message: String) {
        if let Some(sender) = &self.shutdown_sender {
            match sender.send(message) {
                Err(e) => warn!("Error shutting down server: {}", e),
                _ => {}
            }
        } else {
            warn!("No channel to server, cannot shutdown");
        }
    }
}

/// Find the lowest unused player ID
fn get_free_player_id(clients: &Vec<Client>) -> u8 {
    // Get a sorted list of existing player IDs
    let mut player_ids = Vec::new();
    for client in clients {
        player_ids.push(client.player_id);
    }
    player_ids.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    // Now find the lowest unused
    let mut last_id = 0;
    for id in player_ids {
        if id > last_id + 1 {
            return id + 1;
        }
        last_id = id;
    }
    last_id + 1
}
