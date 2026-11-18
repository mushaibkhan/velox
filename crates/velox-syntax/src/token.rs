use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Variable,
    Property,
    Constant,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSpan {
    pub start: usize, // char offset
    pub end: usize,   // char offset
    pub token_type: TokenType,
}

impl TokenSpan {
    pub fn new(start: usize, end: usize, token_type: TokenType) -> Self {
        Self { start, end, token_type }
    }
}
