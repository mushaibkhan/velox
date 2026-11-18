//! Velox Language Server Protocol (LSP) client, transport, and diagnostics store.

pub mod client;
pub mod diagnostics;
pub mod manager;
pub mod transport;

pub use client::{JsonRpcError, LspClient, NotificationMessage};
pub use diagnostics::{
    normalize_path, path_to_uri, uri_to_path, DiagnosticItem, DiagnosticSeverity, DiagnosticStore,
    Url,
};
pub use lsp_types::Uri;
pub use manager::{
    find_executable_on_path, is_executable_available, LspManager, LspServerConfig,
};
pub use transport::{
    encode_json_value, encode_message, parse_frame, write_message, LspFramedReader, LspFramingError,
};

/// High-level errors originating from Language Server Protocol operations.
#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("Framing error: {0}")]
    Framing(#[from] LspFramingError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RPC error: {0}")]
    Rpc(#[from] JsonRpcError),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Client error: {0}")]
    Client(String),

    #[error("Client disconnected")]
    Disconnected,

    #[error("Request timed out")]
    Timeout,
}
