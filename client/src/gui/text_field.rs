use std::any::Any;

use gamework::video::TextAlignment;

use crate::*;

use super::gui_renderer::GuiRenderer;

pub struct TextField {
    text: String,
    text_size: Size,
    padding_x: f32,
    padding_y: f32,
    max_length: usize,
    password_mode: bool,
}

impl TextField {
    pub fn new(text: String, max_length: usize) -> TextField {
        TextField {
            text,
            text_size: Size::new(0.0, 0.0),
            padding_x: 10.0,
            padding_y: 5.0,
            max_length,
            password_mode: false,
        }
    }

    pub fn set_password_mode(&mut self, active: bool) {
        self.password_mode = active;
    }
}

impl Widget<GuiRenderer> for TextField {
    fn event(&mut self, _widget_id: WidgetId, event: &InputEvent, focus: bool) -> Option<GuiEvent> {
        match event {
            InputEvent::KeyPress { key, shift } => {
                if focus {
                    if let Some(character) = key.to_char() {
                        let character = if *shift {
                            character
                        } else {
                            character.to_ascii_lowercase()
                        };
                        if self.text.len() < self.max_length {
                            self.text.push(character);
                        }
                    } else {
                        match key {
                            Key::Backspace => {
                                let len = self.text.len();
                                if len > 0 {
                                    self.text.truncate(len - 1);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn layout(&mut self, _max_size: &Size, context: &mut GuiRenderer) -> Size {
        // Hack to generate a test string of maximum possible width
        let test_string = std::iter::repeat("W")
            .take(self.max_length)
            .collect::<String>();
        let mut size = context.text().measure_string(&test_string, 0);
        size.width += self.padding_x * 2.0;
        size.height += self.padding_y * 2.0;
        self.text_size = size.clone();
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
        let back_color = if focus {
            &config.input_focus_color
        } else {
            &config.input_color
        };
        context.render_rect(
            x,
            y,
            size.width,
            size.height,
            focus,
            GUI_LAYER_BACKGROUND,
            back_color,
        );
        let x = x + size.width - self.padding_x;
        let y = y + size.height / 2.0;
        if self.password_mode {
            let display_text = std::iter::repeat("•")
                .take(self.text.len())
                .collect::<String>();
            context.text_mut().place_string(
                x,
                y,
                &display_text,
                TextAlignment::End,
                TextAlignment::Center,
                0,
            );
        } else {
            context.text_mut().place_string(
                x,
                y,
                &self.text,
                TextAlignment::End,
                TextAlignment::Center,
                0,
            );
        }
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
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
