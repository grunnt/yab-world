use crate::{
    block_select::BlockSelectState,
    gui::{gui_renderer::GuiRenderer, Label, ProgressBar},
    *,
};
use common::block::*;
use common::inventory::Inventory;
use common::{chunk::*, player::PlayerData};
use floating_duration::TimeAsFloat;
use gamework::{InputEvent, MouseButton};
use gamework::{Key, StateCommand};
use gui::ProfileChart;
use log::*;

const POS_UPDATE_INTERVAL: f64 = 1.0 / 20.0;
const PHYSICS_TIME_STEP: f32 = 0.02;
const CAMERA_Z_OFFSET: f32 = 0.45;

const MIN_JUMP_VELOCITY: f32 = 0.25;
const AIR_JUMP_ENERGY: f32 = 2.5;
const AIR_JUMP_VELOCITY: f32 = 0.5;

const BLOCK_PLACE_TIME_S: f32 = 0.15;
const BLOCK_REMOVE_TIME_S: f32 = 0.15;

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
    gui: Gui<GuiRenderer>,
    debug_label: WidgetId,
    resource_label: WidgetId,
    selected_label: WidgetId,
    total_energy_bar: WidgetId,
    block_place_timer: f32,
    block_remove_timer: f32,
    profile_chart_id: WidgetId,
}

impl InGameState {
    pub fn new() -> Self {
        let buffer_col = ChunkColumnPos::from_chunk_pos(ChunkPos::zero());
        let open_column_requests: usize = 0;
        let debug_info = false;
        let player_flying = false;
        let mut gui = Gui::new(
            vec![
                fixed_col(10.0),
                flex_col(1.0),
                flex_col(1.0),
                fixed_col(10.0),
            ],
            vec![
                fixed_row(10.0),
                fixed_row(50.0),
                flex_row(1.0),
                fixed_row(50.0),
                fixed_row(20.0),
                fixed_row(10.0),
            ],
        );
        // gui.set_debug_render(true);
        let debug_label = gui.place(
            gui.root_id(),
            1,
            1,
            Box::new(Label::new("".to_string())),
            CellAlignment::TopLeft,
        );
        let profile_chart_id = gui.place(
            gui.root_id(),
            2,
            1,
            Box::new(ProfileChart::new(75.0)),
            CellAlignment::Center,
        );
        let selected_label = gui.place(
            gui.root_id(),
            1,
            3,
            Box::new(Label::new("".to_string())),
            CellAlignment::BottomLeft,
        );
        let resource_label = gui.place(
            gui.root_id(),
            2,
            3,
            Box::new(Label::new("".to_string())),
            CellAlignment::BottomRight,
        );
        let total_energy_bar = gui.place(
            gui.root_id(),
            1,
            4,
            Box::new(ProgressBar::new(0.0, 1.0)),
            CellAlignment::Fill,
        );

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
            gui,
            debug_label,
            resource_label,
            selected_label,
            total_energy_bar,
            block_remove_timer: 0.0,
            block_place_timer: 0.0,
            profile_chart_id,
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
        data: &mut GameContext,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        match event {
            InputEvent::MouseMove { dx, dy, .. } => {
                if context.input().get_mouse_captured() {
                    self.rendering_mut().camera.rotate(
                        *dx as f32 * data.config.camera_sensitivity_x * 0.0025,
                        *dy as f32 * data.config.camera_sensitivity_y * 0.0025,
                    );
                }
            }
            InputEvent::KeyPress { key, shift } => match key {
                Key::Space => {
                    // -------- Normal jumps --------
                    let object = data.physics.get_object_mut(self.player_body);
                    if object.on_ground {
                        object.velocity.z = object.velocity.z.max(MIN_JUMP_VELOCITY);
                    } else {
                        if data.energy >= AIR_JUMP_ENERGY {
                            object.velocity.z += object.velocity.z.max(AIR_JUMP_VELOCITY);
                            data.energy -= AIR_JUMP_ENERGY;
                        }
                    }
                }
                Key::Tab => {
                    context.input_mut().set_mouse_captured(false);
                    return StateCommand::OpenState {
                        state: Box::new(BlockSelectState::new(&data.block_registry)),
                    };
                }
                Key::X => {
                    let captured = context.input().get_mouse_captured();
                    context.input_mut().set_mouse_captured(!captured);
                    debug!("Mouse captured: {}", context.input().get_mouse_captured());
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
                        data.physics
                            .set_object_colliding(self.player_body, !self.player_flying);
                        debug!("Player flying: {}", self.player_flying);
                    }
                }
                Key::Escape => {
                    if let Some(server) = &mut data.server {
                        server.shutdown("server player exiting game".to_string());
                    }
                    context.input_mut().set_mouse_captured(false);
                    return StateCommand::CloseState;
                }
                Key::Num1 => self.set_selected_block(data, Block::dirt_block()),
                Key::Num2 => self.set_selected_block(data, Block::grass_block()),
                Key::Num3 => self.set_selected_block(data, Block::rock_block()),
                Key::Num4 => self.set_selected_block(data, Block::sand_block()),
                Key::Num5 => self.set_selected_block(data, Block::sandstone_block()),
                Key::Num6 => self.set_selected_block(data, Block::wood_block()),
                Key::Num7 => self.set_selected_block(data, Block::lamp_block()),
                // Key::Num8 => self.set_selected_block(data, Block::water_block()),
                _ => (),
            },
            _ => {}
        }
        StateCommand::None
    }

    fn update_resource_label(&mut self, data: &mut GameContext) {
        let mut new_text = String::new();
        for (resource, resource_def) in data.resource_registry.resources() {
            let count = data.inventory.count(*resource);
            new_text += format!("{} {} ", resource_def.name, count).as_str();
        }
        self.gui
            .set_value(&self.resource_label, GuiValue::String(new_text));
        self.set_selected_block(data, data.selected_block);
    }

    fn update_debug_label(&mut self, data: &mut GameContext) {
        let particle_count = data.particles.as_ref().unwrap().particle_count();
        self.gui.set_value(
            &self.debug_label,
            GuiValue::String(format!(
                "time: {} particles {}",
                (data.daynight.get_time() * 24.0) as i16,
                particle_count
            )),
        );
        self.set_selected_block(data, data.selected_block);
    }

    fn set_selected_block(&mut self, data: &mut GameContext, selected: Block) {
        data.selected_block = selected;
        let mut block_count = 999;
        for (resource, count) in &data.block_registry.get(data.selected_block).resource_cost {
            let inv_count = data.inventory.count(*resource);
            let max_blocks = if *count > 0 { inv_count / count } else { 0 };
            if max_blocks < block_count {
                block_count = max_blocks;
            }
        }
        let text = format!(
            "{} ({})",
            data.block_registry.get(data.selected_block).name,
            block_count
        );
        self.gui
            .set_value(&self.selected_label, GuiValue::String(text));
    }

    fn handle_game_input(
        &mut self,
        context: &mut SystemContext,
        data: &mut GameContext,
        delta: f32,
    ) {
        if !context.input().get_mouse_captured() {
            return;
        }
        if context.input().is_mouse_button_down(MouseButton::Right) {
            if self.block_place_timer <= std::f32::EPSILON {
                // Place a block
                if let Some(hit) = data.world().cast_ray(
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
                    if data.world().chunks.are_all_neighbours_stored(
                        ChunkColumnPos::from_world_block_coords(wbx, wby),
                    ) {
                        if !data.is_occopied_by_body(wbx, wby, wbz) {
                            // Do we have sufficient resources?
                            let selected_block = data.selected_block;
                            let block_def = data.block_registry.get(selected_block);
                            let mut sufficient = true;
                            for (resource_type, count) in &block_def.resource_yield {
                                if data.inventory.count(*resource_type) < *count {
                                    sufficient = false;
                                    break;
                                }
                            }
                            if sufficient {
                                // Remove resources from inventory
                                for (resource_type, count) in &block_def.resource_yield {
                                    data.inventory.remove(*resource_type, *count);
                                }
                                self.update_resource_label(data);
                                // Place block
                                data.world_mut().set_block_add_dirty(
                                    wbx,
                                    wby,
                                    wbz,
                                    selected_block,
                                    &mut self.changed_chunks_to_mesh,
                                );
                                // Send change to the server
                                data.comms_client_mut()
                                    .send(ClientMessage::SetBlock {
                                        wbx,
                                        wby,
                                        wbz,
                                        block: selected_block,
                                    })
                                    .unwrap();
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
        if context.input().is_mouse_button_down(MouseButton::Left) {
            self.block_remove_timer += delta;
            if self.block_remove_timer > BLOCK_REMOVE_TIME_S {
                // See if we need to remove a block
                if let Some(hit) = data.world().cast_ray(
                    &self.rendering().camera.position,
                    self.rendering().camera.get_direction(),
                    8.0,
                    false,
                ) {
                    let (wbx, wby, wbz) = (
                        (hit.hit_block_pos.x).floor() as i16,
                        (hit.hit_block_pos.y).floor() as i16,
                        (hit.hit_block_pos.z).floor() as i16,
                    );
                    let block = data.world().chunks.get_block(wbx, wby, wbz);
                    data.dig_effect(
                        Vec3::new(wbx as f32 + 0.5, wby as f32 + 0.5, wbz as f32 + 0.5),
                        block.kind(),
                    );
                    if data.world().chunks.are_all_neighbours_stored(
                        ChunkColumnPos::from_world_block_coords(wbx, wby),
                    ) {
                        // Add resources in block to inventory
                        let block_def = data.block_registry.get(block as Block);
                        for (resource_type, count) in &block_def.resource_yield {
                            data.inventory.add(*resource_type, *count);
                        }
                        self.update_resource_label(data);
                        // Clear the block
                        data.world_mut().set_block_add_dirty(
                            wbx,
                            wby,
                            wbz,
                            Block::empty_block(),
                            &mut self.changed_chunks_to_mesh,
                        );
                        // Send change to the server
                        data.comms_client_mut()
                            .send(ClientMessage::SetBlock {
                                wbx,
                                wby,
                                wbz,
                                block: u16::empty_block(),
                            })
                            .unwrap();
                    }
                }
                self.block_remove_timer -= BLOCK_REMOVE_TIME_S;
            }
        } else {
            self.block_remove_timer = 0.0;
        }
    }

    fn profile_chart(&mut self) -> &mut ProfileChart {
        self.gui
            .get_widget_mut(&self.profile_chart_id)
            .as_any_mut()
            .downcast_mut::<ProfileChart>()
            .unwrap()
    }
}

impl State<GameContext> for InGameState {
    fn initialize(&mut self, data: &mut GameContext, context: &mut SystemContext) {
        debug!("Activating {}", self.type_name());
        self.update_resource_label(data);
        self.set_selected_block(data, Block::sand_block());

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
        self.player_body = data.physics.new_object(
            data.starting_position.x,
            data.starting_position.y,
            data.starting_position.z,
            Vec3::new(0.75, 0.75, 0.9),
        );

        self.update_debug_label(data);

        self.rendering = Some(Rendering::new(data, context));
        self.rendering_mut().camera.yaw = data.starting_yaw;
        self.rendering_mut().camera.pitch = data.starting_pitch;

        context.input_mut().set_mouse_captured(true);
    }

    fn update(
        &mut self,
        delta: f32,
        data: &mut GameContext,
        input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        context.fill_profile_buffer(self.profile_chart().buffer_mut());

        self.gui.update(
            input_events,
            context.video().screen_size(),
            data.gui_renderer_mut(),
        );
        for event in input_events {
            let state_command = self.handle_event(event, data, context);
            match state_command {
                StateCommand::None => {}
                _ => {
                    return state_command;
                }
            }
        }
        self.handle_game_input(context, data, delta);

        self.rendering_mut().camera.update();

        let player_handle = self.player_body;

        let player_pos =
            data.physics.get_object_position(player_handle) + Vec3::new(0.0, 0.0, CAMERA_Z_OFFSET);
        self.in_water = data
            .world()
            .chunks
            .get_block(
                player_pos.x as i16,
                player_pos.y as i16,
                player_pos.z as i16,
            )
            .kind()
            == Block::water_block();

        data.daynight.update(delta);

        let camera = self.rendering().camera().clone();
        let camera_direction = camera.get_direction().clone();
        let frustum = self.rendering().camera().frustum_checker();

        data.physics
            .set_object_facing(player_handle, &camera_direction);
        data.physics.set_object_controls(
            player_handle,
            PhysicsObjectControls {
                left: context.input().key_pressed(Key::A),
                right: context.input().key_pressed(Key::D),
                forward: context.input().key_pressed(Key::W),
                backward: context.input().key_pressed(Key::S),
                up: context.input().key_pressed(Key::Space),
                down: context.input().key_pressed(Key::LCtrl),
                slower: context.input().key_pressed(Key::LShift),
            },
        );

        // -------- Networking --------
        // Do we need to send a position update to the server?
        let last_position = data.last_position.clone();
        if data.last_pos_update_time.elapsed().as_fractional_secs() > POS_UPDATE_INTERVAL {
            if camera.position != last_position {
                data.comms_client_mut()
                    .send(ClientMessage::PositionUpdate {
                        x: camera.position.x,
                        y: camera.position.y,
                        z: camera.position.z,
                        yaw: camera.yaw,
                        pitch: camera.pitch,
                    })
                    .unwrap();
                data.last_position = camera.position;
            }
            data.last_pos_update_time = Instant::now();
        }

        // Check if the camera moved a chunk
        let cam_cp = ChunkPos::from_world_pos(camera.position);
        let cam_chunk_col = ChunkColumnPos::from_chunk_pos(cam_cp);
        if cam_chunk_col != self.buffer_col {
            self.buffer_col = cam_chunk_col;
            data.world_mut().set_buffer_position(self.buffer_col);
        }

        if let Some(col) = pop(&mut self.out_of_range_columns) {
            let mut columns = HashSet::new();
            data.world_mut().remove_column(&col);
            columns.insert(col);
            data.block_renderer_mut().remove_col_set(&columns);
            data.comms_client_mut()
                .send(ClientMessage::Unsubscribe { columns })
                .unwrap();
        }

        // Any incoming messages from the server?
        while let Some(message) = data.comms_client_mut().try_receive() {
            match message {
                ServerMessage::SetBlock {
                    wbx,
                    wby,
                    wbz,
                    block,
                } => {
                    data.world_mut().set_block_add_dirty(
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
                    data.players.push(PlayerData {
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
                    data.players.retain(|p| p.player_id != player_id);
                }
                ServerMessage::PositionUpdate {
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    player_id,
                } => {
                    for player in &mut data.players {
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
        while let Some((_, status)) = data.world_mut().try_receive_status() {
            if status == ColumnStatus::Received && self.open_column_requests > 0 {
                // debug!("Received {:?}", col);
                self.open_column_requests -= 1;
            }
        }

        // Any fresh columns received from the server?
        data.world_mut().try_receive_columns();

        // Request new generator work if needed
        if self.open_column_requests < MAX_OPEN_COLUMN_REQUESTS {
            let mut columns = Vec::new();
            while self.open_column_requests < MAX_OPEN_COLUMN_REQUESTS {
                if let Some(col) = data.world_mut().get_next_request(Some(&frustum)) {
                    // debug!("Requested {:?}", col);
                    columns.push(col);
                    self.open_column_requests += 1;
                } else {
                    break;
                }
            }
            if !columns.is_empty() {
                data.comms_client_mut()
                    .send(ClientMessage::Subscribe { columns })
                    .unwrap();
            }
        }

        // -------- Physics --------
        self.delta_accumulator += delta;
        while self.delta_accumulator >= PHYSICS_TIME_STEP {
            data.step_physics();
            self.delta_accumulator -= PHYSICS_TIME_STEP;
        }

        // -------- Energy --------
        if data.max_energy > 0.0 {
            // Normal charging
            if data.energy < data.max_energy {
                data.energy += data.charge_per_s * delta;
                if data.energy > data.max_energy {
                    data.energy = data.max_energy;
                }
            }

            // Update energy bar
            self.gui.set_value(
                &self.total_energy_bar,
                GuiValue::Float(data.energy / data.max_energy),
            );
        }

        let pos = data.physics.get_object_position(self.player_body).clone();
        self.rendering_mut().camera.position.x = pos.x;
        self.rendering_mut().camera.position.y = pos.y;
        self.rendering_mut().camera.position.z = pos.z + CAMERA_Z_OFFSET;

        // -------- Meshing --------

        // Receive new meshes from mesher thread
        if let Some((cp, vertices, translucent_vertices)) = data.world_mut().try_receive_vertices()
        {
            if let Some(mesh) = BlockMesh::new(&self.rendering().gl, &vertices, false, true) {
                if data.block_renderer().meshes.contains_key(&cp) {
                    warn!("Duplicate mesh for {:?}", cp);
                }
                data.block_renderer_mut().insert_mesh_pos(cp, mesh);
            }
            if let Some(mesh) =
                BlockMesh::new(&self.rendering().gl, &translucent_vertices, false, true)
            {
                if data.block_renderer().translucent_meshes.contains_key(&cp) {
                    warn!("Duplicate mesh for {:?}", cp);
                }
                data.block_renderer_mut()
                    .insert_translucent_mesh_pos(cp, mesh);
            }
        }

        // Do immediate meshing if needed
        let mesh_list = self.changed_chunks_to_mesh.clone();
        self.changed_chunks_to_mesh.clear();
        for cp in &mesh_list {
            let cp = *cp;
            let col = ChunkColumnPos::from_chunk_pos(cp);
            if data.world().chunks.are_all_neighbours_propagated(col) && cp.z >= 0 {
                let (vertices, translucent_vertices) = self
                    .rendering()
                    .world_mesher
                    .mesh_chunk(cp, &data.world().chunks);
                if let Some(mesh) = BlockMesh::new(&self.rendering().gl, &vertices, false, false) {
                    data.block_renderer_mut().insert_mesh_pos(cp, mesh);
                } else {
                    data.block_renderer_mut().remove_mesh_pos(cp);
                }
                if let Some(mesh) =
                    BlockMesh::new(&self.rendering().gl, &translucent_vertices, false, false)
                {
                    data.block_renderer_mut()
                        .insert_translucent_mesh_pos(cp, mesh);
                } else {
                    data.block_renderer_mut().remove_translucent_mesh_pos(cp);
                }
            }
        }

        data.particles.as_mut().unwrap().update(
            delta,
            self.rendering().camera().position + Vec3::new(0.0, 0.0, -0.25),
        );

        self.update_debug_label(data);

        self.profile_chart().update(context);

        StateCommand::None
    }

    fn resize(&mut self, data: &mut GameContext, context: &mut SystemContext) {
        self.rendering_mut().resize(data, context);
        self.rendering_mut()
            .camera
            .set_aspect(context.video().aspect_ratio());
        data.gui_renderer_mut()
            .resize(context.video().screen_size());
    }

    fn render(&mut self, game_context: &mut GameContext, context: &mut SystemContext) {
        let in_water = self.in_water;
        let mut out_of_range = HashSet::new();
        self.rendering_mut()
            .render(game_context, context, in_water, &mut out_of_range);

        game_context.particles.as_mut().unwrap().draw(
            self.rendering().camera.get_view(),
            self.rendering().camera.get_projection(),
            context.video().height() as f32,
        );

        self.out_of_range_columns.extend(out_of_range);
        self.gui.paint(game_context.gui_renderer_mut());
        game_context.gui_renderer_mut().render();
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
