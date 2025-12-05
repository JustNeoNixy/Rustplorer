use crate::file_system::file_tree::FileNode;
use crate::ui::settings::Settings;

// Get sorted indices based on settings
pub fn get_sorted_indices_for_vec(children: &[FileNode], settings: &Settings) -> Vec<usize> {
    let mut folder_indices: Vec<usize> = Vec::new();
    let mut file_indices: Vec<usize> = Vec::new();

    for (idx, child) in children.iter().enumerate() {
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
        let sort_fn = |&a: &usize, &b: &usize| {
            children[a]
                .name
                .to_lowercase()
                .cmp(&children[b].name.to_lowercase())
        };
        folder_indices.sort_by(sort_fn);
        file_indices.sort_by(sort_fn);
    }

    if settings.sort_folders_first {
        folder_indices.into_iter().chain(file_indices).collect()
    } else {
        let mut all = folder_indices
            .into_iter()
            .chain(file_indices)
            .collect::<Vec<_>>();
        if settings.sort_items {
            all.sort_by(|&a, &b| {
                children[a]
                    .name
                    .to_lowercase()
                    .cmp(&children[b].name.to_lowercase())
            });
        }
        all
    }
}
