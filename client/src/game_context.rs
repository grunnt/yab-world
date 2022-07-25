use crate::render::BlockRenderer;
use crate::world::worldhandler::WorldHandler;
use crate::*;
use crate::{client_config::ClientConfig, gui::gui_renderer::GuiRenderer};
use common::block::*;
use common::comms::*;
use common::daynight::DayNight;
use common::inventory::Inventory;
use common::player::PlayerData;
use gamework::video::*;
use nalgebra_glm::*;
use server::YabServer;
use std::sync::mpsc::Receiver;

pub struct GameContext {
    pub world: Option<WorldHandler>,
    pub block_renderer: Option<BlockRenderer>,
    pub gui_renderer: Option<GuiRenderer>,
    pub particles: Option<ParticleSystem>,
    pub dig_common_emitter: Option<EmitterDef>,
    pub dig_iron_emitter: Option<EmitterDef>,
    pub dig_gold_emitter: Option<EmitterDef>,
    pub dig_explosion_emitter: Option<EmitterDef>,
    pub dig_beam_emitter_handle: EmitterHandle,
    pub player_position_handle: ParticlePositionHandle,
    pub player_target_handle: ParticlePositionHandle,
    pub block_registry: BlockRegistry,
    pub seed: u32,
    pub description: String,
    pub server: Option<YabServer>,
    pub server_address: Option<String>,
    pub connect_to_address: Option<String>,
    pub world_type: Option<GeneratorType>,
    pub comms_client: Option<CommsClient>,
    pub col_receiver: Option<Receiver<ServerMessage>>,
    pub config: ClientConfig,
    pub daynight: DayNight,
    pub starting_position: Vec3,
    pub starting_yaw: f32,
    pub starting_pitch: f32,
    pub physics: Option<Physics>,
    pub last_position: Vec3,
    pub last_pos_update_time: Instant,
    pub inventory: Inventory,
    pub selected_block: Block,
    pub player_id: Option<u8>,
    pub players: Vec<PlayerData>,
    pub last_sound_position: Vec3,
}

impl GameContext {
    pub fn new(_assets: &Assets) -> GameContext {
        GameContext {
            player_id: None,
            block_registry: BlockRegistry::empty(),
            particles: None,
            dig_common_emitter: None,
            dig_iron_emitter: None,
            dig_gold_emitter: None,
            dig_explosion_emitter: None,
            dig_beam_emitter_handle: 0,
            player_position_handle: 0,
            player_target_handle: 0,
            seed: 0,
            description: "".to_string(),
            server_address: None,
            connect_to_address: None,
            world_type: None,
            server: None,
            comms_client: None,
            col_receiver: None,
            daynight: DayNight::new(10.0 * 60.0),
            starting_position: Vec3::new(0.0, 0.0, 0.0),
            starting_yaw: 0.0,
            starting_pitch: 0.0,
            world: None,
            block_renderer: None,
            physics: None,
            last_position: Vec3::new(0.0, 0.0, 0.0),
            last_pos_update_time: Instant::now(),
            inventory: Inventory::new(),
            selected_block: 2,
            config: ClientConfig::load(),
            gui_renderer: None,
            players: Vec::new(),
            last_sound_position: Vec3::zeros(),
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn comms_client_mut(&mut self) -> &mut CommsClient {
        self.comms_client.as_mut().unwrap()
    }

    pub fn world(&self) -> &WorldHandler {
        self.world.as_ref().unwrap()
    }

    pub fn world_mut(&mut self) -> &mut WorldHandler {
        self.world.as_mut().unwrap()
    }

    pub fn block_renderer(&self) -> &BlockRenderer {
        self.block_renderer.as_ref().unwrap()
    }

    pub fn block_renderer_mut(&mut self) -> &mut BlockRenderer {
        self.block_renderer.as_mut().unwrap()
    }

    pub fn particles_mut(&mut self) -> &mut ParticleSystem {
        self.particles.as_mut().unwrap()
    }

    pub fn gui_renderer(&self) -> &GuiRenderer {
        self.gui_renderer.as_ref().unwrap()
    }

    pub fn gui_renderer_mut(&mut self) -> &mut GuiRenderer {
        self.gui_renderer.as_mut().unwrap()
    }

    pub fn physics(&self) -> &Physics {
        self.physics.as_ref().unwrap()
    }

    pub fn physics_mut(&mut self) -> &mut Physics {
        self.physics.as_mut().unwrap()
    }

    pub fn step_physics(&mut self) {
        let world = self.world.as_ref().unwrap();
        self.physics.as_mut().unwrap().step(&world);
    }

    pub fn is_occopied_by_body(&mut self, wbx: i16, wby: i16, wbz: i16) -> bool {
        let world = self.world.as_mut().unwrap();
        self.physics
            .as_mut()
            .unwrap()
            .is_occopied_by_body(wbx, wby, wbz, world)
    }

    pub fn dig_effect(&mut self, block_position: Vec3) {
        let emitter_def = self.dig_common_emitter.as_ref().unwrap().clone();
        let player_position_handle = self.player_position_handle;
        self.particles_mut().emitter(
            ParticlePosition::Fixed(block_position),
            ParticlePosition::Handle(player_position_handle),
            emitter_def,
        );
        let expl_def = self.dig_explosion_emitter.as_ref().unwrap().clone();
        self.particles_mut().emitter(
            ParticlePosition::Fixed(block_position),
            ParticlePosition::None,
            expl_def,
        );
    }

    fn particle(&self, name: &str) -> f32 {
        self.particles
            .as_ref()
            .unwrap()
            .texture_array()
            .get_layer_by_name(name)
            .unwrap()
            .clone()
    }
}

impl SharedContext for GameContext {
    fn initialize(&mut self, context: &mut SystemContext) {
        self.gui_renderer = Some(GuiRenderer::new(
            &context.video().gl(),
            &context.assets(),
            context.video().width(),
            context.video().height(),
        ));
        let texture_array = TextureArray::load_directory(
            &context.assets().assets_path("particles"),
            TextureFormat::RGBA8,
            TextureWrap::None,
            TextureFilter::MipMapNearest,
            context.video().gl(),
        );
        self.particles = Some(ParticleSystem::new(
            context.video().gl(),
            context.assets(),
            texture_array,
        ));
        self.player_position_handle = self.particles_mut().new_position_handle();
        self.player_target_handle = self.particles_mut().new_position_handle();
        self.dig_common_emitter = Some(EmitterDef {
            pitch: 0.0,
            yaw: 0.0,
            spread_angle: std::f32::consts::PI * 2.0,
            delay: 0.0,
            duration: 0.1,
            continuous: false,
            particle_interval_s: 0.001,
            start_area: Some(Vec3::new(0.5, 0.5, 0.5)),
            size: Range::new(0.005, 0.02),
            life: Range::new(0.1, 0.5),
            velocity: Range::new(10.0, 50.0),
            texture_layers: vec![self.particle("dig_common")],
        });
        self.dig_iron_emitter = Some(EmitterDef {
            pitch: 0.0,
            yaw: 0.0,
            spread_angle: std::f32::consts::PI * 2.0,
            delay: 0.0,
            duration: 0.1,
            continuous: false,
            particle_interval_s: 0.001,
            start_area: Some(Vec3::new(0.5, 0.5, 0.5)),
            size: Range::new(0.005, 0.02),
            life: Range::new(0.1, 0.5),
            velocity: Range::new(10.0, 50.0),
            texture_layers: vec![
                self.particle("spark"),
                self.particle("dig_iron"),
                self.particle("dig_iron"),
            ],
        });
        self.dig_gold_emitter = Some(EmitterDef {
            pitch: 0.0,
            yaw: 0.0,
            spread_angle: std::f32::consts::PI * 2.0,
            delay: 0.0,
            duration: 0.1,
            continuous: false,
            particle_interval_s: 0.001,
            start_area: Some(Vec3::new(0.5, 0.5, 0.5)),
            size: Range::new(0.01, 0.03),
            life: Range::new(0.1, 0.5),
            velocity: Range::new(5.0, 25.0),
            texture_layers: vec![
                self.particle("spark"),
                self.particle("dig_gold"),
                self.particle("dig_gold"),
            ],
        });
        self.dig_explosion_emitter = Some(EmitterDef {
            pitch: 0.0,
            yaw: 0.0,
            spread_angle: std::f32::consts::PI * 2.0,
            delay: 0.0,
            duration: 0.2,
            continuous: false,
            particle_interval_s: 0.001,
            start_area: Some(Vec3::new(0.5, 0.5, 0.5)),
            size: Range::new(0.02, 0.1),
            life: Range::new(0.1, 0.2),
            velocity: Range::new(0.5, 1.5),
            texture_layers: vec![self.particle("spark")],
        });
        let beam_emitter_def = EmitterDef {
            pitch: 0.0,
            yaw: 0.0,
            spread_angle: 0.0,
            delay: 0.0,
            duration: 0.0,
            continuous: true,
            particle_interval_s: 0.001,
            start_area: None,
            size: Range::new(0.01, 0.02),
            life: Range::new(0.1, 0.2),
            velocity: Range::new(20.0, 50.0),
            texture_layers: vec![self.particle("spark")],
        };
        let player_position_handle = self.player_position_handle;
        let player_target_handle = self.player_target_handle;
        self.dig_beam_emitter_handle = self.particles_mut().emitter(
            ParticlePosition::Handle(player_position_handle),
            ParticlePosition::Handle(player_target_handle),
            beam_emitter_def,
        );
    }
}
