pub mod cursor;
pub mod find;
pub mod minimap;
pub mod state;

pub use cursor::{normalize_selections, Direction};
pub use find::FindState;
pub use minimap::{
    click_to_scroll_line, compute_thumb_rect, editor_line_to_minimap_y,
    minimap_line_to_editor_line, minimap_y_to_editor_line, thumb_y_to_scroll_line,
    MinimapLine, MinimapState, MinimapThumb,
};
pub use state::EditorState;

#[cfg(test)]
mod tests {
    use super::*;
    use rustforge_text_buffer::TextBuffer;

    #[test]
    fn test_editor_insertion_and_deletion() {
        let buf = TextBuffer::from_str("Hello");
        let mut editor = EditorState::new(buf);

        editor.insert_text(" World");
        assert_eq!(editor.buffer.slice_to_string(0, editor.buffer.total_chars()), " WorldHello");

        editor.select_all();
        editor.insert_text("RustForge");
        assert_eq!(editor.buffer.slice_to_string(0, editor.buffer.total_chars()), "RustForge");

        editor.backspace();
        assert_eq!(editor.buffer.slice_to_string(0, editor.buffer.total_chars()), "RustForg");
    }

    #[test]
    fn test_editor_movement() {
        let buf = TextBuffer::from_str("Line 1\nLine 2");
        let mut editor = EditorState::new(buf);

        editor.move_cursor(Direction::Right, false);
        assert_eq!(editor.primary_cursor().head, 1);

        editor.move_cursor(Direction::Down, false);
        let pos = editor.primary_position();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
    }
}
