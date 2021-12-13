use std::any::Any;

use gamework::video::color::ColorRGBA;

use crate::*;

use super::gui_renderer::GuiRenderer;

pub struct ImageButton {
    texture_name: String,
    texture_id: Option<usize>,
    padding: f32,
}

impl ImageButton {
    pub fn new(texture_name: &String) -> ImageButton {
        ImageButton {
            texture_name: texture_name.clone(),
            texture_id: None,
            padding: 7.0,
        }
    }
}

impl Widget<GuiRenderer> for ImageButton {
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

    fn layout(&mut self, max_size: &Size, context: &mut GuiRenderer) -> Size {
        // Get texture ID by name if we do not have it yet
        if self.texture_id.is_none() {
            self.texture_id = context
                .sprite_batcher_mut()
                .texture_atlas()
                .find_id(&self.texture_name);
        }
        let frame_size = context
            .sprite_batcher_mut()
            .texture_atlas()
            .calculate_frame_size_pixels(self.texture_id.unwrap());
        let mut size = Size::new(
            frame_size.width + self.padding * 2.0,
            frame_size.height + self.padding * 2.0,
        );
        if size.width > max_size.width {
            size.width = max_size.width;
        }
        if size.height > max_size.height {
            size.height = max_size.height;
        }
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
        context.sprite_batcher_mut().add(
            x + self.padding,
            y + self.padding,
            size.width - 2.0 * self.padding,
            size.height - 2.0 * self.padding,
            0.0,
            &ColorRGBA::white(),
            self.texture_id.unwrap(),
        )
    }

    fn get_value(&self) -> GuiValue {
        GuiValue::None
    }

    fn set_value(&mut self, _value: GuiValue) {}

    fn is_focusable(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
