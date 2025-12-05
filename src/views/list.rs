use crate::file_system::{file_tree::FileNode, formatting, operations};
use crate::ui::settings::Settings;
use crate::utils::{drag_drop, sorting};
use crate::views::common;
use eframe::egui;

pub fn render_list_view(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    settings: &Settings,
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
    let original_children = node.children.clone();
    let mut sorted_indices = sorting::get_sorted_indices(node, settings);

    egui::ScrollArea::vertical()
        .max_width(ui.available_width())
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(32.0, 16.0);

            let mut dragged_idx: Option<usize> = None;
            let mut folder_rects: Vec<(usize, usize, egui::Rect)> = Vec::new();

            let sorted_indices_snapshot = sorted_indices.clone();

            let response = egui_dnd::dnd(ui, "file_explorer_dnd").show_vec(
                &mut sorted_indices,
                |ui, &mut child_idx, handle, state| {
                    let child = &node.children[child_idx];
                    let is_folder = child.is_dir;

                    if state.dragged {
                        dragged_idx = Some(state.index);
                    }

                    ui.vertical(|ui| {
                        handle.ui(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width() - 3.0, 30.0),
                                egui::Sense::click(),
                            );

                            if is_folder {
                                let original_idx = sorted_indices_snapshot[state.index];
                                folder_rects.push((state.index, original_idx, rect));
                            }

                            let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                            let is_drag_active = ui.input(|i| i.pointer.is_decidedly_dragging());
                            let is_drop_target = is_folder
                                && is_drag_active
                                && !state.dragged
                                && Some(state.index) != dragged_idx
                                && pointer_pos.map_or(false, |pos| rect.contains(pos));

                            common::draw_item_feedback(
                                ui,
                                rect,
                                state.dragged,
                                is_drop_target,
                                resp.hovered(),
                                is_drag_active,
                            );

                            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                ui.horizontal(|ui| {
                                    let icon = formatting::get_file_icon(&child.name, child.is_dir);
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
                                                        egui::RichText::new(
                                                            formatting::format_file_size(size),
                                                        )
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

                            if resp.double_clicked() && is_folder && !is_drag_active {
                                nav_request = Some(child.path.clone());
                            }

                            resp.context_menu(|ui| {
                                if let Some(idx) = operations::show_context_menu(
                                    ui,
                                    child,
                                    child_idx,
                                    confirm_delete_id,
                                ) {
                                    delete_request = Some(idx);
                                }
                            });
                        });
                    });
                },
            );

            if let Some(update) = response.final_update() {
                if let Some(move_req) =
                    drag_drop::handle_drop(ui, &update, &sorted_indices_snapshot, &folder_rects)
                {
                    move_request = Some(move_req);
                } else {
                    node.children = original_children.clone();
                }
            }
        });

    // Handle delete confirmation modal
    if let Some(idx) = operations::show_delete_confirmation_modal(ui, confirm_delete_id) {
        delete_request = Some(idx);
    }

    // Execute operations
    if let Some(idx) = delete_request {
        operations::execute_delete(node, idx);
    }

    if let Some((from_idx, target_folder_idx)) = move_request {
        operations::execute_move(node, from_idx, target_folder_idx, original_children);
    }

    nav_request
}
