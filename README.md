# Velox

<p align="center">
  <strong>A swift, modular, cross-platform code editor & IDE in 100% pure Rust.</strong>
</p>

<p align="center">
  <a href="https://github.com/mushaibkhan/velox/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/mushaibkhan/velox/ci.yml?branch=master&style=flat-square&label=CI" alt="CI Status" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square" alt="License" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square" alt="Rust 1.80+" /></a>
  <a href="https://github.com/mushaibkhan/velox/releases"><img src="https://img.shields.io/badge/Release-v0.3.1-emerald.svg?style=flat-square" alt="Version 0.3.1" /></a>
</p>

---

## Overview

**Velox** is an experimental next-generation source code editor and IDE designed for developers who demand instant keystroke response, minimal memory overhead, and native desktop performance.

Inspired by the low latency and lightweight footprint of **Sublime Text** and **Zed**, fused with the rich workspace tools, integrated terminal, and extensible command ecosystem of **VS Code**.

Velox contains **zero JavaScript, zero webviews, and zero electron runtime overhead**. Every pixel is rendered natively via GPU hardware acceleration using Rust.

---

## Key Features

### 🚀 Performance & Text Engine
- **Rope-Based Text Buffer**: High-performance text engine built on `ropey::Rope`, supporting $O(\log N)$ edits, transactional undo/redo histories, and zero-allocation viewport slicing that effortlessly handles files exceeding 100 MB.
- **Incremental Syntax Highlighting**: Multi-language tokenization supporting Rust, Python, JavaScript/TypeScript, Go, C/C++, JSON, TOML, Markdown, HTML, CSS, Shell, and SQL with theme-aware color mapping.
- **Word Wrap & Minimap**: Dynamic word wrapping with cached layout structures and an interactive overview minimap with viewport scroll-drag indicator.

### 🎨 "Pure Obsidian" Aesthetic & Vector Icon Engine
- **4-Tier Elevation Hierarchy**: Deep Shell (`#0e0f12`), Recessed Panels (`#121317`), Editor Canvas (`#15161b`), and Overlays (`#1c1d24`) providing crisp visual separation.
- **Pure-Rust Vector Icons**: Custom geometric icons rendered via `egui::Painter` primitives with zero external font file dependencies, sub-pixel High-DPI sharpness, and zero blurry bitmap scaling.
- **Modern Tab Bar & Breadcrumbs**: 2px cyan active tab top accent line, interactive dirty-to-close button transitions, and compact breadcrumbs bar with file-type badges.

### 🌿 Git & Version Control
- **Full Git Integration**: libgit2-backed repository engine (`GitRepository`) supporting status tracking, staging/unstaging, `stage_all`, discard changes, and commit authoring.
- **Upstream Sync & Blame**: Ahead/behind commit counters (`↑2 ↓1`) and line-level blame lookup with human-readable relative timestamps.
- **Visual Diff Viewer**: Dedicated comparison component supporting **Unified** and **Side-by-side** modes with character-level inline span pill highlights and 3px colored gutter strips (`+`, `~`, `-`).

### 🧠 Language Server Protocol (LSP)
- **JSON-RPC 2.0 Client**: Asynchronous client over Tokio child process standard I/O pipes supporting standard language servers (`rust-analyzer`, `pyright`, `typescript-language-server`, `clangd`, `gopls`).
- **In-Editor Intelligence**:
  - Gutter diagnostic badges and squiggly wave underlines.
  - Autocomplete suggestion popup (`Ctrl+Space`) with completion kind icons and prefix replacement.
  - Floating hover documentation card (`Ctrl+I` / `Alt+H`).
  - Go to Definition (`F12`).
  - Document formatting (`Shift+Alt+F`).

### 💻 Multi-Session Tabbed & Split Terminals
- **Integrated PTY Engine**: Full pseudo-terminal powered by `portable-pty` and ANSI/VT100 escape parser (`vte`), auto-detecting your host shell (`bash`, `zsh`, `fish`, `powershell`).
- **Tabs & Splits**: Open multiple terminal sessions, toggle split view (side-by-side or stacked), and route keystrokes directly to the focused pane.

---

## Workspace Architecture

Velox is architected as a modular workspace of 12 decoupled crates:

| Crate | Purpose |
| :--- | :--- |
| [`velox-core`](crates/velox-core) | Core identifiers, positions, commands, action routing, and logging. |
| [`velox-text-buffer`](crates/velox-text-buffer) | Rope-backed text buffer, operations, undo/redo history trees. |
| [`velox-syntax`](crates/velox-syntax) | Tokenizers, language definitions, and syntax highlight themes. |
| [`velox-editor`](crates/velox-editor) | Viewport rendering, cursor navigation, minimap, and gutter state. |
| [`velox-terminal`](crates/velox-terminal) | PTY session manager, ANSI grid buffer, and multi-terminal orchestration. |
| [`velox-workspace`](crates/velox-workspace) | Filesystem directory recursion, file watcher, and cache. |
| [`velox-themes`](crates/velox-themes) | Pure Obsidian, Dark Modern, Tokyo Night, and Monokai Pro themes. |
| [`velox-settings`](crates/velox-settings) | Configuration storage and user preferences (TOML/JSON). |
| [`velox-search`](crates/velox-search) | Workspace regex search & fuzzy matching engine. |
| [`velox-git`](crates/velox-git) | libgit2 repository bindings, diff computation, and blame. |
| [`velox-language-server`](crates/velox-language-server) | LSP client, transport, and diagnostic store. |
| [`velox-app`](crates/velox-app) | Main application entrypoint, egui layout, and UI shell. |

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+P` | Quick Open File |
| `Ctrl+Shift+P` | Command Palette |
| `Ctrl+N` | New File |
| `Ctrl+O` | Open File Dialog |
| `Ctrl+S` | Save Active Buffer |
| `Ctrl+Shift+S` | Save Active Buffer As |
| `Ctrl+W` | Close Active Tab |
| `Ctrl+B` | Toggle Sidebar |
| `Ctrl+Shift+E` | Explorer Panel |
| `Ctrl+Shift+F` | Workspace Search |
| `Ctrl+Shift+G` | Source Control Panel |
| `Ctrl+\`` | Toggle Terminal Panel |
| `Ctrl+Space` | Trigger Autocomplete |
| `Ctrl+I` / `Alt+H` | Show Hover Information |
| `F12` | Go to Definition |
| `Shift+Alt+F` | Format Document |
| `Ctrl+F` | In-buffer Find / Replace |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo |
| `Ctrl+Q` | Quit Velox |

---

## Getting Started

### Prerequisites

- **Rust 1.80+** (stable toolchain)
- **Linux Packages** (if running on Linux):
  ```bash
  sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
  ```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/mushaibkhan/velox.git
cd velox

# Build the project
cargo build --workspace

# Run Velox
cargo run --bin velox
```

### Running Tests

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## License

This project is dual-licensed under either:
* **MIT license** ([LICENSE-MIT](LICENSE-MIT))
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
