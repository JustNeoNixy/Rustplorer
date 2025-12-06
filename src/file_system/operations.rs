use super::file_tree::FileNode;
use eframe::egui;

// Handle delete confirmation modal
pub fn show_delete_confirmation_modal(
    ui: &mut egui::Ui,
    confirm_delete_id: egui::Id,
) -> Option<usize> {
    let mut delete_idx = None;

    if let Some((_, idx, name, _)) =
        ui.data_mut(|d| d.get_temp::<(bool, usize, String, bool)>(confirm_delete_id))
    {
        egui::Window::new("Confirm Delete")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label(format!("Are you sure you want to delete '{}'?", name));
                ui.label("This will delete all contents permanently.");

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        ui.data_mut(|d| d.remove::<(bool, usize, String, bool)>(confirm_delete_id));
                    }
                    if ui.button("Delete").clicked() {
                        delete_idx = Some(idx);
                        ui.data_mut(|d| d.remove::<(bool, usize, String, bool)>(confirm_delete_id));
                    }
                });
            });
    }

    delete_idx
}

// Execute a file/folder deletion
pub fn execute_delete(node: &mut FileNode, idx: usize) {
    // Ensure children are loaded so we can safely index
    node.ensure_children_loaded();
    let children = node.children.as_mut().unwrap();

    let child = &children[idx];

    let delete_result = if child.is_dir {
        std::fs::remove_dir_all(&child.path)
    } else {
        std::fs::remove_file(&child.path)
    };

    match delete_result {
        Ok(_) => {
            children.remove(idx);
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
    let target_folder = original_children[target_folder_idx].clone();
    let target_path = target_folder.path.join(&moved_item.name);

    match std::fs::rename(&moved_item.path, &target_path) {
        Ok(_) => {
            // Rebuild current children list (from original) and remove moved item
            node.ensure_children_loaded();
            let children = node.children.as_mut().unwrap();

            *children = original_children;
            children.retain(|child| child.path != moved_item.path);

            // Refresh the target folder (if it's one of our children)
            if let Some(target) = children.iter_mut().find(|c| c.path == target_folder.path) {
                target.children = None; // mark as needing reload
                target.ensure_children_loaded(); // reload its contents
            }

            println!("Moved {} into {}", moved_item.name, target_folder.name);
        }
        Err(e) => {
            // Restore original state on failure
            node.ensure_children_loaded();
            *node.children.as_mut().unwrap() = original_children;
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
        // For directories, check if we know the child count.
        // We'll assume non-empty if its a dir and we haven't loaded children,
        // or if children exist.
        let is_non_empty_dir = child.is_dir
            && (child.children.is_none() || // not loaded - assume may have content
                child.children.as_ref().map_or(false, |c| !c.is_empty()));

        if is_non_empty_dir {
            // Store confirmation data: (is_dir, index, name, has_children)
            ui.data_mut(|d| {
                d.insert_temp(
                    confirm_delete_id,
                    (
                        true, // is confirmation needed
                        child_idx,
                        child.name.clone(),
                        is_non_empty_dir, // or just true
                    ),
                )
            });
        } else {
            delete_request = Some(child_idx);
        }
        ui.close();
    }

    delete_request
}
