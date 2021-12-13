use std::any::Any;

use gamework::video::color::ColorRGBA;

use crate::*;

use super::gui_renderer::GuiRenderer;

pub struct BlockButton {
    texture_name_up: String,
    texture_name_front: String,
    texture_name_right: String,
    texture_id_up: Option<usize>,
    texture_id_front: Option<usize>,
    texture_id_right: Option<usize>,
    emissive: bool,
    size: f32,
    padding: f32,
}

impl BlockButton {
    pub fn new(
        texture_name_up: &String,
        texture_name_front: &String,
        texture_name_right: &String,
        size: f32,
        emissive: bool,
    ) -> BlockButton {
        BlockButton {
            texture_name_up: texture_name_up.clone(),
            texture_name_front: texture_name_front.clone(),
            texture_name_right: texture_name_right.clone(),
            texture_id_up: None,
            texture_id_front: None,
            texture_id_right: None,
            emissive,
            size,
            padding: 7.0,
        }
    }
}

impl Widget<GuiRenderer> for BlockButton {
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

    fn layout(&mut self, _max_size: &Size, _context: &mut GuiRenderer) -> Size {
        Size::new(self.size, self.size)
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
        // Get texture ID by name if we do not have it yet
        if self.texture_id_up.is_none() {
            self.texture_id_up = context
                .sprite_batcher_mut()
                .texture_atlas()
                .find_id(&self.texture_name_up);
        }
        if self.texture_id_front.is_none() {
            self.texture_id_front = context
                .sprite_batcher_mut()
                .texture_atlas()
                .find_id(&self.texture_name_front);
        }
        if self.texture_id_right.is_none() {
            self.texture_id_right = context
                .sprite_batcher_mut()
                .texture_atlas()
                .find_id(&self.texture_name_right);
        }

        // Render background
        context.render_rect(
            x,
            y,
            size.width,
            size.height,
            focus,
            GUI_LAYER_BACKGROUND,
            &config.background_color,
        );

        // Render 3 sides of a fake cube
        let x = x + self.padding;
        let y = y + self.padding;
        let width = size.width - 2.0 * self.padding;
        let height = size.height - 2.0 * self.padding;
        let x1 = x + width * 0.5;
        let y1 = y;
        let x2 = x + width;
        let y2 = y + height * 0.25;
        let x3 = x + width * 0.5;
        let y3 = y + height * 0.5;
        let x4 = x;
        let y4 = y + height * 0.25;
        let x5 = x;
        let y5 = y + height * 0.75;
        let x6 = x + width * 0.5;
        let y6 = y + height;
        let x7 = x + width;
        let y7 = y + height * 0.75;
        context.sprite_batcher_mut().add_points(
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
            x4,
            y4,
            &ColorRGBA::white(),
            self.texture_id_up.unwrap(),
        );
        context.sprite_batcher_mut().add_points(
            x4,
            y4,
            x3,
            y3,
            x6,
            y6,
            x5,
            y5,
            &if self.emissive {
                ColorRGBA::white()
            } else {
                ColorRGBA::new(0.8, 0.8, 0.8, 1.0)
            },
            self.texture_id_front.unwrap(),
        );
        context.sprite_batcher_mut().add_points(
            x3,
            y3,
            x2,
            y2,
            x7,
            y7,
            x6,
            y6,
            &if self.emissive {
                ColorRGBA::white()
            } else {
                ColorRGBA::new(0.6, 0.6, 0.6, 1.0)
            },
            self.texture_id_right.unwrap(),
        );
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
