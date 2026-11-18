use egui::Color32;
use velox_themes::{Theme, ThemeMode};

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
