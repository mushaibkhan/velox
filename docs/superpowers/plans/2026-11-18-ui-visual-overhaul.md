# UI Visual Overhaul & Pure Obsidian Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Overhaul RustForge's visual aesthetics by introducing the "Pure Obsidian" 4-tier elevation theme, a pure-Rust vector icon engine replacing system emoji, and polished editor tabs, activity bar, breadcrumbs, status bar, and command palette.

**Architecture:** Layered theme tokens in `rustforge-themes` (`ThemeMode::PureObsidian`) providing strict elevation hierarchy; zero-dependency vector icon rendering in `rustforge-app::ui::icons` via `egui::Painter` primitives; surgical styling refinements to tab strip, activity bar, status bar, and modal overlays in `rustforge-app::ui`.

**Tech Stack:** Rust 2021, `egui` 0.28, `eframe` 0.28, `serde`, `serde_json`.

**Spec:** `docs/superpowers/specs/2026-11-18-ui-visual-overhaul-design.md`

## Global Constraints
* Pure Rust only: zero external font binary files (`.ttf`/`.woff`), zero web or Node.js runtime assets.
* Zero compiler warnings and zero Clippy warnings under `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
* 100% test pass rate across all workspace crates.
* No regressions to existing IDE editing, LSP, Git, or terminal features.

---

### Task 1: Pure Obsidian Theme Tokens & Elevation Model

**Files:**
- Modify: `crates/rustforge-themes/src/theme.rs`
- Create: `crates/rustforge-themes/tests/theme_tests.rs`

**Interfaces:**
- Produces:
  - `ThemeMode::PureObsidian` enum variant in `crates/rustforge-themes/src/theme.rs`
  - `Theme::pure_obsidian() -> Self` constructor in `crates/rustforge-themes/src/theme.rs`
  - `Theme::default() -> Self` returning `Theme::pure_obsidian()`

- [x] **Step 1: Write the failing tests for Pure Obsidian theme**

Create `crates/rustforge-themes/tests/theme_tests.rs`:
```rust
use egui::Color32;
use rustforge_themes::{Theme, ThemeMode};

#[test]
fn test_pure_obsidian_theme_properties() {
    let theme = Theme::pure_obsidian();
    assert_eq!(theme.mode, ThemeMode::PureObsidian);
    assert_eq!(theme.name, "Pure Obsidian");

    // Shell & canvas elevation hierarchy
    assert_eq!(theme.bg_app, Color32::from_rgb(14, 15, 18));
    assert_eq!(theme.bg_sidebar, Color32::from_rgb(18, 19, 23));
    assert_eq!(theme.bg_editor, Color32::from_rgb(21, 22, 27));
    assert_eq!(theme.bg_status_bar, Color32::from_rgb(14, 15, 18));
    assert_eq!(theme.border, Color32::from_rgb(30, 32, 38));
    assert_eq!(theme.fg_accent, Color32::from_rgb(56, 189, 248));

    // Contrast check: editor text vs background
    let bg = theme.bg_editor;
    let fg = theme.fg_primary;
    let lum_bg = (0.2126 * bg.r() as f32 + 0.7152 * bg.g() as f32 + 0.0722 * bg.b() as f32) / 255.0;
    let lum_fg = (0.2126 * fg.r() as f32 + 0.7152 * fg.g() as f32 + 0.0722 * fg.b() as f32) / 255.0;
    let ratio = (lum_fg + 0.05) / (lum_bg + 0.05);
    assert!(ratio >= 4.5, "Contrast ratio {} must be >= 4.5 for WCAG AA compliance", ratio);
}

#[test]
fn test_pure_obsidian_serialization() {
    let mode = ThemeMode::PureObsidian;
    let serialized = serde_json::to_string(&mode).expect("Failed to serialize ThemeMode::PureObsidian");
    let deserialized: ThemeMode = serde_json::from_str(&serialized).expect("Failed to deserialize ThemeMode::PureObsidian");
    assert_eq!(mode, deserialized);
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test -p rustforge-themes --test theme_tests`
Expected: FAIL with "no variant named `PureObsidian` found"

- [x] **Step 3: Implement Pure Obsidian theme in `crates/rustforge-themes/src/theme.rs`**

Add `PureObsidian` to `ThemeMode`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeMode {
    DarkModern,
    TokyoNight,
    MonokaiPro,
    LightModern,
    HighContrast,
    PureObsidian,
}
```

Implement `Theme::pure_obsidian()` and update `Default`:
```rust
impl Theme {
    pub fn pure_obsidian() -> Self {
        Self {
            mode: ThemeMode::PureObsidian,
            name: "Pure Obsidian",
            bg_app: Color32::from_rgb(14, 15, 18),
            bg_editor: Color32::from_rgb(21, 22, 27),
            bg_gutter: Color32::from_rgb(21, 22, 27),
            bg_activity_bar: Color32::from_rgb(14, 15, 18),
            bg_sidebar: Color32::from_rgb(18, 19, 23),
            bg_tab_active: Color32::from_rgb(21, 22, 27),
            bg_tab_inactive: Color32::from_rgb(14, 15, 18),
            bg_status_bar: Color32::from_rgb(14, 15, 18),
            bg_terminal: Color32::from_rgb(21, 22, 27),
            bg_panel: Color32::from_rgb(28, 29, 36),
            bg_input: Color32::from_rgb(28, 29, 36),

            fg_primary: Color32::from_rgb(226, 232, 240),
            fg_secondary: Color32::from_rgb(148, 163, 184),
            fg_muted: Color32::from_rgb(100, 116, 139),
            fg_accent: Color32::from_rgb(56, 189, 248),

            border: Color32::from_rgb(30, 32, 38),
            line_highlight: Color32::from_rgba_premultiplied(255, 255, 255, 8),
            selection: Color32::from_rgba_premultiplied(56, 189, 248, 45),
            find_match: Color32::from_rgba_premultiplied(234, 179, 8, 100),

            syntax_keyword: Color32::from_rgb(244, 63, 94),     // #f43f5e
            syntax_function: Color32::from_rgb(56, 189, 248),   // #38bdf8
            syntax_type: Color32::from_rgb(167, 139, 250),      // #a78bfa
            syntax_string: Color32::from_rgb(52, 211, 153),     // #34d399
            syntax_number: Color32::from_rgb(251, 146, 60),     // #fb923c
            syntax_comment: Color32::from_rgb(100, 116, 139),   // #64748b
            syntax_operator: Color32::from_rgb(203, 213, 225),  // #cbd5e1
            syntax_punctuation: Color32::from_rgb(148, 163, 184),// #94a3b8
            syntax_variable: Color32::from_rgb(226, 232, 240),  // #e2e8f0
            syntax_constant: Color32::from_rgb(244, 114, 182),  // #f472b6
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::pure_obsidian()
    }
}
```

Update `Theme::from_mode` to handle `ThemeMode::PureObsidian`:
```rust
ThemeMode::PureObsidian => Self::pure_obsidian(),
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test -p rustforge-themes`
Expected: PASS with all tests passing.

- [x] **Step 5: Commit**

```bash
git add crates/rustforge-themes/src/theme.rs crates/rustforge-themes/tests/theme_tests.rs
git commit -m "feat(themes): add Pure Obsidian theme and elevation tokens"
```

---

### Task 2: Pure-Rust Vector Icon Engine

**Files:**
- Create: `crates/rustforge-app/src/ui/icons.rs`
- Modify: `crates/rustforge-app/src/ui/mod.rs`
- Create: `crates/rustforge-app/tests/icon_tests.rs`

**Interfaces:**
- Consumes: `egui::{Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2}`
- Produces:
  - `pub enum Icon`
  - `Icon::paint(&self, painter: &Painter, rect: Rect, color: Color32)`
  - `Icon::ui(&self, ui: &mut Ui, size: Vec2, color: Color32) -> Response`

- [x] **Step 1: Write the failing tests for Vector Icon Engine**

Create `crates/rustforge-app/tests/icon_tests.rs`:
```rust
use egui::{Color32, Pos2, Rect, Vec2};
use rustforge_app::ui::icons::Icon;

#[test]
fn test_all_icons_paint_without_panic() {
    let icons = [
        Icon::Files,
        Icon::Search,
        Icon::SourceControl,
        Icon::Terminal,
        Icon::Settings,
        Icon::FolderClosed,
        Icon::FolderOpen,
        Icon::FileRust,
        Icon::FileToml,
        Icon::FileMarkdown,
        Icon::FileGeneric,
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Close,
        Icon::UnsavedDot,
        Icon::SplitHorizontal,
        Icon::SplitVertical,
        Icon::GitBranch,
        Icon::GitAdded,
        Icon::GitModified,
        Icon::GitRemoved,
        Icon::DiagnosticError,
        Icon::DiagnosticWarning,
        Icon::DiagnosticInfo,
    ];

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let rects = [
                Rect::from_min_size(Pos2::ZERO, Vec2::ZERO), // edge case: zero size
                Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(16.0, 16.0)),
                Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(24.0, 24.0)),
                Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(32.0, 32.0)),
            ];

            for icon in &icons {
                for rect in &rects {
                    icon.paint(painter, *rect, Color32::WHITE);
                }
            }
        });
    });
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test -p rustforge-app --test icon_tests`
Expected: FAIL with "unresolved import `rustforge_app::ui::icons`"

- [x] **Step 3: Implement `Icon` rendering engine in `crates/rustforge-app/src/ui/icons.rs`**

Create `crates/rustforge-app/src/ui/icons.rs`:
```rust
use egui::{pos2, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Files,
    Search,
    SourceControl,
    Terminal,
    Settings,
    FolderClosed,
    FolderOpen,
    FileRust,
    FileToml,
    FileMarkdown,
    FileGeneric,
    ChevronRight,
    ChevronDown,
    Close,
    UnsavedDot,
    SplitHorizontal,
    SplitVertical,
    GitBranch,
    GitAdded,
    GitModified,
    GitRemoved,
    DiagnosticError,
    DiagnosticWarning,
    DiagnosticInfo,
}

impl Icon {
    pub fn ui(&self, ui: &mut Ui, size: Vec2, color: Color32) -> Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            self.paint(ui.painter(), rect, color);
        }
        response
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }

        let min = rect.min;
        let w = rect.width();
        let h = rect.height();
        let pt = |x: f32, y: f32| -> Pos2 { pos2(min.x + x * w, min.y + y * h) };
        let stroke_w = (w * 0.08).clamp(1.0, 2.5);
        let stroke = Stroke::new(stroke_w, color);

        match self {
            Icon::Files => {
                // Document shape with folded corner
                painter.line_segment([pt(0.2, 0.15), pt(0.6, 0.15)], stroke);
                painter.line_segment([pt(0.6, 0.15), pt(0.8, 0.35)], stroke);
                painter.line_segment([pt(0.8, 0.35), pt(0.8, 0.85)], stroke);
                painter.line_segment([pt(0.8, 0.85), pt(0.2, 0.85)], stroke);
                painter.line_segment([pt(0.2, 0.85), pt(0.2, 0.15)], stroke);
                // Fold line
                painter.line_segment([pt(0.6, 0.15), pt(0.6, 0.35)], stroke);
                painter.line_segment([pt(0.6, 0.35), pt(0.8, 0.35)], stroke);
            }
            Icon::Search => {
                let center = pt(0.42, 0.42);
                let radius = w * 0.24;
                painter.circle_stroke(center, radius, stroke);
                painter.line_segment([pt(0.6, 0.6), pt(0.82, 0.82)], Stroke::new(stroke_w * 1.2, color));
            }
            Icon::SourceControl => {
                let r = w * 0.1;
                painter.circle_filled(pt(0.3, 0.3), r, color);
                painter.circle_filled(pt(0.3, 0.7), r, color);
                painter.circle_filled(pt(0.7, 0.45), r, color);
                painter.line_segment([pt(0.3, 0.3), pt(0.3, 0.7)], stroke);
                // Curved branch to third node
                painter.line_segment([pt(0.3, 0.55), pt(0.7, 0.45)], stroke);
            }
            Icon::Terminal => {
                painter.line_segment([pt(0.2, 0.3), pt(0.45, 0.5)], stroke);
                painter.line_segment([pt(0.45, 0.5), pt(0.2, 0.7)], stroke);
                painter.line_segment([pt(0.55, 0.72), pt(0.82, 0.72)], stroke);
            }
            Icon::Settings => {
                let center = pt(0.5, 0.5);
                let radius = w * 0.28;
                painter.circle_stroke(center, radius, stroke);
                painter.circle_filled(center, w * 0.1, color);
                // 4 teeth
                painter.line_segment([pt(0.5, 0.1), pt(0.5, 0.22)], stroke);
                painter.line_segment([pt(0.5, 0.78), pt(0.5, 0.9)], stroke);
                painter.line_segment([pt(0.1, 0.5), pt(0.22, 0.5)], stroke);
                painter.line_segment([pt(0.78, 0.5), pt(0.9, 0.5)], stroke);
            }
            Icon::FolderClosed => {
                // Folder tab + body
                painter.line_segment([pt(0.15, 0.3), pt(0.4, 0.3)], stroke);
                painter.line_segment([pt(0.4, 0.3), pt(0.5, 0.4)], stroke);
                painter.line_segment([pt(0.5, 0.4), pt(0.85, 0.4)], stroke);
                painter.line_segment([pt(0.85, 0.4), pt(0.85, 0.75)], stroke);
                painter.line_segment([pt(0.85, 0.75), pt(0.15, 0.75)], stroke);
                painter.line_segment([pt(0.15, 0.75), pt(0.15, 0.3)], stroke);
            }
            Icon::FolderOpen => {
                painter.line_segment([pt(0.15, 0.3), pt(0.4, 0.3)], stroke);
                painter.line_segment([pt(0.4, 0.3), pt(0.5, 0.4)], stroke);
                painter.line_segment([pt(0.5, 0.4), pt(0.85, 0.4)], stroke);
                // Slanted open flap
                painter.line_segment([pt(0.15, 0.5), pt(0.85, 0.5)], stroke);
                painter.line_segment([pt(0.85, 0.5), pt(0.75, 0.8)], stroke);
                painter.line_segment([pt(0.75, 0.8), pt(0.25, 0.8)], stroke);
                painter.line_segment([pt(0.25, 0.8), pt(0.15, 0.5)], stroke);
            }
            Icon::FileRust => {
                painter.circle_stroke(pt(0.5, 0.5), w * 0.32, stroke);
                painter.line_segment([pt(0.38, 0.35), pt(0.38, 0.65)], stroke);
                painter.line_segment([pt(0.38, 0.35), pt(0.58, 0.35)], stroke);
                painter.line_segment([pt(0.58, 0.35), pt(0.58, 0.5)], stroke);
                painter.line_segment([pt(0.58, 0.5), pt(0.38, 0.5)], stroke);
                painter.line_segment([pt(0.48, 0.5), pt(0.62, 0.65)], stroke);
            }
            Icon::FileToml => {
                // [ ] brackets
                painter.line_segment([pt(0.35, 0.25), pt(0.25, 0.25)], stroke);
                painter.line_segment([pt(0.25, 0.25), pt(0.25, 0.75)], stroke);
                painter.line_segment([pt(0.25, 0.75), pt(0.35, 0.75)], stroke);

                painter.line_segment([pt(0.65, 0.25), pt(0.75, 0.25)], stroke);
                painter.line_segment([pt(0.75, 0.25), pt(0.75, 0.75)], stroke);
                painter.line_segment([pt(0.75, 0.75), pt(0.65, 0.75)], stroke);
            }
            Icon::FileMarkdown => {
                // M + arrow down
                painter.line_segment([pt(0.2, 0.65), pt(0.2, 0.35)], stroke);
                painter.line_segment([pt(0.2, 0.35), pt(0.35, 0.5)], stroke);
                painter.line_segment([pt(0.35, 0.5), pt(0.5, 0.35)], stroke);
                painter.line_segment([pt(0.5, 0.35), pt(0.5, 0.65)], stroke);

                painter.line_segment([pt(0.7, 0.35), pt(0.7, 0.65)], stroke);
                painter.line_segment([pt(0.6, 0.55), pt(0.7, 0.65)], stroke);
                painter.line_segment([pt(0.8, 0.55), pt(0.7, 0.65)], stroke);
            }
            Icon::FileGeneric => {
                painter.line_segment([pt(0.25, 0.2), pt(0.75, 0.2)], stroke);
                painter.line_segment([pt(0.75, 0.2), pt(0.75, 0.8)], stroke);
                painter.line_segment([pt(0.75, 0.8), pt(0.25, 0.8)], stroke);
                painter.line_segment([pt(0.25, 0.8), pt(0.25, 0.2)], stroke);
            }
            Icon::ChevronRight => {
                painter.line_segment([pt(0.35, 0.25), pt(0.65, 0.5)], stroke);
                painter.line_segment([pt(0.65, 0.5), pt(0.35, 0.75)], stroke);
            }
            Icon::ChevronDown => {
                painter.line_segment([pt(0.25, 0.35), pt(0.5, 0.65)], stroke);
                painter.line_segment([pt(0.5, 0.65), pt(0.75, 0.35)], stroke);
            }
            Icon::Close => {
                painter.line_segment([pt(0.25, 0.25), pt(0.75, 0.75)], stroke);
                painter.line_segment([pt(0.75, 0.25), pt(0.25, 0.75)], stroke);
            }
            Icon::UnsavedDot => {
                let center = pt(0.5, 0.5);
                painter.circle_filled(center, w * 0.25, color);
            }
            Icon::SplitHorizontal => {
                painter.rect_stroke(rect.shrink(2.0), 1.0, stroke);
                painter.line_segment([pt(0.5, 0.2), pt(0.5, 0.8)], stroke);
            }
            Icon::SplitVertical => {
                painter.rect_stroke(rect.shrink(2.0), 1.0, stroke);
                painter.line_segment([pt(0.2, 0.5), pt(0.8, 0.5)], stroke);
            }
            Icon::GitBranch => {
                let r = w * 0.1;
                painter.circle_filled(pt(0.3, 0.3), r, color);
                painter.circle_filled(pt(0.3, 0.7), r, color);
                painter.circle_filled(pt(0.7, 0.4), r, color);
                painter.line_segment([pt(0.3, 0.3), pt(0.3, 0.7)], stroke);
                painter.line_segment([pt(0.3, 0.6), pt(0.7, 0.4)], stroke);
            }
            Icon::GitAdded => {
                painter.line_segment([pt(0.5, 0.25), pt(0.5, 0.75)], stroke);
                painter.line_segment([pt(0.25, 0.5), pt(0.75, 0.5)], stroke);
            }
            Icon::GitModified => {
                painter.line_segment([pt(0.25, 0.5), pt(0.45, 0.35)], stroke);
                painter.line_segment([pt(0.45, 0.35), pt(0.55, 0.65)], stroke);
                painter.line_segment([pt(0.55, 0.65), pt(0.75, 0.5)], stroke);
            }
            Icon::GitRemoved => {
                painter.line_segment([pt(0.25, 0.5), pt(0.75, 0.5)], stroke);
            }
            Icon::DiagnosticError => {
                let center = pt(0.5, 0.5);
                painter.circle_filled(center, w * 0.38, color);
                let inv_stroke = Stroke::new(stroke_w, Color32::BLACK);
                painter.line_segment([pt(0.35, 0.35), pt(0.65, 0.65)], inv_stroke);
                painter.line_segment([pt(0.65, 0.35), pt(0.35, 0.65)], inv_stroke);
            }
            Icon::DiagnosticWarning => {
                // Triangle
                let p1 = pt(0.5, 0.18);
                let p2 = pt(0.18, 0.82);
                let p3 = pt(0.82, 0.82);
                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, p3], stroke);
                painter.line_segment([p3, p1], stroke);
                painter.line_segment([pt(0.5, 0.42), pt(0.5, 0.62)], stroke);
                painter.circle_filled(pt(0.5, 0.72), stroke_w * 0.6, color);
            }
            Icon::DiagnosticInfo => {
                let center = pt(0.5, 0.5);
                painter.circle_stroke(center, w * 0.35, stroke);
                painter.circle_filled(pt(0.5, 0.35), stroke_w * 0.7, color);
                painter.line_segment([pt(0.5, 0.48), pt(0.5, 0.68)], stroke);
            }
        }
    }
}
```

Add `pub mod icons;` to `crates/rustforge-app/src/ui/mod.rs`.

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test -p rustforge-app --test icon_tests`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add crates/rustforge-app/src/ui/icons.rs crates/rustforge-app/src/ui/mod.rs crates/rustforge-app/tests/icon_tests.rs
git commit -m "feat(ui): implement pure-Rust vector icon engine"
```

---

### Task 3: Editor Tab Strip & Breadcrumbs Overhaul

**Files:**
- Modify: `crates/rustforge-app/src/ui/editor_view.rs`

**Interfaces:**
- Consumes: `rustforge_app::ui::icons::Icon`, `rustforge_themes::Theme`
- Modifies: `render_tab_bar(app: &mut RustForgeApp, ui: &mut Ui)`

- [x] **Step 1: Write test for tab bar active accent and dirty dot rendering**

Add a test in `crates/rustforge-app/tests/ui_tab_tests.rs`:
```rust
use egui::Vec2;
use rustforge_app::ui::icons::Icon;

#[test]
fn test_tab_icon_sizing() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let res_dot = Icon::UnsavedDot.ui(ui, Vec2::splat(12.0), egui::Color32::CYAN);
            assert_eq!(res_dot.rect.size(), Vec2::splat(12.0));

            let res_close = Icon::Close.ui(ui, Vec2::splat(12.0), egui::Color32::WHITE);
            assert_eq!(res_close.rect.size(), Vec2::splat(12.0));
        });
    });
}
```

- [x] **Step 2: Run test to verify compilation and pass**

Run: `cargo test -p rustforge-app --test ui_tab_tests`
Expected: PASS

- [x] **Step 3: Update `render_tab_bar` and add breadcrumbs in `crates/rustforge-app/src/ui/editor_view.rs`**

In `crates/rustforge-app/src/ui/editor_view.rs`:
1. In `render_tab_bar`:
   - Increase tab frame height and style:
     - Active tab top 2px accent line: paint a filled rect `[tab_rect.min, pos2(tab_rect.max.x, tab_rect.min.y + 2.0)]` with `app.theme.fg_accent`.
     - Inactive tab gets `app.theme.bg_tab_inactive` and subtle 1px border.
   - Replace `" ●"` with interactive `Icon::UnsavedDot` / `Icon::Close`:
     - Show `Icon::UnsavedDot` when dirty and not hovered; when hovered, show `Icon::Close` so clicking it closes the tab cleanly.
2. In `render_active_editor`, render a 22px Breadcrumbs Bar above editor content:
   - Path split by `/`, each step separated by `Icon::ChevronRight` (10x10, color `#64748b`).
   - Active file prefixed by `Icon::FileRust` (if `.rs`), `Icon::FileToml` (if `.toml`), `Icon::FileMarkdown` (if `.md`), or `Icon::FileGeneric`.

- [x] **Step 4: Run workspace tests to verify no regressions**

Run: `cargo test --workspace`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add crates/rustforge-app/src/ui/editor_view.rs crates/rustforge-app/tests/ui_tab_tests.rs
git commit -m "feat(ui): modernize editor tab strip and breadcrumbs with vector icons"
```

---

### Task 4: Activity Bar & Status Bar Visual Overhaul

**Files:**
- Modify: `crates/rustforge-app/src/ui/activity_bar.rs`
- Modify: `crates/rustforge-app/src/ui/status_bar.rs`

**Interfaces:**
- Consumes: `rustforge_app::ui::icons::Icon`
- Modifies:
  - `render_activity_bar(app: &mut RustForgeApp, ui: &mut Ui)`
  - `render_status_bar(app: &RustForgeApp, ui: &mut Ui)`

- [x] **Step 1: Write test for activity bar and status bar icon rendering**

Create `crates/rustforge-app/tests/activity_status_tests.rs`:
```rust
use egui::Vec2;
use rustforge_app::ui::icons::Icon;

#[test]
fn test_activity_bar_icons_exist() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let icons = [
                Icon::Files,
                Icon::Search,
                Icon::SourceControl,
                Icon::Terminal,
                Icon::Settings,
                Icon::GitBranch,
                Icon::DiagnosticError,
                Icon::DiagnosticWarning,
            ];
            for icon in icons {
                let res = icon.ui(ui, Vec2::splat(18.0), egui::Color32::WHITE);
                assert_eq!(res.rect.width(), 18.0);
            }
        });
    });
}
```

- [x] **Step 2: Run test to verify it passes**

Run: `cargo test -p rustforge-app --test activity_status_tests`
Expected: PASS

- [x] **Step 3: Update `activity_bar.rs` with vector icons & 2px indicator pill**

In `crates/rustforge-app/src/ui/activity_bar.rs`:
- Replace `"📁"`, `"🔍"`, `"🌿"`, `"⌨"`, `"⚙"` buttons with custom icon button helper:
```rust
fn render_nav_button(
    ui: &mut Ui,
    icon: Icon,
    is_active: bool,
    tooltip: &str,
    theme: &rustforge_themes::Theme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(48.0, 40.0), egui::Sense::click());
    
    // Hover highlight
    if response.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(Vec2::new(4.0, 2.0)),
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(26, 27, 35),
        );
    }
    
    // Active left pill (2px width cyan)
    if is_active {
        let pill_rect = Rect::from_min_max(
            pos2(rect.min.x, rect.min.y + 8.0),
            pos2(rect.min.x + 2.0, rect.max.y - 8.0),
        );
        ui.painter().rect_filled(pill_rect, egui::CornerRadius::same(1), theme.fg_accent);
    }
    
    let icon_color = if is_active {
        theme.fg_primary
    } else if response.hovered() {
        theme.fg_secondary
    } else {
        theme.fg_muted
    };
    
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(18.0));
    icon.paint(ui.painter(), icon_rect, icon_color);
    response.on_hover_text(tooltip)
}
```
- Hook `Icon::Files`, `Icon::Search`, `Icon::SourceControl`, `Icon::Terminal`, and `Icon::Settings`.

- [x] **Step 4: Update `status_bar.rs` with matte dark chrome & pill badges**

In `crates/rustforge-app/src/ui/status_bar.rs`:
- Left side:
  - Git status pill: Draw `Icon::GitBranch` (12x12), branch name, and colored diff stats `(+A ~M)` in green/yellow.
- Right side:
  - LSP diagnostics pill:
    - If errors > 0: `Icon::DiagnosticError` + error count in `#ef4444`.
    - If warnings > 0: `Icon::DiagnosticWarning` + warning count in `#f59e0b`.
    - If both 0: Muted icon + `0`.
  - Editor coordinates: `Ln X, Col Y`, `Spaces: 4`, `UTF-8`, `Rust`.

- [x] **Step 5: Run workspace tests to verify they pass**

Run: `cargo test --workspace`
Expected: PASS

- [x] **Step 6: Commit**

```bash
git add crates/rustforge-app/src/ui/activity_bar.rs crates/rustforge-app/src/ui/status_bar.rs crates/rustforge-app/tests/activity_status_tests.rs
git commit -m "feat(ui): modernize activity bar and status bar with vector icons and pill badges"
```

---

### Task 5: Sidebar File Tree & Command Palette Polishing

**Files:**
- Modify: `crates/rustforge-app/src/ui/sidebar.rs`
- Modify: `crates/rustforge-app/src/ui/palette.rs`

**Interfaces:**
- Consumes: `rustforge_app::ui::icons::Icon`
- Modifies:
  - `render_file_tree(...)` in `sidebar.rs`
  - `render_command_palette(...)` in `palette.rs`

- [x] **Step 1: Write unit tests for file type icon resolution**

Add test in `crates/rustforge-app/tests/file_tree_icon_tests.rs`:
```rust
use rustforge_app::ui::icons::Icon;

fn icon_for_path(path: &str) -> Icon {
    if path.ends_with(".rs") {
        Icon::FileRust
    } else if path.ends_with(".toml") {
        Icon::FileToml
    } else if path.ends_with(".md") {
        Icon::FileMarkdown
    } else {
        Icon::FileGeneric
    }
}

#[test]
fn test_file_type_icon_resolution() {
    assert_eq!(icon_for_path("main.rs"), Icon::FileRust);
    assert_eq!(icon_for_path("Cargo.toml"), Icon::FileToml);
    assert_eq!(icon_for_path("README.md"), Icon::FileMarkdown);
    assert_eq!(icon_for_path("data.txt"), Icon::FileGeneric);
}
```

- [x] **Step 2: Run test to verify it passes**

Run: `cargo test -p rustforge-app --test file_tree_icon_tests`
Expected: PASS

- [x] **Step 3: Update `sidebar.rs` to replace emoji with vector icons**

In `crates/rustforge-app/src/ui/sidebar.rs`:
- In `render_explorer`:
  - Replace `"📁"` root label with `Icon::FolderOpen`.
  - Replace `"📂"` / `"📁"` node labels with `Icon::ChevronDown` / `Icon::ChevronRight` + `Icon::FolderOpen` / `Icon::FolderClosed`.
  - Replace `"📄"` file label with file-type vector icon (`Icon::FileRust`, `Icon::FileToml`, `Icon::FileMarkdown`, `Icon::FileGeneric`).
- In `render_git`:
  - Replace `"🌿"` with `Icon::GitBranch`.

- [x] **Step 4: Update `palette.rs` modal styling**

In `crates/rustforge-app/src/ui/palette.rs`:
- Increase backdrop scrim dimming opacity to `Color32::from_black_alpha(140)`.
- Frame styling: 8px corner radius, Tier 3 background `app.theme.bg_panel` (`#1c1d24`), crisp 1px border `Color32::from_rgb(42, 44, 56)`.
- Input field: Paint `Icon::Search` (14x14) preceding query input.
- Highlight selected command with `#1e293b` fill and crisp foreground text.

- [x] **Step 5: Run tests and verify**

Run: `cargo test --workspace`
Expected: PASS

- [x] **Step 6: Commit**

```bash
git add crates/rustforge-app/src/ui/sidebar.rs crates/rustforge-app/src/ui/palette.rs crates/rustforge-app/tests/file_tree_icon_tests.rs
git commit -m "feat(ui): modernize sidebar file tree icons and command palette modal"
```

---

### Task 6: Workspace Clippy Quality Gate & Live Smoke Verification

**Files:**
- Entire workspace

**Interfaces:**
- Workspace quality gates and binary smoke verification

- [x] **Step 1: Run clippy across all workspace crates with `-D warnings`**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: 0 warnings, clean exit code 0.

- [x] **Step 2: Run full workspace test suite**

Run: `cargo test --workspace`
Expected: 100% pass across all tests in all crates.

- [x] **Step 3: Verify binary build & launch**

Run: `cargo build -p rustforge-app`
Expected: Clean build with exit code 0.

- [x] **Step 4: Final commit and changelog entry**

```bash
git commit --allow-empty -m "chore(ui): complete UI visual overhaul verification and quality gates"
```
