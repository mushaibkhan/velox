# Changelog

All notable changes to **RustForge** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.1] - 2026-11-18

### Added (UI Visual Overhaul & Modernization)
- **Pure Obsidian Minimalist Theme (`rustforge-themes`)**:
  - Implemented 4-tier elevation hierarchy (`ThemeMode::PureObsidian`): Tier 0 Deep Shell (`#0e0f12`), Tier 1 Recessed Panels (`#121317`), Tier 2 Working Surfaces (`#15161b`), and Tier 3 Elevated Overlays (`#1c1d24`).
  - Added WCAG AA compliant contrast ratios (≥ 4.5:1) between editor canvas and primary foreground text (`#e2e8f0`).
  - Set `ThemeMode::PureObsidian` as the default theme in `Theme::default()`.
- **Pure-Rust Vector Icon Engine (`rustforge-app::ui::icons`)**:
  - Zero external font files, zero glyph atlas lookups, sub-pixel High-DPI geometric painter engine.
  - Complete icon palette: `Files`, `Search`, `SourceControl`, `Terminal`, `Settings`, `FolderClosed`, `FolderOpen`, `FileRust`, `FileToml`, `FileMarkdown`, `FileGeneric`, `ChevronRight`, `ChevronDown`, `Close`, `UnsavedDot`, `SplitHorizontal`, `SplitVertical`, `GitBranch`, `GitAdded`, `GitModified`, `GitRemoved`, `DiagnosticError`, `DiagnosticWarning`, `DiagnosticInfo`.
  - Replaced all system emoji (`📁`, `🔍`, `🌿`, `⌨`, `❌`, `⚠`) across sidebar tree, navigation buttons, and status bar.
- **Component Polish & Micro-Layout Refinements (`rustforge-app::ui`)**:
  - **Editor Tab Strip**: 2px cyan (`#38bdf8`) active tab top indicator line, seamless bottom border opening into editor canvas, subtle 1px dividers, and hover transition between unsaved dirty dot and `✕` close button.
  - **Breadcrumbs Bar**: Compact 22px bar above editor canvas with `Icon::ChevronRight` separators and file-type badges.
  - **Activity Bar**: 48px fixed-width column with 2px cyan active indicator pill and soft rounded hover background (`#1a1b23`).
  - **Status Bar**: Low-profile `#0e0f12` matte dark chrome with discrete Git branch pills (`↑`/`↓` counters) and LSP diagnostic badges (`✕` error / `⚠` warning).
  - **Command Palette Modal**: 8px rounded window with 1px border (`#2a2c38`), Tier 3 background (`#1c1d24`), `Icon::Search` prefix, and `#1e293b` selection highlights with dimmed backdrop scrim (`Color32::from_black_alpha(140)`).

## [0.3.0] - 2026-11-17

### Added (Milestone 3: Developer Tools)
- **Git Integration & Version Control Subsystem (`rustforge-git`, `rustforge-app`)**:
  - Full libgit2-backed repository engine (`GitRepository`) supporting repository discovery, status categorization (`staged`, `unstaged`, `untracked`, `conflicted`), file staging/unstaging, `stage_all`, discard changes (tracked checkouts and untracked removals), and commit creation with signature fallbacks.
  - Upstream branch tracking and sync metrics (`ahead`/`behind` commit counters).
  - Line-level blame retrieval (`blame_line`) with relative timestamp human-readable formatting.
  - Interactive Source Control sidebar panel (`Ctrl+Shift+G`) with branch switcher, ahead/behind indicators, commit input with `Ctrl+Enter` shortcut, and staged/unstaged/untracked file lists with granular action buttons.
- **Visual Diff Viewer & Gutter Change Strips (`rustforge-git`, `rustforge-editor`, `rustforge-app`)**:
  - Pure-Rust unified and inline diff engine powered by `similar` with character-level `InlineSpan` change highlighting.
  - 3px colored gutter strips on editor margins: Green (`+`) for added lines, Blue (`~`) for modifications, and Red for deleted line markers.
  - Dedicated visual `DiffView` component supporting both **Unified** and **Side-by-side** comparison modes, statistics badges (`+X`, `-Y`, `~Z`), hunk headers (`@@ -old,len +new,len @@`), line background tints, and inline span pill overlays.
  - Seamless diff tab integration (`Tab::new_diff`) allowing instant file comparison when clicking changed files in the sidebar.
- **Language Server Protocol Subsystem (`rustforge-language-server`)**:
  - Asynchronous JSON-RPC 2.0 client actor (`LspClient`) over Tokio child process standard I/O pipes.
  - Streaming `Content-Length: <len>\r\n\r\n` framing parser handling fragmented buffers, multiple concatenated messages, and arbitrary headers.
  - Thread-safe `DiagnosticStore` (`parking_lot::RwLock`) supporting exact path lookups, 0-based line indexing, and multi-line span intersections.
  - Language server discovery orchestrator (`LspManager`) automatically resolving file extensions to binaries on `PATH` (`rust-analyzer`, `pyright`, `typescript-language-server`, `clangd`, `gopls`).
- **Editor LSP Features (`rustforge-editor`, `rustforge-app`)**:
  - Gutter diagnostic severity badges (❌ Error, ⚠ Warning, ℹ Info, 💡 Hint).
  - Diagnostic squiggly wave underlines (`paint_diagnostic_squiggle`) rendered under error/warning ranges.
  - Floating autocomplete suggestion popup (`Ctrl+Space`) with completion kind icons (ƒ function, k keyword, v variable, s struct), keyboard navigation (`Up`/`Down`, `Enter`/`Tab`, `Esc`), and smart word prefix replacement.
  - Floating hover documentation card (`Ctrl+I` / `Alt+H`).
  - Go to Definition command (`F12`) jumping to symbol declaration file and position.
  - Document formatting command (`Shift+Alt+F`) applying LSP `TextEdit`s in reverse line order.
- **Multi-Session Tabbed & Split Terminals (`rustforge-terminal`, `rustforge-app`)**:
  - `TerminalManager` orchestrating concurrent PTY terminal sessions with active index and split state.
  - Terminal tab bar in bottom panel with active/focus indicators, session switching, and per-tab close `✕` buttons.
  - Action buttons: New Session `+`, Toggle Split `◫`, Clear `🗑`, Hide `×`.
  - Side-by-side (Vertical) and stacked (Horizontal) split layout rendering with safe simultaneous mutable session borrowing via `split_at_mut`.
  - Independent PTY resizing and dynamic focus routing ensuring keystrokes deliver directly to the focused terminal pane.
  - Child shell process exit detection with clean tab pruning via `remove_dead_sessions()`.

## [0.2.1] - 2026-11-17

### Fixed
- **Markdown / Symbol Infinite Loop Hang (`rustforge-syntax`)**:
  - Resolved main UI thread freeze when opening `.md` files or lines containing `#` (headings or attributes).
  - Added dedicated Markdown heading recognition in `SyntaxHighlighter`.
  - Added guaranteed forward progress fallback in tokenizer loop ensuring `i` always advances on every iteration regardless of input characters, eliminating infinite loops on unhandled symbols.

## [0.2.0] - 2026-11-17

### Added (Milestone 2: Advanced Editing Engine & Syntax)
- **Tree-sitter Incremental AST Parsing & Code Folding (`rustforge-syntax`)**:
  - Incremental syntax tree parsing via `tree-sitter` for Rust, Python, JSON, and TOML.
  - Fine-grained incremental AST edits (`tree_sitter::InputEdit`) avoiding full document re-parses.
  - AST-driven code folding ranges (`FoldRange`) for functions, blocks, tables, objects, and arrays.
  - Bracket pair matching (`find_matching_bracket`) with string literal, comment (`//`, `/* */`, `#`), and Rust lifetime masking.
  - Seamless non-blocking fallback to token-based highlighting for unsupported grammars.
- **Minimap Gutter & Viewport Tracking (`rustforge-editor`, `rustforge-app`)**:
  - Interactive 100px minimap strip rendering proportional code density bars.
  - Draggable semi-transparent viewport thumb indicating active scroll position with sub-pixel draw culling.
  - Bidirectional click-to-center and drag-to-scroll coordinate translation.
  - Unified track height calculation (`effective_track_height`) eliminating coordinate desynchronization on short files.
  - Tab indentation column normalization ensuring tabbed lines render accurately.
- **Soft Word Wrap Engine (`rustforge-editor`, `rustforge-app`)**:
  - Soft word boundary line wrapping prioritizing whitespace and punctuation runs with hard-character fallback.
  - Bidirectional visual row and buffer position mapping (`visual_row_to_buffer_pos` and `buffer_pos_to_visual_row`).
  - Gutter continuation indicators (`·`) on wrapped sub-lines with physical line number pinning.
  - Layout caching in `EditorState` keyed by column width and buffer mutation flags to eliminate per-frame allocations.
  - Early layout loop termination for off-screen lines.
- **Filesystem Monitoring & Conflict UX (`rustforge-workspace`, `rustforge-app`)**:
  - Asynchronous debounced directory watcher via `notify` v6.1 running on a dedicated worker thread.
  - Event coalescing (50ms debounce window) filtering rapid disk bursts.
  - Path filtering skipping `.git`, `target`, and `node_modules`.
  - Self-save echo suppression (300ms window) preventing false conflict triggers.
  - Non-destructive conflict handling: clean tabs reload automatically; dirty tabs surface an actionable warning banner with "Reload from Disk" vs "Keep Buffer Changes".
  - Transient editor state retention (`tab_size`, `show_find`, `find_state`, `word_wrap`) across disk reloads.
- **100MB+ Large File Performance & Benchmarking (`rustforge-text-buffer`)**:
  - Dedicated stress tests and benchmarks for 10MB and 100MB+ (2.7M lines) buffers.
  - Sub-microsecond viewport slicing (~8.6 µs per 100 lines) via zero-allocation `lines_range` and `viewport_lines`.
  - Sub-millisecond atomic multi-cursor edits (~74 µs for 100 simultaneous positions) via `execute_transaction`.
  - Sample-based line-ending detection and streaming readers (`from_rope`, `from_reader`).
- **Comprehensive Quality Gates**:
  - 53 unit, integration, and stress tests passing across the workspace.
  - Zero warnings under `cargo clippy --workspace --all-targets -- -D warnings`.

## [0.1.0] - 2026-11-17

### Added (Milestone 1: Foundation & Application Shell)
- **Modular Workspace Architecture**:
  - Implemented 10 decoupled crates: `rustforge-core`, `rustforge-text-buffer`, `rustforge-syntax`, `rustforge-editor`, `rustforge-terminal`, `rustforge-workspace`, `rustforge-themes`, `rustforge-settings`, `rustforge-search`, and `rustforge-app`.
- **High-Performance Rope Text Buffer (`rustforge-text-buffer`)**:
  - Built on `ropey::Rope` for $O(\log N)$ insertions, deletions, and viewport slicing.
  - Reversible transaction history supporting coalesced typing undo and redo stacks.
  - Multi-cursor selection arithmetic and normalized cursor merging.
  - Character offset, byte offset, line number, and column coordinate translations.
- **Syntax Highlighting & Language Detection (`rustforge-syntax`)**:
  - Multi-language lexical analysis supporting Rust, Python, JavaScript, TypeScript, Go, C/C++, JSON, TOML, Markdown, HTML, CSS, Shell, and SQL.
  - Theme-based token color classification (Keywords, Functions, Types, Strings, Numbers, Comments, Operators, Punctuation).
- **Decoupled Code Editor Engine (`rustforge-editor`)**:
  - Viewport-based line rendering ensuring sub-millisecond redraws regardless of buffer size.
  - In-buffer Find and Replace engine with regex, case-sensitivity, and match cycling.
  - Navigation controls (Arrows, Home, End, Tab, Backspace, Delete, Select All).
- **Integrated Native PTY Terminal (`rustforge-terminal`)**:
  - Pseudo-terminal session manager built on `portable-pty`.
  - Auto-detection of system shell (`$SHELL`, `bash`, `zsh`, `powershell.exe`).
  - ANSI/VT100 escape parser (`vte`) updating a color grid buffer in background reader threads.
  - Interactive terminal panel supporting shell input, clear screen, and resize sync.
- **Interactive Workspace Tree (`rustforge-workspace`)**:
  - Hierarchical file tree with folder expansion/collapse and file type icons.
  - Automated directory walking with ignore rules (`.git`, `target`, `node_modules`).
- **Global Project Search (`rustforge-search`)**:
  - Ripgrep-inspired file content regex search with line numbers and preview snippet.
  - Substring and fuzzy matching algorithm powering Quick Open (`Ctrl+P`).
- **Command Palette & Quick Open Modals (`rustforge-app`)**:
  - Centered modal dialogs with keyboard navigation (`Up`, `Down`, `Enter`, `Esc`).
  - Global action dispatcher mapped to user commands and shortcuts.
- **Curated Theme Suite (`rustforge-themes`)**:
  - Dark Modern (VS Code aesthetic), Tokyo Night, and Monokai Pro with real-time UI palette switching.
- **Comprehensive Test Suite & Quality Gates**:
  - 100% passing unit tests across all workspace crates.
  - Zero compiler warnings and clean `-D warnings` validation under `cargo clippy`.

### Roadmap / In Progress
- **Phase 2**: Tree-sitter concrete syntax tree integration with incremental parsing and code folding.
- **Phase 3**: Language Server Protocol (LSP) async JSON-RPC client actor over Tokio with diagnostics and code completion.
- **Phase 4**: Debug Adapter Protocol (DAP) client and WASM/WASI plugin sandboxing (`wasmtime`).
- **Phase 5**: Multi-provider AI assistant with local/cloud streaming, repository context, and diff approval gating.
- **Phase 6**: Cross-platform packaging (MSI, AppImage, Flatpak, DMG) and CI/CD pipelines.
