# Contributing to Velox

Thank you for your interest in contributing to **Velox**!

Velox is built with a focus on speed, low latency, and modular pure-Rust architecture. We welcome contributions that maintain these principles.

---

## Getting Started

### Prerequisites

- **Rust**: 1.80+ (stable toolchain)
- **Linux Packages** (if developing on Linux):
  ```bash
  sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
  ```

### Building the Project

```bash
# Clone the repository
git clone https://github.com/mushaibkhan/velox.git
cd velox

# Build workspace in debug mode
cargo build --workspace

# Run the Velox editor
cargo run --bin velox
```

---

## Development Workflow

### 1. Code Quality Gates

Before submitting a Pull Request, ensure all checks pass locally:

```bash
# Run the entire test suite
cargo test --workspace

# Run Clippy with zero warnings allowed
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format check
cargo fmt --check
```

### 2. Architecture Guidelines

- **Pure Rust Only**: Zero JavaScript, webview, or node dependencies.
- **Performance First**: The editor core utilizes rope data structures (`ropey`) for $O(\log N)$ edits. Viewport slicing and rendering must avoid unneeded allocations.
- **No Clippy Warnings**: All code must compile cleanly with zero warnings under `-D warnings`.

### 3. Submitting Pull Requests

1. Fork the repository and create your feature branch from `master`.
2. Keep commits atomic and use clear, conventional commit messages (`feat: ...`, `fix: ...`, `docs: ...`).
3. Open a Pull Request against the `master` branch and describe the changes made.
