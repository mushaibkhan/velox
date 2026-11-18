pub mod theme;

pub use theme::{Theme, ThemeMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themes() {
        let dm = Theme::dark_modern();
        assert_eq!(dm.mode, ThemeMode::DarkModern);
        assert_eq!(dm.name, "Dark Modern");

        let tn = Theme::tokyo_night();
        assert_eq!(tn.name, "Tokyo Night");
    }
}
