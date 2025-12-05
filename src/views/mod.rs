pub mod common;
pub mod grid;
pub mod list;

use crate::file_system::file_tree::FileNode;
use crate::ui::settings::{Settings, View};
use eframe::egui;

pub fn render_file_node(
    ui: &mut egui::Ui,
    node: &mut FileNode,
    settings: &Settings,
) -> Option<std::path::PathBuf> {
    match settings.view {
        View::Grid => grid::render_grid_view(ui, node, settings),
        View::List => list::render_list_view(ui, node, settings),
    }
}
