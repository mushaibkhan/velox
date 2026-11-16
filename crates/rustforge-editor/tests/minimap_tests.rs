use rustforge_editor::minimap::{
    click_to_scroll_line, compute_thumb_rect, editor_line_to_minimap_y,
    effective_track_height, minimap_line_to_editor_line, minimap_y_to_editor_line,
    thumb_y_to_scroll_line, MinimapLine, MinimapState, MinimapThumb,
};
use rustforge_editor::EditorState;
use rustforge_text_buffer::TextBuffer;

#[test]
fn test_minimap_initial_state() {
    let minimap = MinimapState::new();
    assert!(minimap.enabled);
    assert_eq!(minimap.width, 100.0);
    assert_eq!(minimap.max_columns, 100);
    assert_eq!(minimap.min_thumb_height, 20.0);
    assert_eq!(minimap.cached_line_count(), 0);

    let thumb = MinimapThumb::new(10.0, 50.0);
    assert_eq!(thumb.top(), 10.0);
    assert_eq!(thumb.bottom(), 60.0);
    assert!(thumb.contains_y(30.0));
    assert!(!thumb.contains_y(5.0));
    assert!(!thumb.contains_y(65.0));
}

#[test]
fn test_minimap_density_and_line_metrics() {
    // 4 spaces indent, 20 chars total length, max 100 columns
    let line = MinimapLine::new(0, 4, 20, 100);
    assert_eq!(line.line, 0);
    assert_eq!(line.line_idx, 0);
    assert_eq!(line.indent, 4);
    assert_eq!(line.length, 20);
    assert_eq!(line.content_length(), 16);
    assert!((line.indent_ratio - 0.04).abs() < 1e-4);
    assert!((line.width_ratio - 0.20).abs() < 1e-4);
    assert!((line.content_width_ratio() - 0.16).abs() < 1e-4);
    assert!(!line.is_empty());

    // Blank line
    let empty_line = MinimapLine::new(1, 0, 0, 100);
    assert_eq!(empty_line.length, 0);
    assert!(empty_line.is_empty());

    // Whitespace only
    let ws_line = MinimapLine::new(2, 8, 8, 100);
    assert_eq!(ws_line.content_length(), 0);
    assert!(ws_line.is_empty());

    // Line exceeding max columns (should clamp to 1.0)
    let long_line = MinimapLine::new(3, 10, 150, 100);
    assert_eq!(long_line.width_ratio, 1.0);
    assert!((long_line.indent_ratio - 0.10).abs() < 1e-4);
}

#[test]
fn test_minimap_compute_lines_empty_and_cached() {
    let mut minimap = MinimapState::new();
    
    // When cache is empty, compute_minimap_lines generates placeholder lines
    let lines = minimap.compute_minimap_lines(5);
    assert_eq!(lines.len(), 5);
    for (i, l) in lines.iter().enumerate() {
        assert_eq!(l.line, i);
        assert_eq!(l.length, 0);
    }

    // Set line cache explicitly
    minimap.set_line(0, 0, 10);
    minimap.set_line(1, 4, 24);
    let updated = minimap.compute_minimap_lines(2);
    assert_eq!(updated.len(), 2);
    assert_eq!(updated[0].length, 10);
    assert_eq!(updated[1].indent, 4);
    assert_eq!(updated[1].length, 24);
}

#[test]
fn test_minimap_line_length_caching_from_buffer() {
    let code = "fn main() {\n    let answer = 42;\n\n    println!(\"{}\", answer);\n}\n";
    let buffer = TextBuffer::from_str(code);
    let mut minimap = MinimapState::new();
    minimap.update_from_buffer(&buffer);

    assert_eq!(minimap.cached_line_count(), buffer.line_count());
    let lines = minimap.compute_minimap_lines(buffer.line_count());
    assert_eq!(lines.len(), buffer.line_count());

    // Line 0: "fn main() {"
    assert_eq!(lines[0].line, 0);
    assert_eq!(lines[0].indent, 0);
    assert_eq!(lines[0].length, 11);

    // Line 1: "    let answer = 42;"
    assert_eq!(lines[1].line, 1);
    assert_eq!(lines[1].indent, 4);
    assert_eq!(lines[1].length, 20);

    // Line 2: ""
    assert_eq!(lines[2].line, 2);
    assert_eq!(lines[2].length, 0);
    assert!(lines[2].is_empty());

    // Line 3: "    println!(\"{}\", answer);"
    assert_eq!(lines[3].line, 3);
    assert_eq!(lines[3].indent, 4);
    assert_eq!(lines[3].length, 27);

    // Line 4: "}"
    assert_eq!(lines[4].line, 4);
    assert_eq!(lines[4].indent, 0);
    assert_eq!(lines[4].length, 1);
}

#[test]
fn test_minimap_line_length_caching_from_str() {
    let mut minimap = MinimapState::new();
    minimap.update_from_str("hello\n    world\n");
    assert_eq!(minimap.cached_line_count(), 3); // "hello", "    world", ""
    let lines = minimap.cached_lines();
    assert_eq!(lines[0].length, 5);
    assert_eq!(lines[1].indent, 4);
    assert_eq!(lines[1].length, 9);
}

#[test]
fn test_scroll_coordinate_translation() {
    let total_lines = 100;
    let height = 500.0;

    // minimap_line_to_editor_line / minimap_y_to_editor_line
    assert_eq!(minimap_line_to_editor_line(0.0, total_lines, height), 0);
    assert_eq!(minimap_y_to_editor_line(0.0, total_lines, height), 0);

    // Halfway down 500px on 100 lines -> line 50
    assert_eq!(minimap_line_to_editor_line(250.0, total_lines, height), 50);

    // Near the bottom: line 99
    assert_eq!(minimap_line_to_editor_line(499.0, total_lines, height), 99);
    assert_eq!(minimap_line_to_editor_line(500.0, total_lines, height), 99);

    // Clamping outside bounds
    assert_eq!(minimap_line_to_editor_line(-50.0, total_lines, height), 0);
    assert_eq!(minimap_line_to_editor_line(1000.0, total_lines, height), 99);

    // Edge cases
    assert_eq!(minimap_line_to_editor_line(100.0, 0, height), 0);
    assert_eq!(minimap_line_to_editor_line(100.0, total_lines, 0.0), 0);

    // editor_line_to_minimap_y
    assert_eq!(editor_line_to_minimap_y(0, total_lines, height), 0.0);
    assert_eq!(editor_line_to_minimap_y(50, total_lines, height), 250.0);
    assert!((editor_line_to_minimap_y(99, total_lines, height) - 495.0).abs() < 1e-4);
}

#[test]
fn test_viewport_thumb_geometry() {
    let height = 1000.0;
    let total_lines = 100;
    let visible_lines = 25;
    let min_thumb_height = 20.0;

    // 1. Scrolled to top
    let thumb_top = compute_thumb_rect(0, visible_lines, total_lines, height, min_thumb_height);
    assert_eq!(thumb_top.y, 0.0);
    assert_eq!(thumb_top.height, 250.0); // 25% of 1000px

    // 2. Scrolled halfway: max_scroll = 100 - 25 = 75
    let thumb_mid = compute_thumb_rect(37, visible_lines, total_lines, height, min_thumb_height);
    assert!(thumb_mid.y > 0.0 && thumb_mid.bottom() < height);

    // 3. Scrolled to bottom: scroll_line = 75
    let thumb_bottom = compute_thumb_rect(75, visible_lines, total_lines, height, min_thumb_height);
    assert!((thumb_bottom.bottom() - height).abs() < 1e-4);

    // 4. When total_lines <= visible_lines, thumb spans entire height
    let thumb_full = compute_thumb_rect(0, 150, 100, height, min_thumb_height);
    assert_eq!(thumb_full.y, 0.0);
    assert_eq!(thumb_full.height, height);

    // 5. Min thumb height enforcement for huge files
    let thumb_tiny = compute_thumb_rect(0, 20, 10_000, height, min_thumb_height);
    assert_eq!(thumb_tiny.height, min_thumb_height);
}

#[test]
fn test_thumb_drag_to_scroll_line() {
    let height = 1000.0;
    let total_lines = 100;
    let visible_lines = 20;
    let thumb_height = 200.0;
    let travel_height = height - thumb_height; // 800.0
    let max_scroll = total_lines - visible_lines; // 80

    // Top
    assert_eq!(thumb_y_to_scroll_line(0.0, thumb_height, total_lines, visible_lines, height), 0);

    // Midpoint
    assert_eq!(
        thumb_y_to_scroll_line(travel_height / 2.0, thumb_height, total_lines, visible_lines, height),
        max_scroll / 2
    );

    // Bottom
    assert_eq!(
        thumb_y_to_scroll_line(travel_height, thumb_height, total_lines, visible_lines, height),
        max_scroll
    );

    // Overflow clamping
    assert_eq!(
        thumb_y_to_scroll_line(1500.0, thumb_height, total_lines, visible_lines, height),
        max_scroll
    );
}

#[test]
fn test_click_to_scroll_line_centering() {
    let height = 1000.0;
    let total_lines = 100;
    let visible_lines = 20;

    // Click at line 50 (y = 500.0). Centered viewport should have scroll_line = 50 - 10 = 40
    let scroll = click_to_scroll_line(500.0, total_lines, visible_lines, height);
    assert_eq!(scroll, 40);

    // Click near top (y = 10.0 -> line 1). scroll_line = 0 (saturating_sub)
    let scroll_top = click_to_scroll_line(10.0, total_lines, visible_lines, height);
    assert_eq!(scroll_top, 0);

    // Click near bottom (y = 990.0). Should clamp to max_scroll = 80
    let scroll_bottom = click_to_scroll_line(990.0, total_lines, visible_lines, height);
    assert_eq!(scroll_bottom, 80);
}

#[test]
fn test_editor_state_minimap_integration() {
    let buffer = TextBuffer::from_str("fn foo() {\n    let x = 1;\n    let y = 2;\n}\n");
    let mut editor = EditorState::new(buffer);

    // Minimap should be initialized and cached
    assert!(editor.minimap.enabled);
    assert_eq!(editor.minimap.cached_line_count(), editor.buffer.line_count());

    let lines = editor.minimap_lines();
    assert_eq!(lines.len(), editor.buffer.line_count());
    assert_eq!(lines[1].indent, 4);

    let thumb = editor.minimap_thumb(600.0);
    assert!(thumb.height > 0.0);

    // Test text mutation triggers minimap update
    editor.insert_text("// comment\n");
    assert_eq!(editor.minimap.cached_line_count(), editor.buffer.line_count());
}

#[test]
fn test_regression_tab_indentation_consistency() {
    let code = "\t\t\tfoo\n\tlet x = 1;\n\t\t\t\n";
    let buffer = TextBuffer::from_str(code);
    let mut minimap = MinimapState::new();
    minimap.update_from_buffer(&buffer);

    let lines = minimap.cached_lines();
    assert_eq!(lines.len(), 4);

    // Line 0: "\t\t\tfoo" -> 3 tabs = 12 cols indent, total 12 + 3 = 15 cols length
    assert_eq!(lines[0].indent, 12);
    assert_eq!(lines[0].length, 15);
    assert_eq!(lines[0].content_length(), 3);
    assert!(!lines[0].is_empty());
    assert!(lines[0].indent <= lines[0].length);

    // Line 1: "\tlet x = 1;" -> 1 tab = 4 cols indent, total 4 + 10 = 14 cols length
    assert_eq!(lines[1].indent, 4);
    assert_eq!(lines[1].length, 14);
    assert_eq!(lines[1].content_length(), 10);
    assert!(!lines[1].is_empty());

    // Line 2: "\t\t\t" -> 3 tabs whitespace only
    assert_eq!(lines[2].indent, 12);
    assert_eq!(lines[2].length, 12);
    assert_eq!(lines[2].content_length(), 0);
    assert!(lines[2].is_empty());

    // Also test update_from_str
    let mut minimap2 = MinimapState::new();
    minimap2.update_from_str(code);
    let lines2 = minimap2.cached_lines();
    assert_eq!(lines2[0].indent, 12);
    assert_eq!(lines2[0].length, 15);
    assert_eq!(lines2[0].content_length(), 3);
    assert!(!lines2[0].is_empty());
}

#[test]
fn test_regression_short_file_effective_track_height() {
    // 10 lines on a 1000px tall window: natural height = 10 * 3.0 = 30.0px
    let track_h = effective_track_height(10, 1000.0, 3.0);
    assert_eq!(track_h, 30.0);

    // 500 lines on a 1000px window: natural height = 1500px > 1000px -> 1000.0px
    let track_h_large = effective_track_height(500, 1000.0, 3.0);
    assert_eq!(track_h_large, 1000.0);

    // Coordinate translation using effective track height
    // Halfway down 30px track maps to line 5
    assert_eq!(minimap_line_to_editor_line(15.0, 10, track_h), 5);

    // Viewport thumb on 10 lines within 30px track
    let thumb = compute_thumb_rect(0, 5, 10, track_h, 10.0);
    assert_eq!(thumb.y, 0.0);
    assert_eq!(thumb.height, 15.0); // 5/10 of 30px = 15px

    // Thumb at bottom of 10 lines
    let thumb_bottom = compute_thumb_rect(5, 5, 10, track_h, 10.0);
    assert_eq!(thumb_bottom.bottom(), 30.0);
}

#[test]
fn test_regression_minimap_scroll_target_tracking() {
    let mut minimap = MinimapState::new();
    assert_eq!(minimap.scroll_target, None);

    minimap.scroll_target = Some(42);
    assert_eq!(minimap.scroll_target.take(), Some(42));
    assert_eq!(minimap.scroll_target, None);
}
