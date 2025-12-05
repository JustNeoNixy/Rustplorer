use eframe::egui;

// Common visual feedback for drag and drop
pub fn draw_item_feedback(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    is_dragged: bool,
    is_drop_target: bool,
    is_hovered: bool,
    is_drag_active: bool,
) {
    if is_dragged {
        ui.painter().rect_filled(
            rect,
            5.0,
            ui.style()
                .visuals
                .widgets
                .inactive
                .bg_fill
                .gamma_multiply(0.3),
        );
    } else if is_drop_target {
        ui.painter().rect_filled(
            rect,
            5.0,
            egui::Color32::from_rgba_premultiplied(100, 200, 255, 50),
        );
        ui.painter().rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
            egui::StrokeKind::Outside,
        );
    } else if is_hovered && !is_drag_active {
        ui.painter()
            .rect_filled(rect, 5.0, ui.style().visuals.widgets.hovered.weak_bg_fill);
    }
}
