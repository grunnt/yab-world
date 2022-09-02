use crate::{
    join_game::JoinGameState, load_game::LoadGameState, new_game::NewGameState,
    settings::SettingsState,
};
use crate::{start_game::StartGameState, GameContext};
use common::comms::DEFAULT_TCP_PORT;
use common::world_definition::*;
use gamework::*;
use log::*;

pub struct MainMenuState {
    continue_save: Option<WorldDef>,
}

impl MainMenuState {
    pub fn new() -> Self {
        MainMenuState {
            continue_save: None,
        }
    }
}

impl State<GameContext> for MainMenuState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {
        let worlds = WorldsStore::new();
        let world_list = worlds.list_worlds();
        self.continue_save = if world_list.is_empty() {
            None
        } else {
            Some(world_list.get(0).unwrap().clone())
        };
    }

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        gui: &egui::Context,
        _input_events: &Vec<InputEvent>,
        _system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::Area::new("Main Menu").show(gui, |ui| {
            // ui.horizontal_centered(|ui| {
            //     ui.vertical_centered_justified(|ui| {
            ui.heading("YAB-World");
            if ui.button("Continue").clicked() {
                if self.continue_save.is_some() {
                    let save = self.continue_save.as_ref().unwrap();
                    debug!("Continue game");
                    data.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                    data.connect_to_address = Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                    data.seed = save.seed;
                    data.description = save.description.clone();
                    state_command = StateCommand::OpenState {
                        state: Box::new(StartGameState::new()),
                    };
                }
            }
            if ui.button("New").clicked() {
                state_command = StateCommand::OpenState {
                    state: Box::new(NewGameState::new()),
                };
            }
            if ui.button("Load").clicked() {
                state_command = StateCommand::OpenState {
                    state: Box::new(LoadGameState::new()),
                };
            }
            if ui.button("Join").clicked() {
                state_command = StateCommand::OpenState {
                    state: Box::new(JoinGameState::new()),
                };
            }
            if ui.button("Settings").clicked() {
                state_command = StateCommand::OpenState {
                    state: Box::new(SettingsState::new()),
                };
            }
            if ui.button("Exit").clicked() {
                state_command = StateCommand::CloseState;
            }
        });
        //     });
        // });
        state_command
    }

    fn resize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn render(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn shutdown(&mut self) {}
}
