pub mod tree;
pub mod watcher;
pub mod workspace;

pub use tree::FileTreeNode;
pub use watcher::{FileEvent, FileWatcher, WatcherError};
pub use workspace::Workspace;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_workspace_build() {
        let temp_dir = std::env::temp_dir().join("velox_ws_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("main.rs"), "fn main() {}").unwrap();

        let mut ws = Workspace::new();
        ws.open(&temp_dir);

        assert!(ws.file_tree.is_some());
        assert!(!ws.file_cache.is_empty());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
