use std::collections::HashMap;

use crate::{
    gui::{gui_renderer::GuiRenderer, BlockButton, Button, Label},
    GameContext,
};
use common::block::*;
use gamework::*;

pub struct BlockSelectState {
    gui: Gui<GuiRenderer>,
    back_button: WidgetId,
    block_buttons: HashMap<WidgetId, String>,
}

impl BlockSelectState {
    pub fn new(block_registry: &BlockRegistry) -> Self {
        let mut gui = Gui::new(
            vec![flex_col(1.0), fixed_col(800.0), flex_col(1.0)],
            vec![
                flex_row(1.0),
                fixed_row(50.0),
                fixed_row(400.0),
                fixed_row(50.0),
                flex_row(1.0),
            ],
        );
        gui.place(
            gui.root_id(),
            1,
            1,
            Box::new(Label::new("Select block".to_string())),
            CellAlignment::Center,
        );
        let mut cols = Vec::new();
        let blocks_in_row = 10;
        for _ in 0..blocks_in_row {
            cols.push(fixed_col(64.0));
        }
        cols.push(flex_col(1.0));
        let block_grid = gui.grid(
            gui.root_id(),
            1,
            2,
            cols,
            vec![
                fixed_row(64.0),
                fixed_row(64.0),
                fixed_row(64.0),
                fixed_row(64.0),
                flex_row(1.0),
            ],
        );
        let mut column_index = 0;
        let mut row_index = 0;
        let mut block_buttons = HashMap::new();
        for block_def in block_registry.all_blocks() {
            if block_def.buildable {
                let widget_id = gui.place(
                    block_grid,
                    column_index,
                    row_index,
                    Box::new(BlockButton::new(
                        &block_def.textures.get(FACE_ZP).unwrap().clone(),
                        &block_def.textures.get(FACE_XP).unwrap().clone(),
                        &block_def.textures.get(FACE_YP).unwrap().clone(),
                        64.0,
                        block_def.light > 0,
                        200 + (column_index + row_index) as u32,
                    )),
                    CellAlignment::Center,
                );
                block_buttons.insert(widget_id, block_def.code.clone());
                column_index += 1;
                if column_index >= blocks_in_row {
                    column_index = 0;
                    row_index += 1;
                }
            }
        }
        let back_button = gui.place(
            gui.root_id(),
            1,
            3,
            Box::new(Button::new("Back".to_string())),
            CellAlignment::Fill,
        );
        BlockSelectState {
            gui,
            back_button,
            block_buttons,
        }
    }
}

impl State<GameContext> for BlockSelectState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {}

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
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
        let gui_events = self.gui.update(
            input_events,
            context.video().screen_size(),
            data.gui_renderer_mut(),
        );
        for event in gui_events {
            match event {
                GuiEvent::ButtonClicked { widget_id } => {
                    context.audio_mut().play_sound("click");
                    if widget_id == self.back_button {
                        context.input_mut().set_mouse_captured(true);
                        return StateCommand::CloseState;
                    } else if self.block_buttons.contains_key(&widget_id) {
                        let block_type = self.block_buttons.get(&widget_id).unwrap();
                        data.selected_block = data.block_registry.block_kind_from_code(block_type);
                        context.input_mut().set_mouse_captured(true);
                        return StateCommand::CloseState;
                    }
                }
                _ => {}
            }
        }
        StateCommand::None
    }

    fn resize(&mut self, data: &mut GameContext, context: &mut SystemContext) {
        data.gui_renderer_mut()
            .resize(context.video().screen_size());
    }

    fn render(&mut self, data: &mut GameContext, _context: &mut SystemContext) {
        self.gui.paint(data.gui_renderer_mut());
        data.gui_renderer_mut().render();
    }

    fn shutdown(&mut self) {}
}
