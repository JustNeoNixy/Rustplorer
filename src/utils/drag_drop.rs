use eframe::egui;

// Handle drop logic - returns move request if item was dropped on a folder
pub fn handle_drop(
    ui: &egui::Ui,
    update: &egui_dnd::DragUpdate,
    sorted_indices_snapshot: &[usize],
    folder_rects: &[(usize, usize, egui::Rect)],
) -> Option<(usize, usize)> {
    let from_idx = sorted_indices_snapshot[update.from];
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    if let Some(pos) = pointer_pos {
        for (_sorted_pos, folder_idx, rect) in folder_rects {
            if rect.contains(pos) && *folder_idx != from_idx {
                return Some((from_idx, *folder_idx));
            }
        }
    }

    None
}
