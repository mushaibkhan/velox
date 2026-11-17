use std::collections::HashMap;
use similar::{ChangeTag, DiffOp, InlineChange, TextDiff};

/// The type of change applied to a single line in a diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineChange {
    Added,
    Modified,
    Deleted,
    Unchanged,
}

/// An inline span within a line highlighting character-level modifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineSpan {
    /// 0-based character/byte start offset within the line string.
    pub start: usize,
    /// 0-based character/byte end offset within the line string (exclusive).
    pub end: usize,
    /// The change classification for this span.
    pub change: LineChange,
}

/// A single line in a diff hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// 1-based line number in original/old text (if applicable).
    pub old_line_no: Option<usize>,
    /// 1-based line number in modified/new text (if applicable).
    pub new_line_no: Option<usize>,
    /// Change type of the line.
    pub change: LineChange,
    /// The line text content.
    pub content: String,
    /// Inline highlighted character ranges.
    pub inline_spans: Vec<InlineSpan>,
}

/// A contiguous hunk of differences with context lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    /// 1-based starting line in original text.
    pub old_start: usize,
    /// Number of lines in original text for this hunk.
    pub old_lines: usize,
    /// 1-based starting line in new text.
    pub new_start: usize,
    /// Number of lines in new text for this hunk.
    pub new_lines: usize,
    /// Unified diff hunk header, e.g. "@@ -1,5 +1,6 @@".
    pub header: String,
    /// Lines comprising this diff hunk.
    pub lines: Vec<DiffLine>,
}

/// Full line-by-line diff between two versions of a file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileDiff {
    pub hunks: Vec<DiffHunk>,
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub total_new_lines: usize,
}

impl FileDiff {
    /// Returns true if there are no differences between old and new text.
    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }

    /// Query the change type for a 1-based line number in the new/modified file.
    pub fn line_change_at(&self, new_line_no: usize) -> Option<LineChange> {
        if new_line_no == 0 || new_line_no > self.total_new_lines {
            return None;
        }

        for hunk in &self.hunks {
            for (idx, line) in hunk.lines.iter().enumerate() {
                if line.new_line_no == Some(new_line_no) {
                    if line.change != LineChange::Unchanged {
                        return Some(line.change);
                    }
                    // If this line is Unchanged in content, check if a deletion marker attaches to it:
                    // 1. Preceded by Deleted line(s) in this hunk
                    if idx > 0 && hunk.lines[idx - 1].change == LineChange::Deleted {
                        return Some(LineChange::Deleted);
                    }
                    // 2. Trailing Deleted line(s) in this hunk with no following new line
                    let following_new_in_hunk = hunk.lines[idx + 1..]
                        .iter()
                        .any(|l| l.new_line_no.is_some());
                    if !following_new_in_hunk {
                        let has_trailing_delete = hunk.lines[idx + 1..]
                            .iter()
                            .any(|l| l.change == LineChange::Deleted);
                        if has_trailing_delete {
                            return Some(LineChange::Deleted);
                        }
                    }
                    return Some(LineChange::Unchanged);
                }
            }
        }

        // If it falls within the file and was not modified in any hunk, it is unchanged.
        Some(LineChange::Unchanged)
    }

    /// Returns a map from 1-based line numbers in the new file to non-unchanged `LineChange`s.
    pub fn active_line_changes(&self) -> HashMap<usize, LineChange> {
        let mut map = HashMap::new();
        for line_no in 1..=self.total_new_lines {
            if let Some(change) = self.line_change_at(line_no) {
                if change != LineChange::Unchanged {
                    map.insert(line_no, change);
                }
            }
        }
        map
    }
}

fn extract_inline_change_content(
    change: &InlineChange<'_, str>,
    line_change: LineChange,
    collect_spans: bool,
) -> (String, Vec<InlineSpan>) {
    let mut spans = Vec::new();
    let mut offset = 0;
    let mut content = String::new();

    for (emphasized, text) in change.iter_strings_lossy() {
        let len = text.len();
        if collect_spans && emphasized {
            spans.push(InlineSpan {
                start: offset,
                end: offset + len,
                change: line_change,
            });
        }
        offset += len;
        content.push_str(&text);
    }

    (content, spans)
}

/// Compute a line-by-line diff with inline character-level spans using `similar`.
pub fn compute_diff(old_text: &str, new_text: &str) -> FileDiff {
    let diff = TextDiff::from_lines(old_text, new_text);
    let total_new_lines = new_text.lines().count();

    let mut hunks = Vec::new();
    let mut total_additions = 0;
    let mut total_deletions = 0;
    let mut total_modifications = 0;

    for group in diff.grouped_ops(3) {
        if group.is_empty() {
            continue;
        }

        let first = group.first().unwrap();
        let last = group.last().unwrap();
        let old_start = first.old_range().start + 1;
        let new_start = first.new_range().start + 1;
        let old_end = last.old_range().end;
        let new_end = last.new_range().end;
        let old_lines = old_end.saturating_sub(first.old_range().start);
        let new_lines = new_end.saturating_sub(first.new_range().start);
        let header = format!("@@ -{},{} +{},{} @@", old_start, old_lines, new_start, new_lines);

        let mut hunk_lines = Vec::new();

        for op in &group {
            match op {
                DiffOp::Equal { .. } => {
                    for change in diff.iter_inline_changes(op) {
                        let (content, _) = extract_inline_change_content(&change, LineChange::Unchanged, false);
                        hunk_lines.push(DiffLine {
                            old_line_no: change.old_index().map(|i| i + 1),
                            new_line_no: change.new_index().map(|i| i + 1),
                            change: LineChange::Unchanged,
                            content,
                            inline_spans: Vec::new(),
                        });
                    }
                }
                DiffOp::Delete { old_len, .. } => {
                    total_deletions += old_len;
                    for change in diff.iter_inline_changes(op) {
                        let (content, _) = extract_inline_change_content(&change, LineChange::Deleted, false);
                        hunk_lines.push(DiffLine {
                            old_line_no: change.old_index().map(|i| i + 1),
                            new_line_no: None,
                            change: LineChange::Deleted,
                            content,
                            inline_spans: Vec::new(),
                        });
                    }
                }
                DiffOp::Insert { new_len, .. } => {
                    total_additions += new_len;
                    for change in diff.iter_inline_changes(op) {
                        let (content, _) = extract_inline_change_content(&change, LineChange::Added, false);
                        hunk_lines.push(DiffLine {
                            old_line_no: None,
                            new_line_no: change.new_index().map(|i| i + 1),
                            change: LineChange::Added,
                            content,
                            inline_spans: Vec::new(),
                        });
                    }
                }
                DiffOp::Replace {
                    old_len,
                    new_len,
                    ..
                } => {
                    total_deletions += old_len;
                    total_additions += new_len;

                    let mut insert_count = 0;
                    for change in diff.iter_inline_changes(op) {
                        let tag = change.tag();

                        let line_change = match tag {
                            ChangeTag::Delete => LineChange::Deleted,
                            ChangeTag::Insert => {
                                let c = if insert_count < *old_len {
                                    total_modifications += 1;
                                    LineChange::Modified
                                } else {
                                    LineChange::Added
                                };
                                insert_count += 1;
                                c
                            }
                            ChangeTag::Equal => LineChange::Unchanged,
                        };

                        let (content, spans) = extract_inline_change_content(&change, line_change, true);

                        hunk_lines.push(DiffLine {
                            old_line_no: change.old_index().map(|i| i + 1),
                            new_line_no: change.new_index().map(|i| i + 1),
                            change: line_change,
                            content,
                            inline_spans: spans,
                        });
                    }
                }
            }
        }

        hunks.push(DiffHunk {
            old_start,
            old_lines,
            new_start,
            new_lines,
            header,
            lines: hunk_lines,
        });
    }

    FileDiff {
        hunks,
        additions: total_additions,
        deletions: total_deletions,
        modifications: total_modifications,
        total_new_lines,
    }
}
