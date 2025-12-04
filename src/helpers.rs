use super::file_tree::FileNode;
use eframe::egui;

// Get sorted indices based on settings
pub fn get_sorted_indices(node: &FileNode, settings: &crate::settings::Settings) -> Vec<usize> {
    let mut folder_indices: Vec<usize> = Vec::new();
    let mut file_indices: Vec<usize> = Vec::new();

    for (idx, child) in node.children.iter().enumerate() {
        if !settings.show_hidden_files && child.name.starts_with('.') {
            continue;
        }

        if child.is_dir {
            folder_indices.push(idx);
        } else {
            file_indices.push(idx);
        }
    }

    if settings.sort_items {
        let sort_fn = |&a: &usize, &b: &usize| {
            node.children[a]
                .name
                .to_lowercase()
                .cmp(&node.children[b].name.to_lowercase())
        };
        folder_indices.sort_by(sort_fn);
        file_indices.sort_by(sort_fn);
    }

    if settings.sort_folders_first {
        folder_indices
            .into_iter()
            .chain(file_indices)
            .collect::<Vec<_>>()
    } else {
        let mut all = folder_indices
            .into_iter()
            .chain(file_indices)
            .collect::<Vec<_>>();
        if settings.sort_items {
            all.sort_by(|&a, &b| {
                node.children[a]
                    .name
                    .to_lowercase()
                    .cmp(&node.children[b].name.to_lowercase())
            });
        }
        all
    }
}

// Handle the delete confirmation modal
pub fn show_delete_confirmation_modal(
    ui: &mut egui::Ui,
    confirm_delete_id: egui::Id,
) -> Option<usize> {
    let mut delete_request = None;

    if let Some((show, idx, name, count)) =
        ui.data(|d| d.get_temp::<(bool, usize, String, usize)>(confirm_delete_id))
    {
        if show {
            let mut keep_open = true;
            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("The folder '{}' contains {} item(s).", name, count));
                    ui.label("Are you sure you want to delete it?");
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            keep_open = false;
                        }
                        if ui.button("Delete").clicked() {
                            delete_request = Some(idx);
                            keep_open = false;
                        }
                    });
                });

            if !keep_open {
                ui.data_mut(|d| d.remove::<(bool, usize, String, usize)>(confirm_delete_id));
            }
        }
    }

    delete_request
}

// Execute a file/folder deletion
pub fn execute_delete(node: &mut FileNode, idx: usize) {
    let child = &node.children[idx];

    let delete_result = if child.is_dir {
        std::fs::remove_dir_all(&child.path)
    } else {
        std::fs::remove_file(&child.path)
    };

    match delete_result {
        Ok(_) => {
            node.children.remove(idx);
        }
        Err(e) => {
            eprintln!("Failed to delete {}: {}", child.path.display(), e);
        }
    }
}

// Execute a file/folder move operation
pub fn execute_move(
    node: &mut FileNode,
    from_idx: usize,
    target_folder_idx: usize,
    original_children: Vec<FileNode>,
) {
    let moved_item = original_children[from_idx].clone();
    let target_folder = &original_children[target_folder_idx].clone();
    let target_path = target_folder.path.join(&moved_item.name);

    match std::fs::rename(&moved_item.path, &target_path) {
        Ok(_) => {
            node.children = original_children;
            node.children.retain(|child| child.path != moved_item.path);

            if let Some(target) = node
                .children
                .iter_mut()
                .find(|c| c.path == target_folder.path)
            {
                target.refresh_children();
            }

            println!("Moved {} into {}", moved_item.name, target_folder.name);
        }
        Err(e) => {
            node.children = original_children;
            eprintln!("Failed to move file: {}", e);
        }
    }
}

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

//TODO: Implement renaming, copying, (not file clicked) pasting, refreshing, etc.
// Handle context menu for delete
pub fn show_context_menu(
    ui: &mut egui::Ui,
    child: &FileNode,
    child_idx: usize,
    confirm_delete_id: egui::Id,
) -> Option<usize> {
    let mut delete_request = None;

    if ui.button("Delete").clicked() {
        if child.is_dir && !child.children.is_empty() {
            ui.data_mut(|d| {
                d.insert_temp(
                    confirm_delete_id,
                    (true, child_idx, child.name.clone(), child.children.len()),
                )
            });
        } else {
            delete_request = Some(child_idx);
        }
        ui.close();
    }

    delete_request
}

// File size formatting
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f64 = bytes as f64;
    let index = (bytes_f64.log2() / THRESHOLD.log2()).floor() as usize;
    let index = index.min(UNITS.len() - 1);

    let size = bytes_f64 / THRESHOLD.powi(index as i32);

    if index == 0 {
        format!("{} {}", bytes, UNITS[index])
    } else {
        format!("{:.2} {}", size, UNITS[index])
    }
}
