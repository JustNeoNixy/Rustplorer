#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

impl FileNode {
    pub fn new(path: &std::path::Path) -> Self {
        let name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let is_dir = path.is_dir();
        let children = if is_dir { None } else { Some(Vec::new()) };

        Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            children,
        }
    }

    pub fn ensure_children_loaded(&mut self) {
        if self.is_dir && self.children.is_none() {
            let loaded_children = std::fs::read_dir(&self.path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| FileNode::new(&entry.path()))
                .collect::<Vec<_>>();

            let mut dirs = Vec::new();
            let mut files = Vec::new();
            for child in loaded_children {
                if child.is_dir {
                    dirs.push(child);
                } else {
                    files.push(child);
                }
            }
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            dirs.extend(files);

            self.children = Some(dirs);
        }
    }

    pub fn _refresh_children(&mut self) {
        if self.is_dir {
            self.children = None;
            self.ensure_children_loaded();
        }
    }
}

pub fn _build_file_tree(root: &std::path::Path) -> FileNode {
    let mut node = FileNode::new(root);
    node.ensure_children_loaded();
    node
}
