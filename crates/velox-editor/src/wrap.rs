use velox_core::Position;
use std::ops::Range;

/// Represents the visual wrapped line layout for a physical line in the buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedLineLayout {
    pub line_index: usize,
    pub rows: Vec<Range<usize>>,
}

impl WrappedLineLayout {
    pub fn new(line_index: usize, rows: Vec<Range<usize>>) -> Self {
        Self { line_index, rows }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Computes visual row ranges for a single physical line string given max characters per row.
///
/// Features:
/// - Word boundary wrapping on whitespace and ASCII punctuation.
/// - Fallback to hard character boundary if a single word or token exceeds `max_chars_per_row`.
/// - Empty lines (or trailing newlines) produce a single `0..0` row.
#[allow(clippy::single_range_in_vec_init)]
pub fn wrap_line(line_str: &str, max_chars_per_row: usize) -> Vec<Range<usize>> {
    let clean = line_str.trim_end_matches(['\r', '\n']);
    let chars: Vec<char> = clean.chars().collect();
    let total_chars = chars.len();

    if total_chars == 0 {
        return vec![0..0];
    }

    let max_chars = max_chars_per_row.max(1);
    if total_chars <= max_chars {
        return vec![0..total_chars];
    }

    let mut rows = Vec::new();
    let mut current_start = 0;

    while current_start < total_chars {
        let remaining = total_chars - current_start;
        if remaining <= max_chars {
            rows.push(current_start..total_chars);
            break;
        }

        let limit = current_start + max_chars;
        let mut break_idx = None;
        let mut next_start_idx = None;

        // Search backwards from limit down to current_start + 1 for word or punctuation boundaries
        for i in (current_start + 1..=limit.min(total_chars)).rev() {
            if i < total_chars && chars[i].is_whitespace() {
                let mut word_end = i;
                while word_end > current_start && chars[word_end - 1].is_whitespace() {
                    word_end -= 1;
                }
                if word_end > current_start {
                    let mut next_s = i;
                    while next_s < total_chars && chars[next_s].is_whitespace() {
                        next_s += 1;
                    }
                    break_idx = Some(word_end);
                    next_start_idx = Some(next_s);
                    break;
                }
            } else if chars[i - 1].is_whitespace() {
                let mut word_end = i - 1;
                while word_end > current_start && chars[word_end - 1].is_whitespace() {
                    word_end -= 1;
                }
                if word_end > current_start {
                    let mut next_s = i - 1;
                    while next_s < total_chars && chars[next_s].is_whitespace() {
                        next_s += 1;
                    }
                    break_idx = Some(word_end);
                    next_start_idx = Some(next_s);
                    break;
                }
            } else if chars[i - 1].is_ascii_punctuation() {
                let word_end = i;
                let mut next_s = i;
                while next_s < total_chars && chars[next_s].is_whitespace() {
                    next_s += 1;
                }
                break_idx = Some(word_end);
                next_start_idx = Some(next_s);
                break;
            }
        }

        if let (Some(b_end), Some(n_start)) = (break_idx, next_start_idx) {
            rows.push(current_start..b_end);
            current_start = n_start.max(current_start + 1);
        } else {
            // Fallback to hard char boundary if word exceeds max width
            rows.push(current_start..limit);
            current_start = limit;
        }
    }

    if rows.is_empty() {
        rows.push(0..0);
    }

    rows
}

/// Maps a visual row index and visual column offset back to the buffer `Position`.
pub fn visual_row_to_buffer_pos(
    row: usize,
    col: usize,
    layouts: &[WrappedLineLayout],
) -> Position {
    if layouts.is_empty() {
        return Position::new(0, 0);
    }

    let mut accumulated_rows = 0;
    for layout in layouts {
        let layout_rows = layout.rows.len().max(1);
        if row < accumulated_rows + layout_rows {
            let sub_idx = row - accumulated_rows;
            let line_idx = layout.line_index;
            if let Some(range) = layout.rows.get(sub_idx) {
                let char_col = (range.start + col).min(range.end);
                return Position::new(line_idx, char_col);
            } else {
                return Position::new(line_idx, 0);
            }
        }
        accumulated_rows += layout_rows;
    }

    // Past end: clamp to the last layout's last row
    if let Some(last_layout) = layouts.last() {
        let line_idx = last_layout.line_index;
        if let Some(last_range) = last_layout.rows.last() {
            let char_col = (last_range.start + col).min(last_range.end);
            Position::new(line_idx, char_col)
        } else {
            Position::new(line_idx, 0)
        }
    } else {
        Position::new(0, 0)
    }
}

/// Maps a buffer `Position` (physical line, character column) to its visual row index and column.
pub fn buffer_pos_to_visual_row(
    pos: Position,
    layouts: &[WrappedLineLayout],
) -> (usize, usize) {
    if layouts.is_empty() {
        return (0, 0);
    }

    let mut accumulated_rows = 0;
    for layout in layouts {
        if layout.line_index == pos.line {
            if layout.rows.is_empty() {
                return (accumulated_rows, 0);
            }

            for (sub_idx, range) in layout.rows.iter().enumerate() {
                let is_last_sub = sub_idx + 1 == layout.rows.len();
                if is_last_sub {
                    let max_col = range.end.saturating_sub(range.start);
                    let visual_col = pos.column.saturating_sub(range.start).min(max_col);
                    return (accumulated_rows + sub_idx, visual_col);
                } else {
                    let next_range = &layout.rows[sub_idx + 1];
                    if pos.column < next_range.start {
                        let max_col = range.end.saturating_sub(range.start);
                        let visual_col = pos.column.saturating_sub(range.start).min(max_col);
                        return (accumulated_rows + sub_idx, visual_col);
                    }
                }
            }

            return (accumulated_rows, 0);
        }
        accumulated_rows += layout.rows.len().max(1);
    }

    (accumulated_rows.saturating_sub(1), 0)
}
