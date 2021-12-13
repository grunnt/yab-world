use super::gui_renderer::GuiRenderer;
use crate::*;
use gamework::video::TextAlignment;
use std::any::Any;

pub struct Selector {
    values: Vec<String>,
    value_sizes: Vec<Size>,
    selected_index: usize,
    padding_x: f32,
    padding_y: f32,
    row_padding: f32,
}

impl Selector {
    pub fn new(values: Vec<String>) -> Selector {
        Selector {
            values,
            value_sizes: Vec::new(),
            selected_index: 0,
            padding_x: 10.0,
            padding_y: 5.0,
            row_padding: 2.0,
        }
    }
}

impl Widget<GuiRenderer> for Selector {
    fn event(&mut self, widget_id: WidgetId, event: &InputEvent, _focus: bool) -> Option<GuiEvent> {
        match event {
            InputEvent::MouseClick { button, y, .. } => {
                if *button == MouseButton::Left {
                    let mut ty = self.padding_y;
                    let mut i = 0;
                    for value_size in &self.value_sizes {
                        if *y >= ty && *y < ty + value_size.height {
                            self.selected_index = i;
                            return Some(GuiEvent::ValueSelected {
                                widget_id,
                                value: i,
                            });
                        }
                        ty += value_size.height;
                        ty += self.row_padding;
                        i += 1;
                    }
                    return None;
                }
            }
            _ => {}
        }
        None
    }

    fn layout(&mut self, _max_size: &Size, context: &mut GuiRenderer) -> Size {
        self.value_sizes.clear();
        let mut size = Size::zero();
        for value in &self.values {
            if size.height > 0.0 {
                size.height += self.row_padding;
            }
            let value_size = context.text().measure_string(&value, 0);
            self.value_sizes.push(value_size);
            if value_size.width > size.width {
                size.width = value_size.width;
            }
            size.height += value_size.height;
        }
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
        if self.values.len() == self.value_sizes.len() {
            let tx = x + self.padding_x;
            let mut ty = y + self.padding_y;
            for i in 0..self.values.len() {
                if i > 0 {
                    ty += self.row_padding;
                }
                let value_size = &self.value_sizes[i];
                if i == self.selected_index {
                    context.render_rect(
                        tx,
                        ty,
                        value_size.width,
                        value_size.height,
                        focus,
                        GUI_LAYER_FOREGROUND,
                        &config.input_focus_color,
                    );
                }
                let value = &self.values[i];
                context.text_mut().place_string(
                    tx,
                    ty,
                    value,
                    TextAlignment::Start,
                    TextAlignment::Start,
                    0,
                );
                ty += value_size.height;
            }
        }
    }

    fn get_value(&self) -> GuiValue {
        GuiValue::Usize(self.selected_index)
    }

    fn set_value(&mut self, value: GuiValue) {
        match value {
            GuiValue::Usize(selected_index) => {
                if selected_index < self.values.len() {
                    self.selected_index = selected_index;
                }
            }
            _ => panic!("can only set usize as selector value"),
        }
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
