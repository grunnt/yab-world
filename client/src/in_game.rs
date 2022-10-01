use crate::block_button::*;
use crate::{block_select::BlockSelectState, *};
use common::block::*;
use common::inventory::Inventory;
use common::{chunk::*, player::PlayerData};
use egui::plot::{Line, PlotPoints};
use egui::Align2;
use floating_duration::TimeAsFloat;
use gamework::{InputEvent, MouseButton};
use gamework::{Key, StateCommand};
use log::*;

const POS_UPDATE_INTERVAL: f64 = 1.0 / 20.0;
const PHYSICS_TIME_STEP: f32 = 0.02;
const CAMERA_Z_OFFSET: f32 = 0.45;
const MIN_JUMP_VELOCITY: f32 = 0.25;
const BLOCK_PLACE_TIME_S: f32 = 0.25;
const BLOCK_REMOVE_TIME_S: f32 = 0.5;

pub struct InGameState {
    buffer_col: ChunkColumnPos,
    open_column_requests: usize,
    debug_info: bool,
    player_flying: bool,
    player_body: u32,
    delta_accumulator: f32,
    in_water: bool,
    rendering: Option<Rendering>,
    changed_chunks_to_mesh: HashSet<ChunkPos>,
    out_of_range_columns: HashSet<ChunkColumnPos>,
    block_place_timer: f32,
    block_remove_timer: f32,
    show_debug_gui: bool,
}

impl InGameState {
    pub fn new() -> Self {
        let buffer_col = ChunkColumnPos::from_chunk_pos(ChunkPos::zero());
        let open_column_requests: usize = 0;
        let debug_info = false;
        let player_flying = false;

        InGameState {
            buffer_col,
            open_column_requests,
            debug_info,
            player_flying,
            player_body: 0,
            delta_accumulator: 0.0,
            in_water: false,
            rendering: None,
            changed_chunks_to_mesh: HashSet::new(),
            out_of_range_columns: HashSet::new(),
            block_remove_timer: 0.0,
            block_place_timer: 0.0,
            show_debug_gui: false,
        }
    }

    fn rendering(&self) -> &Rendering {
        self.rendering.as_ref().unwrap()
    }

    fn rendering_mut(&mut self) -> &mut Rendering {
        self.rendering.as_mut().unwrap()
    }

    fn handle_event(
        &mut self,
        event: &InputEvent,
        context: &mut GameContext,
        system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        match event {
            InputEvent::MouseMove { dx, dy, .. } => {
                if system.input().get_mouse_captured() {
                    self.rendering_mut().camera.rotate(
                        *dx as f32 * context.config.camera_sensitivity_x * 0.0025,
                        *dy as f32 * context.config.camera_sensitivity_y * 0.0025,
                    );
                }
            }
            InputEvent::KeyPress { key, shift } => match key {
                Key::Space => {
                    let object = context.physics_mut().get_object_mut(self.player_body);
                    if object.on_ground && !self.player_flying {
                        object.velocity.z = object.velocity.z.max(MIN_JUMP_VELOCITY);
                        system.audio_mut().play_sound("jump");
                    }
                }
                Key::Tab => {
                    system.input_mut().set_mouse_captured(false);
                    return StateCommand::OpenState {
                        state: Box::new(BlockSelectState::new(
                            &context.block_registry,
                            &context.inventory,
                        )),
                    };
                }
                Key::X => {
                    let captured = system.input().get_mouse_captured();
                    system.input_mut().set_mouse_captured(!captured);
                    debug!("Mouse captured: {}", system.input().get_mouse_captured());
                }
                Key::I => {
                    self.debug_info = !self.debug_info;
                }
                Key::L => {
                    self.rendering_mut().toggle_render_lines();
                }
                Key::K => {
                    self.rendering_mut().toggle_render_fog();
                }
                Key::F => {
                    if *shift {
                        self.player_flying = !self.player_flying;
                        context
                            .physics_mut()
                            .set_object_colliding(self.player_body, !self.player_flying);
                        debug!("Player flying: {}", self.player_flying);
                    }
                }
                Key::Escape => {
                    if let Some(server) = &mut context.server {
                        server.shutdown("server player exiting game".to_string());
                    }
                    system.input_mut().set_mouse_captured(false);
                    return StateCommand::CloseState;
                }
                Key::G => self.show_debug_gui = !self.show_debug_gui,
                _ => (),
            },
            _ => {}
        }
        StateCommand::None
    }

    fn handle_game_input(
        &mut self,
        system: &mut SystemContext,
        context: &mut GameContext,
        delta: f32,
    ) {
        if !system.input().get_mouse_captured() {
            return;
        }
        if system.input().is_mouse_button_down(MouseButton::Right) {
            if self.block_place_timer <= std::f32::EPSILON {
                // Place a block
                if let Some(hit) = context.world().cast_ray(
                    &self.rendering().camera().position,
                    self.rendering().camera().get_direction(),
                    8.0,
                    false,
                ) {
                    let (wbx, wby, wbz) = (
                        (hit.hit_block_pos.x + hit.hit_norm.x).floor() as i16,
                        (hit.hit_block_pos.y + hit.hit_norm.y).floor() as i16,
                        (hit.hit_block_pos.z + hit.hit_norm.z).floor() as i16,
                    );
                    if context.world().chunks.are_all_neighbours_stored(
                        ChunkColumnPos::from_world_block_coords(wbx, wby),
                    ) {
                        if !context.is_occopied_by_body(wbx, wby, wbz) {
                            // Do we have sufficient resources?
                            let selected_block = context.selected_block.kind();
                            if context.inventory.count(selected_block) > 0 {
                                // Remove resources from inventory
                                context.inventory.remove(selected_block, 1);
                                // Place block
                                let block = context.block_registry.set_block_flags(selected_block);
                                context.world_mut().set_block_add_dirty(
                                    wbx,
                                    wby,
                                    wbz,
                                    block,
                                    &mut self.changed_chunks_to_mesh,
                                );
                                // Send change to the server
                                context
                                    .comms_client_mut()
                                    .send(ClientMessage::SetBlock {
                                        wbx,
                                        wby,
                                        wbz,
                                        block: selected_block,
                                    })
                                    .unwrap();
                                system.audio_mut().play_sound("build");
                            } else {
                                debug!(
                                    "Insufficient resources to place block {:?}",
                                    selected_block
                                );
                            }
                        }
                    }
                }
                self.block_place_timer = BLOCK_PLACE_TIME_S;
            }
        }
        self.block_place_timer -= delta;
        if self.block_place_timer < 0.0 {
            self.block_place_timer = 0.0;
        }
        // Remove a block
        let dig_beam_emitter_handle = context.dig_beam_emitter_handle;
        let mut dig_beam_active = false;
        if system.input().is_mouse_button_down(MouseButton::Left) {
            if let Some(hit) = context.world().cast_ray(
                &self.rendering().camera.position,
                self.rendering().camera.get_direction(),
                8.0,
                false,
            ) {
                dig_beam_active = true;
                let player_target_handle = context.player_target_handle;
                context
                    .particles_mut()
                    .update_position_handle(player_target_handle, hit.hit_pos);
                self.block_remove_timer += delta;
                if self.block_remove_timer > BLOCK_REMOVE_TIME_S {
                    // See if we need to remove a block
                    let (wbx, wby, wbz) = (
                        (hit.hit_block_pos.x).floor() as i16,
                        (hit.hit_block_pos.y).floor() as i16,
                        (hit.hit_block_pos.z).floor() as i16,
                    );
                    let block = context.world().chunks.get_block(wbx, wby, wbz);
                    context.dig_effect(Vec3::new(
                        wbx as f32 + 0.5,
                        wby as f32 + 0.5,
                        wbz as f32 + 0.5,
                    ));
                    if context.world().chunks.are_all_neighbours_stored(
                        ChunkColumnPos::from_world_block_coords(wbx, wby),
                    ) {
                        // Add block to inventory
                        context.inventory.add(block.kind(), 1);
                        // Clear the block
                        context.world_mut().set_block_add_dirty(
                            wbx,
                            wby,
                            wbz,
                            AIR_BLOCK,
                            &mut self.changed_chunks_to_mesh,
                        );
                        // Send change to the server
                        context
                            .comms_client_mut()
                            .send(ClientMessage::SetBlock {
                                wbx,
                                wby,
                                wbz,
                                block: AIR_BLOCK,
                            })
                            .unwrap();
                    }
                    self.block_remove_timer -= BLOCK_REMOVE_TIME_S;
                    system.audio_mut().play_sound("build");
                }
            }
        } else {
            self.block_remove_timer = 0.0;
        }
        context
            .particles_mut()
            .emitter_mut(dig_beam_emitter_handle)
            .unwrap()
            .active = dig_beam_active;
    }
}

impl State<GameContext> for InGameState {
    fn initialize(&mut self, data: &mut GameContext, context: &mut SystemContext) {
        debug!("Activating {}", self.type_name());
        data.selected_block = data.block_registry.block_from_code("dirt");

        // ----------------------------------
        // Game world
        // ----------------------------------
        info!("Initialize game world");

        let starting_chunk_col =
            ChunkColumnPos::from_chunk_pos(ChunkPos::from_world_pos(data.starting_position));

        info!(
            "Starting pos {:?} in chunk {:?}",
            data.starting_position, starting_chunk_col
        );

        data.last_position = data.starting_position;
        let position = data.starting_position.clone();
        self.player_body = data.physics_mut().new_object(
            position.x,
            position.y,
            position.z,
            Vec3::new(0.6, 0.6, 1.5),
        );

        self.rendering = Some(Rendering::new(data, context));
        self.rendering_mut().camera.yaw = data.starting_yaw;
        self.rendering_mut().camera.pitch = data.starting_pitch;

        context.input_mut().set_mouse_captured(true);
    }

    fn update(
        &mut self,
        delta: f32,
        context: &mut GameContext,
        gui: &egui::Context,
        input_events: &Vec<InputEvent>,
        system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let player_position = context
            .physics()
            .get_object_position(self.player_body)
            .clone();

        // Show the selected block
        egui::Area::new("selected_block")
            .anchor(Align2::LEFT_BOTTOM, [5.0, -5.0])
            .show(gui, |ui| {
                let block = context.block_registry.get(context.selected_block);
                let count = context.inventory.count(context.selected_block);
                let preview_size = egui::Vec2::new(48.0, 48.0);
                if let Some(preview_texture) =
                    context.gui_images.get(&format!("preview_{}", block.code))
                {
                    block_button(ui, preview_texture, preview_size, count);
                }
            });

        // Show the profiler
        if self.show_debug_gui {
            egui::Area::new("profiler")
                .anchor(Align2::RIGHT_TOP, [-5.0, 5.0])
                .show(gui, |ui| {
                    let frame_times: PlotPoints = system
                        .frame_profile()
                        .duration_buffer
                        .iter()
                        .enumerate()
                        .map(|(i, t)| [i as f64, *t as f64])
                        .collect();
                    let frame_time_line = Line::new(frame_times);
                    let render_times: PlotPoints = system
                        .render_profile()
                        .duration_buffer
                        .iter()
                        .enumerate()
                        .map(|(i, t)| [i as f64, *t as f64])
                        .collect();
                    let render_time_line = Line::new(render_times);
                    egui::plot::Plot::new("example_plot")
                        .height(32.0)
                        .width(256.0)
                        .show_axes([false, false])
                        .show_x(false)
                        .show_y(false)
                        .include_y(1.0)
                        .show(ui, |plot_ui| {
                            plot_ui.line(frame_time_line);
                            plot_ui.line(render_time_line);
                        })
                        .response;
                    ui.label(format!(
                        "{}fps max {}ms avg {}ms",
                        system.frame_profile().fps as i32,
                        system.frame_profile().max_ms as i32,
                        system.frame_profile().avg_ms as i32
                    ));
                    ui.label(format!(
                        "Position {},{},{}",
                        player_position.x as i32,
                        player_position.y as i32,
                        player_position.z as i32
                    ));
                });
        }

        for event in input_events {
            let state_command = self.handle_event(event, context, system);
            match state_command {
                StateCommand::None => {}
                _ => {
                    return state_command;
                }
            }
        }
        self.handle_game_input(system, context, delta);

        self.rendering_mut().camera.update();

        // Generate spash sound if going into or out of water
        let player_handle = self.player_body;
        let player_object = context.physics().get_object(player_handle);
        if player_object.in_water != player_object.was_in_water {
            system.audio_mut().play_sound("splash");
        }

        context.daynight.update(delta);

        let camera = self.rendering().camera().clone();
        let camera_direction = camera.get_direction().clone();
        let frustum = self.rendering().camera().frustum_checker();

        context
            .physics_mut()
            .set_object_facing(player_handle, &camera_direction);
        let controls = PhysicsObjectControls {
            left: system.input().key_pressed(Key::A),
            right: system.input().key_pressed(Key::D),
            forward: system.input().key_pressed(Key::W),
            backward: system.input().key_pressed(Key::S),
            up: system.input().key_pressed(Key::Space),
            down: system.input().key_pressed(Key::LCtrl),
            slower: system.input().key_pressed(Key::LShift),
        };
        let on_ground = if self.player_flying {
            false
        } else {
            context
                .physics_mut()
                .get_object_mut(player_handle)
                .on_ground
        };
        if controls.is_moving() && on_ground {
            if camera
                .position
                .metric_distance(&context.last_sound_position)
                > 2.0
            {
                system.audio_mut().play_sound("step");
                context.last_sound_position = camera.position;
            }
        }
        context
            .physics_mut()
            .set_object_controls(player_handle, controls);

        // -------- Networking --------
        // Do we need to send a position update to the server?
        let last_position = context.last_position.clone();
        if context.last_pos_update_time.elapsed().as_fractional_secs() > POS_UPDATE_INTERVAL {
            if camera.position != last_position {
                context
                    .comms_client_mut()
                    .send(ClientMessage::PositionUpdate {
                        x: camera.position.x,
                        y: camera.position.y,
                        z: camera.position.z,
                        yaw: camera.yaw,
                        pitch: camera.pitch,
                    })
                    .unwrap();
                context.last_position = camera.position;
            }
            context.last_pos_update_time = Instant::now();
        }

        // Show camera position
        let cam_cp = ChunkPos::from_world_pos(camera.position);
        // let text = format!(
        //     "Position {},{},{} / Chunk {},{},{}",
        //     camera.position.x as i16,
        //     camera.position.y as i16,
        //     camera.position.z as i16,
        //     cam_cp.x,
        //     cam_cp.y,
        //     cam_cp.z
        // );
        // self.gui
        //     .set_value(&self.position_label, GuiValue::String(text));

        // Check if the camera moved a chunk
        let cam_chunk_col = ChunkColumnPos::from_chunk_pos(cam_cp);
        if cam_chunk_col != self.buffer_col {
            self.buffer_col = cam_chunk_col;
            context.world_mut().set_buffer_position(self.buffer_col);
        }

        if let Some(col) = pop(&mut self.out_of_range_columns) {
            let mut columns = HashSet::new();
            context.world_mut().remove_column(&col);
            columns.insert(col);
            context.block_renderer_mut().remove_col_set(&columns);
            context
                .comms_client_mut()
                .send(ClientMessage::Unsubscribe { columns })
                .unwrap();
        }

        // Any incoming messages from the server?
        while let Some(message) = context.comms_client_mut().try_receive() {
            match message {
                ServerMessage::SetBlock {
                    wbx,
                    wby,
                    wbz,
                    block,
                } => {
                    let block = context.block_registry.set_block_flags(block);
                    context.world_mut().set_block_add_dirty(
                        wbx,
                        wby,
                        wbz,
                        block,
                        &mut self.changed_chunks_to_mesh,
                    );
                }
                ServerMessage::ChunkColumn {
                    col: _,
                    block_data: _,
                } => {
                    panic!("Should be received by WorldHandler");
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
                    info!(
                        "Player {} ({}) spawned at {},{},{}",
                        username, player_id, x, y, z
                    );
                    context.players.push(PlayerData {
                        player_id,
                        x,
                        y,
                        z,
                        yaw,
                        pitch,
                        username,
                        inventory: Inventory::new(),
                    });
                }
                ServerMessage::PlayerDespawn { player_id } => {
                    info!("Player {} despawned", player_id);
                    context.players.retain(|p| p.player_id != player_id);
                }
                ServerMessage::PositionUpdate {
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    player_id,
                } => {
                    for player in &mut context.players {
                        if player.player_id == player_id {
                            player.x = x;
                            player.y = y;
                            player.z = z;
                            player.yaw = yaw;
                            player.pitch = pitch;
                        }
                    }
                }

                ServerMessage::SignInConfirm { .. } => {
                    panic!("unexpected in-game server sign in confirm");
                }
            }
        }

        // See if any generated columns were received from the server.
        // This is done seperately from the columns themselves as they are only received
        // when light has been propagated.
        while let Some((_, status)) = context.world_mut().try_receive_status() {
            if status == ColumnStatus::Received && self.open_column_requests > 0 {
                // debug!("Received {:?}", col);
                self.open_column_requests -= 1;
            }
        }

        // Any fresh columns received from the server?
        context.world_mut().try_receive_columns();

        // Request new generator work if needed
        if self.open_column_requests < MAX_OPEN_COLUMN_REQUESTS {
            let mut columns = Vec::new();
            while self.open_column_requests < MAX_OPEN_COLUMN_REQUESTS {
                if let Some(col) = context.world_mut().get_next_request(Some(&frustum)) {
                    // debug!("Requested {:?}", col);
                    columns.push(col);
                    self.open_column_requests += 1;
                } else {
                    break;
                }
            }
            if !columns.is_empty() {
                context
                    .comms_client_mut()
                    .send(ClientMessage::Subscribe { columns })
                    .unwrap();
            }
        }

        // -------- Physics --------
        self.delta_accumulator += delta;
        while self.delta_accumulator >= PHYSICS_TIME_STEP {
            context.step_physics();
            self.delta_accumulator -= PHYSICS_TIME_STEP;
        }

        self.rendering_mut().camera.position.x = player_position.x;
        self.rendering_mut().camera.position.y = player_position.y;
        self.rendering_mut().camera.position.z = player_position.z + CAMERA_Z_OFFSET;

        // -------- Meshing --------

        // Receive new meshes from mesher thread
        if let Some((cp, vertices, translucent_vertices)) =
            context.world_mut().try_receive_vertices()
        {
            if let Some(mesh) = BlockMesh::new(system.video().gl(), &vertices, true) {
                if context.block_renderer().meshes.contains_key(&cp) {
                    warn!("Duplicate mesh for {:?}", cp);
                }
                context.block_renderer_mut().insert_mesh_pos(cp, mesh);
            }
            if let Some(mesh) = BlockMesh::new(system.video().gl(), &translucent_vertices, true) {
                if context
                    .block_renderer()
                    .translucent_meshes
                    .contains_key(&cp)
                {
                    warn!("Duplicate mesh for {:?}", cp);
                }
                context
                    .block_renderer_mut()
                    .insert_translucent_mesh_pos(cp, mesh);
            }
        }

        // Do immediate meshing if needed
        let mesh_list = self.changed_chunks_to_mesh.clone();
        self.changed_chunks_to_mesh.clear();
        for cp in &mesh_list {
            let cp = *cp;
            let col = ChunkColumnPos::from_chunk_pos(cp);
            if context.world().chunks.are_all_neighbours_propagated(col) && cp.z >= 0 {
                let (vertices, translucent_vertices) = self
                    .rendering()
                    .world_mesher
                    .mesh_chunk(cp, &context.world().chunks);
                if let Some(mesh) = BlockMesh::new(system.video().gl(), &vertices, false) {
                    context.block_renderer_mut().insert_mesh_pos(cp, mesh);
                } else {
                    context.block_renderer_mut().remove_mesh_pos(cp);
                }
                if let Some(mesh) =
                    BlockMesh::new(system.video().gl(), &translucent_vertices, false)
                {
                    context
                        .block_renderer_mut()
                        .insert_translucent_mesh_pos(cp, mesh);
                } else {
                    context.block_renderer_mut().remove_translucent_mesh_pos(cp);
                }
            }
        }

        let player_position_handle = context.player_position_handle;
        context.particles_mut().update_position_handle(
            player_position_handle,
            self.rendering().camera().position + Vec3::new(0.0, 0.0, -0.25),
        );
        context.particles.as_mut().unwrap().update(delta);

        // self.profile_chart().update(context);

        StateCommand::None
    }

    fn resize(&mut self, data: &mut GameContext, context: &mut SystemContext) {
        self.rendering_mut().resize(data, context);
        self.rendering_mut()
            .camera
            .set_aspect(context.video().aspect_ratio());
    }

    fn render(&mut self, game_context: &mut GameContext, context: &mut SystemContext) {
        let in_water = self.in_water;
        let mut out_of_range = HashSet::new();
        self.rendering_mut()
            .render(game_context, context, in_water, &mut out_of_range);

        game_context.particles.as_mut().unwrap().draw(
            context.video().gl(),
            self.rendering().camera.get_view(),
            self.rendering().camera.get_projection(),
            context.video().height() as f32,
        );

        self.out_of_range_columns.extend(out_of_range);
    }

    fn shutdown(&mut self) {}
}

pub fn pop<T>(set: &mut HashSet<T>) -> Option<T>
where
    T: Eq + Clone + std::hash::Hash,
{
    if let Some(elt) = set.iter().next().cloned() {
        set.remove(&elt);
        Some(elt)
    } else {
        None
    }
}
