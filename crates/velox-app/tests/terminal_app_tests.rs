use velox_app::app::VeloxApp;
use velox_terminal::{SplitMode, TerminalSession};

#[test]
fn test_app_terminal_manager_initialization() {
    let app = VeloxApp::new_test_app();
    assert_eq!(app.terminal_manager.session_count(), 0);
    assert_eq!(app.focused_terminal_idx, None);
    assert!(app.terminal().is_none());
}

#[test]
fn test_app_new_and_close_terminal_session() {
    let mut app = VeloxApp::new_test_app();

    // Spawn a session
    let s1 = TerminalSession::spawn(80, 24, None, None).expect("failed to spawn");
    let s1_id = s1.id;
    let idx1 = app.terminal_manager.add_session(s1);
    app.focused_terminal_idx = Some(idx1);

    assert_eq!(app.terminal_manager.session_count(), 1);
    assert_eq!(app.focused_terminal_idx, Some(0));
    assert_eq!(app.terminal().map(|s| s.id), Some(s1_id));

    // Spawn a second session
    let s2 = TerminalSession::spawn(80, 24, None, None).expect("failed to spawn");
    let s2_id = s2.id;
    let idx2 = app.terminal_manager.add_session(s2);
    app.focused_terminal_idx = Some(idx2);

    assert_eq!(app.terminal_manager.session_count(), 2);
    assert_eq!(app.focused_terminal_idx, Some(1));
    assert_eq!(app.terminal().map(|s| s.id), Some(s2_id));

    // Close session 1
    app.close_terminal_session(1);
    assert_eq!(app.terminal_manager.session_count(), 1);
    assert_eq!(app.focused_terminal_idx, Some(0));
    assert_eq!(app.terminal().map(|s| s.id), Some(s1_id));

    // Close remaining session 0
    app.close_terminal_session(0);
    assert_eq!(app.terminal_manager.session_count(), 0);
    assert_eq!(app.focused_terminal_idx, None);
    assert!(app.terminal().is_none());
}

#[test]
fn test_app_terminal_split_mode() {
    let mut app = VeloxApp::new_test_app();

    let s1 = TerminalSession::spawn(80, 24, None, None).expect("failed to spawn");
    let s2 = TerminalSession::spawn(80, 24, None, None).expect("failed to spawn");

    app.terminal_manager.add_session(s1);
    app.terminal_manager.add_session(s2);
    app.terminal_manager.set_active_idx(0);

    assert_eq!(app.terminal_manager.split_mode(), SplitMode::None);

    app.terminal_manager.toggle_split();
    assert_eq!(app.terminal_manager.split_mode(), SplitMode::Vertical);
    assert_eq!(app.terminal_manager.split_idx(), Some(1));

    app.terminal_manager.toggle_split();
    assert_eq!(app.terminal_manager.split_mode(), SplitMode::Horizontal);

    app.terminal_manager.toggle_split();
    assert_eq!(app.terminal_manager.split_mode(), SplitMode::None);
    assert_eq!(app.terminal_manager.split_idx(), None);
}
