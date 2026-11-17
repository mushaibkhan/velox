//! LSP diagnostic representations and thread-safe diagnostic store.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;
use rustforge_core::{Position, Range};
use serde::{Deserialize, Serialize};

/// Type alias aligning `Url` with `lsp_types::Uri`.
pub type Url = lsp_types::Uri;

/// Severity levels for language server diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Information => write!(f, "Information"),
            Self::Hint => write!(f, "Hint"),
        }
    }
}

impl From<lsp_types::DiagnosticSeverity> for DiagnosticSeverity {
    fn from(sev: lsp_types::DiagnosticSeverity) -> Self {
        match sev {
            lsp_types::DiagnosticSeverity::ERROR => Self::Error,
            lsp_types::DiagnosticSeverity::WARNING => Self::Warning,
            lsp_types::DiagnosticSeverity::INFORMATION => Self::Information,
            lsp_types::DiagnosticSeverity::HINT => Self::Hint,
            _ => Self::Error,
        }
    }
}

impl From<DiagnosticSeverity> for lsp_types::DiagnosticSeverity {
    fn from(sev: DiagnosticSeverity) -> Self {
        match sev {
            DiagnosticSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticSeverity::Information => lsp_types::DiagnosticSeverity::INFORMATION,
            DiagnosticSeverity::Hint => lsp_types::DiagnosticSeverity::HINT,
        }
    }
}

/// A normalized diagnostic entry with 0-based line/column range, severity, and message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

impl DiagnosticItem {
    /// Creates a new diagnostic item.
    pub fn new(range: Range, severity: DiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            range,
            severity,
            message: message.into(),
            source: None,
            code: None,
        }
    }

    /// Sets the optional source field (e.g. "rustc", "clippy", "pyright").
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Sets the optional diagnostic error/warning code (e.g. "E0308").
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Converts an LSP `Diagnostic` into a normalized `DiagnosticItem`.
    pub fn from_lsp(d: &lsp_types::Diagnostic) -> Self {
        let severity = d
            .severity
            .map(DiagnosticSeverity::from)
            .unwrap_or(DiagnosticSeverity::Error);

        let range = Range::new(
            Position::new(
                d.range.start.line as usize,
                d.range.start.character as usize,
            ),
            Position::new(d.range.end.line as usize, d.range.end.character as usize),
        );

        let code = d.code.as_ref().map(|c| match c {
            lsp_types::NumberOrString::Number(n) => n.to_string(),
            lsp_types::NumberOrString::String(s) => s.clone(),
        });

        Self {
            range,
            severity,
            message: d.message.clone(),
            source: d.source.clone(),
            code,
        }
    }

    /// 0-based start line.
    pub fn start_line(&self) -> usize {
        self.range.start.line
    }

    /// 0-based end line.
    pub fn end_line(&self) -> usize {
        self.range.end.line
    }

    /// 0-based start column.
    pub fn start_col(&self) -> usize {
        self.range.start.column
    }

    /// 0-based end column.
    pub fn end_col(&self) -> usize {
        self.range.end.column
    }
}

/// Normalizes a file path to canonical form if available, or removes relative prefixes.
pub fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canon) = path.canonicalize() {
        canon
    } else {
        path.to_path_buf()
    }
}

/// Converts an LSP URI to a normalized file path buffer.
pub fn uri_to_path(uri: &lsp_types::Uri) -> PathBuf {
    let s = uri.as_str();
    if let Some(rest) = s.strip_prefix("file://") {
        #[cfg(windows)]
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let decoded = rest.replace("%20", " ");
        normalize_path(Path::new(&decoded))
    } else {
        PathBuf::from(s)
    }
}

/// Converts a file path to an LSP URI.
pub fn path_to_uri(path: &Path) -> Result<lsp_types::Uri, String> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let path_str = abs.to_string_lossy().replace('\\', "/");
    let encoded = path_str.replace(' ', "%20");
    let uri_str = if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    };
    uri_str.parse::<lsp_types::Uri>().map_err(|e| e.to_string())
}

/// Thread-safe in-memory store for language server diagnostics queryable by path and line number.
#[derive(Debug, Default)]
pub struct DiagnosticStore {
    store: RwLock<HashMap<PathBuf, Vec<DiagnosticItem>>>,
}

impl DiagnosticStore {
    /// Creates a new empty `DiagnosticStore`.
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    /// Updates or replaces the diagnostics for a specific LSP document URI.
    pub fn update_diagnostics(&self, uri: &lsp_types::Uri, diagnostics: Vec<DiagnosticItem>) {
        let path = uri_to_path(uri);
        self.update_for_path(&path, diagnostics);
    }

    /// Updates or replaces diagnostics for an LSP document URI using raw LSP diagnostics.
    pub fn update_from_lsp(&self, uri: &lsp_types::Uri, diagnostics: &[lsp_types::Diagnostic]) {
        let items: Vec<DiagnosticItem> = diagnostics.iter().map(DiagnosticItem::from_lsp).collect();
        self.update_diagnostics(uri, items);
    }

    /// Updates or replaces diagnostics for a specific file path.
    pub fn update_for_path(&self, path: &Path, diagnostics: Vec<DiagnosticItem>) {
        let norm = normalize_path(path);
        let mut write = self.store.write();
        if diagnostics.is_empty() {
            write.remove(&norm);
            write.remove(path);
        } else {
            write.insert(norm, diagnostics);
        }
    }

    /// Retrieves all diagnostics for a given file path.
    ///
    /// Supports exact path matches, canonicalized paths, and relative suffix matching.
    pub fn get_diagnostics_for_path(&self, path: &Path) -> Vec<DiagnosticItem> {
        let read = self.store.read();

        // Fast path: direct path match without filesystem canonicalization
        if let Some(items) = read.get(path) {
            return items.clone();
        }

        let norm = normalize_path(path);
        if let Some(items) = read.get(&norm) {
            return items.clone();
        }

        // Relative path fallback: check if any stored path ends with requested path
        for (stored_path, items) in read.iter() {
            if stored_path.ends_with(path) {
                return items.clone();
            }
        }

        Vec::new()
    }

    /// Retrieves all diagnostics for a given document URI.
    pub fn get_diagnostics_for_uri(&self, uri: &lsp_types::Uri) -> Vec<DiagnosticItem> {
        let path = uri_to_path(uri);
        self.get_diagnostics_for_path(&path)
    }

    /// Retrieves all diagnostics that intersect with a given 0-based line number.
    pub fn get_diagnostics_for_line(&self, path: &Path, line_0_based: u32) -> Vec<DiagnosticItem> {
        let target = line_0_based as usize;
        self.get_diagnostics_for_path(path)
            .into_iter()
            .filter(|item| {
                if target < item.range.start.line || target > item.range.end.line {
                    return false;
                }
                // Exclude multi-line ranges that terminate at column 0 of target line
                if target == item.range.end.line
                    && item.range.end.column == 0
                    && item.range.end.line > item.range.start.line
                {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Clears all diagnostics for all files.
    pub fn clear(&self) {
        self.store.write().clear();
    }

    /// Clears diagnostics for a specific file path.
    pub fn clear_path(&self, path: &Path) {
        let norm = normalize_path(path);
        let mut write = self.store.write();
        write.remove(&norm);
        write.remove(path);
    }

    /// Returns the number of files currently having active diagnostics.
    pub fn file_count(&self) -> usize {
        self.store.read().len()
    }

    /// Returns the total number of diagnostic items across all files.
    pub fn total_diagnostics(&self) -> usize {
        self.store.read().values().map(|v| v.len()).sum()
    }

    /// Returns a snapshot of all stored diagnostics keyed by file path.
    pub fn all_diagnostics(&self) -> HashMap<PathBuf, Vec<DiagnosticItem>> {
        self.store.read().clone()
    }
}
