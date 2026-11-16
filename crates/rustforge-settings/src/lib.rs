pub mod settings;

pub use settings::Settings;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_serde() {
        let s = Settings::default();
        let toml_str = toml::to_string(&s).unwrap();
        let deserialized: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.font_size, 14.0);
        assert_eq!(deserialized.tab_size, 4);
    }
}
