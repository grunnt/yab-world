mod client;
mod player_store;
pub mod superchunk;
pub mod world_store;
pub mod generator;
mod object_placer;
mod server_world_handler;

extern crate nalgebra_glm as glm;

use chunk_buffer::ChunkBuffer;
use client::*;
use common::world_definition::WorldList;
use common::world_type::GeneratorType;
use common::{block::*, daynight::DayNight, resource::ResourceRegistry};
use gamework::profile::Profile;
use common::{block::BlockRegistry, comms::*};
use common::{chunk::*, player::PlayerData};
use crossbeam::channel::*;
use crossbeam::unbounded;
use floating_duration::TimeAsFloat;
use glm::Vec3;
use log::*;
use num_cpus;
use player_store::PlayerStore;
use rand::Rng;
use std::{thread::{sleep, Builder}};
use std::time::{Duration, Instant};
use world_store::WorldStore;


use crate::generator::WorldGenerator;

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
                let num_cpus = num_cpus::get();
                let world_list = WorldList::new();
                let world_folder = world_list.get_world_path(seed);
                let mut world_store = WorldStore::new(&world_folder, seed, description.as_str(), world_type);
                let world_type = world_store.world_def().world_type;
                let mut player_store = PlayerStore::load(&world_folder);
                let block_registry = BlockRegistry::new(&world_folder);
                let resource_registry = ResourceRegistry::new(&world_folder);
                let mut generator = WorldGenerator::new(num_cpus - 1, seed);
                let mut clients = Vec::new();
                let mut chunks = ChunkBuffer::new();
                let mut broadcast_to_all = Vec::new();
                let mut loop_profile = Profile::new(1);
                let mut client_profile = Profile::new(1);
                let mut generator_profile = Profile::new(1);
                let mut save_profile = Profile::new(1);
                let mut last_message = Instant::now();
                let mut time = Instant::now();
                let mut delta_accumulator = 0.0;
                let mut daynight = DayNight::new(10.0 * 60.0);
                daynight.set_time(world_store.world_def().gametime);
                let mut rng = rand::thread_rng();
                debug!("World time is {}", daynight.get_time());

                // Server warmup: prepare radius of chunks around starting area
                let starting_chunk_col = ChunkColumnPos::new(REGION_SIZE_BLOCKS / CHUNK_SIZE as i16 / 2, REGION_SIZE_BLOCKS / CHUNK_SIZE as i16 / 2);
                let startup_chunk_radius = 2;
                info!(
                    "Warming up server with {} radius around {:?}",
                    startup_chunk_radius, starting_chunk_col
                );

                // Send work to the generators
                let mut warmup_counter = 0;
                for dx in -startup_chunk_radius..startup_chunk_radius + 1 {
                    for dy in -startup_chunk_radius..startup_chunk_radius + 1 {
                        let col = ChunkColumnPos::new(starting_chunk_col.x + dx, starting_chunk_col.y + dy);
                        // See if we can load the chunk column first
                        if let Some(column) = world_store.load_column(col) {
                            chunks.store_column(ChunkColumn::new(
                                col,
                                ColumnStatus::Stored,
                                column,
                            ));
                        } else {
                            // Otherwise generate a new column
                            chunks.store_column(ChunkColumn::new(
                                col,
                                ColumnStatus::Requested,
                                Vec::new(),
                            ));
                            generator.generate(world_type, col);
                            warmup_counter += 1;
                        }
                    }
                }
                // Wait until warmup completes
                while warmup_counter > 0 {
                    sleep(Duration::from_millis(10));
                    if let Some((col, new_chunks)) = generator.try_receive() {
                        if let Some(column) = chunks.get_mut_column(col.x, col.y) {
                            column.set_status(ColumnStatus::Stored);
                            column.chunks = new_chunks;
                            world_store.enqueue_column_save(col, &column.chunks);
                            warmup_counter -= 1;
                        } else {
                            panic!("Generated column not in cache: {:?}", col);
                        }
                    }
                }
                info!("Server warmup complete.");

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
                                        client.data.inventory.add(0, 200);
                                        client.data.inventory.add(1, 50);
                                        client.data.inventory.add(2, 25);
                                        let spawn_range = CHUNK_SIZE as f32 * 0.25;
                                        client.data.x = REGION_SIZE_BLOCKS as f32 / 2.0 + rng.gen_range(-spawn_range, spawn_range);
                                        client.data.y = REGION_SIZE_BLOCKS as f32 / 2.0 + rng.gen_range(-spawn_range, spawn_range);
                                        client.data.z = chunks
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
                                        block_registry: serde_json::to_string(&block_registry).unwrap(),
                                        resource_registry: serde_json::to_string(&resource_registry).unwrap(),
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
                                        // See if we can load the chunk column first
                                        if let Some(column) = world_store.load_column(col) {
                                            chunks.store_column(ChunkColumn::new(
                                                col,
                                                ColumnStatus::Stored,
                                                column,
                                            ));
                                        }
                                        // Get the column or generate a new one
                                        if let Some(column) = chunks.get_column(col.x, col.y) {
                                            if column.is_stored() {
                                                // Use run-length encoding to save bandwidth
                                                let mut block_data = Vec::new();
                                                for chunk in &column.chunks {
                                                    let mut bytes = Vec::new();
                                                    chunk.blocks.rle_encode_to(&mut bytes).unwrap();
                                                    block_data.push(bytes);
                                                }
                                                client.connection.send(
                                                    ServerMessage::ChunkColumn { col, block_data },
                                                );
                                            }
                                        } else {
                                            // Generate this column
                                            chunks.store_column(ChunkColumn::new(
                                                col,
                                                ColumnStatus::Requested,
                                                Vec::new(),
                                            ));
                                            generator.generate(world_type, col);
                                        }
                                    }
                                }
                                ClientMessage::Unsubscribe { columns } => {
                                    if !client.is_signed_in() {
                                        continue;
                                    }
                                    client.unsubscribe_from_set(&columns);
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
                                    let mut allowed = true;
                                    let store_inventory = &mut player_store.get_mut_player(&client.data.username).unwrap().inventory;
                                    if block.is_empty() {
                                        let old_block = chunks.get_block(wbx, wby, wbz);
                                        let block_def = block_registry.get(old_block);
                                        for (resource_type, count) in &block_def.resource_yield {
                                            client.data.inventory.add(*resource_type, *count);
                                            store_inventory.add(*resource_type, *count);
                                        }
                                    } else {
                                        let block_def = block_registry.get(block);
                                        for (resource_type, count) in &block_def.resource_yield {
                                            if client.data.inventory.count(*resource_type) < *count {
                                                warn!(
                                                    "Player {} ({}) tried to build without sufficient resources",
                                                    client.data.username, client.player_id
                                                );
                                                client.connection.disconnect();
                                                allowed = false;
                                            } else {
                                                client.data.inventory.remove(*resource_type, *count);
                                                store_inventory.remove(*resource_type, *count);
                                            }
                                        }
                                    }
                                    if allowed {
                                        
                                        // Now place the new block (which may be empty)
                                        let cp = ChunkPos::from_world_pos(Vec3::new(
                                            wbx as f32, wby as f32, wbz as f32,
                                        ));
                                        if cp.z >= 0 && cp.z < WORLD_HEIGHT_CHUNKS as i16 {
                                            let col = ChunkColumnPos::from_chunk_pos(cp);
                                            if let Some(column) = chunks.get_mut_column(col.x, col.y) {
                                                let chunk = &mut column.chunks[cp.z as usize];
                                                chunk.set_block(
                                                    (wbx - cp.x * CHUNK_SIZE as i16) as usize,
                                                    (wby - cp.y * CHUNK_SIZE as i16) as usize,
                                                    (wbz - cp.z * CHUNK_SIZE as i16) as usize,
                                                    block,
                                                );
                                                broadcast_to_all.push(ServerMessage::SetBlock {
                                                    wbx,
                                                    wby,
                                                    wbz,
                                                    block,
                                                });
                                                world_store.enqueue_chunk_save(&chunk);
                                            }
                                        }
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
                    if let Some((col, new_chunks)) = generator.try_receive() {
                        if let Some(column) = chunks.get_mut_column(col.x, col.y) {
                            column.set_status(ColumnStatus::Stored);
                            column.chunks = new_chunks;
                            for client in &mut clients {
                                if !client.is_signed_in() {
                                    continue;
                                }
                                if client.is_subscribed_to(col) {
                                    // Use run-length encoding to save bandwidth
                                    let mut block_data = Vec::new();
                                    for chunk in &column.chunks {
                                        let mut bytes = Vec::new();
                                        chunk.blocks.rle_encode_to(&mut bytes).unwrap();
                                        block_data.push(bytes);
                                    }
                                    client
                                        .connection
                                        .send(ServerMessage::ChunkColumn { col, block_data });
                                } else {
                                    warn!("Client not subscribed to column {:?}", col);
                                }
                            }
                            world_store.enqueue_column_save(col, &column.chunks);
                        } else {
                            warn!("Generated column not in cache: {:?}", col);
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

                    save_profile.start();
                    world_store.save_world_if_needed(false, daynight.get_time());
                    player_store.save_if_needed(false);
                    save_profile.end();

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
                        save_profile.frame();
                        // debug!(
                        //     "Loop {}/{} client {}/{} generator {}/{} save {}/{}",
                        //     loop_profile.avg_duration_ms(),
                        //     loop_profile.max_duration_ms(),
                        //     client_profile.avg_duration_ms(),
                        //     client_profile.max_duration_ms(),
                        //     generator_profile.avg_duration_ms(),
                        //     generator_profile.max_duration_ms(),
                        //     save_profile.avg_duration_ms(),
                        //     save_profile.max_duration_ms()
                        // );
                        last_message = Instant::now();
                    }

                    match shutdown_rx.try_recv() {
                        Ok(message) => {
                            info!("Shutting down: {}", message);
                            for client in &mut clients {
                                client.connection.disconnect();
                            }
                            server_comms.shutdown();
                            world_store.save_world_if_needed(true, daynight.get_time());
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
