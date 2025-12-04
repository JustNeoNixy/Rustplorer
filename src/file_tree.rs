use eframe::egui;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FileNode {
    name: String,
    path: std::path::PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
}

impl FileNode {
    fn new(path: &std::path::Path) -> Self {
        let name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let is_dir = path.is_dir();
        let children = if is_dir {
            std::fs::read_dir(path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| FileNode::new(&entry.path()))
                .collect()
        } else {
            Vec::new()
        };

        Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            children,
        }
    }

    pub fn refresh_children(&mut self) {
        if self.is_dir {
            self.children = std::fs::read_dir(&self.path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| FileNode::new(&entry.path()))
                .collect();
        }
    }
}

pub fn build_file_tree(root: &std::path::Path) -> FileNode {
    FileNode::new(root)
}

fn get_file_icon(filename: &str, is_folder: bool) -> &'static str {
    if is_folder {
        return egui_nerdfonts::regular::FOLDER_1;
    }

    let extension = filename.rsplit('.').next().unwrap_or("");
    match extension {
        "rs" | "toml" | "lock" => egui_nerdfonts::regular::LANGUAGE_RUST,
        "js" | "jsx" => egui_nerdfonts::regular::LANGUAGE_JAVASCRIPT,
        "ts" | "tsx" => egui_nerdfonts::regular::LANGUAGE_TYPESCRIPT,
        "py" => egui_nerdfonts::regular::LANGUAGE_PYTHON,
        "html" => egui_nerdfonts::regular::LANGUAGE_HTML5,
        "css" | "scss" | "sass" => egui_nerdfonts::regular::LANGUAGE_CSS3,
        "json" => egui_nerdfonts::regular::JSON,
        "md" => egui_nerdfonts::regular::MARKDOWN,
        "xml" => egui_nerdfonts::regular::XML,
        "zip" | "tar" | "gz" => egui_nerdfonts::regular::FILE_ZIP,
        "jpg" | "jpeg" | "png" | "gif" | "svg" => egui_nerdfonts::regular::FILE_IMAGE,
        "mp4" | "avi" | "mov" => egui_nerdfonts::regular::FILE_VIDEO,
        "mp3" | "wav" | "ogg" => egui_nerdfonts::regular::AUDIO_VIDEO,
        "gitignore" => egui_nerdfonts::regular::GIT,
        "pdf" => egui_nerdfonts::regular::FILE_PDF,
        "c" => egui_nerdfonts::regular::LANGUAGE_C,
        "cpp" => egui_nerdfonts::regular::LANGUAGE_CPP,
        _ => egui_nerdfonts::regular::FILE,
    }
}

pub fn render_file_node(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    settings: &crate::settings::Settings,
) -> Option<std::path::PathBuf> {
    match settings.view {
        crate::settings::View::Grid => render_grid_view(ui, node, settings),
        crate::settings::View::Normal => render_normal_view(ui, node, settings),
    }
}

// File size formatting
fn format_file_size(bytes: u64) -> String {
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

fn render_grid_view(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    settings: &crate::settings::Settings,
) -> Option<std::path::PathBuf> {
    let mut nav_request = None;
    let mut move_request: Option<(usize, usize)> = None;
    let mut delete_request: Option<usize> = None;

    let confirm_delete_id = ui.id().with("confirm_delete");
    let mut show_delete_modal =
        ui.data(|d| d.get_temp::<(bool, usize, String, usize)>(confirm_delete_id));

    // Store the original order for restoring after drag
    let original_children = node.children.clone();

    // Sort children: folders first (A-Z), then files (A-Z)
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
        folder_indices.sort_by(|&a, &b| {
            node.children[a]
                .name
                .to_lowercase()
                .cmp(&node.children[b].name.to_lowercase())
        });
        file_indices.sort_by(|&a, &b| {
            node.children[a]
                .name
                .to_lowercase()
                .cmp(&node.children[b].name.to_lowercase())
        });
    };

    let mut sorted_indices = if settings.sort_folders_first {
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
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(20.0, 20.0);

            // Track which item is being dragged and folder rectangles
            let mut dragged_idx: Option<usize> = None;
            let mut folder_rects: Vec<(usize, usize, egui::Rect)> = Vec::new();

            let av_width = ui.available_width() - ui.spacing().item_spacing.x;
            let columns = (av_width / 100.0).ceil() as usize;
            let width = av_width / columns as f32;
            let size = egui::Vec2::new(width, width) + ui.spacing().item_spacing;

            let sorted_indices_snapshot = sorted_indices.clone();

            // Use egui_dnd to make the children draggable
            let response = egui_dnd::dnd(ui, "file_explorer_dnd").show_vec_sized(
                &mut sorted_indices,
                size,
                |ui, &mut child_idx, handle, state| {
                    let child = &node.children[child_idx];
                    let is_folder = child.is_dir;
                    let icon = get_file_icon(&child.name, child.is_dir);

                    // Track which item is being dragged
                    if state.dragged {
                        dragged_idx = Some(state.index);
                    }

                    // Create a vertical layout for the item
                    ui.vertical(|ui| {
                        // The drag handle needs to wrap the entire visual area
                        handle.ui(ui, |ui| {
                            // Allocate space for the file/folder item
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(120.0, 80.0), egui::Sense::click());

                            // Store folder rectangles for drop detection
                            if is_folder {
                                let original_idx = sorted_indices_snapshot[state.index];
                                folder_rects.push((state.index, original_idx, rect));
                            }

                            // Check if mouse is over this folder during drag
                            let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                            let is_drag_active = ui.input(|i| i.pointer.is_decidedly_dragging());
                            let is_drop_target = is_folder
                                && is_drag_active
                                && !state.dragged // This is NOT the item being dragged
                                && Some(state.index) != dragged_idx // Not dragging onto itself
                                && pointer_pos.map_or(false, |pos| rect.contains(pos));

                            // Visual feedback
                            if state.dragged {
                                // Being dragged - show semi-transparent
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
                            } else if resp.hovered() && !is_drag_active {
                                ui.painter().rect_filled(
                                    rect,
                                    5.0,
                                    ui.style().visuals.widgets.hovered.weak_bg_fill,
                                );
                            }

                            // Draw icon with reduced opacity if being dragged
                            let icon_color = if state.dragged {
                                ui.style().visuals.text_color().gamma_multiply(0.5)
                            } else {
                                ui.style().visuals.text_color()
                            };

                            let icon_pos = rect.center_top() + egui::vec2(0.0, 8.0);
                            ui.painter().text(
                                icon_pos,
                                egui::Align2::CENTER_TOP,
                                icon,
                                egui::FontId::proportional(32.0),
                                icon_color,
                            );

                            // Draw name with reduced opacity if being dragged
                            let name_pos = rect.center() + egui::vec2(0.0, 20.0);
                            ui.painter().text(
                                name_pos,
                                egui::Align2::CENTER_TOP,
                                &child.name,
                                egui::FontId::proportional(14.0),
                                icon_color,
                            );

                            // Double-click to navigate into folders
                            if resp.double_clicked() && is_folder && !is_drag_active {
                                nav_request = Some(child.path.clone());
                            }

                            // Right click context menu
                            resp.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    // Check if it's a folder with files
                                    if is_folder && !child.children.is_empty() {
                                        //show_delete_modal = true;
                                        ui.data_mut(|d| {
                                            d.insert_temp(
                                                confirm_delete_id,
                                                (
                                                    true,
                                                    child_idx,
                                                    child.name.clone(),
                                                    child.children.len(),
                                                ),
                                            )
                                        });
                                    } else {
                                        delete_request = Some(child_idx);
                                    }
                                    ui.close();
                                }
                            });
                        });
                    });
                },
            );

            // Handle drop
            if let Some(update) = response.final_update() {
                let from_idx = sorted_indices_snapshot[update.from];

                // Check if pointer was released over a folder
                let pointer_pos = ui.input(|i| i.pointer.hover_pos());

                if let Some(pos) = pointer_pos {
                    // Find which folder (if any) the item was dropped on
                    let mut dropped_on_folder = false;
                    for (_sorted_pos, folder_idx, rect) in &folder_rects {
                        if rect.contains(pos) {
                            if *folder_idx != from_idx {
                                //let target_folder_idx = sorted_indices[*folder_idx];
                                move_request = Some((from_idx, *folder_idx));
                                dropped_on_folder = true;
                                break;
                            }
                        }
                    }

                    // If not dropped on a folder, restore original order
                    if !dropped_on_folder {
                        node.children = original_children.clone();
                    }
                } else {
                    // No valid drop position, restore original order
                    node.children = original_children.clone();
                }
            }
        });
    });

    if let Some((show, idx, name, count)) = show_delete_modal {
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

    // Execute delete if requested
    if let Some(idx) = delete_request {
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

    // Execute move if requested
    if let Some((from_idx, target_folder_idx)) = move_request {
        // Find the original item by name since indices might have changed
        let moved_item = original_children[from_idx].clone();

        // Find the target folder in original children
        let target_folder = &original_children.clone()[target_folder_idx];
        let target_path = target_folder.path.join(&moved_item.name);

        // Perform actual filesystem move
        match std::fs::rename(&moved_item.path, &target_path) {
            Ok(_) => {
                // Successfully moved - restore original order and refresh
                node.children = original_children;

                // Remove the moved item
                node.children.retain(|child| child.path != moved_item.path);

                // Refresh the target folder to show the new item
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
                // Failed to move - restore original order
                node.children = original_children;
                eprintln!("Failed to move file: {}", e);
            }
        }
    }

    nav_request
}

fn render_normal_view(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    settings: &crate::settings::Settings,
) -> Option<std::path::PathBuf> {
    egui::TopBottomPanel::top("placeholder").show_inside(ui, |ui| {
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = ui.visuals().faint_bg_color;
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Name").strong().size(14.0));
            ui.add_space(280.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(35.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(100.0, 20.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(egui::RichText::new("Size").strong().size(14.0));
                    },
                );
                ui.add_space(35.0);
                ui.separator();
                ui.add_space(30.0);
                ui.label(egui::RichText::new("Creation Date").strong().size(14.0));
                ui.add_space(35.0);
                ui.separator();
            });
        });
        ui.add_space(4.0);
    });

    let mut nav_request = None;
    let mut move_request: Option<(usize, usize)> = None;
    let mut delete_request: Option<usize> = None;

    let confirm_delete_id = ui.id().with("confirm_delete");
    let mut show_delete_modal =
        ui.data(|d| d.get_temp::<(bool, usize, String, usize)>(confirm_delete_id));

    // Store the original order for restoring after drag
    let original_children = node.children.clone();

    // Sort children: folders first (A-Z), then files (A-Z)
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
        folder_indices.sort_by(|&a, &b| {
            node.children[a]
                .name
                .to_lowercase()
                .cmp(&node.children[b].name.to_lowercase())
        });
        file_indices.sort_by(|&a, &b| {
            node.children[a]
                .name
                .to_lowercase()
                .cmp(&node.children[b].name.to_lowercase())
        });
    };

    let mut sorted_indices = if settings.sort_folders_first {
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
    };

    egui::ScrollArea::vertical()
        .max_width(ui.available_width())
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(32.0, 16.0);

            // Track which item is being dragged and folder rectangles
            let mut dragged_idx: Option<usize> = None;
            let mut folder_rects: Vec<(usize, usize, egui::Rect)> = Vec::new();

            let sorted_indices_snapshot = sorted_indices.clone();

            let response = egui_dnd::dnd(ui, "file_explorer_dnd").show_vec(
                &mut sorted_indices,
                |ui, &mut child_idx, handle, state| {
                    let child = &node.children[child_idx];
                    let is_folder = child.is_dir;

                    // Track which item is being dragged
                    if state.dragged {
                        dragged_idx = Some(state.index);
                    }

                    ui.vertical(|ui| {
                        handle.ui(ui, |ui| {
                            // Allocate fixed height for horizontal layout
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width() - 3.0, 30.0),
                                egui::Sense::click(),
                            );

                            if is_folder {
                                let original_idx = sorted_indices_snapshot[state.index];
                                folder_rects.push((state.index, original_idx, rect));
                            }

                            // Check if mouse is over this folder during drag
                            let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                            let is_drag_active = ui.input(|i| i.pointer.is_decidedly_dragging());
                            let is_drop_target = is_folder
                                && is_drag_active
                                && !state.dragged // This is NOT the item being dragged
                                && Some(state.index) != dragged_idx // Not dragging onto itself
                                && pointer_pos.map_or(false, |pos| rect.contains(pos));

                            // Visual feedback
                            if state.dragged {
                                // Being dragged - show semi-transparent
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
                            } else if resp.hovered() && !is_drag_active {
                                ui.painter().rect_filled(
                                    rect,
                                    5.0,
                                    ui.style().visuals.widgets.hovered.weak_bg_fill,
                                );
                            }

                            // Horizontal layout for icon + name
                            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                ui.horizontal(|ui| {
                                    let icon = get_file_icon(&child.name, child.is_dir);
                                    let icon_color = if state.dragged {
                                        ui.style().visuals.text_color().gamma_multiply(0.5)
                                    } else {
                                        ui.style().visuals.text_color()
                                    };

                                    ui.label(
                                        egui::RichText::new(icon).color(icon_color).size(24.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(&child.name)
                                            .color(icon_color)
                                            .size(16.0),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if !child.is_dir {
                                                use std::os::unix::fs::MetadataExt;

                                                let meta = std::fs::metadata(&child.path).unwrap();
                                                let size = meta.size();

                                                ui.add_sized(
                                                    [100.0, 20.0],
                                                    egui::Label::new(
                                                        egui::RichText::new(format_file_size(size))
                                                            .color(icon_color)
                                                            .size(16.0)
                                                            .monospace(),
                                                    ),
                                                );
                                            } else {
                                                ui.add_sized(
                                                    [100.0, 20.0],
                                                    egui::Label::new(
                                                        egui::RichText::new("")
                                                            .size(16.0)
                                                            .monospace(),
                                                    ),
                                                );
                                            }

                                            let meta = std::fs::metadata(&child.path).unwrap();
                                            let created: std::time::SystemTime = meta
                                                .created()
                                                .expect("Couldn't get file creation date.");
                                            let datetime: chrono::DateTime<chrono::Utc> =
                                                created.into();
                                            let formatted =
                                                datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                                            ui.label(
                                                egui::RichText::new(formatted)
                                                    .color(icon_color)
                                                    .size(16.0),
                                            );
                                        },
                                    );
                                });
                            });

                            // Double-click to navigate into folder
                            if resp.double_clicked() && is_folder && !is_drag_active {
                                nav_request = Some(child.path.clone());
                            }

                            // Right click context menu
                            resp.context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    // Check if it's a folder with files
                                    if is_folder && !child.children.is_empty() {
                                        //show_delete_modal = true;
                                        ui.data_mut(|d| {
                                            d.insert_temp(
                                                confirm_delete_id,
                                                (
                                                    true,
                                                    child_idx,
                                                    child.name.clone(),
                                                    child.children.len(),
                                                ),
                                            )
                                        });
                                    } else {
                                        delete_request = Some(child_idx);
                                    }
                                    ui.close();
                                }
                            });
                        });
                    });
                },
            );

            // Handle drop
            if let Some(update) = response.final_update() {
                let from_idx = sorted_indices_snapshot[update.from];

                // Check if pointer was released over a folder
                let pointer_pos = ui.input(|i| i.pointer.hover_pos());

                if let Some(pos) = pointer_pos {
                    // Find which folder (if any) the item was dropped on
                    let mut dropped_on_folder = false;
                    for (_sorted_pos, folder_idx, rect) in &folder_rects {
                        if rect.contains(pos) {
                            if *folder_idx != from_idx {
                                //let target_folder_idx = sorted_indices[*folder_idx];
                                move_request = Some((from_idx, *folder_idx));
                                dropped_on_folder = true;
                                break;
                            }
                        }
                    }

                    // If not dropped on a folder, restore original order
                    if !dropped_on_folder {
                        node.children = original_children.clone();
                    }
                } else {
                    // No valid drop position, restore original order
                    node.children = original_children.clone();
                }
            }
        });

    if let Some((show, idx, name, count)) = show_delete_modal {
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

    // Execute delete if requested
    if let Some(idx) = delete_request {
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

    // Execute move if requested
    if let Some((from_idx, target_folder_idx)) = move_request {
        // Find the original item by name since indices might have changed
        let moved_item = original_children[from_idx].clone();

        // Find the target folder in original children
        let target_folder = &original_children.clone()[target_folder_idx];
        let target_path = target_folder.path.join(&moved_item.name);

        // Perform actual filesystem move
        match std::fs::rename(&moved_item.path, &target_path) {
            Ok(_) => {
                // Successfully moved - restore original order and refresh
                node.children = original_children;

                // Remove the moved item
                node.children.retain(|child| child.path != moved_item.path);

                // Refresh the target folder to show the new item
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
                // Failed to move - restore original order
                node.children = original_children;
                eprintln!("Failed to move file: {}", e);
            }
        }
    }

    nav_request
}
