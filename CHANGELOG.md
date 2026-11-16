# Changelog

All notable changes to **RustForge** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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
