use crate::GameContext;
use gamework::*;

pub struct SettingsState {}

impl SettingsState {
    pub fn new() -> Self {
        SettingsState {}
    }
}

impl State<GameContext> for SettingsState {
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
                    ui.heading("Settings");
                    ui.add(
                        egui::Slider::new(&mut data.config.render_range_chunks, 4..=128)
                            .text("Render distance"),
                    );
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
