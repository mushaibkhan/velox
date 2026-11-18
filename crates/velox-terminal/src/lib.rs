pub mod grid;
pub mod manager;
pub mod parser;
pub mod session;

pub use grid::{Cell, Color, TerminalGrid};
pub use manager::{SplitMode, TerminalManager};
pub use session::TerminalSession;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation_and_resize() {
        let mut grid = TerminalGrid::new(80, 24);
        assert_eq!(grid.cols, 80);
        assert_eq!(grid.rows, 24);

        grid.write_char('A');
        assert_eq!(grid.cursor_x, 1);
        assert_eq!(grid.cells[0][0].ch, 'A');

        grid.resize(100, 30);
        assert_eq!(grid.cols, 100);
        assert_eq!(grid.rows, 30);
        assert_eq!(grid.cells[0][0].ch, 'A');
    }
}
