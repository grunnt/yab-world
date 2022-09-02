use crate::physics::physics::Physics;
use crate::world::worldhandler::WorldHandler;
use crate::GameContext;
use crate::{in_game::InGameState, render::*};
use common::block::{BlockDef, BlockRegistry};
use common::chunk::*;
use common::comms::*;
use common::world_type::GeneratorType;
use gamework::*;
use log::*;
use nalgebra_glm::*;
use server::YabServer;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;

const PRELOAD_COLUMN_COUNT: usize = 49; // Make sure this contains all chunks "nearby" the player
const PRELOAD_SLEEP_DURATION: Duration = Duration::from_millis(10);

#[derive(Copy, Clone, Debug)]
pub enum StartGameStage {
    StartingServer,
    Connecting,
    WaitForSignInConfirm,
    RequestingChunks,
    DownloadingChunks,
    EnteringGame,
}

pub struct StartGameState {
    preload_count: usize,
    player_chunk_stored: bool,
    stage: StartGameStage,
}

impl StartGameState {
    pub fn new() -> Self {
        StartGameState {
            preload_count: 0,
            player_chunk_stored: false,
            stage: StartGameStage::StartingServer,
        }
    }
}

impl State<GameContext> for StartGameState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {
        self.preload_count = 0;
        self.player_chunk_stored = false;
        self.stage = StartGameStage::StartingServer;
    }

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        gui: &egui::Context,
        _input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let stage = self.stage;
        let (message, progress) = match stage {
            StartGameStage::StartingServer => {
                // Start the server if needed
                if let Some(server_address) = &data.server_address {
                    let mut server = YabServer::new(server_address);
                    let world_type = if let Some(world_type) = data.world_type {
                        world_type
                    } else {
                        GeneratorType::Default
                    };
                    server.run(false, data.seed, data.description.clone(), world_type);
                    data.server = Some(server);
                }
                self.stage = StartGameStage::Connecting;
                ("Connecting", 0.0)
            }
            StartGameStage::Connecting => {
                // Connect to the server and sign in
                let connect_to_address = data.connect_to_address.as_ref().unwrap().clone();
                let socket_addr: SocketAddr = connect_to_address
                    .parse()
                    .expect("Cannot parse server address");
                let mut comms_client = CommsClient::new(socket_addr);
                comms_client
                    .send(ClientMessage::SignIn {
                        username: "my user".to_string(),
                    })
                    .unwrap();
                data.comms_client = Some(comms_client);
                self.stage = StartGameStage::WaitForSignInConfirm;
                ("Signing in", 0.1)
            }
            StartGameStage::WaitForSignInConfirm => {
                if let Some(message) = data.comms_client.as_mut().unwrap().try_receive() {
                    match message {
                        ServerMessage::SignInConfirm {
                            player_id,
                            x,
                            y,
                            z,
                            yaw,
                            pitch,
                            inventory,
                            gametime,
                            block_registry,
                        } => {
                            let starting_chunk_col = ChunkColumnPos::from_chunk_pos(
                                ChunkPos::from_world_pos(data.starting_position),
                            );
                            info!(
                                "Sign in confirm - player id {} and position {},{},{}, starting {:?}",
                                player_id, x, y, z, starting_chunk_col
                            );
                            data.starting_position = Vec3::new(x, y, z);
                            data.starting_yaw = yaw;
                            data.starting_pitch = pitch;
                            data.player_id = Some(player_id);
                            data.inventory = inventory;
                            debug!("Client gametime {}", gametime);
                            data.daynight.set_time(gametime);
                            let blocks: Vec<BlockDef> =
                                serde_json::from_str(&block_registry).unwrap();
                            debug!("Block registry contains {} blocks", blocks.len());
                            data.block_registry = BlockRegistry::from_blocks(blocks);
                            data.block_renderer = Some(
                                BlockRenderer::new(
                                    &context.video().gl(),
                                    &context.assets(),
                                    &data.block_registry,
                                )
                                .unwrap(),
                            );
                            data.physics = Some(Physics::new(&data.block_registry));
                            // Request chunks for preloading
                            data.world = Some(
                                WorldHandler::new(
                                    data.config.render_range_chunks as usize,
                                    starting_chunk_col,
                                    data.comms_client.as_ref().unwrap().clone_col_receiver(),
                                    data.block_registry.clone(),
                                )
                                .unwrap(),
                            );
                            self.stage = StartGameStage::RequestingChunks;
                        }
                        _ => {
                            panic!("unexpected server response for sign in");
                        }
                    }
                } else {
                    sleep(PRELOAD_SLEEP_DURATION);
                }
                ("Requesting chunks", 0.2)
            }
            StartGameStage::RequestingChunks => {
                let mut columns = Vec::new();
                for _ in 0..PRELOAD_COLUMN_COUNT {
                    if let Some(col) = data.world.as_mut().unwrap().get_next_request(None) {
                        columns.push(col);
                    } else {
                        break;
                    }
                }
                self.preload_count = 0;
                if !columns.is_empty() {
                    self.preload_count += columns.len();
                    data.comms_client
                        .as_mut()
                        .unwrap()
                        .send(ClientMessage::Subscribe { columns })
                        .unwrap();
                }
                self.stage = StartGameStage::DownloadingChunks;
                ("Downloading chunks", 0.3)
            }
            StartGameStage::DownloadingChunks => {
                let world = data.world.as_mut().unwrap();

                if self.preload_count <= 0 {
                    self.stage = StartGameStage::EnteringGame;
                    return StateCommand::None;
                }

                sleep(PRELOAD_SLEEP_DURATION);
                while let Some((_, status)) = world.try_receive_status() {
                    if status == ColumnStatus::Received && self.preload_count > 0 {
                        self.preload_count -= 1;
                    }
                }
                world.try_receive_columns();

                let starting_chunk_col = ChunkColumnPos::from_chunk_pos(ChunkPos::from_world_pos(
                    data.starting_position,
                ));
                if !self.player_chunk_stored {
                    if let Some(col) = world
                        .chunks
                        .get_column(starting_chunk_col.x, starting_chunk_col.y)
                    {
                        if col.status() == ColumnStatus::Meshed {
                            self.player_chunk_stored = true;
                        }
                    }
                }

                let block_renderer = data.block_renderer.as_mut().unwrap();
                while let Some((cp, vertices, translucent_vertices)) = world.try_receive_vertices()
                {
                    if let Some(mesh) = BlockMesh::new(&context.video().gl(), &vertices, false) {
                        if block_renderer.meshes.contains_key(&cp) {
                            warn!("Duplicate mesh for {:?}", cp);
                        }
                        block_renderer.insert_mesh_pos(cp, mesh);
                    }
                    if let Some(mesh) =
                        BlockMesh::new(&context.video().gl(), &translucent_vertices, false)
                    {
                        if block_renderer.translucent_meshes.contains_key(&cp) {
                            warn!("Duplicate mesh for {:?}", cp);
                        }
                        block_renderer.insert_translucent_mesh_pos(cp, mesh);
                    }
                }

                let progress = (PRELOAD_COLUMN_COUNT - self.preload_count) as f32
                    / PRELOAD_COLUMN_COUNT as f32;
                ("Downloading chunks", 0.3 + progress * 0.7)
            }
            StartGameStage::EnteringGame => {
                return StateCommand::ReplaceState {
                    state: Box::new(InGameState::new()),
                };
            }
        };
        show_message(gui, message, progress);

        StateCommand::None
    }

    fn resize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn render(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn shutdown(&mut self) {}
}

fn show_message(gui: &egui::Context, message: &str, progress: f32) {
    egui::CentralPanel::default().show(gui, |ui| {
        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                ui.label(message);
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            },
        );
    });
}
