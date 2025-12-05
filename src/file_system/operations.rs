use super::file_tree::FileNode;
use eframe::egui;

// Handle delete confirmation modal
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
