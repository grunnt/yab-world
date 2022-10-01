use crate::block_button::*;
use crate::GameContext;
use common::{block::*, inventory::Inventory};
use egui::ScrollArea;
use egui::Vec2;
use gamework::*;

pub struct BlockSelectState {
    block_registry: BlockRegistry,
    viewed_block: Option<Block>,
    filter: String,
}

impl BlockSelectState {
    pub fn new(block_registry: &BlockRegistry, _inventory: &Inventory) -> Self {
        BlockSelectState {
            block_registry: block_registry.clone(),
            viewed_block: None,
            filter: "".to_string(),
        }
    }
}

impl State<GameContext> for BlockSelectState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn update(
        &mut self,
        _delta: f32,
        context: &mut GameContext,
        gui: &egui::Context,
        input_events: &Vec<InputEvent>,
        system: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let mut state_command = StateCommand::None;
        egui::TopBottomPanel::top("top").show(gui, |ui| {
            ui.heading("Select block");
            ui.text_edit_singleline(&mut self.filter);
        });
        egui::TopBottomPanel::bottom("bottom").show(gui, |ui| {
            if ui.button("Back").clicked() {
                system.input_mut().set_mouse_captured(true);
                state_command = StateCommand::CloseState;
            }
        });
        egui::SidePanel::right("block_details")
            .resizable(false)
            .show(gui, |ui| {
                if let Some(block) = &self.viewed_block {
                    let block_def = context.block_registry.get(*block);
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
                    let count = context.inventory.count(*block);
                    if count > 0 {
                        ui.label(format!("{} in inventory", count));
                    }
                }
            });
        self.viewed_block = None;
        egui::CentralPanel::default().show(gui, |ui| {
            ScrollArea::vertical()
                .max_height(f32::INFINITY)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut block_id = 0;
                        for block in self.block_registry.all_blocks() {
                            if block.buildable {
                                if self.filter.is_empty()
                                    || block
                                        .name
                                        .to_lowercase()
                                        .contains(&self.filter.to_lowercase())
                                {
                                    let count = context.inventory.count(block_id);
                                    let preview_size = Vec2::new(48.0, 48.0);
                                    if let Some(preview_texture) =
                                        context.gui_images.get(&format!("preview_{}", block.code))
                                    {
                                        let block_button =
                                            block_button(ui, preview_texture, preview_size, count);
                                        if block_button.clicked() {
                                            context.selected_block = context
                                                .block_registry
                                                .block_kind_from_code(&block.code);
                                            system.input_mut().set_mouse_captured(true);
                                            state_command = StateCommand::CloseState;
                                        }
                                        if block_button.hovered() {
                                            self.viewed_block = Some(block_id);
                                        }
                                    }
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
                        system.input_mut().set_mouse_captured(true);
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
