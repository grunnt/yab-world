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
        context: &mut GameContext,
        gui: &egui::Context,
        _input_events: &Vec<InputEvent>,
        system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::SidePanel::left("Settings").show(gui, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Settings");
                    ui.separator();
                    ui.add(
                        egui::Slider::new(&mut context.config.render_range_chunks, 4..=128)
                            .text("Render distance"),
                    );
                    if ui
                        .add(
                            egui::Slider::new(&mut system.audio_mut().volume, 0.0..=1.0)
                                .text("Sound effect volume"),
                        )
                        .drag_released()
                    {
                        system.audio().play_sound("click");
                    };
                    ui.separator();
                    if ui.button("Back").clicked() {
                        context.config.sound_effect_volume = system.audio_mut().volume;
                        context.config.save();
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
