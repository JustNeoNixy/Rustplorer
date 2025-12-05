use eframe::egui;

use crate::file_system::file_tree;
use crate::ui::{settings::Settings, window};
use crate::views;

pub struct MyApp {
    current_root: std::path::PathBuf,
    history: Vec<std::path::PathBuf>,
    history_index: usize,
    file_tree: file_tree::FileNode,
    settings: Settings,
    show_settings: bool,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext, initial_path: std::path::PathBuf) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        egui_nerdfonts::add_to_fonts(&mut fonts, egui_nerdfonts::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        let mut tree = file_tree::FileNode::new(&initial_path);
        tree.ensure_children_loaded();

        Self {
            current_root: initial_path.clone(),
            history: vec![initial_path.clone()],
            history_index: 0,
            file_tree: tree,
            settings: Settings::default(),
            show_settings: false,
        }
    }

    fn go_to_directory(&mut self, path: &std::path::Path) {
        if path.is_dir() && path != self.current_root.as_path() {
            if self.history_index < self.history.len().saturating_sub(1) {
                self.history.truncate(self.history_index + 1);
            }

            self.history.push(path.to_path_buf());
            self.history_index += 1;

            self.current_root = path.to_path_buf();

            let mut node = file_tree::FileNode::new(path);
            node.ensure_children_loaded();
            self.file_tree = node;
        }
    }

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let prev_path = &self.history[self.history_index];
            self.current_root = prev_path.clone();
            let mut node = file_tree::FileNode::new(prev_path);
            node.ensure_children_loaded();
            self.file_tree = node;
        }
    }

    fn can_go_back(&self) -> bool {
        self.history_index > 0
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.settings.apply_theme(ctx);

        if self.show_settings {
            self.settings.ui(ctx, &mut self.show_settings);
        }

        let mut show_settings_toggle = false;

        window::custom_window_frame(ctx, "Rustplorer", &mut show_settings_toggle, |ui| {
            egui::SidePanel::left("Favorites")
                .resizable(true)
                .default_width(150.0)
                .width_range(90.0..=200.0)
                .show_inside(ui, |ui| {
                    ui.heading("Favorites");
                    ui.separator();
                    ui.label("TODO!");
                });

            egui::TopBottomPanel::top("nav_bar").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let back_enabled = self.can_go_back();
                    let back_button = ui
                        .add_enabled(
                            back_enabled,
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{}",
                                    egui_phosphor::regular::ARROW_LEFT
                                ))
                                .size(20.0),
                            ),
                        )
                        .on_hover_text("Go back");

                    if back_button.clicked() {
                        self.go_back();
                    }

                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new(
                            self.current_root
                                .to_string_lossy()
                                .replace('\\', "/")
                                .replace(&format!("/home/{}", whoami::username()).to_string(), "~"),
                        )
                        .monospace()
                        .weak(),
                    );
                });
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if let Some(target) =
                            views::render_file_node(ui, &mut self.file_tree, &self.settings)
                        {
                            self.go_to_directory(&target);
                        }
                    });
            });
        });

        if show_settings_toggle {
            self.show_settings = !self.show_settings;
        }
    }
}
