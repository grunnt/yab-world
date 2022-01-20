mod block_select;
mod client_config;
mod game_context;
pub mod gui;
mod in_game;
mod join_game;
mod load_game;
mod main_menu;
mod new_game;
mod physics;
pub mod render;
mod settings;
mod start_game;
pub mod world;

extern crate nalgebra_glm as glm;

use self::render::*;
use common::comms::*;
use common::world_definition::*;
use common::world_type::GeneratorType;
use game_context::GameContext;
pub use gamework::gl;
use gamework::*;
use log::*;
use main_menu::*;
use nalgebra_glm::*;
use physics::physics::Physics;
use physics::physicsobject::*;
use rand::Rng;
use start_game::StartGameState;
use std::collections::HashSet;
use std::time::Instant;

const MAX_OPEN_COLUMN_REQUESTS: usize = 6;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum StateIDs {
    MainMenu,
    NewGame,
    JoinGame,
    LoadGame,
    Settings,
    StartGame,
    InGame,
    BlockSelect,
}

pub enum StartMode {
    Normal,
    QuickNewWorld,
    Continue,
}

pub struct YabClient {
    start_mode: StartMode,
    world_type: GeneratorType,
}

impl YabClient {
    pub fn new(start_mode: StartMode, world_type: GeneratorType) -> YabClient {
        YabClient {
            start_mode,
            world_type,
        }
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {
        let assets = Assets::default();
        let mut data = GameContext::new(&assets);

        let state: Box<dyn State<GameContext>> = match self.start_mode {
            StartMode::Normal => Box::new(MainMenuState::new()),
            StartMode::QuickNewWorld => {
                data.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                data.connect_to_address = Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                data.world_type = Some(self.world_type);
                data.seed = rand::thread_rng().gen::<u32>();
                data.description = "Quick".to_string();
                Box::new(StartGameState::new())
            }
            StartMode::Continue => {
                let worlds = WorldList::new();
                let world_list = worlds.list_worlds();
                if world_list.is_empty() {
                    panic!("No saved world to continue");
                } else {
                    let world = world_list.get(0).unwrap();
                    debug!("Continue game");
                    data.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                    data.connect_to_address = Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                    data.seed = world.seed;
                    data.description = world.description.clone();
                    Box::new(StartGameState::new())
                }
            }
        };

        App::run(
            "YAB-World",
            1024,
            768,
            true,
            3,
            3,
            state,
            &assets,
            data,
            Box::new(setup),
        );

        info!("Exited");

        Ok(())
    }
}

fn setup(_game: &mut GameContext, system: &mut SystemContext) {
    let sounds = vec!["click", "step", "jump", "build", "splash"];
    for sound in &sounds {
        let path = system
            .assets()
            .assets_path(format!("sounds/{}.wav", sound).as_str());
        system.audio_mut().load_sound(&path);
    }
}
