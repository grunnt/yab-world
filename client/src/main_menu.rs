use crate::{
    gui::{gui_renderer::GuiRenderer, Button, Label},
    join_game::JoinGameState,
    load_game::LoadGameState,
    new_game::NewGameState,
    settings::SettingsState,
};
use crate::{start_game::StartGameState, GameContext};
use common::comms::DEFAULT_TCP_PORT;
use common::world_definition::*;
use gamework::*;
use log::*;

pub struct MainMenuState {
    gui: Gui<GuiRenderer>,
    continue_button: WidgetId,
    new_game_button: WidgetId,
    load_button: WidgetId,
    join_button: WidgetId,
    settings_button: WidgetId,
    exit_button: WidgetId,
    continue_save: Option<WorldDef>,
}

impl MainMenuState {
    pub fn new() -> Self {
        let mut gui = Gui::new(
            vec![flex_col(1.0), fixed_col(200.0), flex_col(1.0)],
            vec![
                flex_row(1.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(50.0),
                fixed_row(50.0),
                flex_row(1.0),
            ],
        );
        gui.place(
            gui.root_id(),
            1,
            1,
            Box::new(Label::new("YAB-World".to_string())),
            CellAlignment::Center,
        );
        let continue_button = gui.place(
            gui.root_id(),
            1,
            2,
            Box::new(Button::new("Continue".to_string())),
            CellAlignment::Fill,
        );
        let new_game_button = gui.place(
            gui.root_id(),
            1,
            3,
            Box::new(Button::new("New".to_string())),
            CellAlignment::Fill,
        );
        let load_button = gui.place(
            gui.root_id(),
            1,
            4,
            Box::new(Button::new("Load".to_string())),
            CellAlignment::Fill,
        );
        let join_button = gui.place(
            gui.root_id(),
            1,
            5,
            Box::new(Button::new("Join".to_string())),
            CellAlignment::Fill,
        );
        let settings_button = gui.place(
            gui.root_id(),
            1,
            6,
            Box::new(Button::new("Settings".to_string())),
            CellAlignment::Fill,
        );
        let exit_button = gui.place(
            gui.root_id(),
            1,
            7,
            Box::new(Button::new("Exit".to_string())),
            CellAlignment::Fill,
        );
        MainMenuState {
            gui,
            continue_button,
            new_game_button,
            load_button,
            join_button,
            settings_button,
            exit_button,
            continue_save: None,
        }
    }
}

impl State<GameContext> for MainMenuState {
    fn initialize(&mut self, _data: &mut GameContext, _context: &mut SystemContext) {
        let worlds = WorldList::new();
        let world_list = worlds.list_worlds();
        self.continue_save = if world_list.is_empty() {
            None
        } else {
            Some(world_list.get(0).unwrap().clone())
        };
    }

    fn update(
        &mut self,
        _delta: f32,
        data: &mut GameContext,
        input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext> {
        let gui_events = self.gui.update(
            input_events,
            context.video().screen_size(),
            data.gui_renderer_mut(),
        );
        for event in gui_events {
            match event {
                GuiEvent::ButtonClicked { widget_id } => {
                    context.play_sound(&data.sound_high_beep);
                    if widget_id == self.continue_button {
                        if self.continue_save.is_some() {
                            let save = self.continue_save.as_ref().unwrap();
                            debug!("Continue game");
                            data.server_address = Some(format!("0.0.0.0:{}", DEFAULT_TCP_PORT));
                            data.connect_to_address =
                                Some(format!("127.0.0.1:{}", DEFAULT_TCP_PORT));
                            data.seed = save.seed;
                            data.description = save.description.clone();
                            return StateCommand::OpenState {
                                state: Box::new(StartGameState::new()),
                            };
                        }
                    } else if widget_id == self.new_game_button {
                        return StateCommand::OpenState {
                            state: Box::new(NewGameState::new()),
                        };
                    } else if widget_id == self.load_button {
                        debug!("Load game");
                        return StateCommand::OpenState {
                            state: Box::new(LoadGameState::new()),
                        };
                    } else if widget_id == self.join_button {
                        debug!("Join game");
                        return StateCommand::OpenState {
                            state: Box::new(JoinGameState::new()),
                        };
                    } else if widget_id == self.settings_button {
                        debug!("Settings");
                        return StateCommand::OpenState {
                            state: Box::new(SettingsState::new()),
                        };
                    } else if widget_id == self.exit_button {
                        debug!("Exit");
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
