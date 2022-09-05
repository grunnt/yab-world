use crate::{start_game::StartGameState, GameContext};
use common::comms::DEFAULT_TCP_PORT;
use gamework::*;
use log::*;

pub struct JoinGameState {
    address: String,
}

impl JoinGameState {
    pub fn new() -> Self {
        JoinGameState {
            address: "".to_string(),
        }
    }
}

impl State<GameContext> for JoinGameState {
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
        egui::SidePanel::left("Join").show(gui, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Join game");
                    ui.separator();
                    ui.add(egui::Label::new("Server address"));
                    ui.add(egui::TextEdit::singleline(&mut self.address));
                    ui.separator();
                    if ui.button("Join").clicked() {
                        let server_address = format!("{}:{}", self.address, DEFAULT_TCP_PORT);
                        let server_address = if server_address == "localhost" {
                            "127.1.1.1".to_string()
                        } else {
                            server_address
                        };
                        debug!("Join server at address {}", server_address);
                        data.connect_to_address = Some(server_address);
                        state_command = StateCommand::OpenState {
                            state: Box::new(StartGameState::new()),
                        };
                    }
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
