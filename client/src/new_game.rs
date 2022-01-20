use crate::{
    gui::{gui_renderer::GuiRenderer, Button, Label, Selector, TextField},
    start_game::StartGameState,
    GameContext,
};
use common::{comms::DEFAULT_TCP_PORT, world_type::GeneratorType};
use gamework::*;
use log::*;
use rand::{prelude::ThreadRng, Rng};

pub struct NewGameState {
    gui: Gui<GuiRenderer>,
    rng: ThreadRng,
    seed_textfield: WidgetId,
    random_button: WidgetId,
    description_textfield: WidgetId,
    world_type_selector: WidgetId,
    create_button: WidgetId,
    back_button: WidgetId,
}

impl NewGameState {
    pub fn new() -> Self {
        let mut gui = Gui::new(
            vec![
                flex_col(1.0),
                fixed_col(200.0),
                fixed_col(400.0),
                fixed_col(50.0),
                flex_col(1.0),
            ],
            vec![
                flex_row(1.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(150.0),
                fixed_row(10.0),
                fixed_row(50.0),
                flex_row(1.0),
            ],
        );
        gui.place(
            gui.root_id(),
            2,
            1,
            Box::new(Label::new("New world".to_string())),
            CellAlignment::Center,
        );
        gui.place(
            gui.root_id(),
            1,
            2,
            Box::new(Label::new("Seed: ".to_string())),
            CellAlignment::Right,
        );
        let seed_textfield = gui.place(
            gui.root_id(),
            2,
            2,
            Box::new(TextField::new("".to_string(), 9)),
            CellAlignment::Fill,
        );
        let random_button = gui.place(
            gui.root_id(),
            3,
            2,
            Box::new(Button::new("Random".to_string())),
            CellAlignment::Left,
        );
        gui.place(
            gui.root_id(),
            1,
            3,
            Box::new(Label::new("Name: ".to_string())),
            CellAlignment::Right,
        );
        let description_textfield = gui.place(
            gui.root_id(),
            2,
            3,
            Box::new(TextField::new("".to_string(), 16)),
            CellAlignment::Fill,
        );
        let world_type_selector = gui.place(
            gui.root_id(),
            2,
            4,
            Box::new(Selector::new(vec![
                "Default".to_string(),
                "Flat".to_string(),
                "Water".to_string(),
                "Alien".to_string(),
            ])),
            CellAlignment::Fill,
        );
        let buttons = gui.grid(
            gui.root_id(),
            2,
            6,
            vec![flex_col(1.0), flex_col(1.0)],
            vec![fixed_row(50.0)],
        );
        let create_button = gui.place(
            buttons,
            0,
            0,
            Box::new(Button::new("Create".to_string())),
            CellAlignment::Fill,
        );
        let back_button = gui.place(
            buttons,
            1,
            0,
            Box::new(Button::new("Back".to_string())),
            CellAlignment::Fill,
        );

        NewGameState {
            gui,
            rng: rand::thread_rng(),
            seed_textfield,
            random_button,
            description_textfield,
            world_type_selector,
            create_button,
            back_button,
        }
    }
}

impl State<GameContext> for NewGameState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {
        self.gui.set_value(
            &self.seed_textfield,
            GuiValue::String(self.rng.gen::<u32>().to_string()),
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
                    context.audio_mut().play_sound("click");
                    if widget_id == self.create_button {
                        let world_type = match self.gui.get_value(&self.world_type_selector) {
                            GuiValue::Usize(selected_index) => match selected_index {
                                0 => GeneratorType::Default,
                                1 => GeneratorType::Flat,
                                2 => GeneratorType::Water,
                                3 => GeneratorType::Alien,
                                _ => panic!(
                                    "Unexpected value for world type selector: {}",
                                    selected_index
                                ),
                            },
                            _ => panic!("Unexpected value type for selector"),
                        };
                        let description = match self.gui.get_value(&self.description_textfield) {
                            GuiValue::String(description) => description,
                            _ => panic!("Unexpected value type for textfield"),
                        };
                        if description.trim().is_empty() {
                            continue;
                        }
                        data.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                        data.connect_to_address = Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                        data.world_type = Some(world_type);
                        let seed_string = match self.gui.get_value(&self.seed_textfield) {
                            GuiValue::String(seed_string) => seed_string,
                            _ => panic!("Unexpected value type for textfield"),
                        };
                        match seed_string.parse() {
                            Ok(seed) => {
                                data.seed = seed;
                                data.description = description;
                                return StateCommand::ReplaceState {
                                    state: Box::new(StartGameState::new()),
                                };
                            }
                            _ => {
                                warn!("Could not parse seed value as u32");
                            }
                        }
                    } else if widget_id == self.random_button {
                        self.gui.set_value(
                            &self.seed_textfield,
                            GuiValue::String(self.rng.gen::<u32>().to_string()),
                        );
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
