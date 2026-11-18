use velox_text_buffer::TextBuffer;

/// A single line's geometry and density metrics for minimap rendering.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MinimapLine {
    /// 0-indexed line number in the document.
    pub line: usize,
    /// Alias for `line` for compatibility.
    pub line_idx: usize,
    /// Leading whitespace character count (spaces or tabs expanded).
    pub indent: usize,
    /// Total character length of the line (excluding newline characters).
    pub length: usize,
    /// Indentation normalized as a ratio between 0.0 and 1.0 against maximum column width.
    pub indent_ratio: f32,
    /// Total length normalized as a ratio between 0.0 and 1.0 against maximum column width.
    pub width_ratio: f32,
}

impl MinimapLine {
    /// Creates a new `MinimapLine` with normalized ratios clamped to 0.0..=1.0.
    pub fn new(line: usize, indent: usize, length: usize, max_cols: usize) -> Self {
        let max_cols_f = (max_cols.max(1)) as f32;
        let indent_ratio = (indent as f32 / max_cols_f).clamp(0.0, 1.0);
        let width_ratio = (length as f32 / max_cols_f).clamp(0.0, 1.0);

        Self {
            line,
            line_idx: line,
            indent,
            length,
            indent_ratio,
            width_ratio,
        }
    }

    /// Returns the length of non-whitespace content in characters.
    pub fn content_length(&self) -> usize {
        self.length.saturating_sub(self.indent)
    }

    /// Returns the normalized width ratio of non-whitespace content.
    pub fn content_width_ratio(&self) -> f32 {
        (self.width_ratio - self.indent_ratio).max(0.0)
    }

    /// Whether this line contains only whitespace or is empty.
    pub fn is_empty(&self) -> bool {
        self.length == 0 || self.indent >= self.length
    }
}

/// Represents the viewport thumb (visible window) within the minimap track.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MinimapThumb {
    /// Top Y position of the thumb in pixels.
    pub y: f32,
    /// Height of the thumb in pixels.
    pub height: f32,
}

impl MinimapThumb {
    pub fn new(y: f32, height: f32) -> Self {
        Self { y, height }
    }

    /// Top coordinate of the thumb.
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Bottom coordinate of the thumb (`y + height`).
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns true if the given pixel Y coordinate falls within the thumb.
    pub fn contains_y(&self, y: f32) -> bool {
        y >= self.y && y <= self.bottom()
    }
}

/// State and cache for the code editor minimap.
#[derive(Debug, Clone, PartialEq)]
pub struct MinimapState {
    /// Whether the minimap gutter is visible.
    pub enabled: bool,
    /// Width of the minimap gutter in pixels (default: 100.0).
    pub width: f32,
    /// Reference maximum column width used for scaling density bars (default: 100).
    pub max_columns: usize,
    /// Minimum height of the viewport thumb in pixels (default: 20.0).
    pub min_thumb_height: f32,
    /// Whether the viewport thumb is actively being dragged.
    pub is_dragging: bool,
    /// Pending scroll target requested by a click or drag on the minimap.
    pub scroll_target: Option<usize>,
    /// Cached line density metrics.
    cached_lines: Vec<MinimapLine>,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self::new()
    }
}

impl MinimapState {
    /// Creates a new `MinimapState` with default settings (100px width, 100 max cols, 20px min thumb).
    pub fn new() -> Self {
        Self {
            enabled: true,
            width: 100.0,
            max_columns: 100,
            min_thumb_height: 20.0,
            is_dragging: false,
            scroll_target: None,
            cached_lines: Vec::new(),
        }
    }

    /// Configures the minimap gutter width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Configures the reference maximum columns.
    pub fn with_max_columns(mut self, max_columns: usize) -> Self {
        self.max_columns = max_columns;
        self
    }

    /// Returns the number of cached lines.
    pub fn cached_line_count(&self) -> usize {
        self.cached_lines.len()
    }

    /// Returns an immutable slice of the cached line metrics.
    pub fn cached_lines(&self) -> &[MinimapLine] {
        &self.cached_lines
    }

    /// Computes or retrieves minimap line metrics for the given total_lines count.
    /// Uses cached lines where available, filling any remaining indices with empty placeholders.
    pub fn compute_minimap_lines(&self, total_lines: usize) -> Vec<MinimapLine> {
        let mut lines = Vec::with_capacity(total_lines);
        for i in 0..total_lines {
            if let Some(cached) = self.cached_lines.get(i) {
                lines.push(cached.clone());
            } else {
                lines.push(MinimapLine::new(i, 0, 0, self.max_columns));
            }
        }
        lines
    }

    /// Updates cached line metrics from a `TextBuffer`.
    pub fn update_from_buffer(&mut self, buffer: &TextBuffer) {
        let total_lines = buffer.line_count();
        self.cached_lines.clear();
        self.cached_lines.reserve(total_lines);

        let rope = buffer.rope();
        for (i, slice) in rope.lines().enumerate().take(total_lines) {
            let mut indent = 0;
            let mut length = 0;
            let mut counting_indent = true;

            for ch in slice.chars() {
                if ch == '\r' || ch == '\n' {
                    break;
                }
                let char_cols = if ch == '\t' { 4 } else { 1 };
                length += char_cols;
                if counting_indent {
                    if ch == ' ' || ch == '\t' {
                        indent += char_cols;
                    } else {
                        counting_indent = false;
                    }
                }
            }

            self.cached_lines.push(MinimapLine::new(i, indent, length, self.max_columns));
        }
    }

    /// Updates cached line metrics from a multi-line string.
    pub fn update_from_str(&mut self, text: &str) {
        let lines: Vec<&str> = text.split('\n').collect();
        self.update_from_lines(lines);
    }

    /// Updates cached line metrics from an iterator of line string slices.
    pub fn update_from_lines<'a, I>(&mut self, lines: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        self.cached_lines.clear();
        for (i, line_str) in lines.into_iter().enumerate() {
            let clean = line_str.trim_end_matches(['\r', '\n']);
            let mut indent = 0;
            let mut length = 0;
            let mut counting = true;
            for ch in clean.chars() {
                let char_cols = if ch == '\t' { 4 } else { 1 };
                length += char_cols;
                if counting {
                    if ch == ' ' || ch == '\t' {
                        indent += char_cols;
                    } else {
                        counting = false;
                    }
                }
            }
            self.cached_lines.push(MinimapLine::new(i, indent, length, self.max_columns));
        }
    }

    /// Returns the effective track height for the minimap content.
    pub fn effective_height(&self, total_lines: usize, available_height: f32) -> f32 {
        effective_track_height(total_lines, available_height, 3.0)
    }

    /// Directly sets or updates metrics for a specific line.
    pub fn set_line(&mut self, line: usize, indent: usize, length: usize) {
        if line >= self.cached_lines.len() {
            self.cached_lines.resize(line + 1, MinimapLine::default());
        }
        self.cached_lines[line] = MinimapLine::new(line, indent, length, self.max_columns);
    }

    /// Calculates the viewport thumb rectangle.
    pub fn compute_thumb(
        &self,
        scroll_line: usize,
        visible_lines: usize,
        total_lines: usize,
        height: f32,
    ) -> MinimapThumb {
        compute_thumb_rect(scroll_line, visible_lines, total_lines, height, self.min_thumb_height)
    }

    /// Translates minimap Y to editor line index.
    pub fn minimap_line_to_editor_line(&self, minimap_y: f32, total_lines: usize, height: f32) -> usize {
        minimap_line_to_editor_line(minimap_y, total_lines, height)
    }

    /// Translates minimap Y to editor line index.
    pub fn minimap_y_to_editor_line(&self, minimap_y: f32, total_lines: usize, height: f32) -> usize {
        minimap_y_to_editor_line(minimap_y, total_lines, height)
    }

    /// Translates editor line to minimap Y.
    pub fn editor_line_to_minimap_y(&self, line: usize, total_lines: usize, height: f32) -> f32 {
        editor_line_to_minimap_y(line, total_lines, height)
    }

    /// Translates thumb top-Y to scroll line.
    pub fn thumb_y_to_scroll_line(
        &self,
        thumb_y: f32,
        thumb_height: f32,
        total_lines: usize,
        visible_lines: usize,
        height: f32,
    ) -> usize {
        thumb_y_to_scroll_line(thumb_y, thumb_height, total_lines, visible_lines, height)
    }
}

// ---------------------------------------------------------------------------
// Standalone geometry & coordinate translation functions
// ---------------------------------------------------------------------------

/// Calculates the effective track height for the minimap content.
///
/// When the document has few lines such that `total_lines * line_spacing < available_height`,
/// the effective track height is bounded to the natural content height (at least 20px) so
/// lines, thumb geometry, and mouse coordinate translations remain perfectly synchronized.
pub fn effective_track_height(total_lines: usize, available_height: f32, line_spacing: f32) -> f32 {
    if total_lines == 0 || available_height <= 0.0 {
        return 0.0;
    }
    let natural_height = total_lines as f32 * line_spacing;
    if natural_height < available_height {
        natural_height.max(20.0).min(available_height)
    } else {
        available_height
    }
}

/// Translates a vertical pixel coordinate in the minimap to a buffer line index.
pub fn minimap_line_to_editor_line(minimap_y: f32, total_lines: usize, height: f32) -> usize {
    if total_lines == 0 || height <= 0.0 {
        return 0;
    }
    let ratio = (minimap_y / height).clamp(0.0, 1.0);
    let line = (ratio * total_lines as f32).floor() as usize;
    line.min(total_lines.saturating_sub(1))
}

/// Alias for `minimap_line_to_editor_line`.
pub fn minimap_y_to_editor_line(minimap_y: f32, total_lines: usize, height: f32) -> usize {
    minimap_line_to_editor_line(minimap_y, total_lines, height)
}

/// Translates an editor line index to a vertical pixel coordinate in the minimap.
pub fn editor_line_to_minimap_y(line: usize, total_lines: usize, height: f32) -> f32 {
    if total_lines <= 1 || height <= 0.0 {
        return 0.0;
    }
    let clamped_line = line.min(total_lines.saturating_sub(1));
    (clamped_line as f32 / total_lines as f32) * height
}

/// Calculates the viewport thumb rectangle in the minimap track.
pub fn compute_thumb_rect(
    scroll_line: usize,
    visible_lines: usize,
    total_lines: usize,
    height: f32,
    min_thumb_height: f32,
) -> MinimapThumb {
    if total_lines == 0 || height <= 0.0 {
        return MinimapThumb::new(0.0, height.max(0.0));
    }
    if total_lines <= visible_lines {
        return MinimapThumb::new(0.0, height);
    }

    let line_scale = height / total_lines as f32;
    let raw_thumb_height = visible_lines as f32 * line_scale;
    let thumb_height = raw_thumb_height.max(min_thumb_height).min(height);

    let max_scroll = total_lines.saturating_sub(visible_lines);
    let travel_height = (height - thumb_height).max(0.0);

    let thumb_y = if max_scroll > 0 {
        let scroll_ratio = (scroll_line as f32 / max_scroll as f32).clamp(0.0, 1.0);
        scroll_ratio * travel_height
    } else {
        0.0
    };

    MinimapThumb::new(thumb_y, thumb_height)
}

/// Translates the top-Y coordinate of a dragged thumb to the target `scroll_line`.
pub fn thumb_y_to_scroll_line(
    thumb_y: f32,
    thumb_height: f32,
    total_lines: usize,
    visible_lines: usize,
    height: f32,
) -> usize {
    if total_lines <= visible_lines || height <= thumb_height {
        return 0;
    }
    let travel_height = height - thumb_height;
    if travel_height <= 0.0 {
        return 0;
    }
    let ratio = (thumb_y / travel_height).clamp(0.0, 1.0);
    let max_scroll = total_lines.saturating_sub(visible_lines);
    let scroll_line = (ratio * max_scroll as f32).round() as usize;
    scroll_line.min(max_scroll)
}

/// Translates a mouse click in the minimap to a centered `scroll_line`.
pub fn click_to_scroll_line(
    click_y: f32,
    total_lines: usize,
    visible_lines: usize,
    height: f32,
) -> usize {
    if total_lines <= visible_lines || height <= 0.0 {
        return 0;
    }
    let target_line = minimap_line_to_editor_line(click_y, total_lines, height);
    let scroll_line = target_line.saturating_sub(visible_lines / 2);
    let max_scroll = total_lines.saturating_sub(visible_lines);
    scroll_line.min(max_scroll)
}
