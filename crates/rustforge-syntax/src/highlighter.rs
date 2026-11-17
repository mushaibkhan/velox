use crate::language::Language;
use crate::token::{TokenSpan, TokenType};

pub struct SyntaxHighlighter {
    language: Language,
}

impl SyntaxHighlighter {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn highlight_line(&self, line: &str, line_char_offset: usize) -> Vec<TokenSpan> {
        if self.language == Language::PlainText || line.is_empty() {
            return vec![TokenSpan::new(
                line_char_offset,
                line_char_offset + line.chars().count(),
                TokenType::Text,
            )];
        }

        let mut tokens = Vec::new();
        let chars: Vec<(usize, char)> = line.char_indices().collect();
        let num_chars = chars.len();
        let mut i = 0;

        let keywords = self.language.keywords();
        let types = self.language.types();

        while i < num_chars {
            let (byte_pos, ch) = chars[i];

            // 1. Comments
            let is_slash_comment = ch == '/' && i + 1 < num_chars && chars[i + 1].1 == '/';
            let is_hash_comment = (self.language == Language::Python || self.language == Language::Shell) && ch == '#';

            if is_slash_comment || is_hash_comment {
                tokens.push(TokenSpan::new(
                    line_char_offset + i,
                    line_char_offset + num_chars,
                    TokenType::Comment,
                ));
                break;
            }

            // Markdown Headings
            if self.language == Language::Markdown && ch == '#' {
                let start_idx = i;
                while i < num_chars && chars[i].1 == '#' {
                    i += 1;
                }
                tokens.push(TokenSpan::new(
                    line_char_offset + start_idx,
                    line_char_offset + i,
                    TokenType::Keyword,
                ));
                continue;
            }

            // 2. Strings
            if ch == '"' || ch == '\'' || ch == '`' {
                let quote = ch;
                let start_idx = i;
                i += 1;
                let mut escaped = false;
                while i < num_chars {
                    let cur_ch = chars[i].1;
                    if escaped {
                        escaped = false;
                    } else if cur_ch == '\\' {
                        escaped = true;
                    } else if cur_ch == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                tokens.push(TokenSpan::new(
                    line_char_offset + start_idx,
                    line_char_offset + i,
                    TokenType::String,
                ));
                continue;
            }

            // 3. Numbers
            if ch.is_ascii_digit() {
                let start_idx = i;
                while i < num_chars && (chars[i].1.is_ascii_alphanumeric() || chars[i].1 == '.' || chars[i].1 == '_') {
                    i += 1;
                }
                tokens.push(TokenSpan::new(
                    line_char_offset + start_idx,
                    line_char_offset + i,
                    TokenType::Number,
                ));
                continue;
            }

            // 4. Identifiers, Keywords & Types
            if ch.is_alphabetic() || ch == '_' {
                let start_idx = i;
                while i < num_chars && (chars[i].1.is_alphanumeric() || chars[i].1 == '_') {
                    i += 1;
                }

                let end_byte = if i < num_chars {
                    chars[i].0
                } else {
                    line.len()
                };
                let ident = &line[byte_pos..end_byte];

                let token_type = if keywords.contains(&ident) {
                    TokenType::Keyword
                } else if types.contains(&ident) {
                    TokenType::Type
                } else if i < num_chars && chars[i].1 == '(' {
                    TokenType::Function
                } else if ident.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_') && ident.len() > 1 {
                    TokenType::Constant
                } else {
                    TokenType::Variable
                };

                tokens.push(TokenSpan::new(
                    line_char_offset + start_idx,
                    line_char_offset + i,
                    token_type,
                ));
                continue;
            }

            // 5. Operators & Punctuation
            if "=+-*/%&|^!<>?:~".contains(ch) {
                tokens.push(TokenSpan::new(
                    line_char_offset + i,
                    line_char_offset + i + 1,
                    TokenType::Operator,
                ));
                i += 1;
                continue;
            } else if "{}()[].,;:#@$\\".contains(ch) {
                tokens.push(TokenSpan::new(
                    line_char_offset + i,
                    line_char_offset + i + 1,
                    TokenType::Punctuation,
                ));
                i += 1;
                continue;
            }

            // 6. Whitespace / default text
            let start_idx = i;
            while i < num_chars
                && chars[i].1.is_whitespace()
            {
                i += 1;
            }
            if i > start_idx {
                tokens.push(TokenSpan::new(
                    line_char_offset + start_idx,
                    line_char_offset + i,
                    TokenType::Text,
                ));
            } else {
                // Guaranteed progress: consume the character as Text
                tokens.push(TokenSpan::new(
                    line_char_offset + i,
                    line_char_offset + i + 1,
                    TokenType::Text,
                ));
                i += 1;
            }
        }

        tokens
    }
}
