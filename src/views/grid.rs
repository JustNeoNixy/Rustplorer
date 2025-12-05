use crate::file_system::{file_tree::FileNode, formatting::get_file_icon, operations};
use crate::ui::settings::Settings;
use crate::utils::{drag_drop, sorting};
use crate::views::common;
use eframe::egui;

pub fn render_grid_view(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    settings: &Settings,
) -> Option<std::path::PathBuf> {
    let mut nav_request = None;
    let mut move_request: Option<(usize, usize)> = None;
    let mut delete_request: Option<usize> = None;

    let confirm_delete_id = ui.id().with("confirm_delete");

    node.ensure_children_loaded();
    let children = node.children.as_mut().unwrap();

    let original_children = children.clone();
    let mut sorted_indices = sorting::get_sorted_indices_for_vec(children, settings);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(20.0, 20.0);

            let mut dragged_idx: Option<usize> = None;
            let mut folder_rects: Vec<(usize, usize, egui::Rect)> = Vec::new();

            let av_width = ui.available_width() - ui.spacing().item_spacing.x;
            let columns = (av_width / 100.0).ceil() as usize;
            let width = av_width / columns as f32;
            let size = egui::Vec2::new(width, width) + ui.spacing().item_spacing;

            let sorted_indices_snapshot = sorted_indices.clone();

            let response = egui_dnd::dnd(ui, "file_explorer_dnd").show_vec_sized(
                &mut sorted_indices,
                size,
                |ui, &mut child_idx, handle, state| {
                    let child = &children[child_idx];
                    let is_folder = child.is_dir;
                    let icon = get_file_icon(&child.name, child.is_dir);

                    if state.dragged {
                        dragged_idx = Some(state.index);
                    }

                    ui.vertical(|ui| {
                        handle.ui(ui, |ui| {
                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(120.0, 80.0), egui::Sense::click());

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

                            let name_pos = rect.center() + egui::vec2(0.0, 20.0);
                            ui.painter().text(
                                name_pos,
                                egui::Align2::CENTER_TOP,
                                &child.name,
                                egui::FontId::proportional(14.0),
                                icon_color,
                            );

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
                    *children = original_children.clone();
                }
            }
        });
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
