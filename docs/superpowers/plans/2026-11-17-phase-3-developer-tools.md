# Phase 3: Developer Tools Implementation Plan (LSP, Git, Multi-Terminal)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Git repository management & visual diff viewer, asynchronous Language Server Protocol (LSP) client actor, and multi-session tabbed/split terminals in pure Rust for RustForge.

**Architecture:** Pure Rust modular crates. `rustforge-git` handles libgit2 repository management and `similar` line/char diffing. `rustforge-language-server` runs an asynchronous Tokio actor over JSON-RPC 2.0 child process pipes. `rustforge-terminal` and `rustforge-app` provide multi-session tabbed shell execution.

**Tech Stack:** Rust 1.80+, `git2` 0.20+, `similar` 2.6+, `lsp-types` 0.97+, `tokio` 1.40+, `serde_json` 1.0+, `portable-pty` 0.8+, `egui` 0.29.

**Spec:** [phase_3_developer_tools_plan.md](file:///home/musahibkhan/.gemini/antigravity-cli/brain/84cdcd75-763b-4944-88e4-d1c94f1f8160/phase_3_developer_tools_plan.md)

## Global Constraints

- 100% Pure Rust implementation. Zero Node.js or JavaScript dependencies.
- All code must pass `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Non-blocking UI operations: git repository scans, diff calculations, and LSP JSON-RPC message framing must run on Tokio background actors or worker threads. The egui UI loop must never stall.
- Backward compatibility: when a workspace is not a git repository, or when no language server binary is installed on `PATH`, the editor must continue to operate cleanly with Tree-sitter syntax highlighting and standard buffer editing.

---

### Task 1: `rustforge-git` Core Engine (Status, Staging & `similar` Diff)

**Files:**
- Create: `crates/rustforge-git/Cargo.toml`
- Create: `crates/rustforge-git/src/lib.rs`
- Create: `crates/rustforge-git/src/repository.rs`
- Create: `crates/rustforge-git/src/diff.rs`
- Create: `crates/rustforge-git/src/blame.rs`
- Modify: `Cargo.toml` (add `crates/rustforge-git` to workspace members and dependencies)
- Test: `crates/rustforge-git/tests/git_tests.rs`

**Interfaces:**
- Consumes: `std::path::Path`, `git2`, `similar`
- Produces: `GitRepository`, `FileDiff`, `LineChange`, `RepoStatus`, `BranchInfo`, `BlameInfo`
  - `pub fn discover(path: &Path) -> Option<Self>`
  - `pub fn status(&self) -> Result<RepoStatus, GitError>`
  - `pub fn stage_file(&self, path: &Path) -> Result<(), GitError>`
  - `pub fn unstage_file(&self, path: &Path) -> Result<(), GitError>`
  - `pub fn compute_diff(old_text: &str, new_text: &str) -> FileDiff`
  - `pub fn blame_line(&self, path: &Path, line: usize) -> Option<BlameInfo>`

- [ ] **Step 1: Write integration tests in `crates/rustforge-git/tests/git_tests.rs` creating a temp git repo and asserting status, staging, diffing, and blame**
- [ ] **Step 2: Add `rustforge-git` to workspace Cargo.toml and configure crate dependencies**
- [ ] **Step 3: Implement `diff.rs` using `similar` for line-by-line and inline character change detection**
- [ ] **Step 4: Implement `repository.rs` and `blame.rs` wrapping `git2` with clean error handling**
- [ ] **Step 5: Verify tests pass with `cargo test -p rustforge-git`**
- [ ] **Step 6: Run clippy and commit**

---

### Task 2: Editor Git Gutter Markers & Visual Diff Viewer

**Files:**
- Modify: `crates/rustforge-editor/src/state.rs`
- Modify: `crates/rustforge-editor/src/lib.rs`
- Modify: `crates/rustforge-app/src/ui/editor_view.rs`
- Create: `crates/rustforge-app/src/ui/diff_view.rs`
- Test: `crates/rustforge-editor/tests/git_gutter_tests.rs`

**Interfaces:**
- Consumes: `FileDiff`, `LineChange` from `rustforge-git`
- Produces: `EditorState::git_diff`, Gutter change strips, Visual Diff Viewer component
  - Render 3px colored strip on gutter edge: green `+` for added lines, blue `~` for modified, red marker for deletions.
  - Side-by-side or unified diff tab in `diff_view.rs`.

- [ ] **Step 1: Write tests for line change mapping and diff hunk visual indexing**
- [ ] **Step 2: Add `git_diff: Option<FileDiff>` to `EditorState` and wire line change queries**
- [ ] **Step 3: Render gutter status strips in `editor_view.rs` alongside line numbers**
- [ ] **Step 4: Implement `diff_view.rs` for visual file comparison tabs**
- [ ] **Step 5: Verify tests pass with `cargo test -p rustforge-editor`**
- [ ] **Step 6: Run clippy and commit**

---

### Task 3: Interactive Git Sidebar Panel

**Files:**
- Modify: `crates/rustforge-app/src/app.rs`
- Modify: `crates/rustforge-app/src/ui/sidebar.rs`
- Test: `crates/rustforge-app/tests/git_panel_tests.rs`

**Interfaces:**
- Consumes: `GitRepository`, `RepoStatus`
- Produces: Sidebar Git panel:
  - Branch switcher dropdown with ahead/behind counts.
  - Staged Changes list with `[-]` unstage buttons.
  - Changes (Unstaged) list with `[+]` stage buttons and `[↺]` discard buttons.
  - Untracked files list.
  - Commit message text input and `Commit` action (`Ctrl+Enter`).

- [ ] **Step 1: Add Git repository instance and status polling to `RustForgeApp`**
- [ ] **Step 2: Implement Git sidebar view in `sidebar.rs` when `active_sidebar == SidebarTab::Git`**
- [ ] **Step 3: Wire clicking a modified file to open its Visual Diff tab**
- [ ] **Step 4: Verify integration tests pass**
- [ ] **Step 5: Run clippy and commit**

---

### Task 4: `rustforge-language-server` Async JSON-RPC Actor

**Files:**
- Create: `crates/rustforge-language-server/Cargo.toml`
- Create: `crates/rustforge-language-server/src/lib.rs`
- Create: `crates/rustforge-language-server/src/transport.rs`
- Create: `crates/rustforge-language-server/src/client.rs`
- Create: `crates/rustforge-language-server/src/diagnostics.rs`
- Create: `crates/rustforge-language-server/src/manager.rs`
- Modify: `Cargo.toml` (add `crates/rustforge-language-server` to workspace members)
- Test: `crates/rustforge-language-server/tests/lsp_tests.rs`

**Interfaces:**
- Consumes: `lsp-types`, `tokio`, child processes
- Produces: `LspManager`, `LspClient`, `DiagnosticStore`, `DiagnosticItem`
  - Framing: `Content-Length: <N>\r\n\r\n<JSON>`
  - Asynchronous JSON-RPC 2.0 request/response/notification dispatcher.
  - Language server discovery: maps file extension to server executable on `PATH`.
  - Non-blocking diagnostic collection.

- [ ] **Step 1: Write integration tests for LSP transport framing, request-response pairing, and diagnostic storage**
- [ ] **Step 2: Implement `transport.rs` with asynchronous header parsing and framed reading/writing**
- [ ] **Step 3: Implement `client.rs` and `diagnostics.rs` with Tokio actors**
- [ ] **Step 4: Implement `manager.rs` for server discovery and lifecycle orchestration**
- [ ] **Step 5: Verify tests pass with `cargo test -p rustforge-language-server`**
- [ ] **Step 6: Run clippy and commit**

---

### Task 5: Editor LSP Features (Diagnostics, Completion, Hover, Definition, Formatting)

**Files:**
- Modify: `crates/rustforge-editor/src/state.rs`
- Modify: `crates/rustforge-editor/src/lib.rs`
- Modify: `crates/rustforge-app/src/app.rs`
- Modify: `crates/rustforge-app/src/ui/editor_view.rs`
- Test: `crates/rustforge-editor/tests/lsp_editor_tests.rs`

**Interfaces:**
- Consumes: `LspManager`, `DiagnosticStore`, `lsp-types`
- Produces:
  - Gutter diagnostic markers (❌, ⚠, ℹ, 💡) and red/yellow squiggly underlines.
  - Autocomplete suggestion popup (`Ctrl+Space`) with keyboard navigation (`Up`, `Down`, `Enter`, `Tab`, `Esc`).
  - Hover documentation tooltip floating near cursor.
  - Go to Definition command (`F12`).
  - Document formatting command (`Shift+Alt+F`).

- [ ] **Step 1: Write tests for diagnostic span mapping and autocomplete suggestion filtering**
- [ ] **Step 2: Add diagnostic and completion state to `EditorState`**
- [ ] **Step 3: Implement gutter diagnostic icons and squiggly line rendering in `editor_view.rs`**
- [ ] **Step 4: Implement floating completion popup and hover card in `editor_view.rs`**
- [ ] **Step 5: Wire `F12` (definition) and `Shift+Alt+F` (formatting) keybindings**
- [ ] **Step 6: Verify tests pass with `cargo test` and clippy passes cleanly**

---

### Task 6: Multi-Session Tabbed & Split Terminals

**Files:**
- Modify: `crates/rustforge-terminal/src/session.rs`
- Modify: `crates/rustforge-terminal/src/lib.rs`
- Modify: `crates/rustforge-app/src/app.rs`
- Modify: `crates/rustforge-app/src/ui/terminal_view.rs`
- Test: `crates/rustforge-terminal/tests/multi_session_tests.rs`

**Interfaces:**
- Consumes: `portable-pty`, `vte`
- Produces: Multi-session terminal manager:
  - Tab bar: `+` new tab, `×` close tab, tab title (`1: bash`, `2: zsh`).
  - Split terminal mode: side-by-side or stacked terminal panels.
  - Active terminal switching with keyboard shortcut (`Ctrl+Shift+\``).

- [ ] **Step 1: Write tests verifying concurrent multiple PTY sessions with independent grids**
- [ ] **Step 2: Update `RustForgeApp` to manage `terminals: Vec<TerminalSession>` and active tab index**
- [ ] **Step 3: Implement tab bar header in `terminal_view.rs` with `+` and `×` controls**
- [ ] **Step 4: Implement split-terminal rendering in `terminal_view.rs`**
- [ ] **Step 5: Verify all workspace tests pass and clippy clean**
