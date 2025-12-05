#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

mod app;
mod file_system;
mod ui;
mod utils;
mod views;

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
        viewport: eframe::egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([800.0, 600.0])
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "Rustplorer",
        options,
        Box::new(move |cc| Ok(Box::new(app::MyApp::new(cc, initial_path)))),
    )
}
