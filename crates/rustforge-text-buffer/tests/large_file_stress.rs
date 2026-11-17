use rustforge_core::Selection;
use rustforge_text_buffer::{Operation, TextBuffer, Transaction};
use std::time::Instant;

fn generate_synthetic_data(num_lines: usize) -> String {
    // Generate lines averaging ~42 bytes per line
    let template_lines = [
        "// Copyright (c) 2026 RustForge Team. All rights reserved.\n",
        "pub fn calculate_metrics_for_block(id: u64, factor: f64) -> Result<f64, Error> {\n",
        "    let mut accumulator: f64 = factor * 1.41421356237;\n",
        "    if id % 2 == 0 {\n",
        "        accumulator += (id as f64).sqrt() * 0.70710678;\n",
        "    } else {\n",
        "        accumulator -= (id as f64).ln_1p() * 0.5;\n",
        "    }\n",
        "    Ok(accumulator.max(0.0))\n",
        "}\n",
    ];

    let pattern_block: String = template_lines.concat();
    let lines_per_block = template_lines.len();
    let num_blocks = num_lines.div_ceil(lines_per_block);

    let mut output = String::with_capacity(pattern_block.len() * num_blocks);
    for _ in 0..num_blocks {
        output.push_str(&pattern_block);
    }
    output
}

#[test]
fn test_10mb_buffer_rapid_editing() {
    // ~250,000 lines -> ~10MB
    let text = generate_synthetic_data(250_000);
    assert!(text.len() >= 9_000_000);

    let t0 = Instant::now();
    let mut buf = TextBuffer::from_str(&text);
    let creation_dur = t0.elapsed();
    println!("10MB buffer creation: {:?}", creation_dur);

    // Rapid editing: 50 insertions and 50 deletions at various offsets
    let t_edit = Instant::now();
    for i in 0..50 {
        let offset = (i * 3000) % buf.total_chars();
        buf.insert(offset, "// rapid edit\n", vec![Selection::point(offset)]);
    }
    for i in 0..50 {
        let offset = (i * 3000) % buf.total_chars();
        buf.delete_range(offset, offset + 5, vec![Selection::point(offset + 5)]);
    }
    let edit_dur = t_edit.elapsed();
    println!("100 rapid edits on 10MB: {:?}", edit_dur);

    // Undo all 100 edits
    let t_undo = Instant::now();
    for _ in 0..100 {
        assert!(buf.undo().is_some());
    }
    let undo_dur = t_undo.elapsed();
    println!("100 undos on 10MB: {:?}", undo_dur);
}

#[test]
fn test_100mb_buffer_stress_and_latency() {
    // 2,700,000 lines -> > 100,000,000 bytes
    let num_lines = 2_700_000;
    let text = generate_synthetic_data(num_lines);
    assert!(text.len() > 100_000_000, "Text must exceed 100MB");
    println!("Generated dataset size: {} bytes, {} lines", text.len(), num_lines);

    let t_create = Instant::now();
    let mut buf = TextBuffer::from_str(&text);
    let create_dur = t_create.elapsed();
    println!("100MB buffer creation: {:?}", create_dur);

    // In release mode, buffer creation must be under 1.5s
    if !cfg!(debug_assertions) {
        assert!(
            create_dur.as_millis() < 1500,
            "100MB buffer creation took {:?}, expected < 1.5s",
            create_dur
        );
    }

    let total_lines = buf.line_count();
    println!("Buffer line count: {}, total chars: {}", total_lines, buf.total_chars());

    // 1. Viewport slicing of 100 lines at fixed and pseudo-random offsets
    let fixed_offsets = [
        ("beginning", 0),
        ("middle", total_lines / 2),
        ("near_end", total_lines.saturating_sub(150)),
    ];

    for (name, start_line) in fixed_offsets {
        let t_slice = Instant::now();
        let slices = buf.viewport_lines(start_line, 100);
        let slice_dur = t_slice.elapsed();
        let total_chars_in_slice: usize = slices.iter().map(|s| s.len_chars()).sum();
        println!(
            "Viewport 100 lines at {} (line {}): {:?} ({} chars, {} slices)",
            name, start_line, slice_dur, total_chars_in_slice, slices.len()
        );
        assert_eq!(slices.len(), 100);

        // Viewport slicing of 100 lines must complete in sub-millisecond time (< 1ms in release)
        let budget_micros = if cfg!(debug_assertions) { 2000 } else { 1000 };
        assert!(
            slice_dur.as_micros() < budget_micros,
            "Viewport slice at {} took {:?}, expected < {}µs",
            name, slice_dur, budget_micros
        );
    }

    // Pseudo-random offsets across the 2,700,000 lines
    let pseudo_random_lines = [
        123_456,
        456_789,
        987_654,
        1_500_000,
        2_100_000,
        2_600_000,
    ];
    for &rand_line in &pseudo_random_lines {
        let t_slice = Instant::now();
        let slices = buf.viewport_lines(rand_line, 100);
        let slice_dur = t_slice.elapsed();
        assert_eq!(slices.len(), 100);
        let budget_micros = if cfg!(debug_assertions) { 2000 } else { 1000 };
        assert!(
            slice_dur.as_micros() < budget_micros,
            "Random viewport slice at line {} took {:?}, expected < {}µs",
            rand_line, slice_dur, budget_micros
        );
    }

    // 2. Multi-cursor edits on 100 positions simultaneously: INSERTIONS
    let num_cursors = 100;
    let spacing = buf.total_chars() / (num_cursors + 1);
    let mut cursor_offsets: Vec<usize> = (1..=num_cursors).map(|i| i * spacing).collect();
    cursor_offsets.sort_by_key(|&pos| std::cmp::Reverse(pos));

    let initial_chars = buf.total_chars();
    let cursors_before: Vec<Selection> = cursor_offsets.iter().map(|&pos| Selection::point(pos)).collect();
    let mut insert_tx = Transaction::new(cursors_before.clone());

    for &pos in &cursor_offsets {
        insert_tx.operations.push(Operation::Insert {
            char_offset: pos,
            text: "/* MC */ ".to_string(),
        });
    }

    let t_multicursor_insert = Instant::now();
    buf.execute_transaction(insert_tx);
    let multicursor_insert_dur = t_multicursor_insert.elapsed();
    println!("Multi-cursor 100 simultaneous inserts on 100MB buffer: {:?}", multicursor_insert_dur);
    let edit_budget_ms = if cfg!(debug_assertions) { 20 } else { 10 };
    assert!(
        multicursor_insert_dur.as_millis() < edit_budget_ms,
        "Multi-cursor inserts took {:?}, expected < {}ms",
        multicursor_insert_dur,
        edit_budget_ms
    );
    assert_eq!(buf.total_chars(), initial_chars + num_cursors * 9);

    // 3. Multi-cursor edits on 100 positions simultaneously: DELETIONS
    let mut delete_tx = Transaction::new(cursors_before.clone());
    for &pos in &cursor_offsets {
        delete_tx.operations.push(Operation::Delete {
            char_offset: pos,
            text: "/* MC */ ".to_string(),
        });
    }

    let t_multicursor_delete = Instant::now();
    buf.execute_transaction(delete_tx);
    let multicursor_delete_dur = t_multicursor_delete.elapsed();
    println!("Multi-cursor 100 simultaneous deletes on 100MB buffer: {:?}", multicursor_delete_dur);
    assert!(
        multicursor_delete_dur.as_millis() < edit_budget_ms,
        "Multi-cursor deletes took {:?}, expected < {}ms",
        multicursor_delete_dur,
        edit_budget_ms
    );
    assert_eq!(buf.total_chars(), initial_chars);

    // 4. Undo/Redo of batch operations via TextBuffer undo/redo
    // Undo deletion -> buffer should have the inserted text back
    let t_undo_delete = Instant::now();
    let res = buf.undo();
    let undo_delete_dur = t_undo_delete.elapsed();
    println!("Undo batch delete on 100MB buffer: {:?}", undo_delete_dur);
    assert!(res.is_some());
    assert_eq!(buf.total_chars(), initial_chars + num_cursors * 9);
    assert!(
        undo_delete_dur.as_millis() < edit_budget_ms,
        "Batch undo took {:?}, expected < {}ms",
        undo_delete_dur,
        edit_budget_ms
    );

    // Undo insertion -> buffer should be back to initial state
    let t_undo_insert = Instant::now();
    let res = buf.undo();
    let undo_insert_dur = t_undo_insert.elapsed();
    println!("Undo batch insert on 100MB buffer: {:?}", undo_insert_dur);
    assert!(res.is_some());
    assert_eq!(buf.total_chars(), initial_chars);
    assert!(
        undo_insert_dur.as_millis() < edit_budget_ms,
        "Batch undo took {:?}, expected < {}ms",
        undo_insert_dur,
        edit_budget_ms
    );

    // Redo insertion
    let t_redo_insert = Instant::now();
    let res = buf.redo();
    let redo_insert_dur = t_redo_insert.elapsed();
    println!("Redo batch insert on 100MB buffer: {:?}", redo_insert_dur);
    assert!(res.is_some());
    assert_eq!(buf.total_chars(), initial_chars + num_cursors * 9);
    assert!(
        redo_insert_dur.as_millis() < edit_budget_ms,
        "Batch redo took {:?}, expected < {}ms",
        redo_insert_dur,
        edit_budget_ms
    );

    // Redo deletion
    let t_redo_delete = Instant::now();
    let res = buf.redo();
    let redo_delete_dur = t_redo_delete.elapsed();
    println!("Redo batch delete on 100MB buffer: {:?}", redo_delete_dur);
    assert!(res.is_some());
    assert_eq!(buf.total_chars(), initial_chars);
    assert!(
        redo_delete_dur.as_millis() < edit_budget_ms,
        "Batch redo took {:?}, expected < {}ms",
        redo_delete_dur,
        edit_budget_ms
    );
}
