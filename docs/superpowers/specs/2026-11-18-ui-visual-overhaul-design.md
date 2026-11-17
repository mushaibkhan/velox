# RustForge UI Visual Overhaul Design Specification

**Date:** 2026-11-18  
**Status:** Approved  
**Topic:** UI Visual Overhaul & "Pure Obsidian" Aesthetic Modernization  
**Target:** `crates/rustforge-themes`, `crates/rustforge-app`

---

## 1. Executive Summary & Goals

### 1.1 Problem Statement
The current RustForge user interface has functional features across editor tabs, sidebar panels, terminal sessions, and Git/LSP integration, but appears visually flat, raw, and unpolished. Specific aesthetic shortcomings include:
1. **Toy-like iconography**: Reliance on system emoji (`📁`, `🔍`, `🌿`, `⌨`, `❌`, `⚠`) which render inconsistently and look amateurish in native desktop windows.
2. **Flat contrast & muddy grays**: Lack of elevation hierarchy (`#181818`, `#252526`), creating muddy borders and indistinct panel transitions.
3. **Legacy chrome accents**: Saturated VS Code bright blue (`#007acc`) on the bottom status bar distracts from code content and clashes with modern dark aesthetics.
4. **Unrefined tab and micro-component styling**: Absence of active tab top indicators, subtle inactive tab recession, crisp breadcrumbs, and floating modal scrims.

### 1.2 Overhaul Goals
* **Elevated Aesthetic Direction**: Implement the "Pure Obsidian" minimalist theme inspired by modern developer tooling (such as Zed), featuring four distinct elevation tiers, crisp 1px borders, and refined cyan (`#38bdf8`) focus indicators.
* **Pure-Rust Vector Icon Engine**: Introduce an in-crate vector painting engine (`rustforge_app::ui::icons`) leveraging `egui::Painter` primitives. Zero external font dependencies, zero glyph atlas lookups, and pixel-perfect sharpness across all DPI scales.
* **Component-Level Polish**: Overhaul the editor Tab Strip (2px cyan top border, smooth dirty-to-close transitions), Activity Bar (2px indicator pills, 48px standard spacing), Breadcrumbs (compact 22px bar with chevron separators), Status Bar (matte `#0e0f12` chrome with discrete pill badges), and Command Palette (floating modal with backdrop scrim).
* **Zero Regressions & Clean Code**: Retain 100% existing IDE functionality, maintain 0 compiler and Clippy warnings under `-D warnings`, and achieve 100% test pass rate across the workspace.

### 1.3 Non-Goals
* Re-architecting state management or splitting/tab models.
* Adding external C/C++ or web runtime dependencies (pure Rust and pure `egui` remain absolute requirements).
* Redesigning syntax highlighting lexers or grammar definitions (theme syntax tokens are updated, but lexer architecture remains untouched).

---

## 2. Architecture & Theme System ("Pure Obsidian")

### 2.1 Theme Elevation Tiers
The theme system in `crates/rustforge-themes/src/theme.rs` is expanded to support `ThemeMode::PureObsidian` (and update default dark values) using a structured 4-tier elevation hierarchy:

| Surface Layer | Hex Code | RGB | Purpose & Usage |
|---|---|---|---|
| **Tier 0: Deep Shell** | `#0e0f12` | `(14, 15, 18)` | Window frame, inactive tab bar, Activity Bar background, Status Bar. |
| **Tier 1: Recessed Panels** | `#121317` | `(18, 19, 23)` | File tree, Git changes panel, Search toolview, Breadcrumbs strip. |
| **Tier 2: Canvas Working Area** | `#15161b` | `(21, 22, 27)` | Editor code canvas, active tab surface, terminal viewport, diff view panels. |
| **Tier 3: Elevated Overlays** | `#1c1d24` | `(28, 29, 36)` | Command Palette modal, context menus, hover tooltips, LSP parameter hints. |

### 2.2 Borders, Dividers & Text Hierarchy
* **Borders / Separators**: `#1e2026` (`rgb(30, 32, 38)`). Replaces thick or muddy gray lines with crisp 1px boundaries.
* **Primary Foreground**: `#e2e8f0` (`rgb(226, 232, 240)`). High-readability crisp off-white for code and active UI text.
* **Secondary Foreground**: `#94a3b8` (`rgb(148, 163, 184)`). Used for inactive tab titles, breadcrumbs, and panel headers.
* **Muted Foreground**: `#64748b` (`rgb(100, 116, 139)`). Line numbers, comment text, and inactive icon states.

### 2.3 Accent & Indicator Tokens
* **Primary Accent (Cyan)**: `#38bdf8` (`rgb(56, 189, 248)`).
  * 2px active tab top border.
  * 2px active Activity Bar left vertical pill.
  * Unsaved buffer dirty indicator dot.
  * Focused input rings and cursor blink.
* **Status Badges**:
  * **Success / Git Added**: `#22c55e` (`rgb(34, 197, 94)`).
  * **Warning / Diagnostics**: `#f59e0b` (`rgb(245, 158, 11)`).
  * **Error / Diagnostics / Git Deleted**: `#ef4444` (`rgb(239, 68, 68)`).
  * **Info / Git Modified**: `#38bdf8` (`rgb(56, 189, 248)`).

---

## 3. Pure-Rust Vector Icon Engine

### 3.1 Module Location & Interface
The icon engine is located in `crates/rustforge-app/src/ui/icons.rs` and exposes an enum `Icon`:

```rust
use egui::{Color32, Painter, Pos2, Rect, Response, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    // Activity Bar Navigation
    Files,
    Search,
    SourceControl,
    Terminal,
    Settings,
    
    // File Tree & File Types
    FolderClosed,
    FolderOpen,
    FileRust,
    FileToml,
    FileMarkdown,
    FileGeneric,
    ChevronRight,
    ChevronDown,
    
    // Controls & State
    Close,
    UnsavedDot,
    SplitHorizontal,
    SplitVertical,
    
    // Git & Status
    GitBranch,
    GitAdded,
    GitModified,
    GitRemoved,
    
    // Diagnostics
    DiagnosticError,
    DiagnosticWarning,
    DiagnosticInfo,
}

impl Icon {
    /// Paint the icon directly onto an egui painter within the designated rect.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32);

    /// Helper to allocate UI space and paint the icon, returning an egui Response.
    pub fn ui(&self, ui: &mut Ui, size: Vec2, color: Color32) -> Response;
}
```

### 3.2 Vector Geometry Primitives
All icons are drawn using normalized unit coordinates `[0.0, 1.0]` and scaled dynamically to target `Rect`:
1. **`Icon::Files`**: Draws a folded document sheet with an outline stroke (1.2px) and a diagonal fold in the top-right corner.
2. **`Icon::Search`**: Circle at center `(0.42, 0.42)` with radius `0.28`, and a 45° angled handle stroke extending to `(0.85, 0.85)` with rounded caps.
3. **`Icon::SourceControl`**: Two filled node circles at `(0.3, 0.3)` and `(0.3, 0.7)` connected by a vertical trunk stroke, with a branch arc diverging to a third node circle at `(0.7, 0.45)`.
4. **`Icon::Terminal`**: Prompt chevron `>` paths on the left, plus a horizontal cursor underline stroke at `(0.55, 0.75)` to `(0.85, 0.75)`.
5. **`Icon::Settings`**: Outer circle with 6 radial tooth trapezoids and a hollow center.
6. **`Icon::Close`**: Two anti-aliased 45° diagonal line segments intersecting at the center of the bounding rect.
7. **`Icon::UnsavedDot`**: Anti-aliased circle with radius equal to `rect.width() * 0.28`, filled with `color`.
8. **`Icon::ChevronRight` / `Icon::ChevronDown`**: Smooth 1.5px angled stroke joints for tree expansion and breadcrumbs.
9. **`Icon::DiagnosticError` / `Warning` / `Info`**: Crisp geometric shapes (circle with cross for error, triangle with exclamation mark for warning, circle with `i` for info).

---

## 4. Component Refinements & Micro-Layouts

### 4.1 Tab Strip & Tabs (`rustforge_app::ui::tabs`)
* **Tab Dimensions & Padding**: Standard height of 34px with 12px horizontal padding.
* **Active Tab Accent**:
  * An `egui::Rect` corresponding to the top 2px of the active tab is filled with `theme.fg_accent` (`#38bdf8`).
  * Tab background equals Tier 2 canvas (`#15161b`).
  * The bottom border of the tab is deliberately omitted to visually merge the active tab with the editor canvas.
* **Inactive Tabs**: Background equals Tier 0 (`#0e0f12`), separated from adjacent tabs by 1px vertical borders (`#1e2026`).
* **Interactive Close & Dirty Handling**:
  * If the buffer is modified, an `Icon::UnsavedDot` is displayed.
  * When hovering the tab or the dirty dot, the dirty dot transitions to `Icon::Close` with a subtle hover circle or rounded rect (`#262833`).

### 4.2 Activity Bar (`rustforge_app::ui::activity_bar`)
* **Width**: Fixed 48px width.
* **Vertical Structure**: Primary tabs (`Files`, `Search`, `SourceControl`, `Terminal`) aligned to the top; utility items (`Settings`) pinned to the bottom.
* **Active State**: The active item displays:
  * An `Icon` rendered in `theme.fg_primary` (`#e2e8f0`).
  * A 2px wide vertical indicator pill in `theme.fg_accent` (`#38bdf8`) pinned to the leftmost edge `[left, top + 8px, left + 2px, bottom - 8px]`.
* **Hover State**: Non-active items hover with a soft rounded background `egui::CornerRadius::same(4)` in `#1a1b23`.

### 4.3 Breadcrumbs Bar (`rustforge_app::ui::breadcrumbs`)
* **Height**: Compact 22px bar immediately above the editor line numbers.
* **Background**: Tier 1 (`#121317`) with a 1px bottom border (`#1e2026`).
* **Layout**: Displays path segments separated by `Icon::ChevronRight` (size: 10x10, color: `#64748b`). The active file segment includes its corresponding `Icon::FileRust` (or relevant file type).

### 4.4 Status Bar (`rustforge_app::ui::status_bar`)
* **Height & Styling**: 22px height, Tier 0 matte dark background (`#0e0f12`), with a 1px top border (`#1e2026`).
* **Left Status Cluster**:
  * Git pill: `Icon::GitBranch` + branch name (e.g., `master`) + concise diff stat badges `(+2 ~1)` in respective theme colors.
  * Read/Write status or line/col summary.
* **Right Status Cluster**:
  * Diagnostic pill: `Icon::DiagnosticError` count + `Icon::DiagnosticWarning` count. Only highlights with red/yellow colors when counts > 0.
  * Editor format metrics: `Ln X, Col Y`, `Spaces: 4`, `UTF-8`, `Rust`.

### 4.5 Command Palette Modal (`rustforge_app::ui::command_palette`)
* **Overlay**: Renders in the front layer with a subtle dark backdrop scrim (`Color32::from_black_alpha(140)`).
* **Modal Window**: Centered, 540px width, rounded corners (8px radius), 1px border (`#2a2c38`), background Tier 3 (`#1c1d24`).
* **Input Box**: Prefix `Icon::Search`, placeholder text, auto-focus, and subtle cyan bottom focus line.
* **Selection Highlighting**: Selected command item highlighted with `#1e293b` background and white text; shortcut chips rendered on the right in muted badges.

---

## 5. Error Handling & Edge Cases

1. **High-DPI / Scale Factors**:
   * All icon vector geometry scales based on `rect.size()`. Painter strokes scale stroke width proportionally to avoid fractional aliasing or hairline blurring at 125%/150%/200% scaling.
2. **Tab Overflow**:
   * Tab strip layout gracefully clips text labels with ellipsis if tab count exceeds window width, and prioritizes active tab visibility.
3. **Missing File Types**:
   * Any unrecognized file extension falls back to `Icon::FileGeneric`.
4. **Zero Diagnostic Count**:
   * Diagnostic indicators display in muted gray (`#64748b`) when error/warning counts are zero to avoid visual noise.

---

## 6. Verification & Testing Strategy

### 6.1 Unit Tests
* **Theme Tests (`rustforge-themes/tests/theme_tests.rs`)**:
  * Verify `Theme::pure_obsidian()` provides valid non-zero alpha colors for all tokens.
  * Verify `ThemeMode::PureObsidian` round-trips correctly through `serde_json`.
  * Ensure color contrast ratio between `bg_editor` (`#15161b`) and `fg_primary` (`#e2e8f0`) meets WCAG AA standards (≥ 4.5:1).
* **Icon Engine Tests (`rustforge-app/tests/icon_tests.rs`)**:
  * Iterate over all `Icon` enum variants and execute `paint()` with arbitrary rect bounds (including zero, small 8x8, standard 16x16, large 64x64) to confirm panic-free execution.
  * Verify bounding box and aspect ratio consistency.

### 6.2 Quality Gates
* `cargo check --workspace --all-targets` succeeds.
* `cargo test --workspace` passes with 100% success.
* `cargo clippy --workspace --all-targets --all-features -- -D warnings` completes with **0 warnings**.
* Visual inspection via `cargo run -p rustforge-app` to verify live aesthetics.
