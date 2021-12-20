use crate::{
    gui::{gui_renderer::GuiRenderer, Button, Label, TextField},
    GameContext,
};
use gamework::*;
use log::*;

pub struct SettingsState {
    gui: Gui<GuiRenderer>,
    render_distance_textfield: WidgetId,
    back_button: WidgetId,
}

impl SettingsState {
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
            Box::new(Label::new("Settings".to_string())),
            CellAlignment::Center,
        );
        gui.place(
            gui.root_id(),
            1,
            2,
            Box::new(Label::new("Render range (chunks): ".to_string())),
            CellAlignment::Right,
        );
        let render_distance_textfield = gui.place(
            gui.root_id(),
            2,
            2,
            Box::new(TextField::new("".to_string(), 3)),
            CellAlignment::Fill,
        );
        let buttons = gui.grid(
            gui.root_id(),
            2,
            5,
            vec![flex_col(1.0), flex_col(1.0)],
            vec![fixed_row(50.0)],
        );
        let back_button = gui.place(
            buttons,
            0,
            0,
            Box::new(Button::new("Back".to_string())),
            CellAlignment::Fill,
        );

        SettingsState {
            gui,
            render_distance_textfield,
            back_button,
        }
    }
}

impl State<GameContext> for SettingsState {
    fn initialize(&mut self, data: &mut GameContext, _context: &mut SystemContext) {
        self.gui.set_value(
            &self.render_distance_textfield,
            GuiValue::String(data.config.render_range_chunks.to_string()),
        );
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
                    Key::Tab => {
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
                    if widget_id == self.back_button {
                        let distance_string =
                            match self.gui.get_value(&self.render_distance_textfield) {
                                GuiValue::String(distance_string) => distance_string,
                                _ => panic!("Unexpected value type for textfield"),
                            };
                        match distance_string.parse::<u16>() {
                            Ok(distance) => {
                                if distance >= 4 && distance <= 512 {
                                    data.config.render_range_chunks = distance;
                                    data.config.save();
                                    return StateCommand::CloseState;
                                }
                            }
                            _ => {
                                warn!("Could not parse seed value as u32");
                            }
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
