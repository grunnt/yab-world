use std::any::Any;

use gamework::video::TextAlignment;

use crate::*;

use super::gui_renderer::GuiRenderer;

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: String) -> Label {
        Label { text }
    }
}

impl Widget<GuiRenderer> for Label {
    fn event(
        &mut self,
        _widget_id: WidgetId,
        _event: &InputEvent,
        _focus: bool,
    ) -> Option<GuiEvent> {
        None
    }

    fn layout(&mut self, _max_size: &Size, context: &mut GuiRenderer) -> Size {
        context.text().measure_string(&self.text, 0)
    }

    fn paint(
        &mut self,
        x: f32,
        y: f32,
        size: Size,
        context: &mut GuiRenderer,
        _config: &GuiConfig,
        _focus: bool,
    ) {
        let tx = x + size.width / 2.0;
        let ty = y + size.height / 2.0;
        context.text_mut().place_string(
            tx,
            ty,
            &self.text,
            TextAlignment::Center,
            TextAlignment::Center,
            0,
        );
    }

    fn get_value(&self) -> GuiValue {
        GuiValue::String(self.text.clone())
    }

    fn set_value(&mut self, value: GuiValue) {
        match value {
            GuiValue::String(string) => {
                self.text = string;
            }
            _ => panic!("can only set string as button value"),
        }
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
