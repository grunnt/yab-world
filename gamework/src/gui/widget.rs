use crate::*;
use std::{
    any::Any,
    sync::atomic::{AtomicU64, Ordering},
};

pub const NO_WIDGET: WidgetId = WidgetId(0);
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct WidgetId(u64);

impl WidgetId {
    pub fn next() -> WidgetId {
        static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        WidgetId(WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub trait Widget<T> {
    fn event(&mut self, widget_id: WidgetId, event: &InputEvent, focus: bool) -> Option<GuiEvent>;

    fn layout(&mut self, max_size: &Size, context: &mut T) -> Size;

    fn paint(
        &mut self,
        x: f32,
        y: f32,
        size: Size,
        context: &mut T,
        config: &GuiConfig,
        focus: bool,
    );

    fn get_value(&self) -> GuiValue;

    fn set_value(&mut self, value: GuiValue);

    fn is_focusable(&self) -> bool;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    #[doc(hidden)]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
