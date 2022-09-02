use crate::{start_game::StartGameState, GameContext};
use common::comms::DEFAULT_TCP_PORT;
use common::world_definition::*;
use egui::ScrollArea;
use gamework::*;

pub struct LoadGameState {
    worlds: Vec<WorldDef>,
}

impl LoadGameState {
    pub fn new() -> Self {
        let worlds_store = WorldsStore::new();
        let worlds = worlds_store.list_worlds();
        LoadGameState { worlds }
    }
}

impl State<GameContext> for LoadGameState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        gui: &egui::Context,
        _input_events: &Vec<InputEvent>,
        _context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::CentralPanel::default().show(gui, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Load game");
                    ScrollArea::vertical()
                        .max_height(200.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                for world in &self.worlds {
                                    if ui.button(&world.description).clicked() {
                                        data.server_address =
                                            Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                                        data.connect_to_address =
                                            Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                                        data.seed = world.seed;
                                        state_command = StateCommand::ReplaceState {
                                            state: Box::new(StartGameState::new()),
                                        };
                                    }
                                }
                            });
                        });

                    if ui.button("Back").clicked() {
                        state_command = StateCommand::CloseState;
                    }
                },
            );
        });
        state_command
    }

    fn resize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn render(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn shutdown(&mut self) {}
}
