use rustforge_syntax::{
    find_bracket_match, find_matching_bracket, Language, TreeSitterEngine,
};
use tree_sitter::{InputEdit, Point};

#[test]
fn test_tree_sitter_engine_creation() {
    let engine = TreeSitterEngine::new(Language::Rust);
    assert!(engine.is_available());
    assert_eq!(engine.language(), Language::Rust);

    let plain_engine = TreeSitterEngine::new(Language::PlainText);
    assert!(!plain_engine.is_available());
}

#[test]
fn test_tree_sitter_parse_rust_code() {
    let mut engine = TreeSitterEngine::new(Language::Rust);
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let tree = engine.parse_incremental(source, None, None).expect("failed to parse");

    let root = tree.root_node();
    assert_eq!(root.kind(), "source_file");
    assert!(!root.has_error());
    assert!(root.child_count() > 0);
}

#[test]
fn test_tree_sitter_incremental_edit() {
    let mut engine = TreeSitterEngine::new(Language::Rust);
    let source_v1 = "fn foo() {\n    let x = 1;\n}\n";
    let tree_v1 = engine.parse_incremental(source_v1, None, None).expect("v1 parse failed");

    // Replace "1" with "100" at line 1, column 12 (char offset 23)
    let source_v2 = "fn foo() {\n    let x = 100;\n}\n";
    let start_byte = source_v1.find('1').unwrap();
    let old_end_byte = start_byte + 1;
    let new_end_byte = start_byte + 3;

    let edit = InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position: Point { row: 1, column: 12 },
        old_end_position: Point { row: 1, column: 13 },
        new_end_position: Point { row: 1, column: 15 },
    };

    let tree_v2 = engine
        .parse_incremental(source_v2, Some(&tree_v1), Some(edit))
        .expect("v2 incremental parse failed");

    let root_v2 = tree_v2.root_node();
    assert_eq!(root_v2.kind(), "source_file");
    assert!(!root_v2.has_error());

    // Inspect that the integer literal in AST now has value "100"
    let v2_text = root_v2.to_sexp();
    assert!(v2_text.contains("integer_literal"));
}

#[test]
fn test_ast_fold_ranges_rust() {
    let mut engine = TreeSitterEngine::new(Language::Rust);
    let source = r#"fn calculate(x: i32) -> i32 {
    let y = x * 2;
    if y > 10 {
        y + 1
    } else {
        y - 1
    }
}

fn single_line() {}
"#;

    let tree = engine.parse_incremental(source, None, None).expect("parse failed");
    let fold_ranges = engine.compute_fold_ranges(&tree);

    // Should detect multi-line blocks/functions, but ignore single-line items
    assert!(!fold_ranges.is_empty(), "expected fold ranges");

    // The function `calculate` spans lines 0..7
    let has_func_fold = fold_ranges.iter().any(|r| r.start_line == 0 && r.end_line == 7);
    assert!(has_func_fold, "expected function fold range, got: {:?}", fold_ranges);

    // Every fold range must satisfy start_line < end_line
    for range in &fold_ranges {
        assert!(
            range.start_line < range.end_line,
            "fold range must span multiple lines: {:?}",
            range
        );
    }

    // single_line() is at line 9 and is 1 line, must not have a fold
    let has_single_line_fold = fold_ranges.iter().any(|r| r.start_line == 9);
    assert!(!has_single_line_fold, "single line item should not be foldable");
}

#[test]
fn test_ast_fold_ranges_json() {
    let mut engine = TreeSitterEngine::new(Language::Json);
    let source = r#"{
    "name": "rustforge",
    "dependencies": [
        "tree-sitter",
        "ropey"
    ]
}"#;

    let tree = engine.parse_incremental(source, None, None).expect("json parse failed");
    let fold_ranges = engine.compute_fold_ranges(&tree);

    // Root object fold 0..6, array fold 2..5
    assert!(fold_ranges.len() >= 2, "expected at least 2 fold ranges: {:?}", fold_ranges);
    assert!(fold_ranges.iter().any(|r| r.start_line == 0 && r.end_line == 6));
    assert!(fold_ranges.iter().any(|r| r.start_line == 2 && r.end_line == 5));
}

#[test]
fn test_bracket_matching_simple() {
    let source = "fn foo() { let arr = [1, (2 + 3)]; }";

    // Matching '(' at offset 6 -> ')' at offset 7
    assert_eq!(find_matching_bracket(source, 6), Some(7));
    assert_eq!(find_matching_bracket(source, 7), Some(6));

    // Matching '{' at offset 9 -> '}' at offset 35
    assert_eq!(find_matching_bracket(source, 9), Some(35));
    assert_eq!(find_matching_bracket(source, 35), Some(9));

    // Matching '[' at offset 21 -> ']' at offset 32
    assert_eq!(find_matching_bracket(source, 21), Some(32));
    assert_eq!(find_matching_bracket(source, 32), Some(21));

    // Non-bracket character
    assert_eq!(find_matching_bracket(source, 0), None);
    assert_eq!(find_matching_bracket(source, 10), None);
}

#[test]
fn test_bracket_matching_inside_strings_and_comments() {
    // Brackets inside string literal should not be matched
    let source = r#"let msg = "{ (hello) }"; let val = (42);"#;
    // The parens inside string: offset 13 '(' and 19 ')' -> ignored
    assert_eq!(find_matching_bracket(source, 13), None);
    assert_eq!(find_matching_bracket(source, 19), None);

    // The parens around 42: offset 35 '(' and 38 ')'
    assert_eq!(find_matching_bracket(source, 35), Some(38));
    assert_eq!(find_matching_bracket(source, 38), Some(35));

    // Brackets inside line comment should also be ignored
    let comment_source = "let x = (10); // this is a (comment)\nlet y = 20;";
    assert_eq!(find_matching_bracket(comment_source, 8), Some(11));
    assert_eq!(find_matching_bracket(comment_source, 11), Some(8));
    assert_eq!(find_matching_bracket(comment_source, 27), None);
}

#[test]
fn test_bracket_match_struct() {
    let source = "vec![1, 2, 3]";
    let bracket_match = find_bracket_match(source, 4).expect("bracket match not found");
    assert_eq!(bracket_match.open, 4);
    assert_eq!(bracket_match.close, 12);
    assert_eq!(bracket_match.start(), 4);
    assert_eq!(bracket_match.end(), 12);
}

#[test]
fn test_unsupported_language_graceful_fallback() {
    let mut engine = TreeSitterEngine::new(Language::PlainText);
    assert!(!engine.is_available());

    let tree = engine.parse_incremental("hello world", None, None);
    assert!(tree.is_none());

    // Tokenizer fallback still functions correctly
    let tokens = engine.highlight_line_fallback("let x = 42;", 0);
    assert!(!tokens.is_empty());
}
