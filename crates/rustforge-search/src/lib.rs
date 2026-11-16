pub mod search;

pub use search::{SearchEngine, SearchMatch};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matching() {
        assert!(SearchEngine::fuzzy_match("rf", "rustforge").is_some());
        assert!(SearchEngine::fuzzy_match("buf", "text_buffer.rs").is_some());
        assert!(SearchEngine::fuzzy_match("xyz", "rustforge").is_none());
    }
}
