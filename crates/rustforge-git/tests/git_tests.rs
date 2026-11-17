use std::fs;
use std::path::Path;
use tempfile::TempDir;

use rustforge_git::{
    compute_diff, FileChangeType, GitRepository, LineChange,
};

fn init_temp_repo() -> (TempDir, GitRepository) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let repo = git2::Repository::init(temp_dir.path()).expect("failed to init git2 repo");
    // Set dummy git config for user
    let mut config = repo.config().expect("failed to get config");
    config.set_str("user.name", "Test User").expect("set user.name");
    config.set_str("user.email", "test@example.com").expect("set user.email");

    let git_repo = GitRepository::open(temp_dir.path()).expect("failed to open GitRepository");
    (temp_dir, git_repo)
}

#[test]
fn test_repo_discovery() {
    let (temp_dir, _repo) = init_temp_repo();
    let root = temp_dir.path();

    // Direct path discovery
    let discovered = GitRepository::discover(root);
    assert!(discovered.is_some());

    // Subdirectory discovery
    let sub = root.join("nested").join("subfolder");
    fs::create_dir_all(&sub).unwrap();
    let discovered_sub = GitRepository::discover(&sub);
    assert!(discovered_sub.is_some());

    // Non-git path discovery
    let non_git = TempDir::new().unwrap();
    assert!(GitRepository::discover(non_git.path()).is_none());
}

#[test]
fn test_status_staging_and_commit() {
    let (temp_dir, git_repo) = init_temp_repo();
    let root = temp_dir.path();

    // Create untracked file
    let file_path = root.join("hello.txt");
    fs::write(&file_path, "Hello, RustForge!\n").unwrap();

    let status = git_repo.status().expect("status call failed");
    assert_eq!(status.untracked, vec![std::path::PathBuf::from("hello.txt")]);
    assert!(status.staged.is_empty());
    assert!(status.unstaged.is_empty());

    // Stage file
    git_repo.stage_file(&file_path).expect("stage failed");
    let status_after_stage = git_repo.status().unwrap();
    assert!(status_after_stage.untracked.is_empty());
    assert_eq!(status_after_stage.staged.len(), 1);
    assert_eq!(status_after_stage.staged[0].path, std::path::PathBuf::from("hello.txt"));
    assert_eq!(status_after_stage.staged[0].status, FileChangeType::Added);

    // Unstage file
    git_repo.unstage_file(&file_path).expect("unstage failed");
    let status_after_unstage = git_repo.status().unwrap();
    assert_eq!(status_after_unstage.untracked, vec![std::path::PathBuf::from("hello.txt")]);
    assert!(status_after_unstage.staged.is_empty());

    // Stage again and commit
    git_repo.stage_file(&file_path).unwrap();
    let commit_oid = git_repo
        .commit("Initial commit", "Test User", "test@example.com")
        .expect("commit failed");
    assert!(!commit_oid.is_zero());

    let status_after_commit = git_repo.status().unwrap();
    assert!(status_after_commit.is_clean());

    // Modify file
    fs::write(&file_path, "Hello, RustForge!\nLine 2 added.\n").unwrap();
    let status_modified = git_repo.status().unwrap();
    assert_eq!(status_modified.unstaged.len(), 1);
    assert_eq!(status_modified.unstaged[0].path, std::path::PathBuf::from("hello.txt"));
    assert_eq!(status_modified.unstaged[0].status, FileChangeType::Modified);

    // Stage modified file
    git_repo.stage_file(Path::new("hello.txt")).unwrap();
    let status_mod_staged = git_repo.status().unwrap();
    assert_eq!(status_mod_staged.staged.len(), 1);
    assert_eq!(status_mod_staged.staged[0].status, FileChangeType::Modified);
    assert!(status_mod_staged.unstaged.is_empty());

    // Unstage modified file
    git_repo.unstage_file(Path::new("hello.txt")).unwrap();
    let status_mod_unstaged = git_repo.status().unwrap();
    assert!(status_mod_unstaged.staged.is_empty());
    assert_eq!(status_mod_unstaged.unstaged.len(), 1);
    assert_eq!(status_mod_unstaged.unstaged[0].status, FileChangeType::Modified);
}

#[test]
fn test_stage_all_and_discard() {
    let (temp_dir, git_repo) = init_temp_repo();
    let root = temp_dir.path();

    fs::write(root.join("f1.txt"), "File 1\n").unwrap();
    fs::write(root.join("f2.txt"), "File 2\n").unwrap();

    git_repo.stage_all().expect("stage all failed");
    let status = git_repo.status().unwrap();
    assert_eq!(status.staged.len(), 2);
    assert!(status.untracked.is_empty());

    git_repo.commit("Add files", "Test User", "test@example.com").unwrap();

    // Modify f1.txt
    fs::write(root.join("f1.txt"), "File 1 modified\n").unwrap();
    assert_eq!(git_repo.status().unwrap().unstaged.len(), 1);

    // Discard unstaged changes in f1.txt
    git_repo.discard_file(Path::new("f1.txt")).expect("discard failed");
    assert!(git_repo.status().unwrap().is_clean());
    let content = fs::read_to_string(root.join("f1.txt")).unwrap();
    assert_eq!(content, "File 1\n");
}

#[test]
fn test_branch_info_and_status() {
    let (temp_dir, git_repo) = init_temp_repo();
    let root = temp_dir.path();

    fs::write(root.join("init.txt"), "init\n").unwrap();
    git_repo.stage_all().unwrap();
    git_repo.commit("Initial", "Test", "test@example.com").unwrap();

    let branch = git_repo.current_branch().unwrap();
    assert!(!branch.is_empty());

    let branch_status = git_repo.branch_status().unwrap();
    assert_eq!(branch_status.ahead, 0);
    assert_eq!(branch_status.behind, 0);

    let branches = git_repo.branches().unwrap();
    assert!(!branches.is_empty());
    assert!(branches.iter().any(|b| b.is_head && b.name == branch));
}

#[test]
fn test_compute_diff_and_inline_spans() {
    let old_text = "fn main() {\n    let a = 1;\n    println!(\"hi\");\n}\n";
    let new_text = "fn main() {\n    let a = 2;\n    let b = 3;\n    println!(\"hi\");\n}\n";

    let diff = compute_diff(old_text, new_text);
    assert_eq!(diff.additions, 2); // 'let a = 2' replaced (1 add + 1 del) and 'let b = 3' added
    assert_eq!(diff.deletions, 1);
    assert!(!diff.hunks.is_empty());

    // Check line change mapping
    // Line 1: fn main() { -> Unchanged
    assert_eq!(diff.line_change_at(1), Some(LineChange::Unchanged));
    // Line 2: let a = 2 -> Modified
    assert_eq!(diff.line_change_at(2), Some(LineChange::Modified));
    // Line 3: let b = 3 -> Added
    assert_eq!(diff.line_change_at(3), Some(LineChange::Added));

    // Verify hunk has inline spans on modified line
    let hunk = &diff.hunks[0];
    let modified_line = hunk.lines.iter().find(|l| l.new_line_no == Some(2)).unwrap();
    assert_eq!(modified_line.change, LineChange::Modified);
    assert!(!modified_line.inline_spans.is_empty());
    let span = &modified_line.inline_spans[0];
    assert_eq!(&modified_line.content[span.start..span.end], "2;");
}

#[test]
fn test_blame_line() {
    let (temp_dir, git_repo) = init_temp_repo();
    let root = temp_dir.path();
    let file = root.join("code.rs");

    fs::write(&file, "line 1\nline 2\n").unwrap();
    git_repo.stage_all().unwrap();
    git_repo.commit("First commit", "Alice Author", "alice@example.com").unwrap();

    fs::write(&file, "line 1\nline 2 updated\n").unwrap();
    git_repo.stage_all().unwrap();
    git_repo.commit("Second commit", "Bob Builder", "bob@example.com").unwrap();

    let blame1 = git_repo.blame_line(Path::new("code.rs"), 1).expect("blame line 1 failed");
    assert_eq!(blame1.author, "Alice Author");
    assert_eq!(blame1.summary, "First commit");

    let blame2 = git_repo.blame_line(Path::new("code.rs"), 2).expect("blame line 2 failed");
    assert_eq!(blame2.author, "Bob Builder");
    assert_eq!(blame2.summary, "Second commit");

    // Blame line 0 or out of range returns None
    assert!(git_repo.blame_line(Path::new("code.rs"), 0).is_none());
    assert!(git_repo.blame_line(Path::new("code.rs"), 99).is_none());
}

#[test]
fn test_diff_file_against_head() {
    let (temp_dir, git_repo) = init_temp_repo();
    let root = temp_dir.path();
    let file = root.join("src").join("main.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();

    fs::write(&file, "fn main() {\n    // v1\n}\n").unwrap();
    git_repo.stage_all().unwrap();
    git_repo.commit("Init main.rs", "Dev", "dev@example.com").unwrap();

    // Modify file
    fs::write(&file, "fn main() {\n    // v2 modified\n    let x = 10;\n}\n").unwrap();

    let diff = git_repo.diff_file(Path::new("src/main.rs")).expect("diff_file failed");
    assert!(!diff.is_empty());
    assert_eq!(diff.additions, 2);
    assert_eq!(diff.deletions, 1);
    assert_eq!(diff.modifications, 1);
}

#[test]
fn test_diff_edge_cases() {
    // Identical texts
    let empty_diff = compute_diff("same\ntext\n", "same\ntext\n");
    assert!(empty_diff.is_empty());
    assert_eq!(empty_diff.line_change_at(1), Some(LineChange::Unchanged));
    assert_eq!(empty_diff.line_change_at(2), Some(LineChange::Unchanged));
    assert_eq!(empty_diff.line_change_at(3), None);

    // Empty texts
    let zero_diff = compute_diff("", "");
    assert!(zero_diff.is_empty());
    assert_eq!(zero_diff.line_change_at(1), None);

    // Completely new file
    let add_diff = compute_diff("", "line 1\nline 2\n");
    assert!(!add_diff.is_empty());
    assert_eq!(add_diff.additions, 2);
    assert_eq!(add_diff.deletions, 0);
    assert_eq!(add_diff.line_change_at(1), Some(LineChange::Added));
    assert_eq!(add_diff.line_change_at(2), Some(LineChange::Added));

    // Active changes map
    let active = add_diff.active_line_changes();
    assert_eq!(active.get(&1), Some(&LineChange::Added));
    assert_eq!(active.get(&2), Some(&LineChange::Added));

    // Deletion detection
    let del_diff = compute_diff("line 1\nline 2\nline 3\n", "line 1\nline 3\n");
    assert_eq!(del_diff.line_change_at(1), Some(LineChange::Unchanged));
    assert_eq!(del_diff.line_change_at(2), Some(LineChange::Deleted));
    assert_eq!(del_diff.line_change_at(3), None);
}

#[test]
fn test_format_relative_time() {
    use rustforge_git::format_relative_time;
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    assert_eq!(format_relative_time(now - 10), "just now");
    assert_eq!(format_relative_time(now - 120), "2 minutes ago");
    assert_eq!(format_relative_time(now - 7200), "2 hours ago");
    assert_eq!(format_relative_time(now - 172800), "2 days ago");
    assert_eq!(format_relative_time(now - 6000000), "2 months ago");
    assert_eq!(format_relative_time(now - 70000000), "2 years ago");
}

