use rustforge_core::{Position, Range, Selection};
use rustforge_editor::{DiagnosticItem, DiagnosticSeverity, EditorState};
use rustforge_text_buffer::TextBuffer;

#[test]
fn test_diagnostic_assignment_and_query_by_line() {
    let text = "fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}\n";
    let buffer = TextBuffer::from_str(text);
    let mut editor = EditorState::new(buffer);

    let d1 = DiagnosticItem::new(
        Range::new(Position::new(1, 4), Position::new(1, 9)),
        DiagnosticSeverity::Warning,
        "unused variable `x`",
    );
    let d2 = DiagnosticItem::new(
        Range::new(Position::new(2, 4), Position::new(2, 22)),
        DiagnosticSeverity::Error,
        "syntax error in format macro",
    );

    editor.set_diagnostics(vec![d1.clone(), d2.clone()]);
    assert_eq!(editor.diagnostics.len(), 2);

    // Line 0: no diagnostics
    let diags_0 = editor.diagnostics_for_line(0);
    assert!(diags_0.is_empty());

    // Line 1: unused variable `x`
    let diags_1 = editor.diagnostics_for_line(1);
    assert_eq!(diags_1.len(), 1);
    assert_eq!(diags_1[0].message, "unused variable `x`");
    assert_eq!(diags_1[0].severity, DiagnosticSeverity::Warning);

    // Line 2: syntax error
    let diags_2 = editor.diagnostics_for_line(2);
    assert_eq!(diags_2.len(), 1);
    assert_eq!(diags_2[0].message, "syntax error in format macro");
    assert_eq!(diags_2[0].severity, DiagnosticSeverity::Error);

    // Line 3: no diagnostics
    let diags_3 = editor.diagnostics_for_line(3);
    assert!(diags_3.is_empty());
}

#[test]
fn test_highest_severity_calculation() {
    let buffer = TextBuffer::from_str("line 0\nline 1\nline 2\nline 3\nline 4\n");
    let mut editor = EditorState::new(buffer);

    let err = DiagnosticItem::new(
        Range::new(Position::new(0, 0), Position::new(0, 5)),
        DiagnosticSeverity::Error,
        "error message",
    );
    let warn = DiagnosticItem::new(
        Range::new(Position::new(0, 2), Position::new(0, 6)),
        DiagnosticSeverity::Warning,
        "warning message",
    );
    let info = DiagnosticItem::new(
        Range::new(Position::new(1, 0), Position::new(1, 5)),
        DiagnosticSeverity::Information,
        "info message",
    );
    let hint = DiagnosticItem::new(
        Range::new(Position::new(1, 2), Position::new(1, 4)),
        DiagnosticSeverity::Hint,
        "hint message",
    );
    let only_hint = DiagnosticItem::new(
        Range::new(Position::new(2, 0), Position::new(2, 3)),
        DiagnosticSeverity::Hint,
        "only hint message",
    );

    editor.set_diagnostics(vec![err, warn, info, hint, only_hint]);

    // Line 0 has both Error and Warning -> Error is highest
    assert_eq!(
        editor.highest_severity_for_line(0),
        Some(DiagnosticSeverity::Error)
    );

    // Line 1 has Information and Hint -> Information is highest
    assert_eq!(
        editor.highest_severity_for_line(1),
        Some(DiagnosticSeverity::Information)
    );

    // Line 2 has only Hint -> Hint
    assert_eq!(
        editor.highest_severity_for_line(2),
        Some(DiagnosticSeverity::Hint)
    );

    // Line 3 has no diagnostics -> None
    assert_eq!(editor.highest_severity_for_line(3), None);
}

#[test]
fn test_multiline_diagnostic_intersection() {
    let buffer = TextBuffer::from_str("0\n1\n2\n3\n4\n5\n6\n7\n");
    let mut editor = EditorState::new(buffer);

    // Diagnostic spanning lines 2 to 5
    let multi = DiagnosticItem::new(
        Range::new(Position::new(2, 3), Position::new(5, 4)),
        DiagnosticSeverity::Error,
        "multiline error",
    );

    // Diagnostic ending at col 0 of line 4 (lines 3 to 4 with end.col = 0)
    let end_at_zero = DiagnosticItem::new(
        Range::new(Position::new(3, 1), Position::new(4, 0)),
        DiagnosticSeverity::Warning,
        "terminates at col 0",
    );

    editor.set_diagnostics(vec![multi, end_at_zero]);

    // Line 1: outside
    assert!(editor.diagnostics_for_line(1).is_empty());

    // Line 2: intersects multi
    let d2 = editor.diagnostics_for_line(2);
    assert_eq!(d2.len(), 1);
    assert_eq!(d2[0].message, "multiline error");

    // Line 3: intersects both multi and end_at_zero
    let d3 = editor.diagnostics_for_line(3);
    assert_eq!(d3.len(), 2);

    // Line 4: intersects multi, but end_at_zero is excluded because end.column == 0
    let d4 = editor.diagnostics_for_line(4);
    assert_eq!(d4.len(), 1);
    assert_eq!(d4[0].message, "multiline error");

    // Line 5: intersects multi (end.col is 4)
    let d5 = editor.diagnostics_for_line(5);
    assert_eq!(d5.len(), 1);
    assert_eq!(d5[0].message, "multiline error");

    // Line 6: outside
    assert!(editor.diagnostics_for_line(6).is_empty());
}

#[test]
fn test_completion_item_navigation() {
    let buffer = TextBuffer::from_str("let x = ");
    let mut editor = EditorState::new(buffer);

    let item1 = lsp_types::CompletionItem {
        label: "apple".to_string(),
        ..Default::default()
    };
    let item2 = lsp_types::CompletionItem {
        label: "banana".to_string(),
        ..Default::default()
    };
    let item3 = lsp_types::CompletionItem {
        label: "cherry".to_string(),
        ..Default::default()
    };

    editor.set_completions(vec![item1, item2, item3]);
    assert!(editor.show_completion);
    assert_eq!(editor.completion_selected_idx, 0);
    assert_eq!(editor.selected_completion().unwrap().label, "apple");

    // Next -> banana (idx 1)
    editor.select_next_completion();
    assert_eq!(editor.completion_selected_idx, 1);
    assert_eq!(editor.selected_completion().unwrap().label, "banana");

    // Next -> cherry (idx 2)
    editor.select_next_completion();
    assert_eq!(editor.completion_selected_idx, 2);
    assert_eq!(editor.selected_completion().unwrap().label, "cherry");

    // Next wraps around -> apple (idx 0)
    editor.select_next_completion();
    assert_eq!(editor.completion_selected_idx, 0);
    assert_eq!(editor.selected_completion().unwrap().label, "apple");

    // Prev wraps around -> cherry (idx 2)
    editor.select_prev_completion();
    assert_eq!(editor.completion_selected_idx, 2);
    assert_eq!(editor.selected_completion().unwrap().label, "cherry");

    // Prev -> banana (idx 1)
    editor.select_prev_completion();
    assert_eq!(editor.completion_selected_idx, 1);
    assert_eq!(editor.selected_completion().unwrap().label, "banana");

    // Close completion
    editor.close_completion();
    assert!(!editor.show_completion);
    assert!(editor.completion_items.is_empty());
    assert_eq!(editor.completion_selected_idx, 0);
}

#[test]
fn test_apply_selected_completion_and_cursor_advancement() {
    let buffer = TextBuffer::from_str("let x = ");
    let mut editor = EditorState::new(buffer);
    // Cursor at end of "let x = " (index 8)
    editor.cursors = vec![Selection::point(8)];
    editor.primary_cursor_idx = 0;

    let item = lsp_types::CompletionItem {
        label: "compute_value".to_string(),
        insert_text: Some("compute_value(10)".to_string()),
        ..Default::default()
    };

    editor.set_completions(vec![item]);
    assert!(editor.show_completion);

    let applied = editor.apply_selected_completion();
    assert!(applied);
    assert!(!editor.show_completion);

    let result_text = editor.buffer.slice_to_string(0, editor.buffer.total_chars());
    assert_eq!(result_text, "let x = compute_value(10)");

    // Cursor should now be at position 8 + "compute_value(10)".len() = 25
    assert_eq!(editor.primary_cursor().head, 25);
    assert_eq!(editor.primary_cursor().anchor, 25);
}

#[test]
fn test_hover_text_setting_and_clearing() {
    let buffer = TextBuffer::from_str("fn test() {}\n");
    let mut editor = EditorState::new(buffer);

    assert!(!editor.show_hover);
    assert_eq!(editor.hover_text, None);

    editor.set_hover(Some("fn test() -> ()\nBasic function documentation".to_string()));
    assert!(editor.show_hover);
    assert_eq!(
        editor.hover_text.as_deref(),
        Some("fn test() -> ()\nBasic function documentation")
    );

    editor.close_hover();
    assert!(!editor.show_hover);
    assert_eq!(editor.hover_text, None);

    // Setting None directly
    editor.set_hover(Some("another doc".to_string()));
    assert!(editor.show_hover);
    editor.set_hover(None);
    assert!(!editor.show_hover);
    assert_eq!(editor.hover_text, None);
}

#[test]
fn test_apply_selected_completion_with_prefix_replacement() {
    let buffer = TextBuffer::from_str("let x = comp");
    let mut editor = EditorState::new(buffer);
    // Cursor at end of "comp" (index 12)
    editor.cursors = vec![Selection::point(12)];
    editor.primary_cursor_idx = 0;

    let item = lsp_types::CompletionItem {
        label: "compute_value".to_string(),
        insert_text: Some("compute_value(10)".to_string()),
        ..Default::default()
    };

    editor.set_completions(vec![item]);
    let applied = editor.apply_selected_completion();
    assert!(applied);

    let result_text = editor.buffer.slice_to_string(0, editor.buffer.total_chars());
    assert_eq!(result_text, "let x = compute_value(10)");
}
