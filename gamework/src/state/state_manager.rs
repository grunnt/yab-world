use crate::state::*;
use crate::InputEvent;
use crate::*;
use log::*;

pub enum StateCommand<GameContext> {
    None,
    CloseState,
    ReplaceState { state: Box<dyn State<GameContext>> },
    OpenState { state: Box<dyn State<GameContext>> },
}

pub struct StateManager<GameContext> {
    state_stack: Vec<Box<dyn State<GameContext>>>,
    game_context: GameContext,
}

impl<GameContext: SharedContext> StateManager<GameContext> {
    pub fn new(data: GameContext) -> StateManager<GameContext> {
        StateManager {
            state_stack: Vec::new(),
            game_context: data,
        }
    }

    pub fn activate(&mut self, mut state: Box<dyn State<GameContext>>, system: &mut SystemContext) {
        state.initialize(&mut self.game_context, system);
        self.state_stack.push(state);
    }

    pub fn update(
        &mut self,
        delta: f32,
        gui: &egui::Context,
        input_events: &Vec<InputEvent>,
        system: &mut SystemContext,
    ) -> bool {
        assert!(!self.state_stack.is_empty());
        let top_state = self.state_stack.last_mut().unwrap();
        let mut exit = false;
        match top_state.update(delta, &mut self.game_context, gui, input_events, system) {
            StateCommand::OpenState { mut state } => {
                debug!("Open state {}", state.type_name());
                state.initialize(&mut self.game_context, system);
                self.state_stack.push(state);
            }
            StateCommand::ReplaceState { mut state } => {
                debug!(
                    "Replace {} state with {}",
                    top_state.type_name(),
                    state.type_name()
                );
                top_state.shutdown();
                self.state_stack.pop();
                state.initialize(&mut self.game_context, system);
                self.state_stack.push(state);
            }
            StateCommand::CloseState {} => {
                debug!("Close state {}", top_state.type_name());
                top_state.shutdown();
                self.state_stack.pop();
                if self.state_stack.is_empty() {
                    exit = true;
                }
            }
            StateCommand::None => {}
        };
        if exit {
            for state in &mut self.state_stack {
                state.shutdown();
            }
        }
        exit
    }

    pub fn render(&mut self, system: &mut SystemContext) {
        let top_state = self.state_stack.last_mut().unwrap();
        top_state.render(&mut self.game_context, system);
    }

    pub fn resize(&mut self, system: &mut SystemContext) {
        let top_state = self.state_stack.last_mut().unwrap();
        top_state.resize(&mut self.game_context, system);
    }
}
