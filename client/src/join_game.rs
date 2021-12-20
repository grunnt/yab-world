use crate::{
    gui::{gui_renderer::GuiRenderer, Button, Label, TextField},
    start_game::StartGameState,
    GameContext,
};
use common::comms::DEFAULT_TCP_PORT;
use gamework::*;
use log::*;

pub struct JoinGameState {
    gui: Gui<GuiRenderer>,
    address_field: WidgetId,
    join_button: WidgetId,
    back_button: WidgetId,
}

impl JoinGameState {
    pub fn new() -> Self {
        let mut gui = Gui::new(
            vec![
                flex_col(1.0),
                fixed_col(200.0),
                fixed_col(400.0),
                flex_col(1.0),
            ],
            vec![
                flex_row(1.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(10.0),
                fixed_row(50.0),
                flex_row(1.0),
            ],
        );
        gui.place(
            gui.root_id(),
            2,
            1,
            Box::new(Label::new("Join server".to_string())),
            CellAlignment::Center,
        );
        gui.place(
            gui.root_id(),
            1,
            2,
            Box::new(Label::new("Server address: ".to_string())),
            CellAlignment::Right,
        );
        let address_field = gui.place(
            gui.root_id(),
            2,
            2,
            Box::new(TextField::new("".to_string(), 15)),
            CellAlignment::Fill,
        );
        let buttons = gui.grid(
            gui.root_id(),
            2,
            4,
            vec![flex_col(1.0), flex_col(1.0)],
            vec![fixed_row(50.0)],
        );
        let join_button = gui.place(
            buttons,
            0,
            0,
            Box::new(Button::new("Join".to_string())),
            CellAlignment::Fill,
        );
        let back_button = gui.place(
            buttons,
            1,
            0,
            Box::new(Button::new("Back".to_string())),
            CellAlignment::Fill,
        );

        JoinGameState {
            gui,
            address_field,
            join_button,
            back_button,
        }
    }
}

impl State<GameContext> for JoinGameState {
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
                    context.play_sound(&data.sound_high_beep);
                    if widget_id == self.join_button {
                        let address = self.gui.get_value(&self.address_field);
                        match address {
                            GuiValue::String(value) => {
                                let value = if value == "localhost" {
                                    "127.1.1.1".to_string()
                                } else {
                                    value
                                };
                                let server_address = format!("{}:{}", value, DEFAULT_TCP_PORT);
                                debug!("Join server at address {}", server_address);
                                data.connect_to_address = Some(server_address);
                                return StateCommand::OpenState {
                                    state: Box::new(StartGameState::new()),
                                };
                            }
                            _ => panic!("Unexpected value type for text field"),
                        }
                    } else if widget_id == self.back_button {
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
