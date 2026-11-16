use egui::Color32;
use rustforge_syntax::TokenType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeMode {
    DarkModern,
    TokyoNight,
    MonokaiPro,
    LightModern,
    HighContrast,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub name: &'static str,
    pub bg_app: Color32,
    pub bg_editor: Color32,
    pub bg_gutter: Color32,
    pub bg_activity_bar: Color32,
    pub bg_sidebar: Color32,
    pub bg_tab_active: Color32,
    pub bg_tab_inactive: Color32,
    pub bg_status_bar: Color32,
    pub bg_terminal: Color32,
    pub bg_panel: Color32,
    pub bg_input: Color32,

    pub fg_primary: Color32,
    pub fg_secondary: Color32,
    pub fg_muted: Color32,
    pub fg_accent: Color32,

    pub border: Color32,
    pub line_highlight: Color32,
    pub selection: Color32,
    pub find_match: Color32,

    // Syntax colors
    pub syntax_keyword: Color32,
    pub syntax_function: Color32,
    pub syntax_type: Color32,
    pub syntax_string: Color32,
    pub syntax_number: Color32,
    pub syntax_comment: Color32,
    pub syntax_operator: Color32,
    pub syntax_punctuation: Color32,
    pub syntax_variable: Color32,
    pub syntax_constant: Color32,
}

impl Theme {
    pub fn dark_modern() -> Self {
        Self {
            mode: ThemeMode::DarkModern,
            name: "Dark Modern",
            bg_app: Color32::from_rgb(24, 24, 24),
            bg_editor: Color32::from_rgb(30, 30, 30),
            bg_gutter: Color32::from_rgb(30, 30, 30),
            bg_activity_bar: Color32::from_rgb(24, 24, 24),
            bg_sidebar: Color32::from_rgb(37, 37, 38),
            bg_tab_active: Color32::from_rgb(30, 30, 30),
            bg_tab_inactive: Color32::from_rgb(45, 45, 45),
            bg_status_bar: Color32::from_rgb(0, 122, 204),
            bg_terminal: Color32::from_rgb(24, 24, 24),
            bg_panel: Color32::from_rgb(30, 30, 30),
            bg_input: Color32::from_rgb(60, 60, 60),

            fg_primary: Color32::from_rgb(212, 212, 212),
            fg_secondary: Color32::from_rgb(170, 170, 170),
            fg_muted: Color32::from_rgb(110, 110, 110),
            fg_accent: Color32::from_rgb(0, 122, 204),

            border: Color32::from_rgb(45, 45, 45),
            line_highlight: Color32::from_rgba_premultiplied(255, 255, 255, 10),
            selection: Color32::from_rgba_premultiplied(38, 79, 120, 160),
            find_match: Color32::from_rgba_premultiplied(234, 179, 8, 120),

            syntax_keyword: Color32::from_rgb(86, 156, 214),   // #569cd6
            syntax_function: Color32::from_rgb(220, 220, 170), // #dcdcaa
            syntax_type: Color32::from_rgb(78, 201, 176),      // #4ec9b0
            syntax_string: Color32::from_rgb(206, 145, 120),   // #ce9178
            syntax_number: Color32::from_rgb(181, 206, 168),   // #b5cea8
            syntax_comment: Color32::from_rgb(106, 153, 85),   // #6a9955
            syntax_operator: Color32::from_rgb(212, 212, 212),
            syntax_punctuation: Color32::from_rgb(212, 212, 212),
            syntax_variable: Color32::from_rgb(156, 220, 254),  // #9cdcfe
            syntax_constant: Color32::from_rgb(79, 193, 255),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            mode: ThemeMode::TokyoNight,
            name: "Tokyo Night",
            bg_app: Color32::from_rgb(26, 27, 38),
            bg_editor: Color32::from_rgb(26, 27, 38),
            bg_gutter: Color32::from_rgb(26, 27, 38),
            bg_activity_bar: Color32::from_rgb(22, 22, 30),
            bg_sidebar: Color32::from_rgb(31, 35, 53),
            bg_tab_active: Color32::from_rgb(26, 27, 38),
            bg_tab_inactive: Color32::from_rgb(31, 35, 53),
            bg_status_bar: Color32::from_rgb(114, 135, 253),
            bg_terminal: Color32::from_rgb(22, 22, 30),
            bg_panel: Color32::from_rgb(26, 27, 38),
            bg_input: Color32::from_rgb(41, 46, 66),

            fg_primary: Color32::from_rgb(192, 202, 245),
            fg_secondary: Color32::from_rgb(169, 177, 214),
            fg_muted: Color32::from_rgb(86, 95, 137),
            fg_accent: Color32::from_rgb(122, 162, 247),

            border: Color32::from_rgb(41, 46, 66),
            line_highlight: Color32::from_rgba_premultiplied(41, 46, 66, 120),
            selection: Color32::from_rgba_premultiplied(54, 76, 126, 160),
            find_match: Color32::from_rgba_premultiplied(224, 175, 104, 140),

            syntax_keyword: Color32::from_rgb(187, 154, 247),  // #bb9af7
            syntax_function: Color32::from_rgb(122, 162, 247), // #7aa2f7
            syntax_type: Color32::from_rgb(42, 195, 222),      // #2ac3de
            syntax_string: Color32::from_rgb(158, 206, 106),   // #9ece6a
            syntax_number: Color32::from_rgb(255, 158, 100),   // #ff9e64
            syntax_comment: Color32::from_rgb(86, 95, 137),    // #565f89
            syntax_operator: Color32::from_rgb(137, 221, 255),
            syntax_punctuation: Color32::from_rgb(192, 202, 245),
            syntax_variable: Color32::from_rgb(192, 202, 245),
            syntax_constant: Color32::from_rgb(255, 158, 100),
        }
    }

    pub fn monokai_pro() -> Self {
        Self {
            mode: ThemeMode::MonokaiPro,
            name: "Monokai Pro",
            bg_app: Color32::from_rgb(45, 42, 46),
            bg_editor: Color32::from_rgb(45, 42, 46),
            bg_gutter: Color32::from_rgb(45, 42, 46),
            bg_activity_bar: Color32::from_rgb(34, 31, 34),
            bg_sidebar: Color32::from_rgb(39, 36, 40),
            bg_tab_active: Color32::from_rgb(45, 42, 46),
            bg_tab_inactive: Color32::from_rgb(34, 31, 34),
            bg_status_bar: Color32::from_rgb(255, 97, 136),
            bg_terminal: Color32::from_rgb(34, 31, 34),
            bg_panel: Color32::from_rgb(45, 42, 46),
            bg_input: Color32::from_rgb(64, 62, 65),

            fg_primary: Color32::from_rgb(252, 252, 250),
            fg_secondary: Color32::from_rgb(193, 192, 192),
            fg_muted: Color32::from_rgb(114, 112, 114),
            fg_accent: Color32::from_rgb(255, 97, 136),

            border: Color32::from_rgb(64, 62, 65),
            line_highlight: Color32::from_rgba_premultiplied(64, 62, 65, 80),
            selection: Color32::from_rgba_premultiplied(87, 85, 88, 180),
            find_match: Color32::from_rgba_premultiplied(255, 216, 102, 140),

            syntax_keyword: Color32::from_rgb(255, 97, 136),   // #ff6188
            syntax_function: Color32::from_rgb(169, 220, 118), // #a9dc76
            syntax_type: Color32::from_rgb(120, 220, 232),     // #78dce8
            syntax_string: Color32::from_rgb(255, 216, 102),   // #ffd866
            syntax_number: Color32::from_rgb(171, 157, 242),   // #ab9df2
            syntax_comment: Color32::from_rgb(114, 112, 114),  // #727072
            syntax_operator: Color32::from_rgb(255, 97, 136),
            syntax_punctuation: Color32::from_rgb(147, 146, 147),
            syntax_variable: Color32::from_rgb(252, 252, 250),
            syntax_constant: Color32::from_rgb(171, 157, 242),
        }
    }

    pub fn get_token_color(&self, token_type: TokenType) -> Color32 {
        match token_type {
            TokenType::Keyword => self.syntax_keyword,
            TokenType::Function => self.syntax_function,
            TokenType::Type => self.syntax_type,
            TokenType::String => self.syntax_string,
            TokenType::Number => self.syntax_number,
            TokenType::Comment => self.syntax_comment,
            TokenType::Operator => self.syntax_operator,
            TokenType::Punctuation => self.syntax_punctuation,
            TokenType::Variable | TokenType::Property => self.syntax_variable,
            TokenType::Constant => self.syntax_constant,
            TokenType::Text => self.fg_primary,
        }
    }
}
