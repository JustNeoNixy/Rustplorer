pub fn format_file_size(bytes: u64) -> String {
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

pub fn get_file_icon(filename: &str, is_folder: bool) -> &'static str {
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
