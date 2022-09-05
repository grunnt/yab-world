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
        egui::TopBottomPanel::top("top").show(gui, |ui| {
            ui.heading("Select block");
        });
        egui::TopBottomPanel::bottom("'bottom'").show(gui, |ui| {
            if ui.button("Back").clicked() {
                context.input_mut().set_mouse_captured(true);
                state_command = StateCommand::CloseState;
            }
        });
        egui::CentralPanel::default().show(gui, |ui| {
            ScrollArea::vertical()
                .max_height(f32::INFINITY)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut block_id = 0;
                        for block in self.block_registry.all_blocks() {
                            let count = data.inventory.count(block_id);
                            if count > 0 {
                                if ui.button(format!("{} ({})", block.name, count)).clicked() {
                                    data.selected_block =
                                        data.block_registry.block_kind_from_code(&block.code);
                                    context.input_mut().set_mouse_captured(true);
                                    state_command = StateCommand::CloseState;
                                }
                            } else {
                                ui.label(&block.name);
                            }
                            block_id += 1;
                        }
                    });
                });
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
