use crate::block_button::*;
use crate::GameContext;
use common::{block::*, inventory::Inventory};
use egui::ScrollArea;
use gamework::*;

pub struct BlockSelectState {
    block_registry: BlockRegistry,
    viewed_block: Option<Block>,
}

impl BlockSelectState {
    pub fn new(block_registry: &BlockRegistry, _inventory: &Inventory) -> Self {
        BlockSelectState {
            block_registry: block_registry.clone(),
            viewed_block: None,
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
        egui::TopBottomPanel::bottom("bottom").show(gui, |ui| {
            if ui.button("Back").clicked() {
                context.input_mut().set_mouse_captured(true);
                state_command = StateCommand::CloseState;
            }
        });
        egui::SidePanel::right("block_details")
            .resizable(false)
            .show(gui, |ui| {
                if let Some(block) = &self.viewed_block {
                    let block_def = data.block_registry.get(*block);
                    ui.label(&block_def.name);
                    if block_def.buildable {
                        ui.label("Buildable");
                    }
                    if block_def.solid {
                        ui.label("Solid");
                    }
                    if block_def.transparent {
                        ui.label("Transparent");
                    }
                    if block_def.light > 0 {
                        ui.label("Emits light");
                    }
                    let count = data.inventory.count(*block);
                    if count > 0 {
                        ui.label(format!("{} in inventory", count));
                    }
                }
            });
        egui::CentralPanel::default().show(gui, |ui| {
            ScrollArea::vertical()
                .max_height(f32::INFINITY)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut block_id = 0;
                        for block in self.block_registry.all_blocks() {
                            if block.buildable {
                                let count = data.inventory.count(block_id);
                                let block_button =
                                    block_button(ui, block.name.clone().into(), count);
                                if block_button.clicked() {
                                    data.selected_block =
                                        data.block_registry.block_kind_from_code(&block.code);
                                    context.input_mut().set_mouse_captured(true);
                                    state_command = StateCommand::CloseState;
                                }
                                if block_button.hovered() {
                                    self.viewed_block = Some(block_id);
                                }
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
