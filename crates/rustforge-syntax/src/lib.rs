pub mod folding;
pub mod highlighter;
pub mod language;
pub mod token;
pub mod tree_sitter_engine;

pub use folding::{
    compute_fold_ranges, find_bracket_match, find_matching_bracket, BracketMatch, FoldRange,
};
pub use highlighter::SyntaxHighlighter;
pub use language::Language;
pub use token::{TokenSpan, TokenType};
pub use tree_sitter_engine::TreeSitterEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_highlighting() {
        let hl = SyntaxHighlighter::new(Language::Rust);
        let tokens = hl.highlight_line("fn main() { let x = 42; // comment }", 0);
        
        let keyword = tokens.iter().find(|t| t.token_type == TokenType::Keyword).unwrap();
        assert_eq!(keyword.start, 0);
        assert_eq!(keyword.end, 2);

        let comment = tokens.iter().find(|t| t.token_type == TokenType::Comment).unwrap();
        assert!(comment.start > 0);
    }

    #[test]
    fn test_markdown_heading_highlighting() {
        let hl = SyntaxHighlighter::new(Language::Markdown);
        let tokens = hl.highlight_line("# RustForge", 0);
        assert!(!tokens.is_empty());
        let kw = tokens.iter().find(|t| t.token_type == TokenType::Keyword).unwrap();
        assert_eq!(kw.start, 0);
        assert_eq!(kw.end, 1);
    }

    #[test]
    fn test_markdown_complex_document_lines() {
        let hl = SyntaxHighlighter::new(Language::Markdown);
        let lines = [
            "# Title",
            "## Subtitle with `code`",
            "- List item 1",
            "* List item 2 with **bold**",
            "> Blockquote",
            "```rust",
            "fn main() {}",
            "```",
            "[Link text](https://example.com)",
        ];
        for line in lines {
            let tokens = hl.highlight_line(line, 0);
            assert!(!tokens.is_empty(), "Failed to tokenize: {}", line);
        }
    }
}
