use velox_terminal::{SplitMode, TerminalManager, TerminalSession};

#[test]
fn test_terminal_manager_new_is_empty() {
    let mgr = TerminalManager::new();
    assert_eq!(mgr.session_count(), 0);
    assert!(mgr.is_empty());
    assert_eq!(mgr.active_idx(), 0);
    assert_eq!(mgr.split_mode(), SplitMode::None);
    assert_eq!(mgr.split_idx(), None);
    assert!(mgr.active_session().is_none());
    assert!(mgr.split_session().is_none());
}

#[test]
fn test_session_addition_and_active_indexing() {
    let mut mgr = TerminalManager::new();

    let s1 = TerminalSession::spawn(80, 24, None, None)
        .expect("Failed to spawn session 1")
        .with_title("bash");
    let s1_id = s1.id;

    let idx1 = mgr.add_session(s1);
    assert_eq!(idx1, 0);
    assert_eq!(mgr.session_count(), 1);
    assert!(!mgr.is_empty());
    assert_eq!(mgr.active_idx(), 0);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(s1_id));

    // Title formatting
    let title1 = mgr.tab_title(0).expect("tab_title(0) should exist");
    assert_eq!(title1, "1: bash");

    let s2 = TerminalSession::spawn(80, 24, None, None)
        .expect("Failed to spawn session 2")
        .with_title("sh");
    let s2_id = s2.id;

    let idx2 = mgr.add_session(s2);
    assert_eq!(idx2, 1);
    assert_eq!(mgr.session_count(), 2);
    assert_eq!(mgr.active_idx(), 1);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(s2_id));

    let title2 = mgr.tab_title(1).expect("tab_title(1) should exist");
    assert_eq!(title2, "2: sh");
}

#[test]
fn test_tab_switching() {
    let mut mgr = TerminalManager::new();
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("term1");
    let s2 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("term2");
    let id1 = s1.id;
    let id2 = s2.id;

    mgr.add_session(s1);
    mgr.add_session(s2);
    assert_eq!(mgr.active_idx(), 1);

    mgr.set_active_idx(0);
    assert_eq!(mgr.active_idx(), 0);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id1));

    mgr.set_active_idx(1);
    assert_eq!(mgr.active_idx(), 1);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id2));

    // Out of bounds active index is safely clamped/ignored
    mgr.set_active_idx(99);
    assert_eq!(mgr.active_idx(), 1);
}

#[test]
fn test_closing_sessions_and_clamping() {
    let mut mgr = TerminalManager::new();
    let s0 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s0");
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s1");
    let s2 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s2");
    let id0 = s0.id;
    let id1 = s1.id;
    let id2 = s2.id;

    mgr.add_session(s0);
    mgr.add_session(s1);
    mgr.add_session(s2);
    assert_eq!(mgr.session_count(), 3);
    assert_eq!(mgr.active_idx(), 2);

    // Close session 0 (non-active). Active index 2 shifts to 1.
    let closed0 = mgr.close_session(0);
    assert_eq!(closed0.map(|s| s.id), Some(id0));
    assert_eq!(mgr.session_count(), 2);
    assert_eq!(mgr.active_idx(), 1);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id2));

    // Active is 1 (id2). Close active session 1.
    // Active index clamps to 0 (which is now id1).
    let closed2 = mgr.close_session(1);
    assert_eq!(closed2.map(|s| s.id), Some(id2));
    assert_eq!(mgr.session_count(), 1);
    assert_eq!(mgr.active_idx(), 0);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id1));

    // Close remaining session 0.
    let closed1 = mgr.close_session(0);
    assert_eq!(closed1.map(|s| s.id), Some(id1));
    assert_eq!(mgr.session_count(), 0);
    assert!(mgr.is_empty());
    assert_eq!(mgr.active_idx(), 0);

    // Closing out of bounds returns None
    assert!(mgr.close_session(0).is_none());
}

#[test]
fn test_split_mode_toggling_and_indexing() {
    let mut mgr = TerminalManager::new();
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s1");
    let s2 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s2");
    let id1 = s1.id;
    let id2 = s2.id;

    mgr.add_session(s1);
    mgr.add_session(s2);
    mgr.set_active_idx(0);

    assert_eq!(mgr.split_mode(), SplitMode::None);
    assert_eq!(mgr.split_idx(), None);
    assert!(mgr.split_session().is_none());

    // Toggle 1: None -> Vertical
    mgr.toggle_split();
    assert_eq!(mgr.split_mode(), SplitMode::Vertical);
    assert_eq!(mgr.split_idx(), Some(1));
    assert_eq!(mgr.split_session().map(|s| s.id), Some(id2));
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id1));

    // Toggle 2: Vertical -> Horizontal
    mgr.toggle_split();
    assert_eq!(mgr.split_mode(), SplitMode::Horizontal);
    assert_eq!(mgr.split_idx(), Some(1));
    assert_eq!(mgr.split_session().map(|s| s.id), Some(id2));

    // Toggle 3: Horizontal -> None
    mgr.toggle_split();
    assert_eq!(mgr.split_mode(), SplitMode::None);
    assert_eq!(mgr.split_idx(), None);
    assert!(mgr.split_session().is_none());
}

#[test]
fn test_split_swapping_on_active_switch() {
    let mut mgr = TerminalManager::new();
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s1");
    let s2 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s2");
    let id1 = s1.id;
    let id2 = s2.id;

    mgr.add_session(s1);
    mgr.add_session(s2);
    mgr.set_active_idx(0);
    mgr.set_split(Some(1), SplitMode::Vertical);

    assert_eq!(mgr.active_idx(), 0);
    assert_eq!(mgr.split_idx(), Some(1));

    // Clicking the split tab switches active to 1, and swaps split to 0
    mgr.set_active_idx(1);
    assert_eq!(mgr.active_idx(), 1);
    assert_eq!(mgr.split_idx(), Some(0));
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id2));
    assert_eq!(mgr.split_session().map(|s| s.id), Some(id1));
}

#[test]
fn test_remove_dead_sessions() {
    let mut mgr = TerminalManager::new();
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s1");
    let s2 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s2");
    let id2 = s2.id;

    mgr.add_session(s1);
    mgr.add_session(s2);
    mgr.set_active_idx(0);

    // Terminate session 0
    mgr.sessions_mut()[0].terminate();
    assert!(!mgr.sessions()[0].is_alive());
    assert!(mgr.sessions()[1].is_alive());

    mgr.remove_dead_sessions();
    assert_eq!(mgr.session_count(), 1);
    assert_eq!(mgr.active_idx(), 0);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id2));
}

#[test]
fn test_close_split_session_collapses_or_adjusts() {
    let mut mgr = TerminalManager::new();
    let s0 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s0");
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s1");
    let id0 = s0.id;

    mgr.add_session(s0);
    mgr.add_session(s1);
    mgr.set_active_idx(0);
    mgr.set_split(Some(1), SplitMode::Vertical);

    assert_eq!(mgr.split_mode(), SplitMode::Vertical);
    assert_eq!(mgr.split_idx(), Some(1));

    // Closing the split session leaves only 1 session, which collapses split_mode to None
    mgr.close_session(1);
    assert_eq!(mgr.session_count(), 1);
    assert_eq!(mgr.split_mode(), SplitMode::None);
    assert_eq!(mgr.split_idx(), None);
    assert_eq!(mgr.active_session().map(|s| s.id), Some(id0));
}

#[test]
fn test_active_and_split_mut() {
    let mut mgr = TerminalManager::new();
    let s0 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s0");
    let s1 = TerminalSession::spawn(80, 24, None, None).unwrap().with_title("s1");
    let id0 = s0.id;
    let id1 = s1.id;

    mgr.add_session(s0);
    mgr.add_session(s1);
    mgr.set_active_idx(0);
    mgr.set_split(Some(1), SplitMode::Vertical);

    let (act, spl) = mgr.active_and_split_mut();
    assert_eq!(act.as_ref().map(|s| s.id), Some(id0));
    assert_eq!(spl.as_ref().map(|s| s.id), Some(id1));

    if let Some(a) = act {
        a.title = "s0_modified".to_string();
    }
    if let Some(s) = spl {
        s.title = "s1_modified".to_string();
    }

    assert_eq!(mgr.active_session().unwrap().title, "s0_modified");
    assert_eq!(mgr.split_session().unwrap().title, "s1_modified");
}
