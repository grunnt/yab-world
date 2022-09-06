use egui::NumExt;

pub fn block_button(ui: &mut egui::Ui, name: egui::WidgetText, count: u32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(64.0), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = if count > 0 {
            ui.style().interact(&response)
        } else {
            ui.style().noninteractive()
        };

        // Background
        let rect = rect.expand(visuals.expansion);
        ui.painter()
            .rect(rect, 0.0, visuals.bg_fill, visuals.bg_stroke);

        // TODO block image (resize to fit)

        // TODO block count
    }

    response
}
