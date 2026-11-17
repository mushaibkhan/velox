use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use rustforge_app::RustForgeApp;
use rustforge_git::FileChangeType;

fn init_temp_git_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let repo = git2::Repository::init(temp_dir.path()).expect("failed to init git repo");
    let mut config = repo.config().expect("failed to get config");
    config.set_str("user.name", "Test User").expect("set user.name");
    config.set_str("user.email", "test@example.com").expect("set user.email");

    // Create an initial commit so HEAD exists
    let initial_file = temp_dir.path().join("README.md");
    fs::write(&initial_file, "# Test Repo\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    temp_dir
}

#[test]
fn test_git_repo_discovery_and_initialization() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();

    // Initially no repo is loaded
    assert!(app.git_repo.is_none());
    assert!(app.git_status.is_none());

    // Open workspace in git repo
    app.open_workspace(temp_dir.path());

    assert!(app.git_repo.is_some());
    assert!(app.git_status.is_some());
    assert!(app.git_status.as_ref().unwrap().is_clean());
    assert!(!app.git_branch.is_empty());
}

#[test]
fn test_staging_and_unstaging_files() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    // Create a new untracked file
    let foo_path = temp_dir.path().join("foo.txt");
    fs::write(&foo_path, "hello world\n").unwrap();

    app.refresh_git();
    let status = app.git_status.as_ref().unwrap();
    assert_eq!(status.untracked, vec![PathBuf::from("foo.txt")]);
    assert!(status.staged.is_empty());

    // Stage the file
    app.stage_file(&foo_path);
    let status = app.git_status.as_ref().unwrap();
    assert!(status.untracked.is_empty());
    assert_eq!(status.staged.len(), 1);
    assert_eq!(status.staged[0].path, PathBuf::from("foo.txt"));
    assert_eq!(status.staged[0].status, FileChangeType::Added);

    // Unstage the file
    app.unstage_file(&foo_path);
    let status = app.git_status.as_ref().unwrap();
    assert!(status.staged.is_empty());
    assert_eq!(status.untracked, vec![PathBuf::from("foo.txt")]);
}

#[test]
fn test_stage_all() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    // Create two new files and modify README
    fs::write(temp_dir.path().join("a.txt"), "a\n").unwrap();
    fs::write(temp_dir.path().join("b.txt"), "b\n").unwrap();
    fs::write(temp_dir.path().join("README.md"), "# Modified\n").unwrap();

    app.refresh_git();
    assert_eq!(app.git_status.as_ref().unwrap().untracked.len(), 2);
    assert_eq!(app.git_status.as_ref().unwrap().unstaged.len(), 1);

    app.stage_all();

    let status = app.git_status.as_ref().unwrap();
    assert!(status.untracked.is_empty());
    assert!(status.unstaged.is_empty());
    assert_eq!(status.staged.len(), 3);
}

#[test]
fn test_committing_changes() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    // Create and stage a new file
    let new_file = temp_dir.path().join("commit_test.txt");
    fs::write(&new_file, "commit me\n").unwrap();
    app.stage_file(&new_file);

    assert_eq!(app.git_status.as_ref().unwrap().staged.len(), 1);

    // Attempting commit with empty message fails
    app.git_commit_message.clear();
    let err = app.commit();
    assert!(err.is_err());

    // Commit with valid message succeeds
    app.git_commit_message = "feat: add commit_test.txt".to_string();
    let commit_res = app.commit();
    assert!(commit_res.is_ok());

    // Commit message should be cleared and repo should be clean
    assert!(app.git_commit_message.is_empty());
    assert!(app.git_status.as_ref().unwrap().is_clean());
}

#[test]
fn test_discarding_changes() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    let readme_path = temp_dir.path().join("README.md");
    // Open README in an editor tab
    app.open_file(&readme_path);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.tabs[0].editor.buffer.rope().to_string(), "# Test Repo\n");

    // Modify file on disk
    fs::write(&readme_path, "# Corrupted content\n").unwrap();
    // Simulate reloading tab to reflect change
    app.tabs[0].reload_from_disk().unwrap();
    assert_eq!(app.tabs[0].editor.buffer.rope().to_string(), "# Corrupted content\n");

    app.refresh_git();
    assert_eq!(app.git_status.as_ref().unwrap().unstaged.len(), 1);

    // Discard unstaged changes
    app.discard_file(&readme_path);

    // File on disk should be restored
    let disk_content = fs::read_to_string(&readme_path).unwrap();
    assert_eq!(disk_content, "# Test Repo\n");

    // Open tab should also be reloaded
    assert_eq!(app.tabs[0].editor.buffer.rope().to_string(), "# Test Repo\n");
    assert!(app.git_status.as_ref().unwrap().is_clean());

    // Test discarding untracked file deletes it
    let untracked = temp_dir.path().join("temp_delete.txt");
    fs::write(&untracked, "to delete\n").unwrap();
    app.refresh_git();
    assert_eq!(app.git_status.as_ref().unwrap().untracked.len(), 1);

    app.discard_file(&untracked);
    assert!(!untracked.exists());
    assert!(app.git_status.as_ref().unwrap().is_clean());
}

#[test]
fn test_open_diff_tab() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    let readme_path = temp_dir.path().join("README.md");
    fs::write(&readme_path, "# Test Repo\nNew Line Added\n").unwrap();

    app.refresh_git();
    assert_eq!(app.git_status.as_ref().unwrap().unstaged.len(), 1);

    // Open diff tab
    app.open_diff_tab(&readme_path);

    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.active_tab_idx, Some(0));

    let tab = &app.tabs[0];
    assert!(tab.diff_view.is_some());
    assert_eq!(tab.title, "Diff: README.md");

    let diff_state = tab.diff_view.as_ref().unwrap();
    assert_eq!(diff_state.diff.additions, 1);
    assert_eq!(diff_state.old_label, "HEAD:README.md");
    assert_eq!(diff_state.new_label, "Working Tree:README.md");

    // Opening diff tab for the same file should switch to it without adding duplicate tab
    app.open_diff_tab(&readme_path);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.active_tab_idx, Some(0));
}

#[test]
fn test_editor_tab_git_gutter_diff_updated() {
    let temp_dir = init_temp_git_repo();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    let readme_path = temp_dir.path().join("README.md");
    app.open_file(&readme_path);

    assert_eq!(app.tabs.len(), 1);
    let editor_diff = app.tabs[0].editor.git_diff.as_ref();
    assert!(editor_diff.is_some());
    assert!(editor_diff.unwrap().is_empty());

    // Modify file
    fs::write(&readme_path, "# Test Repo\nAdded line\n").unwrap();
    app.refresh_git();

    let updated_diff = app.tabs[0].editor.git_diff.as_ref().unwrap();
    assert_eq!(updated_diff.additions, 1);
}

#[test]
fn test_non_git_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let mut app = RustForgeApp::new_test_app();
    app.open_workspace(temp_dir.path());

    assert!(app.git_repo.is_none());
    assert!(app.git_status.is_none());
    assert!(app.git_branch.is_empty());
    assert_eq!(app.git_ahead_behind, (0, 0));

    app.git_commit_message = "Should fail".to_string();
    let err = app.commit();
    assert!(err.is_err());
    assert_eq!(err.unwrap_err(), "Not a git repository");
}
