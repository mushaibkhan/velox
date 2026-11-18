use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Json,
    Toml,
    Markdown,
    Html,
    Css,
    C,
    Cpp,
    Go,
    Shell,
    Sql,
    PlainText,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Json => "JSON",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Go => "Go",
            Language::Shell => "Shell Script",
            Language::Sql => "SQL",
            Language::PlainText => "Plain Text",
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => Language::Rust,
                "py" => Language::Python,
                "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
                "ts" | "tsx" => Language::TypeScript,
                "json" => Language::Json,
                "toml" => Language::Toml,
                "md" | "markdown" => Language::Markdown,
                "html" | "htm" => Language::Html,
                "css" | "scss" | "less" => Language::Css,
                "c" | "h" => Language::C,
                "cpp" | "cxx" | "cc" | "hpp" => Language::Cpp,
                "go" => Language::Go,
                "sh" | "bash" | "zsh" => Language::Shell,
                "sql" => Language::Sql,
                _ => Language::PlainText,
            }
        } else if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            match file_name {
                "Cargo.toml" | "Cargo.lock" => Language::Toml,
                "package.json" | "tsconfig.json" => Language::Json,
                "Dockerfile" => Language::Shell,
                _ => Language::PlainText,
            }
        } else {
            Language::PlainText
        }
    }

    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "type",
                "const", "static", "use", "mod", "as", "where", "for", "in", "loop",
                "while", "if", "else", "match", "return", "break", "continue", "move",
                "async", "await", "unsafe", "ref", "self", "Self", "crate", "super",
            ],
            Language::Python => &[
                "def", "class", "import", "from", "as", "return", "if", "elif", "else",
                "for", "while", "break", "continue", "try", "except", "finally", "raise",
                "with", "yield", "async", "await", "lambda", "pass", "None", "True", "False",
                "is", "in", "not", "and", "or", "global", "nonlocal", "assert",
            ],
            Language::JavaScript | Language::TypeScript => &[
                "function", "const", "let", "var", "class", "interface", "type", "export",
                "import", "from", "return", "if", "else", "for", "while", "switch", "case",
                "default", "break", "continue", "new", "this", "super", "try", "catch",
                "finally", "throw", "async", "await", "yield", "null", "undefined", "true",
                "false", "typeof", "instanceof", "in", "of",
            ],
            Language::Go => &[
                "func", "package", "import", "type", "struct", "interface", "var", "const",
                "return", "if", "else", "for", "range", "switch", "case", "default", "go",
                "chan", "select", "defer", "map", "nil", "true", "false",
            ],
            Language::C | Language::Cpp => &[
                "int", "char", "float", "double", "void", "auto", "const", "static", "struct",
                "class", "enum", "union", "typedef", "return", "if", "else", "for", "while",
                "do", "switch", "case", "default", "break", "continue", "goto", "sizeof",
                "namespace", "using", "public", "private", "protected", "virtual", "template",
            ],
            _ => &[],
        }
    }

    pub fn types(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "i8", "i16", "i32", "i64", "i128", "isize",
                "u8", "u16", "u32", "u64", "u128", "usize",
                "f32", "f64", "bool", "char", "str", "String",
                "Vec", "Option", "Result", "Box", "Rc", "Arc", "Some", "None", "Ok", "Err",
            ],
            Language::Python => &[
                "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes", "object",
            ],
            Language::JavaScript | Language::TypeScript => &[
                "number", "string", "boolean", "any", "void", "never", "unknown",
                "Array", "Promise", "Record", "Map", "Set", "Object",
            ],
            _ => &[],
        }
    }
}
