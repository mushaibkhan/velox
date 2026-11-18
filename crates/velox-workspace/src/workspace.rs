use crate::tree::FileTreeNode;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Workspace {
    pub root_path: Option<PathBuf>,
    pub file_tree: Option<FileTreeNode>,
    pub file_cache: Vec<PathBuf>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            root_path: None,
            file_tree: None,
            file_cache: Vec::new(),
        }
    }

    pub fn open(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        self.root_path = Some(path.clone());
        self.refresh();
    }

    pub fn refresh(&mut self) {
        if let Some(root) = &self.root_path {
            let mut tree = FileTreeNode::build(root, 6);
            if let Some(t) = &mut tree {
                t.is_expanded = true;
            }
            self.file_tree = tree;

            // Rebuild indexed file cache for Quick Open
            let mut files = Vec::new();
            for entry in WalkDir::new(root)
                .max_depth(8)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && name != "target" && name != "node_modules"
                })
                .flatten()
            {
                if entry.file_type().is_file() {
                    files.push(entry.into_path());
                }
            }
            self.file_cache = files;
        }
    }

    pub fn toggle_folder(&mut self, path: &Path) {
        if let Some(tree) = &mut self.file_tree {
            tree.toggle_path(path);
        }
    }

    pub fn root_name(&self) -> String {
        self.root_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "No Folder Opened".to_string())
    }
}
