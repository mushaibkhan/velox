use velox_core::Selection;
use velox_text_buffer::{Operation, TextBuffer, Transaction};
use std::time::{Duration, Instant};

fn generate_synthetic_data(num_lines: usize) -> String {
    let template_lines = [
        "// Copyright (c) 2026 Velox Team. All rights reserved.\n",
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

struct BenchResult {
    name: &'static str,
    iterations: usize,
    avg: Duration,
    min: Duration,
    max: Duration,
}

impl BenchResult {
    fn print(&self) {
        println!(
            "{:<48} | {:>6} iters | avg: {:>9.3?} | min: {:>9.3?} | max: {:>9.3?}",
            self.name, self.iterations, self.avg, self.min, self.max
        );
    }
}

fn bench_fn<F>(name: &'static str, iterations: usize, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    // Warmup
    f();

    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    let min = *durations.iter().min().unwrap();
    let max = *durations.iter().max().unwrap();
    let total: Duration = durations.iter().sum();
    let avg = total / (iterations as u32);

    BenchResult {
        name,
        iterations,
        avg,
        min,
        max,
    }
}

fn main() {
    println!("================================================================================");
    println!(" Velox Text Buffer - 100MB+ Large File Benchmark Suite");
    println!("================================================================================");

    // 1. Generation & Buffer Creation
    println!("\n[1/5] Benchmarking Buffer Creation...");
    let text_10mb = generate_synthetic_data(250_000);
    println!("Generated 10MB dataset: {} bytes, {} lines", text_10mb.len(), 250_000);

    let res_create_10mb = bench_fn("10MB TextBuffer::from_str", 5, || {
        let _ = TextBuffer::from_str(&text_10mb);
    });
    res_create_10mb.print();

    let text_100mb = generate_synthetic_data(2_700_000);
    println!("Generated 100MB+ dataset: {} bytes, {} lines", text_100mb.len(), 2_700_000);

    let res_create_100mb = bench_fn("100MB+ TextBuffer::from_str", 3, || {
        let _ = TextBuffer::from_str(&text_100mb);
    });
    res_create_100mb.print();

    let mut buf = TextBuffer::from_str(&text_100mb);
    let total_lines = buf.line_count();

    // 2. Viewport Slicing Benchmarks
    println!("\n[2/5] Benchmarking Viewport Slicing (100 lines)...");
    let res_slice_beg = bench_fn("Viewport 100 lines at beginning (line 0)", 100, || {
        let lines = buf.viewport_lines(0, 100);
        assert_eq!(lines.len(), 100);
    });
    res_slice_beg.print();

    let mid_line = total_lines / 2;
    let res_slice_mid = bench_fn("Viewport 100 lines at middle", 100, || {
        let lines = buf.viewport_lines(mid_line, 100);
        assert_eq!(lines.len(), 100);
    });
    res_slice_mid.print();

    let end_line = total_lines.saturating_sub(150);
    let res_slice_end = bench_fn("Viewport 100 lines near end", 100, || {
        let lines = buf.viewport_lines(end_line, 100);
        assert_eq!(lines.len(), 100);
    });
    res_slice_end.print();

    let res_direct_slice = bench_fn("Direct line_slice 100 lines at middle", 100, || {
        for idx in mid_line..mid_line + 100 {
            let _ = buf.line_slice(idx);
        }
    });
    res_direct_slice.print();

    // 3. Multi-cursor Editing (100 positions simultaneously)
    println!("\n[3/5] Benchmarking Multi-Cursor Editing (100 simultaneous positions)...");
    let num_cursors = 100;
    let spacing = buf.total_chars() / (num_cursors + 1);
    let mut cursor_offsets: Vec<usize> = (1..=num_cursors).map(|i| i * spacing).collect();
    cursor_offsets.sort_by_key(|&pos| std::cmp::Reverse(pos));

    let cursors_before: Vec<Selection> = cursor_offsets.iter().map(|&pos| Selection::point(pos)).collect();

    // Insertion
    let make_insert_tx = || {
        let mut tx = Transaction::new(cursors_before.clone());
        for &pos in &cursor_offsets {
            tx.operations.push(Operation::Insert {
                char_offset: pos,
                text: "/* edit */ ".to_string(),
            });
        }
        tx
    };

    let res_mc_insert = bench_fn("Multi-cursor 100 simultaneous inserts (execute_tx)", 20, || {
        let tx = make_insert_tx();
        buf.execute_transaction(tx);
        buf.undo(); // keep buffer state clean
    });
    res_mc_insert.print();

    // Deletion
    let make_delete_tx = || {
        let mut tx = Transaction::new(cursors_before.clone());
        for &pos in &cursor_offsets {
            tx.operations.push(Operation::Delete {
                char_offset: pos,
                text: "/* edit */ ".to_string(),
            });
        }
        tx
    };

    // Apply insert once so we can benchmark multi-cursor delete
    let tx_ins = make_insert_tx();
    buf.execute_transaction(tx_ins);

    let res_mc_delete = bench_fn("Multi-cursor 100 simultaneous deletes (execute_tx)", 20, || {
        let tx = make_delete_tx();
        buf.execute_transaction(tx);
        buf.undo();
    });
    res_mc_delete.print();
    buf.undo(); // revert the initial insert

    // 4. Undo / Redo Latency on 100MB buffer
    println!("\n[4/5] Benchmarking Undo / Redo Operations...");
    let tx = make_insert_tx();
    buf.execute_transaction(tx);

    let res_undo = bench_fn("Undo 100-cursor transaction on 100MB buffer", 50, || {
        buf.undo();
        buf.redo();
    });
    res_undo.print();

    let res_redo = bench_fn("Redo 100-cursor transaction on 100MB buffer", 50, || {
        buf.redo();
        buf.undo();
    });
    res_redo.print();
    buf.undo(); // final cleanup

    // 5. Position <-> Char conversions
    println!("\n[5/5] Benchmarking Coordinate Conversions...");
    let res_pos_to_char = bench_fn("position_to_char at middle line", 1000, || {
        let _ = buf.position_to_char(velox_core::Position::new(mid_line, 15));
    });
    res_pos_to_char.print();

    let res_char_to_pos = bench_fn("char_to_position at middle offset", 1000, || {
        let _ = buf.char_to_position(buf.total_chars() / 2);
    });
    res_char_to_pos.print();

    println!("\n================================================================================");
    println!(" Latency Budget Verification Summary:");
    println!(" - 100MB creation:     {:>9.3?} (budget < 1500ms) -> {}", res_create_100mb.avg, if res_create_100mb.avg < Duration::from_millis(1500) { "PASSED" } else { "FAILED" });
    println!(" - Viewport slicing:    {:>9.3?} (budget < 1.0ms)  -> {}", res_slice_mid.avg, if res_slice_mid.avg < Duration::from_millis(1) { "PASSED" } else { "FAILED" });
    println!(" - Multi-cursor insert: {:>9.3?} (budget < 10.0ms) -> {}", res_mc_insert.avg, if res_mc_insert.avg < Duration::from_millis(10) { "PASSED" } else { "FAILED" });
    println!(" - Multi-cursor delete: {:>9.3?} (budget < 10.0ms) -> {}", res_mc_delete.avg, if res_mc_delete.avg < Duration::from_millis(10) { "PASSED" } else { "FAILED" });
    println!("================================================================================");
}
