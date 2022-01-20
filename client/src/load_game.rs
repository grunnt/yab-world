use crate::{
    gui::{gui_renderer::GuiRenderer, Button, Label},
    start_game::StartGameState,
    GameContext,
};
use common::comms::DEFAULT_TCP_PORT;
use common::world_definition::*;
use gamework::*;

const SAVES_PER_PAGE: usize = 7;

pub struct LoadGameState {
    gui: Gui<GuiRenderer>,
    prev_button: WidgetId,
    next_button: WidgetId,
    back_button: WidgetId,
    offset: usize,
    worlds_list: WidgetId,
    worlds_buttons: Vec<(WidgetId, WorldDef)>,
    worlds: WorldList,
    world_count: usize,
}

impl LoadGameState {
    pub fn new() -> Self {
        let mut gui = Gui::new(
            vec![flex_col(1.0), fixed_col(500.0), flex_col(1.0)],
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
            Box::new(Label::new("Load world".to_string())),
            CellAlignment::Center,
        );
        let worlds_list = gui.grid(
            gui.root_id(),
            1,
            2,
            vec![flex_col(1.0)],
            vec![fixed_row(50.0); SAVES_PER_PAGE],
        );
        let buttons = gui.grid(
            gui.root_id(),
            1,
            3,
            vec![flex_col(1.0), flex_col(1.0), flex_col(1.0)],
            vec![fixed_row(50.0)],
        );
        let prev_button = gui.place(
            buttons,
            0,
            0,
            Box::new(Button::new("<".to_string())),
            CellAlignment::Fill,
        );
        let back_button = gui.place(
            buttons,
            1,
            0,
            Box::new(Button::new("Back".to_string())),
            CellAlignment::Fill,
        );
        let next_button = gui.place(
            buttons,
            2,
            0,
            Box::new(Button::new(">".to_string())),
            CellAlignment::Fill,
        );

        let worlds = WorldList::new();

        LoadGameState {
            gui,
            prev_button,
            next_button,
            back_button,
            offset: 0,
            worlds_list,
            worlds_buttons: Vec::new(),
            worlds,
            world_count: 0,
        }
    }

    fn update_paging(&mut self) {
        // Remove old buttons
        self.worlds_buttons.clear();
        for r in 0..SAVES_PER_PAGE {
            self.gui.remove(self.worlds_list, 0, r);
        }
        // Place new buttons
        let worlds = self.worlds.list_worlds();
        self.world_count = worlds.len();
        if self.offset >= self.world_count {
            self.offset = if self.world_count > SAVES_PER_PAGE {
                self.world_count - SAVES_PER_PAGE
            } else {
                0
            };
        }
        for r in 0..SAVES_PER_PAGE {
            let world_index = self.offset + r;
            if world_index >= worlds.len() {
                break;
            }
            let world = &worlds[world_index];
            let world_button_id = self.gui.place(
                self.worlds_list,
                0,
                r,
                Box::new(Button::new(
                    format!(
                        "{} ({}) - v{}",
                        world.description, world.seed, world.version
                    )
                    .to_string(),
                )),
                CellAlignment::Fill,
            );
            self.worlds_buttons.push((world_button_id, world.clone()));
        }
    }
}

impl State<GameContext> for LoadGameState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {
        self.update_paging();
    }

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
                    Key::Escape => {
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
                        return StateCommand::CloseState;
                    } else if widget_id == self.next_button {
                        self.offset = if self.offset + SAVES_PER_PAGE < self.world_count {
                            self.offset + SAVES_PER_PAGE
                        } else {
                            self.offset
                        };
                        self.update_paging();
                    } else if widget_id == self.prev_button {
                        self.offset = if self.offset > SAVES_PER_PAGE {
                            self.offset - SAVES_PER_PAGE
                        } else {
                            0
                        };
                        self.update_paging();
                    }

                    for (button_id, world) in &self.worlds_buttons {
                        if widget_id == *button_id {
                            data.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                            data.connect_to_address =
                                Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                            data.seed = world.seed;
                            return StateCommand::ReplaceState {
                                state: Box::new(StartGameState::new()),
                            };
                        }
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
