use eframe::egui;

#[derive(Debug, Clone)]
pub struct Settings {
    pub show_hidden_files: bool,
    pub sort_folders_first: bool,
    pub sort_items: bool,
    pub theme: Theme,
    pub view: View,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Grid,
    List,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_hidden_files: false,
            sort_folders_first: true,
            sort_items: true,
            theme: Theme::System,
            view: View::List,
        }
    }
}

impl Settings {
    pub fn ui(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new("Settings")
            .open(open)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("General");
                ui.separator();

                ui.add_space(8.0);

                ui.checkbox(&mut self.show_hidden_files, "Show hidden files");
                ui.add_space(4.0);
                ui.checkbox(&mut self.sort_folders_first, "Sort folders first");
                ui.add_space(4.0);
                ui.checkbox(&mut self.sort_items, "Sort items");

                ui.add_space(16.0);

                ui.heading("Appearence");
                ui.separator();

                ui.add_space(8.0);

                ui.label("Theme:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.theme, Theme::Light, "Light");
                    ui.radio_value(&mut self.theme, Theme::Dark, "Dark");
                    ui.radio_value(&mut self.theme, Theme::System, "System");
                });
                ui.add_space(4.0);
                ui.label("View:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.view, View::Grid, "Grid");
                    ui.radio_value(&mut self.view, View::List, "List");
                });

                ui.add_space(16.0);

                ui.separator();
                ui.add_space(8.0);

                if ui.button("Reset to defaults").clicked() {
                    *self = Settings::default();
                }
            });
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        match self.theme {
            Theme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
            Theme::System => {
                // default
            }
        }
    }
}
