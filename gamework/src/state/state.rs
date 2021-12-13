use crate::*;

pub trait State<GameContext: SharedContext> {
    fn initialize(&mut self, data: &mut GameContext, context: &mut SystemContext);

    fn update(
        &mut self,
        delta: f32,
        data: &mut GameContext,
        input_events: &Vec<InputEvent>,
        context: &mut SystemContext,
    ) -> StateCommand<GameContext>;

    fn resize(&mut self, data: &mut GameContext, context: &mut SystemContext);

    fn render(&mut self, data: &mut GameContext, context: &mut SystemContext);

    fn shutdown(&mut self);

    #[doc(hidden)]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// Application-specific context shared between states
pub trait SharedContext {
    fn initialize(&mut self, context: &mut SystemContext);
}
