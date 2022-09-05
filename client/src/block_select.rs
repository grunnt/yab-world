use crate::GameContext;
use common::{block::*, inventory::Inventory};
use egui::ScrollArea;
use gamework::*;

pub struct BlockSelectState {
    block_registry: BlockRegistry,
}

impl BlockSelectState {
    pub fn new(block_registry: &BlockRegistry, _inventory: &Inventory) -> Self {
        BlockSelectState {
            block_registry: block_registry.clone(),
        }
    }
}

impl State<GameContext> for BlockSelectState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        gui: &egui::Context,
        input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::CentralPanel::default().show(gui, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Select block");
                    ScrollArea::vertical()
                        .max_height(200.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                for block in self.block_registry.all_blocks() {
                                    if ui.button(&block.name).clicked() {
                                        data.selected_block =
                                            data.block_registry.block_kind_from_code(&block.code);
                                        context.input_mut().set_mouse_captured(true);
                                        state_command = StateCommand::CloseState;
                                    }
                                }
                            });
                        });

                    if ui.button("Back").clicked() {
                        context.input_mut().set_mouse_captured(true);
                        state_command = StateCommand::CloseState;
                    }
                },
            );
        });
        for event in input_events {
            match event {
                InputEvent::KeyPress { key, .. } => match key {
                    Key::Tab => {
                        context.input_mut().set_mouse_captured(true);
                        return StateCommand::CloseState;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        state_command
    }

    fn resize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn render(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn shutdown(&mut self) {}
}
