#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    fn new(path: &std::path::Path) -> Self {
        let name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let is_dir = path.is_dir();
        let children = if is_dir {
            std::fs::read_dir(path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| FileNode::new(&entry.path()))
                .collect()
        } else {
            Vec::new()
        };

        Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            children,
        }
    }

    pub fn refresh_children(&mut self) {
        if self.is_dir {
            self.children = std::fs::read_dir(&self.path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| FileNode::new(&entry.path()))
                .collect();
        }
    }
}

pub fn build_file_tree(root: &std::path::Path) -> FileNode {
    FileNode::new(root)
}
