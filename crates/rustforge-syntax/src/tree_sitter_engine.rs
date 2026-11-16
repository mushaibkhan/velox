use crate::folding::{self, BracketMatch, FoldRange};
use crate::highlighter::SyntaxHighlighter;
use crate::language::Language;
use crate::token::TokenSpan;
use tree_sitter::{InputEdit, Parser, Tree};

/// Incremental AST parsing and analysis engine powered by Tree-sitter.
pub struct TreeSitterEngine {
    language: Language,
    parser: Option<Parser>,
}

impl TreeSitterEngine {
    /// Creates a new `TreeSitterEngine` for the given language.
    ///
    /// If an AST grammar is available for the language, the Tree-sitter parser
    /// is initialized. If not, `is_available()` will return `false`, and
    /// `parse_incremental` will return `None`.
    pub fn new(language: Language) -> Self {
        let parser = Self::init_parser_for_language(language);
        Self { language, parser }
    }

    /// Returns `true` if an AST grammar parser is available and loaded for this language.
    pub fn is_available(&self) -> bool {
        self.parser.is_some()
    }

    /// Returns the currently configured language.
    pub fn language(&self) -> Language {
        self.language
    }

    /// Updates the target language for this engine.
    /// Returns `true` if an AST grammar was successfully loaded.
    pub fn set_language(&mut self, language: Language) -> bool {
        self.language = language;
        self.parser = Self::init_parser_for_language(language);
        self.parser.is_some()
    }

    /// Parses the entire source text from scratch.
    pub fn parse(&mut self, source: &str) -> Option<Tree> {
        self.parse_incremental(source, None, None)
    }

    /// Parses `source` incrementally.
    ///
    /// - `old_tree`: The previous syntax tree, if available.
    /// - `edit`: The edit applied between `old_tree` and `source`. If provided,
    ///   `old_tree` is edited before parsing.
    ///
    /// Returns the newly updated `Tree`, or `None` if the parser is unavailable.
    pub fn parse_incremental(
        &mut self,
        source: &str,
        old_tree: Option<&Tree>,
        edit: Option<InputEdit>,
    ) -> Option<Tree> {
        let parser = self.parser.as_mut()?;

        let mut tree_to_use = old_tree.cloned();
        if let (Some(tree), Some(ref input_edit)) = (&mut tree_to_use, &edit) {
            tree.edit(input_edit);
        }

        parser.parse(source, tree_to_use.as_ref())
    }

    /// Computes code fold ranges from a parsed Tree-sitter AST.
    pub fn compute_fold_ranges(&self, tree: &Tree) -> Vec<FoldRange> {
        folding::compute_fold_ranges(tree)
    }

    /// Finds the character offset of the matching bracket for the bracket at `char_offset`.
    pub fn find_matching_bracket(&self, source: &str, char_offset: usize) -> Option<usize> {
        folding::find_matching_bracket(source, char_offset)
    }

    /// Finds the `BracketMatch` pair for the bracket at `char_offset`.
    pub fn find_bracket_match(&self, source: &str, char_offset: usize) -> Option<BracketMatch> {
        folding::find_bracket_match(source, char_offset)
    }

    /// Fallback line-based tokenizer highlighter when an AST grammar is not loaded
    /// or for unsupported languages.
    pub fn highlight_line_fallback(&self, line: &str, line_char_offset: usize) -> Vec<TokenSpan> {
        let highlighter = SyntaxHighlighter::new(self.language);
        highlighter.highlight_line(line, line_char_offset)
    }

    fn init_parser_for_language(language: Language) -> Option<Parser> {
        let mut parser = Parser::new();
        let ts_lang = match language {
            Language::Rust => tree_sitter::Language::from(tree_sitter_rust::LANGUAGE),
            Language::Python => tree_sitter::Language::from(tree_sitter_python::LANGUAGE),
            Language::Json => tree_sitter::Language::from(tree_sitter_json::LANGUAGE),
            Language::Toml => tree_sitter::Language::from(tree_sitter_toml_ng::LANGUAGE),
            _ => return None,
        };

        if parser.set_language(&ts_lang).is_ok() {
            Some(parser)
        } else {
            None
        }
    }
}
