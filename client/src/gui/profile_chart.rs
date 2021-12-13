use gamework::{
    video::{color::ColorRGBA, TextAlignment},
    GuiConfig, GuiEvent, GuiValue, InputEvent, Size, SystemContext, Widget, WidgetId,
    GUI_LAYER_BACKGROUND, PROFILE_SAMPLES,
};
use std::any::Any;

use super::gui_renderer::GuiRenderer;

pub struct ProfileChart {
    padding: f32,
    size_x: f32,
    size_y: f32,
    text: String,
    text_size: Size,
    buffer: [[f32; 4]; PROFILE_SAMPLES],
}

impl ProfileChart {
    pub fn new(size_y: f32) -> ProfileChart {
        ProfileChart {
            padding: 7.0,
            size_x: PROFILE_SAMPLES as f32,
            size_y,
            text: "...".to_string(),
            text_size: Size::new(0.0, 0.0),
            buffer: [[0.0; 4]; PROFILE_SAMPLES],
        }
    }

    pub fn update(&mut self, context: &SystemContext) {
        self.text = format!(
            "frame {} ms / max {} ms / {} fps",
            context.frame_profile().current_ms.round(),
            context.frame_profile().max_ms.round(),
            context.frame_profile().fps.round()
        );
    }

    pub fn buffer_mut(&mut self) -> &mut [[f32; 4]; PROFILE_SAMPLES] {
        &mut self.buffer
    }
}

impl Widget<GuiRenderer> for ProfileChart {
    fn event(
        &mut self,
        _widget_id: WidgetId,
        _event: &InputEvent,
        _focus: bool,
    ) -> Option<GuiEvent> {
        None
    }

    fn layout(&mut self, _max_size: &Size, context: &mut GuiRenderer) -> Size {
        self.text_size = context.text().measure_string(&self.text, 1);
        Size::new(
            self.padding * 2.0 + self.size_x,
            self.padding * 2.0 + self.size_y,
        )
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

        let tx = x + self.padding + size.width / 2.0;
        let ty = y + self.padding;
        context.text_mut().place_string(
            tx,
            ty,
            &self.text,
            TextAlignment::Center,
            TextAlignment::Start,
            1,
        );

        let mut dx = 0.0;
        let y1 = y + self.size_y + self.padding;
        for frame in &self.buffer {
            let y_scale = 2.0;
            let other_duration = frame[0] * y_scale;
            let update_duration = frame[1] * y_scale;
            let render_duration = frame[2] * y_scale;
            let swap_duration = frame[3] * y_scale;
            let x = x + dx + self.padding;
            let y2 = y1 - other_duration;
            context
                .primitive_render_mut()
                .line(x, y1, x, y2, &ColorRGBA::new(0.6, 0.6, 0.6, 1.0));
            let y3 = y2 - update_duration;
            context
                .primitive_render_mut()
                .line(x, y2, x, y3, &ColorRGBA::new(0.4, 0.9, 0.4, 1.0));
            let y4 = y3 - render_duration;
            context
                .primitive_render_mut()
                .line(x, y3, x, y4, &ColorRGBA::new(0.8, 0.4, 0.4, 1.0));
            let y5 = y4 - swap_duration;
            context
                .primitive_render_mut()
                .line(x, y4, x, y5, &ColorRGBA::new(1.0, 1.0, 1.0, 1.0));
            dx += 1.0;
        }
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
