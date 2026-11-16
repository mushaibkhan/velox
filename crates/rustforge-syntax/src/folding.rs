/// AST fold range representation and calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
}

impl FoldRange {
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
        }
    }

    /// Number of lines spanned by this fold range.
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Returns true if the line falls within this fold range.
    pub fn contains_line(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Returns inclusive range of lines spanned by this fold.
    pub fn range(&self) -> std::ops::RangeInclusive<usize> {
        self.start_line..=self.end_line
    }
}

/// Represents a matched pair of brackets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BracketMatch {
    pub open: usize,
    pub close: usize,
}

impl BracketMatch {
    pub fn new(open: usize, close: usize) -> Self {
        Self { open, close }
    }

    /// Returns the start (smaller) character offset.
    pub fn start(&self) -> usize {
        self.open.min(self.close)
    }

    /// Returns the end (larger) character offset.
    pub fn end(&self) -> usize {
        self.open.max(self.close)
    }

    /// Returns the opening bracket character offset.
    pub fn open_pos(&self) -> usize {
        self.open
    }

    /// Returns the closing bracket character offset.
    pub fn close_pos(&self) -> usize {
        self.close
    }
}

/// Computes AST fold ranges for a parsed Tree-sitter tree.
pub fn compute_fold_ranges(tree: &tree_sitter::Tree) -> Vec<FoldRange> {
    let mut folds = Vec::new();
    let mut cursor = tree.walk();
    let root = tree.root_node();
    let root_id = root.id();

    let mut visited_children = false;
    loop {
        let node = cursor.node();
        if !visited_children {
            if node.id() != root_id {
                let start_row = node.start_position().row;
                let end_row = node.end_position().row;
                if end_row > start_row && is_foldable_node(node.kind()) {
                    folds.push(FoldRange::new(start_row, end_row));
                }
            }

            if cursor.goto_first_child() {
                continue;
            }
        }

        if cursor.goto_next_sibling() {
            visited_children = false;
            continue;
        }

        if cursor.goto_parent() {
            visited_children = true;
            continue;
        }

        break;
    }

    // Sort by start_line ascending, then end_line descending (outer ranges first)
    folds.sort_by(|a, b| {
        a.start_line
            .cmp(&b.start_line)
            .then_with(|| b.end_line.cmp(&a.end_line))
    });
    folds.dedup_by(|a, b| a.start_line == b.start_line && a.end_line == b.end_line);

    folds
}

fn is_foldable_node(kind: &str) -> bool {
    // 1. Block and list constructs
    if kind.contains("block")
        || kind.contains("declaration_list")
        || kind.contains("variant_list")
        || kind.ends_with("_list")
        || kind.contains("body")
    {
        return true;
    }

    // 2. Definition and item constructs
    if kind.ends_with("_item")
        || kind.ends_with("_definition")
        || kind.ends_with("_statement")
        || kind.ends_with("_expression")
        || kind.ends_with("_declaration")
    {
        return true;
    }

    // 3. Composite literals & data structures
    matches!(
        kind,
        "object"
            | "array"
            | "table"
            | "inline_table"
            | "dictionary"
            | "list"
            | "set"
            | "tuple"
            | "arguments"
            | "parameters"
            | "struct_expression"
    ) || kind.contains("comment")
}

/// Finds the matching bracket character offset in `source` for the bracket at `char_offset`.
///
/// Ignores brackets that appear inside string literals or comments.
pub fn find_matching_bracket(source: &str, char_offset: usize) -> Option<usize> {
    let chars: Vec<char> = source.chars().collect();
    if char_offset >= chars.len() {
        return None;
    }

    let mask = compute_literal_and_comment_mask(&chars);
    if mask[char_offset] {
        return None;
    }

    let target_ch = chars[char_offset];
    let (match_ch, is_forward) = match target_ch {
        '(' => (')', true),
        '{' => ('}', true),
        '[' => (']', true),
        ')' => ('(', false),
        '}' => ('{', false),
        ']' => ('[', false),
        _ => return None,
    };

    if is_forward {
        let mut depth = 1usize;
        for (i, &ch) in chars.iter().enumerate().skip(char_offset + 1) {
            if mask[i] {
                continue;
            }
            if ch == target_ch {
                depth += 1;
            } else if ch == match_ch {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    } else {
        let mut depth = 1usize;
        for i in (0..char_offset).rev() {
            if mask[i] {
                continue;
            }
            let ch = chars[i];
            if ch == target_ch {
                depth += 1;
            } else if ch == match_ch {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    }

    None
}

/// Returns a `BracketMatch` if `char_offset` points to a bracket with a valid counterpart.
pub fn find_bracket_match(source: &str, char_offset: usize) -> Option<BracketMatch> {
    let matched = find_matching_bracket(source, char_offset)?;
    if char_offset < matched {
        Some(BracketMatch::new(char_offset, matched))
    } else {
        Some(BracketMatch::new(matched, char_offset))
    }
}

/// Computes a boolean mask where `true` means the character at that index
/// is part of a string literal or a comment.
fn compute_literal_and_comment_mask(chars: &[char]) -> Vec<bool> {
    let len = chars.len();
    let mut mask = vec![false; len];

    let mut i = 0;
    while i < len {
        let ch = chars[i];

        // 1. Line comment //
        if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                mask[i] = true;
                i += 1;
            }
            continue;
        }

        // 2. Block comment /* ... */
        if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
            mask[i] = true;
            mask[i + 1] = true;
            i += 2;
            while i < len {
                mask[i] = true;
                if chars[i] == '*' && i + 1 < len && chars[i + 1] == '/' {
                    mask[i + 1] = true;
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // 3. String literal: double quote "
        if ch == '"' {
            mask[i] = true;
            i += 1;
            let mut escaped = false;
            while i < len {
                mask[i] = true;
                let cur = chars[i];
                if escaped {
                    escaped = false;
                } else if cur == '\\' {
                    escaped = true;
                } else if cur == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // 4. Character or single quote literal '
        if ch == '\'' {
            // Check if this single quote is closed on the same line (to distinguish from lifetimes like 'a)
            let mut closed = false;
            let mut j = i + 1;
            let mut esc = false;
            while j < len && chars[j] != '\n' {
                if esc {
                    esc = false;
                } else if chars[j] == '\\' {
                    esc = true;
                } else if chars[j] == '\'' {
                    closed = true;
                    break;
                }
                j += 1;
            }

            if closed {
                mask[i..=j].fill(true);
                i = j + 1;
                continue;
            }
        }

        // 5. Backtick string literal `
        if ch == '`' {
            mask[i] = true;
            i += 1;
            let mut escaped = false;
            while i < len {
                mask[i] = true;
                let cur = chars[i];
                if escaped {
                    escaped = false;
                } else if cur == '\\' {
                    escaped = true;
                } else if cur == '`' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        i += 1;
    }

    mask
}
