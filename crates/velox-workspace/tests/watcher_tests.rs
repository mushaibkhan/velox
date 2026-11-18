use velox_workspace::{FileEvent, FileWatcher};
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_watcher_create_and_modify_events() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let root_path = temp_dir.path().canonicalize().unwrap_or_else(|_| temp_dir.path().to_path_buf());

    let (tx, rx) = mpsc::channel();
    let mut watcher = FileWatcher::new();
    watcher
        .start_watching(&root_path, move |event| {
            let _ = tx.send(event);
        })
        .expect("failed to start watching");

    assert!(watcher.is_watching());

    // 1. Create a file
    let file_path = root_path.join("sample.txt");
    fs::write(&file_path, "initial content").expect("failed to write initial file");

    let event = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("timed out waiting for create event");

    match event {
        FileEvent::Created(p) | FileEvent::Modified(p) => {
            assert_eq!(p, file_path);
        }
        FileEvent::Deleted(_) => panic!("unexpected delete event on creation"),
    }

    // Drain any remaining create events and pause
    std::thread::sleep(Duration::from_millis(200));
    while rx.try_recv().is_ok() {}

    // 2. Modify the file
    fs::write(&file_path, "modified content").expect("failed to update file");

    let event = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("timed out waiting for modify event");

    match event {
        FileEvent::Modified(p) | FileEvent::Created(p) => {
            assert_eq!(p, file_path);
        }
        other => panic!("expected FileEvent::Modified or Created, got {:?}", other),
    }

    // 3. Stop watcher
    watcher.stop_watching();
    assert!(!watcher.is_watching());
}

#[test]
fn test_watcher_ignores_git_target_node_modules() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let root_path = temp_dir.path().canonicalize().unwrap_or_else(|_| temp_dir.path().to_path_buf());

    let git_dir = root_path.join(".git");
    let target_dir = root_path.join("target");
    let node_modules_dir = root_path.join("node_modules");

    fs::create_dir_all(&git_dir).unwrap();
    fs::create_dir_all(&target_dir).unwrap();
    fs::create_dir_all(&node_modules_dir).unwrap();

    let (tx, rx) = mpsc::channel();
    let mut watcher = FileWatcher::new();
    watcher
        .start_watching(&root_path, move |event| {
            let _ = tx.send(event);
        })
        .expect("failed to start watching");

    // Write files in ignored folders
    fs::write(git_dir.join("config"), "git config").unwrap();
    fs::write(target_dir.join("build.log"), "build output").unwrap();
    fs::write(node_modules_dir.join("pkg.json"), "{}").unwrap();

    // Also write a valid file to know watcher is alive
    std::thread::sleep(Duration::from_millis(50));
    let valid_file = root_path.join("valid.txt");
    fs::write(&valid_file, "valid content").unwrap();

    let event = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("timed out waiting for valid file event");

    // The ONLY event received should be for valid.txt
    assert_eq!(event.path(), &valid_file);

    // No other events should be pending
    std::thread::sleep(Duration::from_millis(150));
    assert!(rx.try_recv().is_err());
}

#[test]
fn test_watcher_deletion_event() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let root_path = temp_dir.path().canonicalize().unwrap_or_else(|_| temp_dir.path().to_path_buf());

    let file_path = root_path.join("to_delete.txt");
    fs::write(&file_path, "will be deleted").expect("failed to write initial file");

    let (tx, rx) = mpsc::channel();
    let mut watcher = FileWatcher::new();
    watcher
        .start_watching(&root_path, move |event| {
            let _ = tx.send(event);
        })
        .expect("failed to start watching");

    // Drain initial create/modify event if any arrived
    std::thread::sleep(Duration::from_millis(300));
    while rx.try_recv().is_ok() {}

    // Delete the file
    fs::remove_file(&file_path).expect("failed to delete file");

    let mut received_deleted = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        if let Ok(FileEvent::Deleted(p)) = rx.recv_timeout(Duration::from_millis(500)) {
            assert_eq!(p, file_path);
            received_deleted = true;
            break;
        }
    }
    assert!(received_deleted, "expected FileEvent::Deleted");
}
