use crate::{gui::Label, GameContext};
use crate::{
    gui::{gui_renderer::GuiRenderer, ProgressBar},
    world::worldhandler::WorldHandler,
};
use crate::{in_game::InGameState, render::*};
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
    gui: Gui<GuiRenderer>,
    preload_count: usize,
    player_chunk_stored: bool,
    stage: StartGameStage,
    progress_label: WidgetId,
    progress_bar: WidgetId,
}

impl StartGameState {
    pub fn new() -> Self {
        let mut gui = Gui::new(
            vec![flex_col(1.0), fixed_col(400.0), flex_col(1.0)],
            vec![
                flex_row(1.0),
                fixed_row(50.0),
                fixed_row(30.0),
                fixed_row(50.0),
                flex_row(1.0),
            ],
        );
        gui.place(
            gui.root_id(),
            1,
            1,
            Box::new(Label::new("Loading...".to_string())),
            CellAlignment::Center,
        );
        let progress_bar = gui.place(
            gui.root_id(),
            1,
            2,
            Box::new(ProgressBar::new(0.0, 1.0)),
            CellAlignment::Fill,
        );
        let progress_label = gui.place(
            gui.root_id(),
            1,
            3,
            Box::new(Label::new("".to_string())),
            CellAlignment::Fill,
        );
        StartGameState {
            gui,
            preload_count: 0,
            player_chunk_stored: false,
            stage: StartGameStage::StartingServer,
            progress_label,
            progress_bar,
        }
    }
}

impl State<GameContext> for StartGameState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {
        self.gui.set_value(
            &self.progress_label,
            GuiValue::String("Starting server".to_string()),
        );
        self.gui.set_value(&self.progress_bar, GuiValue::Float(0.0));
        self.preload_count = 0;
        self.player_chunk_stored = false;
        self.stage = StartGameStage::StartingServer;
    }

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        self.gui.update(
            input_events,
            context.video().screen_size(),
            data.gui_renderer_mut(),
        );
        let stage = self.stage;
        match stage {
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
                self.gui.set_value(
                    &self.progress_label,
                    GuiValue::String("Connecting to server".to_string()),
                );
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
                self.gui.set_value(
                    &self.progress_label,
                    GuiValue::String("Waiting for server response".to_string()),
                );
                self.stage = StartGameStage::WaitForSignInConfirm;
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
                            resource_registry,
                        } => {
                            data.starting_position = Vec3::new(x, y, z);
                            data.starting_yaw = yaw;
                            data.starting_pitch = pitch;
                            data.player_id = Some(player_id);
                            data.inventory = inventory;
                            data.daynight.set_time(gametime);
                            debug!("Client gametime {}", gametime);
                            data.block_registry = serde_json::from_str(&block_registry).unwrap();
                            data.block_renderer = Some(
                                BlockRenderer::new(
                                    &context.video().gl(),
                                    &context.assets(),
                                    &data.block_registry,
                                )
                                .unwrap(),
                            );
                            data.resource_registry =
                                serde_json::from_str(&resource_registry).unwrap();
                            // Request chunks for preloading
                            let starting_chunk_col = ChunkColumnPos::from_chunk_pos(
                                ChunkPos::from_world_pos(data.starting_position),
                            );
                            info!(
                                "Sign in confirm - player id {} and position {},{},{}, starting {:?}",
                                player_id, x, y, z, starting_chunk_col
                            );
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
                            self.gui.set_value(
                                &self.progress_label,
                                GuiValue::String("Requesting world data".to_string()),
                            );
                        }
                        _ => {
                            panic!("unexpected server response for sign in");
                        }
                    }
                } else {
                    sleep(PRELOAD_SLEEP_DURATION);
                }
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
                self.gui.set_value(
                    &self.progress_label,
                    GuiValue::String("Receiving world data".to_string()),
                );
            }
            StartGameStage::DownloadingChunks => {
                let world = data.world.as_mut().unwrap();

                if self.preload_count <= 0 && self.player_chunk_stored {
                    self.gui.set_value(
                        &self.progress_label,
                        GuiValue::String("Entering world".to_string()),
                    );
                    self.stage = StartGameStage::EnteringGame;
                    return StateCommand::None;
                } else if self.preload_count <= 0 {
                    self.gui.set_value(
                        &self.progress_label,
                        GuiValue::String("Waiting for player chunk".to_string()),
                    );
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
                    if let Some(mesh) =
                        BlockMesh::new(&context.video().gl(), &vertices, true, false)
                    {
                        if block_renderer.meshes.contains_key(&cp) {
                            warn!("Duplicate mesh for {:?}", cp);
                        }
                        block_renderer.insert_mesh_pos(cp, mesh);
                    }
                    if let Some(mesh) =
                        BlockMesh::new(&context.video().gl(), &translucent_vertices, true, false)
                    {
                        if block_renderer.translucent_meshes.contains_key(&cp) {
                            warn!("Duplicate mesh for {:?}", cp);
                        }
                        block_renderer.insert_translucent_mesh_pos(cp, mesh);
                    }
                }

                let progress = (PRELOAD_COLUMN_COUNT - self.preload_count) as f32
                    / PRELOAD_COLUMN_COUNT as f32;
                self.gui
                    .set_value(&self.progress_bar, GuiValue::Float(progress));
                self.gui.set_value(
                    &self.progress_label,
                    GuiValue::String(
                        format!("Receiving world data ({}%)", (progress * 100.0) as i16)
                            .to_string(),
                    ),
                );
            }
            StartGameStage::EnteringGame => {
                self.gui.set_value(
                    &self.progress_label,
                    GuiValue::String("Entering world".to_string()),
                );
                return StateCommand::ReplaceState {
                    state: Box::new(InGameState::new()),
                };
            }
        }

        StateCommand::None
    }

    fn resize(&mut self, data: &mut GameContext, context: &mut SystemContext) {
        data.gui_renderer_mut()
            .resize(context.video().screen_size());
    }

    fn render(&mut self, data: &mut GameContext, _context: &mut SystemContext) {
        self.gui.paint(data.gui_renderer_mut());
        data.gui_renderer_mut().render();
    }

    fn shutdown(&mut self) {}
}
