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
}
