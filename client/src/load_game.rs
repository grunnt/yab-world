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
        system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::SidePanel::left("Load").show(gui, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Load game");
                    ui.separator();
                    ScrollArea::vertical()
                        .max_height(200.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::Min),
                                |ui| {
                                    for world in &self.worlds {
                                        if ui
                                            .button(format!(
                                                "{}\n{}",
                                                world.description,
                                                world.timestamp.format("%Y-%m-%d %H:%M:%S")
                                            ))
                                            .clicked()
                                        {
                                            system.audio().play_sound("click");
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
                                },
                            );
                        });
                    ui.separator();
                    if ui.button("Back").clicked() {
                        system.audio().play_sound("click");
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
