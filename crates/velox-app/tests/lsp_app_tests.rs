use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::duplex;

use velox_app::VeloxApp;
use velox_core::{Position, Range};
use velox_language_server::{
    path_to_uri, DiagnosticItem, DiagnosticSeverity, DiagnosticStore, LspClient, LspFramedReader,
};
use serde_json::json;

#[test]
fn test_lsp_manager_initialization() {
    let app = VeloxApp::new_test_app();
    assert!(app.lsp_manager.is_some());
}

#[test]
fn test_lsp_sync_diagnostics() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("main.rs");
    fs::write(&file_path, "fn main() {\n    let x = 1;\n}\n").unwrap();

    let mut app = VeloxApp::new_test_app();
    app.open_file(&file_path);
    assert_eq!(app.tabs.len(), 1);

    // Populate diagnostics in the shared store
    let store = app.lsp_manager.as_ref().unwrap().diagnostic_store();
    let uri = path_to_uri(&file_path).unwrap();
    store.update_diagnostics(
        &uri,
        vec![
            DiagnosticItem::new(
                Range::new(Position::new(1, 8), Position::new(1, 9)),
                DiagnosticSeverity::Warning,
                "unused variable `x`",
            ),
            DiagnosticItem::new(
                Range::new(Position::new(1, 8), Position::new(1, 9)),
                DiagnosticSeverity::Error,
                "type annotation needed",
            ),
        ],
    );

    // Synchronize diagnostics to editor tabs
    app.sync_lsp_diagnostics();

    let tab = &app.tabs[0];
    assert_eq!(tab.editor.diagnostics.len(), 2);
    assert_eq!(
        tab.editor.highest_severity_for_line(1),
        Some(DiagnosticSeverity::Error)
    );
    assert_eq!(tab.editor.highest_severity_for_line(0), None);
}

#[test]
fn test_lsp_mock_client_features() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("lib.rs");
    fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();

    let mut app = VeloxApp::new_test_app();
    app.open_file(&file_path);
    let uri = path_to_uri(&file_path).unwrap();

    // Create a mock duplex LSP client
    let (client_io, server_io) = duplex(4096);
    let (client_read, client_write) = tokio::io::split(client_io);
    let (server_read, mut server_write) = tokio::io::split(server_io);

    let store = Arc::new(DiagnosticStore::new());
    let client = {
        let _guard = app.tokio_rt.as_ref().unwrap().enter();
        Arc::new(LspClient::from_split(client_read, client_write, store))
    };

    // Register mock client for rust
    app.lsp_manager
        .as_ref()
        .unwrap()
        .register_client("rust", Arc::clone(&client));

    // Spawn mock server on dedicated OS thread
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    let server_handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let mut reader = LspFramedReader::new(server_read);

            loop {
                let msg = tokio::select! {
                    _ = &mut stop_rx => break,
                    res = reader.read_message() => match res {
                        Ok(Some(m)) => m,
                        _ => break,
                    }
                };

                let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
                let id = parsed.get("id").and_then(|v| v.as_u64());
                let method = parsed.get("method").and_then(|v| v.as_str()).unwrap_or_default();

                if method == "textDocument/completion" {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id.unwrap(),
                        "result": [
                            {
                                "label": "add",
                                "kind": 3,
                                "detail": "fn(i32, i32) -> i32",
                                "insertText": "add(1, 2)"
                            }
                        ]
                    });
                    let encoded = velox_language_server::encode_message(&resp.to_string());
                    use tokio::io::AsyncWriteExt;
                    server_write.write_all(&encoded).await.unwrap();
                } else if method == "textDocument/hover" {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id.unwrap(),
                        "result": {
                            "contents": {
                                "kind": "markdown",
                                "value": "```rust\npub fn add(a: i32, b: i32) -> i32\n```\nAdds two numbers."
                            }
                        }
                    });
                    let encoded = velox_language_server::encode_message(&resp.to_string());
                    use tokio::io::AsyncWriteExt;
                    server_write.write_all(&encoded).await.unwrap();
                } else if method == "textDocument/formatting" {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id.unwrap(),
                        "result": [
                            {
                                "range": {
                                    "start": { "line": 0, "character": 0 },
                                    "end": { "line": 0, "character": 0 }
                                },
                                "newText": "// Header\n"
                            }
                        ]
                    });
                    let encoded = velox_language_server::encode_message(&resp.to_string());
                    use tokio::io::AsyncWriteExt;
                    server_write.write_all(&encoded).await.unwrap();
                } else if method == "textDocument/definition" {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id.unwrap(),
                        "result": {
                            "uri": uri.to_string(),
                            "range": {
                                "start": { "line": 1, "character": 4 },
                                "end": { "line": 1, "character": 9 }
                            }
                        }
                    });
                    let encoded = velox_language_server::encode_message(&resp.to_string());
                    use tokio::io::AsyncWriteExt;
                    server_write.write_all(&encoded).await.unwrap();
                }
            }
        });
    });

    // 1. Test Completion Trigger
    app.trigger_completion();
    let tab = &app.tabs[0];
    assert!(tab.editor.show_completion);
    assert_eq!(tab.editor.completion_items.len(), 1);
    assert_eq!(tab.editor.completion_items[0].label, "add");

    // 2. Test Hover Trigger
    app.trigger_hover();
    let tab = &app.tabs[0];
    assert!(tab.editor.show_hover);
    assert!(tab.editor.hover_text.as_ref().unwrap().contains("Adds two numbers."));

    // 3. Test Formatting Trigger
    app.trigger_formatting();
    let tab = &app.tabs[0];
    let content = tab.editor.buffer.slice_to_string(0, tab.editor.buffer.total_chars());
    assert!(content.starts_with("// Header\n"));

    // 4. Test Goto Definition Trigger
    app.trigger_goto_definition();
    let tab = &app.tabs[0];
    let pos = tab.editor.primary_position();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 4);

    let _ = stop_tx.send(());
    let _ = server_handle.join();
}
