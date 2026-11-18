use velox_core::Selection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub fn normalize_selections(selections: &mut Vec<Selection>) {
    if selections.is_empty() {
        selections.push(Selection::point(0));
        return;
    }

    selections.sort_by_key(|s| s.start());

    let mut merged = Vec::with_capacity(selections.len());
    let mut current = selections[0];

    for &next in &selections[1..] {
        if next.start() <= current.end() {
            // Overlapping or touching
            let new_start = current.start();
            let new_end = current.end().max(next.end());
            let head = if current.head >= current.anchor { new_end } else { new_start };
            let anchor = if current.head >= current.anchor { new_start } else { new_end };
            current = Selection::new(anchor, head);
        } else {
            merged.push(current);
            current = next;
        }
    }
    merged.push(current);
    *selections = merged;
}
