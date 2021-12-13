use std::any::Any;

use crate::*;

use super::gui_renderer::GuiRenderer;

pub struct ProgressBar {
    value: f32,
    max_value: f32,
}

impl ProgressBar {
    pub fn new(value: f32, max_value: f32) -> ProgressBar {
        assert!(max_value > 0.0);
        ProgressBar { value, max_value }
    }
}

impl Widget<GuiRenderer> for ProgressBar {
    fn event(
        &mut self,
        _widget_id: WidgetId,
        _event: &InputEvent,
        _focus: bool,
    ) -> Option<GuiEvent> {
        None
    }

    fn layout(&mut self, max_size: &Size, _context: &mut GuiRenderer) -> Size {
        *max_size
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
        let progress = self.value / self.max_value;
        context.render_rect(
            x + 2.0,
            y + 2.0,
            (size.width - 4.0) * progress,
            size.height - 4.0,
            focus,
            GUI_LAYER_FOREGROUND,
            &config.input_focus_color,
        );
    }

    fn get_value(&self) -> GuiValue {
        GuiValue::Float(self.value)
    }

    fn set_value(&mut self, value: GuiValue) {
        match value {
            GuiValue::Float(value) => {
                self.value = value;
            }
            _ => panic!("can only set float as progress bar value"),
        }
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
