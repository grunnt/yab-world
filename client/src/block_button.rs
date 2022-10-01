use egui::{Align2, Color32, FontId, Mesh, Pos2, Rect, Shape, TextureHandle, Vec2};

pub fn block_button(
    ui: &mut egui::Ui,
    preview_texture: &TextureHandle,
    preview_size: Vec2,
    count: u32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(preview_size, egui::Sense::click());

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
        // Preview image
        let mut mesh = Mesh::with_texture(preview_texture.id());
        let rect = rect.shrink(5.0);
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().add(Shape::mesh(mesh));
        // Block count
        if count > 0 {
            ui.painter().text(
                rect.max,
                Align2::RIGHT_BOTTOM,
                format!("{}", count),
                FontId::proportional(14.0),
                Color32::WHITE,
            );
        }
    }

    response
}
