use velox_app::ui::icons::Icon;

fn icon_for_path(path: &str) -> Icon {
    if path.ends_with(".rs") {
        Icon::FileRust
    } else if path.ends_with(".toml") {
        Icon::FileToml
    } else if path.ends_with(".md") {
        Icon::FileMarkdown
    } else {
        Icon::FileGeneric
    }
}

#[test]
fn test_file_type_icon_resolution() {
    assert_eq!(icon_for_path("main.rs"), Icon::FileRust);
    assert_eq!(icon_for_path("Cargo.toml"), Icon::FileToml);
    assert_eq!(icon_for_path("README.md"), Icon::FileMarkdown);
    assert_eq!(icon_for_path("data.txt"), Icon::FileGeneric);
}
