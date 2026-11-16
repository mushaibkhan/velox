use rustforge_themes::ThemeMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub font_size: f32,
    pub tab_size: usize,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub show_minimap: bool,
    pub theme_mode: ThemeMode,
    pub auto_save: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            tab_size: 4,
            show_line_numbers: true,
            word_wrap: false,
            show_minimap: false,
            theme_mode: ThemeMode::DarkModern,
            auto_save: false,
        }
    }
}

impl Settings {
    pub fn config_path() -> Option<PathBuf> {
        let base = dirs_next::config_dir().or_else(|| {
            std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
        })?;
        Some(base.join("rustforge").join("settings.toml"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(settings) = toml::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::other(e.to_string())
        })?;
        fs::write(path, content)
    }
}

mod dirs_next {
    use std::path::PathBuf;
    pub fn config_dir() -> Option<PathBuf> {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
            })
    }
}
