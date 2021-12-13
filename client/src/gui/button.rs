use std::any::Any;

use gamework::video::TextAlignment;

use crate::*;

use super::gui_renderer::GuiRenderer;

pub struct Button {
    text: String,
    text_size: Size,
    padding_x: f32,
    padding_y: f32,
}

impl Button {
    pub fn new(text: String) -> Button {
        Button {
            text,
            text_size: Size::new(0.0, 0.0),
            padding_x: 10.0,
            padding_y: 5.0,
        }
    }
}

impl Widget<GuiRenderer> for Button {
    fn event(&mut self, widget_id: WidgetId, event: &InputEvent, _focus: bool) -> Option<GuiEvent> {
        match event {
            InputEvent::MouseClick { button, .. } => {
                if *button == MouseButton::Left {
                    return Some(GuiEvent::ButtonClicked { widget_id });
                }
            }
            _ => {}
        }
        None
    }

    fn layout(&mut self, _max_size: &Size, context: &mut GuiRenderer) -> Size {
        let mut size = context.text().measure_string(&self.text, 0);
        self.text_size = size.clone();
        size.width += self.padding_x * 2.0;
        size.height += self.padding_y * 2.0;
        size
    }

    fn paint(
        &mut self,
        x: f32,
        y: f32,
        size: Size,
        context: &mut GuiRenderer,
        config: &GuiConfig,
        focus: bool,
    ) {
        context.render_rect(
            x,
            y,
            size.width,
            size.height,
            focus,
            GUI_LAYER_BACKGROUND,
            &config.background_color,
        );
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
