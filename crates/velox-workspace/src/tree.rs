use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<FileTreeNode>,
}

impl FileTreeNode {
    pub fn build(path: impl AsRef<Path>, max_depth: usize) -> Option<Self> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = path.is_dir();
        let mut children = Vec::new();

        if is_dir && max_depth > 0 {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut dir_entries = Vec::new();
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    let file_name = entry_path.file_name().and_then(|f| f.to_str()).unwrap_or("");

                    // Skip ignored directories
                    if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                        continue;
                    }

                    if let Some(child_node) = FileTreeNode::build(&entry_path, max_depth - 1) {
                        dir_entries.push(child_node);
                    }
                }

                // Sort: folders first, then alphabetically
                dir_entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                });

                children = dir_entries;
            }
        }

        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            is_expanded: false,
            children,
        })
    }

    pub fn toggle_path(&mut self, target: &Path) -> bool {
        if self.path == target {
            if self.is_dir {
                self.is_expanded = !self.is_expanded;
            }
            return true;
        }

        for child in &mut self.children {
            if child.toggle_path(target) {
                return true;
            }
        }

        false
    }

    pub fn flatten_visible(&self) -> Vec<(usize, &FileTreeNode)> {
        let mut result = Vec::new();
        self.collect_visible(0, &mut result);
        result
    }

    fn collect_visible<'a>(&'a self, depth: usize, out: &mut Vec<(usize, &'a FileTreeNode)>) {
        out.push((depth, self));
        if self.is_dir && self.is_expanded {
            for child in &self.children {
                child.collect_visible(depth + 1, out);
            }
        }
    }
}
