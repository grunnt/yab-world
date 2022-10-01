use crate::{start_game::StartGameState, GameContext};
use common::{comms::DEFAULT_TCP_PORT, world_type::GeneratorType};
use gamework::*;
use log::*;
use rand::{prelude::ThreadRng, RngCore};

pub struct NewGameState {
    rng: ThreadRng,
    seed: String,
    name: String,
    world_type: GeneratorType,
}

impl NewGameState {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        NewGameState {
            seed: rng.next_u32().to_string(),
            rng,
            name: "New World".to_string(),
            world_type: GeneratorType::Default,
        }
    }
}

impl State<GameContext> for NewGameState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn update(
        &mut self,
        _delta: f32,
        context: &mut GameContext,
        gui: &egui::Context,
        _input_events: &Vec<InputEvent>,
        system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::SidePanel::left("New").show(gui, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("New game");
                    ui.separator();
                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.add(egui::Label::new("Seed"));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.seed).desired_width(100.0),
                                );
                                if ui.button("Random").clicked() {
                                    system.audio().play_sound("click");
                                    self.seed = self.rng.next_u32().to_string();
                                }
                            });
                            ui.end_row();
                            ui.add(egui::Label::new("Name"));
                            ui.add(egui::TextEdit::singleline(&mut self.name));
                            ui.end_row();
                            ui.add(egui::Label::new("Type"));
                            egui::ComboBox::from_id_source("world_type")
                                .selected_text(format!("{:?}", self.world_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.world_type,
                                        GeneratorType::Default,
                                        "Default",
                                    );
                                    ui.selectable_value(
                                        &mut self.world_type,
                                        GeneratorType::Alien,
                                        "Alien",
                                    );
                                    ui.selectable_value(
                                        &mut self.world_type,
                                        GeneratorType::Flat,
                                        "Flat",
                                    );
                                    ui.selectable_value(
                                        &mut self.world_type,
                                        GeneratorType::Water,
                                        "Water",
                                    );
                                });
                            ui.end_row();
                        });
                    ui.separator();
                    if ui.button("Create").clicked() {
                        system.audio().play_sound("click");
                        context.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                        context.connect_to_address =
                            Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                        context.world_type = Some(self.world_type);
                        match self.seed.parse() {
                            Ok(seed) => {
                                context.seed = seed;
                                context.description = self.name.clone();
                                state_command = StateCommand::ReplaceState {
                                    state: Box::new(StartGameState::new()),
                                };
                            }
                            _ => {
                                warn!("Could not parse seed value as u32");
                            }
                        }
                    }
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
