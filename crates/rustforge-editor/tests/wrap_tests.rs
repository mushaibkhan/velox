#![allow(clippy::single_range_in_vec_init)]

use rustforge_core::Position;
use rustforge_editor::wrap::{
    buffer_pos_to_visual_row, visual_row_to_buffer_pos, wrap_line, WrappedLineLayout,
};
use rustforge_editor::EditorState;
use rustforge_text_buffer::TextBuffer;

#[test]
fn test_wrap_empty_lines() {
    assert_eq!(wrap_line("", 80), vec![0..0]);
    assert_eq!(wrap_line("\n", 80), vec![0..0]);
    assert_eq!(wrap_line("\r\n", 80), vec![0..0]);
    assert_eq!(wrap_line("", 0), vec![0..0]);
}

#[test]
fn test_wrap_short_line_no_wrap() {
    assert_eq!(wrap_line("hello", 10), vec![0..5]);
    assert_eq!(wrap_line("hello", 5), vec![0..5]);
}

#[test]
fn test_wrap_word_boundary_whitespace() {
    // "hello world" wrapped at 5 chars -> "hello" (0..5), "world" (6..11)
    let rows = wrap_line("hello world", 5);
    assert_eq!(rows, vec![0..5, 6..11]);

    // "hello world" wrapped at 8 chars -> still splits between words cleanly
    let rows = wrap_line("hello world", 8);
    assert_eq!(rows, vec![0..5, 6..11]);

    // Multiple words: "the quick brown fox" with max 10
    // "the quick" is 9 chars (0..9)
    // "brown fox" is 9 chars (10..19)
    let rows = wrap_line("the quick brown fox", 10);
    assert_eq!(rows, vec![0..9, 10..19]);
}

#[test]
fn test_wrap_word_boundary_punctuation() {
    // Hyphenated word wrap: "foo-bar" at 5 chars -> "foo-" (0..4), "bar" (4..7)
    let rows = wrap_line("foo-bar", 5);
    assert_eq!(rows, vec![0..4, 4..7]);

    // Comma with space: "foo, bar" at 5 chars -> "foo," (0..4), "bar" (5..8)
    let rows = wrap_line("foo, bar", 5);
    assert_eq!(rows, vec![0..4, 5..8]);

    // Series of punctuated tokens: "apple,banana,cherry" at 7 chars
    let rows = wrap_line("apple,banana,cherry", 7);
    assert_eq!(rows, vec![0..6, 6..13, 13..19]);
}

#[test]
fn test_wrap_fallback_hard_boundary() {
    // Long unbroken string exceeds max width: fallback to exact char boundary
    let text = "abcdefghijklmnop"; // 16 chars
    let rows = wrap_line(text, 5);
    assert_eq!(rows, vec![0..5, 5..10, 10..15, 15..16]);

    // Word boundary first, then long unbroken word: "a supercalifragilistic"
    // "a " -> 0..1, then "supercalifragilistic" (20 chars) in chunks of 6 starting at 2
    let rows = wrap_line("a supercalifragilistic", 6);
    assert_eq!(rows, vec![0..1, 2..8, 8..14, 14..20, 20..22]);
}

#[test]
fn test_wrap_zero_max_chars_fallback() {
    // Guard against div-by-zero or infinite loop when max_chars_per_row is 0
    let rows = wrap_line("abc", 0);
    assert_eq!(rows, vec![0..1, 1..2, 2..3]);
}

#[test]
fn test_visual_row_to_buffer_pos_mapping() {
    // Layout representing two physical lines:
    // Line 0: "hello world" -> sub-rows [0..5, 6..11]
    // Line 1: "rustforge"   -> sub-row  [0..9]
    let layouts = vec![
        WrappedLineLayout::new(0, vec![0..5, 6..11]),
        WrappedLineLayout::new(1, vec![0..9]),
    ];

    // Visual row 0 (Line 0, row 0..5):
    assert_eq!(visual_row_to_buffer_pos(0, 0, &layouts), Position::new(0, 0));
    assert_eq!(visual_row_to_buffer_pos(0, 3, &layouts), Position::new(0, 3));
    assert_eq!(visual_row_to_buffer_pos(0, 5, &layouts), Position::new(0, 5));
    // Clamping past end of sub-row
    assert_eq!(visual_row_to_buffer_pos(0, 10, &layouts), Position::new(0, 5));

    // Visual row 1 (Line 0, row 6..11):
    assert_eq!(visual_row_to_buffer_pos(1, 0, &layouts), Position::new(0, 6));
    assert_eq!(visual_row_to_buffer_pos(1, 4, &layouts), Position::new(0, 10));
    assert_eq!(visual_row_to_buffer_pos(1, 5, &layouts), Position::new(0, 11));

    // Visual row 2 (Line 1, row 0..9):
    assert_eq!(visual_row_to_buffer_pos(2, 0, &layouts), Position::new(1, 0));
    assert_eq!(visual_row_to_buffer_pos(2, 4, &layouts), Position::new(1, 4));
}

#[test]
fn test_buffer_pos_to_visual_row_mapping() {
    let layouts = vec![
        WrappedLineLayout::new(0, vec![0..5, 6..11]),
        WrappedLineLayout::new(1, vec![0..9]),
    ];

    // Line 0 col 0 -> visual row 0, col 0
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 0), &layouts), (0, 0));
    // Line 0 col 5 -> visual row 0, col 5 (end of first row)
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 5), &layouts), (0, 5));
    // Line 0 col 6 -> visual row 1, col 0 (start of second row)
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 6), &layouts), (1, 0));
    // Line 0 col 11 -> visual row 1, col 5 (end of second row)
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 11), &layouts), (1, 5));

    // Line 1 col 4 -> visual row 2, col 4
    assert_eq!(buffer_pos_to_visual_row(Position::new(1, 4), &layouts), (2, 4));
}

#[test]
fn test_editor_state_word_wrap_integration() {
    let text = "Line 1 is long enough to wrap\nLine 2";
    let buffer = TextBuffer::from_str(text);
    let mut editor = EditorState::new(buffer);

    assert!(!editor.word_wrap);
    editor.word_wrap = true;
    assert!(editor.word_wrap);

    let layouts = editor.compute_wrapped_layouts(10);
    assert_eq!(layouts.len(), 2);
    assert!(layouts[0].row_count() > 1);
    assert_eq!(layouts[1].row_count(), 1);
}

#[test]
fn test_regression_whitespace_gap_cursor_mapping() {
    // "hello   world" (indices 0..5 "hello", 5..8 "   ", 8..13 "world")
    // Wrapped at 5 chars: row 0 is 0..5, row 1 is 8..13.
    let rows = wrap_line("hello   world", 5);
    assert_eq!(rows, vec![0..5, 8..13]);

    let layouts = vec![WrappedLineLayout::new(0, rows)];

    // Characters within row 0:
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 0), &layouts), (0, 0));
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 4), &layouts), (0, 4));
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 5), &layouts), (0, 5));

    // Characters in the omitted whitespace gap (cols 6 and 7):
    // Must NOT teleport to (0, 0); must clamp to the end of visual row 0.
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 6), &layouts), (0, 5));
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 7), &layouts), (0, 5));

    // Start of second visual row (col 8, 'w'):
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 8), &layouts), (1, 0));
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 10), &layouts), (1, 2));
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 13), &layouts), (1, 5));

    // Beyond EOL: must clamp visual column to (range.end - range.start)
    assert_eq!(buffer_pos_to_visual_row(Position::new(0, 50), &layouts), (1, 5));
}

#[test]
fn test_regression_wrap_layout_caching() {
    let text = "Line 1 is long enough to wrap\nLine 2";
    let buffer = TextBuffer::from_str(text);
    let mut editor = EditorState::new(buffer);

    // Initial state
    assert!(editor.wrap_dirty);
    assert_eq!(editor.cached_wrap_cols, 0);
    assert!(editor.cached_wrapped_layouts.is_empty());

    // First update computes layouts and caches them
    editor.update_wrapped_layouts(10);
    assert!(!editor.wrap_dirty);
    assert_eq!(editor.cached_wrap_cols, 10);
    assert_eq!(editor.cached_wrapped_layouts.len(), 2);
    let initial_row_count = editor.cached_wrapped_layouts[0].row_count();
    assert!(initial_row_count > 1);

    // Marker sentinel to verify cache hit
    editor.cached_wrapped_layouts[0].line_index = 999;

    // Second call with same column count: must be a cache hit (line_index remains 999)
    editor.update_wrapped_layouts(10);
    assert_eq!(editor.cached_wrapped_layouts[0].line_index, 999);

    // Column count change invalidates cache and recomputes
    editor.update_wrapped_layouts(40);
    assert_eq!(editor.cached_wrap_cols, 40);
    assert_eq!(editor.cached_wrapped_layouts[0].line_index, 0); // recomputed!

    // Typing text sets wrap_dirty = true
    editor.insert_text(" additional text");
    assert!(editor.wrap_dirty);

    // Next update recomputes
    editor.update_wrapped_layouts(40);
    assert!(!editor.wrap_dirty);

    // Backspace marks wrap_dirty
    editor.backspace();
    assert!(editor.wrap_dirty);

    editor.update_wrapped_layouts(40);
    assert!(!editor.wrap_dirty);

    // Undo marks wrap_dirty
    editor.undo();
    assert!(editor.wrap_dirty);
}
