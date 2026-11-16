# RustForge

**RustForge** is a next-generation, high-performance, cross-platform source code editor and IDE built in pure Rust.

Inspired by the speed, low latency, and lightweight responsiveness of **Sublime Text** and **Zed**, fused with the rich workspace tools, integrated terminal, and extensible command ecosystem of **VS Code** and **JetBrains IDEs**.

---

## Key Highlights

- **Pure Rust Core & UI**: Built on `eframe` and `egui` with zero JavaScript or embedded webview runtime overhead.
- **Rope-Based Text Buffer**: High-performance text engine built on `ropey::Rope`, supporting $O(\log N)$ edits, unlimited undo/redo history trees, and zero-allocation viewport slicing that effortlessly handles files exceeding 100 MB.
- **Incremental Syntax Highlighting**: Multi-language tokenization supporting Rust, Python, JavaScript/TypeScript, Go, C/C++, JSON, TOML, Markdown, HTML, CSS, Shell, and SQL with theme-aware color mapping.
- **Integrated Native PTY Terminal**: Full pseudo-terminal powered by `portable-pty` and ANSI/VT100 escape parser (`vte`), auto-detecting your host shell (`bash`, `zsh`, `fish`, `powershell`).
- **Interactive File Explorer**: Real filesystem tree viewer with collapsible folders, instant file opening, and icons.
- **Global Workspace Search**: Rapid regex and fuzzy search across project files with live line preview and navigation.
- **Command Palette & Quick Open**: VS Code-style modal palette (`Ctrl+Shift+P` / `Ctrl+P`) with fuzzy filtering and instant keyboard execution.
- **Curated Themes**: Dark Modern (VS Code style), Tokyo Night, and Monokai Pro with instant theme switching.

---

## Workspace Architecture

```
rustforge/
├── Cargo.toml                    # Root workspace manifest
├── crates/
│   ├── rustforge-core/           # Unique IDs, Positions, Actions, Command Registry, Logging
│   ├── rustforge-text-buffer/    # Rope text buffer, transactional undo/redo, multi-cursor
│   ├── rustforge-syntax/         # Syntax highlighting, tokenizers, language definitions
│   ├── rustforge-editor/         # Editor state, viewport slicing, cursors, in-buffer find/replace
│   ├── rustforge-terminal/       # Portable PTY session manager, ANSI grid buffer, vte parser
│   ├── rustforge-workspace/      # Filesystem tree, folder recursion, ignore filter, cache
│   ├── rustforge-themes/         # Dark Modern, Tokyo Night, Monokai Pro theme definitions
│   ├── rustforge-settings/       # Configuration storage and user preferences (TOML/JSON)
│   ├── rustforge-search/         # Workspace directory search & fuzzy match engine
│   └── rustforge-app/            # Main GUI application binary and layout composition
├── CHANGELOG.md                  # Development history and milestone tracking
└── README.md                     # Architecture documentation & guide
```

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+N` | New File |
| `Ctrl+O` | Open File Dialog |
| `Ctrl+S` | Save Active Buffer |
| `Ctrl+Shift+S` | Save Active Buffer As |
| `Ctrl+W` | Close Active Tab |
| `Ctrl+P` | Quick Open File |
| `Ctrl+Shift+P` | Command Palette |
| `Ctrl+B` | Toggle Primary Sidebar |
| `Ctrl+\`` | Toggle Integrated Terminal |
| `Ctrl+F` | In-buffer Find / Replace |
| `Ctrl+Z` | Undo Edit |
| `Ctrl+Y` | Redo Edit |
| `Ctrl+A` | Select All |
| `Ctrl+Q` | Quit Application |

---

## Building & Running

### Prerequisites
- Rust 1.80+ (stable toolchain)
- Graphics driver with OpenGL / Vulkan support

### Build & Check
```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Launch
```bash
cargo run --bin rustforge
```

---

## License
MIT OR Apache-2.0
