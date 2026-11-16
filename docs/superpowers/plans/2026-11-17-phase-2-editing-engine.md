# Phase 2: Advanced Editing Engine & Syntax Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Tree-sitter incremental AST parsing, code folding ranges, bracket pair matching, soft word wrap, editor minimap, `notify` filesystem watcher, and 100MB+ large-file benchmarks for RustForge.

**Architecture:** Pure Rust modular crates. `rustforge-syntax` manages ASTs and incremental `InputEdit`s. `rustforge-editor` handles minimap rasterization and soft word wrap. `rustforge-workspace` integrates non-blocking `notify` file monitoring. `rustforge-text-buffer` delivers verified $O(\log N)$ chunked access on large files.

**Tech Stack:** Rust 1.80+, `tree-sitter` 0.22+, `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-json`, `tree-sitter-toml`, `notify` 6.1+, `ropey` 1.6, `egui` 0.28.

**Spec:** [rustforge_implementation_plan.md](file:///home/musahibkhan/.gemini/antigravity-cli/brain/84cdcd75-763b-4944-88e4-d1c94f1f8160/rustforge_implementation_plan.md)

## Global Constraints

- Pure Rust implementation. No Node.js or JavaScript dependencies.
- All code must pass `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
- No blocking UI operations: file watching and heavy parsing must run off the main frame thread or be incremental.
- Backward compatibility: existing token-based highlighting serves as instantaneous fallback if an AST parser is unavailable for a language.

---

### Task 1: Tree-sitter Incremental Parser & AST Folding in `rustforge-syntax`

**Files:**
- Modify: `crates/rustforge-syntax/Cargo.toml`
- Create: `crates/rustforge-syntax/src/tree_sitter_engine.rs`
- Create: `crates/rustforge-syntax/src/folding.rs`
- Modify: `crates/rustforge-syntax/src/lib.rs`
- Test: `crates/rustforge-syntax/tests/tree_sitter_tests.rs`

**Interfaces:**
- Consumes: `Language`, `TokenSpan`, `TokenType` from `rustforge-syntax`
- Produces: `TreeSitterEngine`, `FoldRange`, `BracketMatch`
  - `pub fn parse_incremental(&mut self, source: &str, old_tree: Option<&tree_sitter::Tree>, edit: Option<tree_sitter::InputEdit>) -> Option<tree_sitter::Tree>`
  - `pub fn compute_fold_ranges(&self, tree: &tree_sitter::Tree) -> Vec<FoldRange>`
  - `pub fn find_matching_bracket(&self, source: &str, char_offset: usize) -> Option<usize>`

- [ ] **Step 1: Write integration tests for Tree-sitter parsing, incremental edit, and bracket matching**
- [ ] **Step 2: Add `tree-sitter` and language grammars to `rustforge-syntax/Cargo.toml`**
- [ ] **Step 3: Implement `TreeSitterEngine` and `folding.rs`**
- [ ] **Step 4: Verify tests pass with `cargo test -p rustforge-syntax`**
- [ ] **Step 5: Run clippy and commit**

---

### Task 2: Minimap Gutter & Viewport Tracking in `rustforge-editor`

**Files:**
- Create: `crates/rustforge-editor/src/minimap.rs`
- Modify: `crates/rustforge-editor/src/state.rs`
- Modify: `crates/rustforge-editor/src/lib.rs`
- Modify: `crates/rustforge-app/src/ui/editor_view.rs`
- Test: `crates/rustforge-editor/tests/minimap_tests.rs`

**Interfaces:**
- Consumes: `EditorState`, `TextBuffer`
- Produces: `MinimapState`, `MinimapLine`
  - `pub fn compute_minimap_lines(&self, total_lines: usize) -> Vec<MinimapLine>`
  - `pub fn minimap_line_to_editor_line(minimap_y: f32, total_lines: usize, height: f32) -> usize`

- [ ] **Step 1: Write test for minimap density and scroll coordinate translation**
- [ ] **Step 2: Implement `minimap.rs` with proportional scale and line length caching**
- [ ] **Step 3: Integrate minimap strip rendering into `editor_view.rs` with draggable thumb**
- [ ] **Step 4: Verify tests pass with `cargo test -p rustforge-editor`**
- [ ] **Step 5: Run clippy and commit**

---

### Task 3: Soft Word Wrap Computation in `rustforge-editor`

**Files:**
- Create: `crates/rustforge-editor/src/wrap.rs`
- Modify: `crates/rustforge-editor/src/state.rs`
- Modify: `crates/rustforge-editor/src/lib.rs`
- Modify: `crates/rustforge-app/src/ui/editor_view.rs`
- Test: `crates/rustforge-editor/tests/wrap_tests.rs`

**Interfaces:**
- Consumes: `TextBuffer`, `EditorState`
- Produces: `WrappedLineLayout`
  - `pub fn wrap_line(line_str: &str, max_chars_per_row: usize) -> Vec<std::ops::Range<usize>>`
  - `pub fn visual_row_to_buffer_pos(row: usize, col: usize, layouts: &[WrappedLineLayout]) -> Position`

- [ ] **Step 1: Write tests for line wrapping with word boundaries and fallback character wrap**
- [ ] **Step 2: Implement `wrap.rs`**
- [ ] **Step 3: Add `word_wrap: bool` toggle to `EditorState` and wire into `editor_view.rs`**
- [ ] **Step 4: Verify tests pass with `cargo test -p rustforge-editor`**
- [ ] **Step 5: Run clippy and commit**

---

### Task 4: Filesystem Watcher & Reload Prompt in `rustforge-workspace`

**Files:**
- Modify: `crates/rustforge-workspace/Cargo.toml`
- Create: `crates/rustforge-workspace/src/watcher.rs`
- Modify: `crates/rustforge-workspace/src/lib.rs`
- Modify: `crates/rustforge-app/src/app.rs`
- Test: `crates/rustforge-workspace/tests/watcher_tests.rs`

**Interfaces:**
- Consumes: `std::path::Path`, `notify::Watcher`
- Produces: `FileWatcher`, `FileEvent`
  - `pub enum FileEvent { Modified(PathBuf), Created(PathBuf), Deleted(PathBuf) }`
  - `pub fn start_watching<F: Fn(FileEvent) + Send + 'static>(&mut self, path: &Path, callback: F) -> Result<(), Error>`

- [ ] **Step 1: Write integration test watching temporary directory and receiving create/modify events**
- [ ] **Step 2: Add `notify` dependency to `rustforge-workspace/Cargo.toml`**
- [ ] **Step 3: Implement `FileWatcher` with debounced event channel**
- [ ] **Step 4: Integrate into `RustForgeApp` to notify open tabs when their file changes on disk**
- [ ] **Step 5: Verify tests pass with `cargo test -p rustforge-workspace`**
- [ ] **Step 6: Run clippy and commit**

---

### Task 5: 100MB+ Large File Benchmark in `rustforge-text-buffer`

**Files:**
- Create: `crates/rustforge-text-buffer/benches/large_file.rs`
- Modify: `crates/rustforge-text-buffer/Cargo.toml`
- Modify: `crates/rustforge-text-buffer/src/buffer.rs`
- Test: `crates/rustforge-text-buffer/tests/large_file_stress.rs`

**Interfaces:**
- Consumes: `TextBuffer`, `Rope`
- Produces: Stress test assertions:
  - 100MB+ synthetic buffer creation in < 1.5s
  - Random viewport slicing of 100 lines anywhere in 2,500,000 lines in < 1ms
  - Multi-cursor insertion/deletion on 100 positions simultaneously in < 10ms

- [ ] **Step 1: Write integration stress test `large_file_stress.rs` with 10MB and 100MB synthetic datasets**
- [ ] **Step 2: Optimize rope slicing and chunk iteration if any hot spots detected**
- [ ] **Step 3: Verify all benchmarks pass within target latency budgets**
- [ ] **Step 4: Run clippy and commit**
