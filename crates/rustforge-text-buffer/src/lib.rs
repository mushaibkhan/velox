pub mod buffer;
pub mod history;
pub mod operation;

pub use buffer::{LineEnding, TextBuffer};
pub use history::History;
pub use operation::{Operation, Transaction};

#[cfg(test)]
mod tests {
    use super::*;
    use rustforge_core::Selection;

    #[test]
    fn test_buffer_insert_and_delete() {
        let mut buf = TextBuffer::from_str("Hello World!");
        assert_eq!(buf.total_chars(), 12);
        assert_eq!(buf.line_count(), 1);

        let new_cursor = buf.insert(5, " Beautiful", vec![Selection::point(5)]);
        assert_eq!(new_cursor, 15);
        assert_eq!(buf.slice_to_string(0, buf.total_chars()), "Hello Beautiful World!");

        let deleted_cursor = buf.delete_range(5, 15, vec![Selection::point(15)]);
        assert_eq!(deleted_cursor, 5);
        assert_eq!(buf.slice_to_string(0, buf.total_chars()), "Hello World!");
    }

    #[test]
    fn test_buffer_undo_redo() {
        let mut buf = TextBuffer::from_str("Initial");
        buf.insert(7, " Edit", vec![Selection::point(7)]);
        assert_eq!(buf.slice_to_string(0, buf.total_chars()), "Initial Edit");

        let undo_cursors = buf.undo();
        assert!(undo_cursors.is_some());
        assert_eq!(buf.slice_to_string(0, buf.total_chars()), "Initial");

        let redo_cursors = buf.redo();
        assert!(redo_cursors.is_some());
        assert_eq!(buf.slice_to_string(0, buf.total_chars()), "Initial Edit");
    }

    #[test]
    fn test_position_conversion() {
        let text = "Line 1\nLine 2 is longer\nLine 3";
        let buf = TextBuffer::from_str(text);

        assert_eq!(buf.line_count(), 3);
        let pos1 = buf.char_to_position(0);
        assert_eq!(pos1.line, 0);
        assert_eq!(pos1.column, 0);

        let pos2 = buf.char_to_position(7);
        assert_eq!(pos2.line, 1);
        assert_eq!(pos2.column, 0);

        let off = buf.position_to_char(rustforge_core::Position::new(1, 4));
        assert_eq!(off, 11);
    }
}
