//! Show a custom window frame instead of the default OS window chrome decorations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, ViewportCommand};

mod file_tree;
mod settings;

// For some reason it doesnt want to show a window if you run it in home directory.
// TODO: Fixes
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let initial_path = if args.len() > 1 {
        let path_str = if args[1].starts_with("~/") {
            args[1].replacen(
                "~",
                &std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
                1,
            )
        } else if args[1] == "~" {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        } else {
            args[1].clone()
        };

        let path = std::path::PathBuf::from(&path_str);

        if path.exists() {
            if path.is_file() {
                path.parent().unwrap_or(&path).to_path_buf()
            } else {
                path
            }
        } else {
            eprintln!(
                "Warning: Path '{}' does not exist. Using current directory.",
                args[1]
            );
            std::env::current_dir().unwrap_or_else(|_| "./".into())
        }
    } else {
        std::env::current_dir().unwrap_or_else(|_| "./".into())
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false) // Hide the OS-specific "chrome" around the window
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([800.0, 600.0])
            .with_transparent(true), // To have rounded corners we need transparency

        ..Default::default()
    };
    eframe::run_native(
        "Rustplorer", // unused title
        options,
        Box::new(move |cc| Ok(Box::new(MyApp::new(cc, initial_path)))),
    )
}

struct MyApp {
    current_root: std::path::PathBuf,
    history: Vec<std::path::PathBuf>,
    history_index: usize,
    file_tree: file_tree::FileNode,
    settings: settings::Settings,
    show_settings: bool,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext, initial_path: std::path::PathBuf) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        egui_nerdfonts::add_to_fonts(&mut fonts, egui_nerdfonts::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        let tree = file_tree::build_file_tree(&initial_path);

        Self {
            current_root: initial_path.clone(),
            history: vec![initial_path.clone()],
            history_index: 0,
            file_tree: tree,
            settings: settings::Settings::default(),
            show_settings: false,
        }
    }

    fn go_to_directory(&mut self, path: &std::path::Path) {
        if path.is_dir() && path != self.current_root.as_path() {
            // Add to history (only if not going back in history)
            if self.history_index < self.history.len().saturating_sub(1) {
                self.history.truncate(self.history_index + 1);
            }

            self.history.push(path.to_path_buf());
            self.history_index += 1;

            self.current_root = path.to_path_buf();
            self.file_tree = file_tree::build_file_tree(path);
        }
    }

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let prev_path = &self.history[self.history_index];
            self.current_root = prev_path.clone();
            self.file_tree = file_tree::build_file_tree(prev_path);
        }
    }

    fn can_go_back(&self) -> bool {
        self.history_index > 0
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.settings.apply_theme(ctx);

        if self.show_settings {
            self.settings.ui(ctx, &mut self.show_settings);
        }

        let mut show_settings_toggle = false;

        custom_window_frame(ctx, "Rustplorer", &mut show_settings_toggle, |ui| {
            egui::SidePanel::left("Favorites")
                .resizable(true)
                .default_width(150.0)
                .width_range(90.0..=200.0)
                .show_inside(ui, |ui| {
                    ui.heading("Favorites");
                    ui.separator();
                    ui.label("TODO!");
                });

            // Top navigation bar
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

                    // Show current path
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

            // Main file tree area
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if let Some(target) =
                            file_tree::render_file_node(ui, &mut self.file_tree, &self.settings)
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

fn custom_window_frame(
    ctx: &egui::Context,
    title: &str,
    show_settings_toggle: &mut bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::{CentralPanel, UiBuilder};

    let panel_frame = egui::Frame::new()
        .fill(ctx.style().visuals.window_fill())
        .corner_radius(10)
        .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
        .outer_margin(1); // so the stroke is within the bounds

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, title_bar_rect, title, show_settings_toggle);

        // Add the contents:
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    title_bar_rect: eframe::epaint::Rect,
    title: &str,
    show_settings_toggle: &mut bool,
) {
    use egui::{Align2, FontId, Id, PointerButton, Sense, UiBuilder, vec2};

    let painter = ui.painter();

    let title_bar_response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    }

    if title_bar_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);

            let settings_response = ui
                .add(egui::Button::new(
                    egui::RichText::new(format!("{}", egui_nerdfonts::regular::GEAR_1)).size(18.0),
                ))
                .on_hover_text("Settings");

            if settings_response.clicked() {
                *show_settings_toggle = true;
            }
        },
    );

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui);
        },
    );
}

/// Show some close/maximize/minimize buttons for the native window.
fn close_maximize_minimize(ui: &mut egui::Ui) {
    use egui::{Button, RichText};

    let button_height = 18.0;

    let close_response = ui
        .add(Button::new(
            RichText::new(format!("{}", egui_phosphor::regular::X)).size(button_height),
        ))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
        let maximized_response = ui
            .add(Button::new(
                RichText::new(format!("{}", egui_phosphor::regular::CORNERS_IN))
                    .size(button_height),
            ))
            .on_hover_text("Restore window");
        if maximized_response.clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(false));
        }
    } else {
        let maximized_response = ui
            .add(Button::new(
                RichText::new(format!("{}", egui_phosphor::regular::CORNERS_OUT))
                    .size(button_height),
            ))
            .on_hover_text("Maximize window");
        if maximized_response.clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
        }
    }

    let minimized_response = ui
        .add(Button::new(
            RichText::new(format!("{}", egui_phosphor::regular::CARET_DOWN)).size(button_height),
        ))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}
