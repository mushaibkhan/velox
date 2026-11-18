use velox_editor::{EditorState, LineChange};
use velox_git::compute_diff;
use velox_text_buffer::TextBuffer;

#[test]
fn test_gutter_initial_and_cleared_diff() {
    let buffer = TextBuffer::from_str("line 1\nline 2\nline 3\n");
    let mut editor = EditorState::new(buffer);

    // Initial state: no git_diff set -> all queries return None
    assert!(editor.git_diff.is_none());
    assert_eq!(editor.line_git_change(1), None);
    assert_eq!(editor.line_git_change(2), None);
    assert_eq!(editor.line_git_change(3), None);

    // Set diff
    let diff = compute_diff("line 1\nline 2\n", "line 1\nline 2\nline 3\n");
    editor.set_git_diff(Some(diff));
    assert!(editor.git_diff.is_some());
    assert_eq!(editor.line_git_change(3), Some(LineChange::Added));

    // Clear diff
    editor.set_git_diff(None);
    assert!(editor.git_diff.is_none());
    assert_eq!(editor.line_git_change(1), None);
    assert_eq!(editor.line_git_change(2), None);
    assert_eq!(editor.line_git_change(3), None);
}

#[test]
fn test_gutter_unchanged_file() {
    let text = "alpha\nbeta\ngamma\n";
    let buffer = TextBuffer::from_str(text);
    let mut editor = EditorState::new(buffer);

    let diff = compute_diff(text, text);
    assert!(diff.is_empty());
    editor.set_git_diff(Some(diff));

    assert_eq!(editor.line_git_change(1), Some(LineChange::Unchanged));
    assert_eq!(editor.line_git_change(2), Some(LineChange::Unchanged));
    assert_eq!(editor.line_git_change(3), Some(LineChange::Unchanged));
}

#[test]
fn test_gutter_added_and_modified_lines() {
    let old_text = "fn main() {\n    let a = 1;\n    println!(\"hi\");\n}\n";
    let new_text = "fn main() {\n    let a = 2;\n    let b = 3;\n    println!(\"hi\");\n}\n";

    let buffer = TextBuffer::from_str(new_text);
    let mut editor = EditorState::new(buffer);

    let diff = compute_diff(old_text, new_text);
    assert_eq!(diff.additions, 2);
    assert_eq!(diff.deletions, 1);
    assert_eq!(diff.modifications, 1);

    editor.set_git_diff(Some(diff));

    // Line 1: fn main() { -> Unchanged
    assert_eq!(editor.line_git_change(1), Some(LineChange::Unchanged));
    // Line 2: let a = 2; -> Modified
    assert_eq!(editor.line_git_change(2), Some(LineChange::Modified));
    // Line 3: let b = 3; -> Added
    assert_eq!(editor.line_git_change(3), Some(LineChange::Added));
    // Line 4: println!("hi"); -> Unchanged
    assert_eq!(editor.line_git_change(4), Some(LineChange::Unchanged));
    // Line 5: } -> Unchanged
    assert_eq!(editor.line_git_change(5), Some(LineChange::Unchanged));
}

#[test]
fn test_gutter_deleted_markers() {
    // 1. Deletion in the middle
    let old_text = "line 1\nline 2\nline 3\n";
    let new_text = "line 1\nline 3\n";
    let buffer = TextBuffer::from_str(new_text);
    let mut editor = EditorState::new(buffer);

    let diff = compute_diff(old_text, new_text);
    assert_eq!(diff.deletions, 1);
    editor.set_git_diff(Some(diff));

    assert_eq!(editor.line_git_change(1), Some(LineChange::Unchanged));
    // Line 2 in new text receives the deletion marker
    assert_eq!(editor.line_git_change(2), Some(LineChange::Deleted));

    // 2. Deletion at start
    let old_start = "deleted line\nkept line\n";
    let new_start = "kept line\n";
    let mut editor_start = EditorState::new(TextBuffer::from_str(new_start));
    editor_start.set_git_diff(Some(compute_diff(old_start, new_start)));
    assert_eq!(editor_start.line_git_change(1), Some(LineChange::Deleted));

    // 3. Deletion at end
    let old_end = "kept line\ndeleted line\n";
    let new_end = "kept line\n";
    let mut editor_end = EditorState::new(TextBuffer::from_str(new_end));
    editor_end.set_git_diff(Some(compute_diff(old_end, new_end)));
    assert_eq!(editor_end.line_git_change(1), Some(LineChange::Deleted));
}

#[test]
fn test_gutter_out_of_bounds() {
    let text = "first\nsecond\n";
    let buffer = TextBuffer::from_str(text);
    let mut editor = EditorState::new(buffer);

    let diff = compute_diff("first\n", text);
    editor.set_git_diff(Some(diff));

    // 1-based indexing: line 0 is out of bounds
    assert_eq!(editor.line_git_change(0), None);
    // Line 1 and 2 are within bounds
    assert_eq!(editor.line_git_change(1), Some(LineChange::Unchanged));
    assert_eq!(editor.line_git_change(2), Some(LineChange::Added));
    // Beyond buffer line count is out of bounds
    assert_eq!(editor.line_git_change(3), None);
    assert_eq!(editor.line_git_change(100), None);
}
