use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use lsp_types::{
    CompletionResponse, Diagnostic, DiagnosticSeverity as LspSeverity, HoverContents, MarkedString,
    Position as LspPosition, Range as LspRange,
};
use velox_core::{Position, Range};
use velox_language_server::{
    encode_json_value, encode_message, find_executable_on_path, is_executable_available,
    parse_frame, path_to_uri, write_message, DiagnosticItem, DiagnosticSeverity, DiagnosticStore,
    LspClient, LspError, LspFramedReader, LspFramingError, LspManager, LspServerConfig,
};
use serde_json::json;
use tokio::io::duplex;

// =========================================================================
// 1. Framing Encoder & Decoder Tests
// =========================================================================

#[test]
fn test_encode_and_parse_single_frame() {
    let payload = r#"{"jsonrpc":"2.0","method":"test","params":{}}"#;
    let encoded = encode_message(payload);

    let expected_header = format!("Content-Length: {}\r\n\r\n", payload.len());
    assert!(encoded.starts_with(expected_header.as_bytes()));

    let (consumed, parsed) = parse_frame(&encoded).unwrap().expect("Should parse frame");
    assert_eq!(consumed, encoded.len());
    assert_eq!(parsed, payload);
}

#[test]
fn test_encode_json_value() {
    let val = json!({"id": 1, "result": "ok"});
    let encoded = encode_json_value(&val).unwrap();

    let (_, parsed) = parse_frame(&encoded).unwrap().expect("Should parse frame");
    let parsed_json: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert_eq!(parsed_json, val);
}

#[test]
fn test_parse_multiple_frames_in_single_buffer() {
    let msg1 = r#"{"id":1}"#;
    let msg2 = r#"{"id":2,"message":"hello world"}"#;
    let msg3 = r#"{"id":3,"result":true}"#;

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&encode_message(msg1));
    buffer.extend_from_slice(&encode_message(msg2));
    buffer.extend_from_slice(&encode_message(msg3));

    // Frame 1
    let (c1, p1) = parse_frame(&buffer).unwrap().unwrap();
    assert_eq!(p1, msg1);
    buffer.drain(..c1);

    // Frame 2
    let (c2, p2) = parse_frame(&buffer).unwrap().unwrap();
    assert_eq!(p2, msg2);
    buffer.drain(..c2);

    // Frame 3
    let (c3, p3) = parse_frame(&buffer).unwrap().unwrap();
    assert_eq!(p3, msg3);
    buffer.drain(..c3);

    // Buffer is now empty
    assert!(buffer.is_empty());
    assert_eq!(parse_frame(&buffer).unwrap(), None);
}

#[test]
fn test_parse_incomplete_buffer() {
    let msg = r#"{"id":1,"status":"ok"}"#;
    let encoded = encode_message(msg);

    // Partial header
    assert_eq!(parse_frame(&encoded[..10]).unwrap(), None);

    // Header complete but partial payload
    let split_pt = encoded.len() - 5;
    assert_eq!(parse_frame(&encoded[..split_pt]).unwrap(), None);

    // Full frame
    assert!(parse_frame(&encoded).unwrap().is_some());
}

#[test]
fn test_parse_with_additional_headers() {
    let payload = r#"{"jsonrpc":"2.0","result":42}"#;
    let framed = format!(
        "Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\nX-Custom: value\r\n\r\n{}",
        payload.len(),
        payload
    );

    let (consumed, parsed) = parse_frame(framed.as_bytes()).unwrap().unwrap();
    assert_eq!(consumed, framed.len());
    assert_eq!(parsed, payload);
}

#[test]
fn test_parse_missing_content_length() {
    let bad_frame = b"Content-Type: text/plain\r\n\r\nHello";
    let res = parse_frame(bad_frame);
    assert!(matches!(res, Err(LspFramingError::MissingContentLength)));
}

#[tokio::test]
async fn test_fragmented_stream_with_framed_reader() {
    let payload = r#"{"jsonrpc":"2.0","method":"window/logMessage","params":{"message":"fragmented"}}"#;
    let encoded = encode_message(payload);

    let (mut client, server) = duplex(1024);

    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        // Write in tiny chunks of 3 bytes
        for chunk in encoded.chunks(3) {
            client.write_all(chunk).await.unwrap();
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });

    let mut reader = LspFramedReader::new(server);
    let msg = reader.read_message().await.unwrap().expect("Should read message");
    assert_eq!(msg, payload);
}

// =========================================================================
// 2. Diagnostic Store Tests
// =========================================================================

#[test]
fn test_diagnostic_store_crud() {
    let store = DiagnosticStore::new();
    let uri = path_to_uri(Path::new("/workspace/src/lib.rs")).unwrap();

    let diag1 = DiagnosticItem::new(
        Range::new(Position::new(10, 4), Position::new(10, 15)),
        DiagnosticSeverity::Error,
        "mismatched types",
    )
    .with_code("E0308")
    .with_source("rustc");

    let diag2 = DiagnosticItem::new(
        Range::new(Position::new(20, 0), Position::new(20, 10)),
        DiagnosticSeverity::Warning,
        "unused variable `x`",
    )
    .with_code("unused_variables")
    .with_source("rustc");

    store.update_diagnostics(&uri, vec![diag1.clone(), diag2.clone()]);

    assert_eq!(store.file_count(), 1);
    assert_eq!(store.total_diagnostics(), 2);

    let diags = store.get_diagnostics_for_path(Path::new("/workspace/src/lib.rs"));
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0], diag1);
    assert_eq!(diags[1], diag2);

    // Query via URI
    let uri_diags = store.get_diagnostics_for_uri(&uri);
    assert_eq!(uri_diags.len(), 2);

    // Clear single file
    store.clear_path(Path::new("/workspace/src/lib.rs"));
    assert_eq!(store.file_count(), 0);
    assert_eq!(store.total_diagnostics(), 0);
}

#[test]
fn test_diagnostic_store_query_by_line() {
    let store = DiagnosticStore::new();
    let path = Path::new("/project/src/main.rs");
    let uri = path_to_uri(path).unwrap();

    let diag_line5 = DiagnosticItem::new(
        Range::new(Position::new(5, 0), Position::new(5, 10)),
        DiagnosticSeverity::Error,
        "error on line 5",
    );

    let diag_span_7_to_9 = DiagnosticItem::new(
        Range::new(Position::new(7, 2), Position::new(9, 8)),
        DiagnosticSeverity::Warning,
        "warning spanning lines 7 through 9",
    );

    store.update_diagnostics(&uri, vec![diag_line5.clone(), diag_span_7_to_9.clone()]);

    // Line 5: only diag_line5
    let at_5 = store.get_diagnostics_for_line(path, 5);
    assert_eq!(at_5.len(), 1);
    assert_eq!(at_5[0].message, "error on line 5");

    // Line 6: empty
    let at_6 = store.get_diagnostics_for_line(path, 6);
    assert!(at_6.is_empty());

    // Lines 7, 8, 9: matches diag_span_7_to_9
    assert_eq!(store.get_diagnostics_for_line(path, 7).len(), 1);
    assert_eq!(store.get_diagnostics_for_line(path, 8).len(), 1);
    assert_eq!(store.get_diagnostics_for_line(path, 9).len(), 1);

    // Line 10: empty
    assert!(store.get_diagnostics_for_line(path, 10).is_empty());
}

#[test]
fn test_diagnostic_store_relative_path_lookup() {
    let store = DiagnosticStore::new();
    let abs_path = Path::new("/home/user/code/my-app/src/parser.rs");
    let uri = path_to_uri(abs_path).unwrap();

    store.update_diagnostics(
        &uri,
        vec![DiagnosticItem::new(
            Range::new(Position::new(1, 0), Position::new(1, 5)),
            DiagnosticSeverity::Information,
            "info note",
        )],
    );

    // Direct lookup by relative suffix "src/parser.rs"
    let relative_path = Path::new("src/parser.rs");
    let found = store.get_diagnostics_for_path(relative_path);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].message, "info note");
}

#[test]
fn test_diagnostic_from_lsp_types() {
    let lsp_diag = Diagnostic {
        range: LspRange {
            start: LspPosition {
                line: 12,
                character: 4,
            },
            end: LspPosition {
                line: 12,
                character: 16,
            },
        },
        severity: Some(LspSeverity::ERROR),
        code: Some(lsp_types::NumberOrString::String("E0425".to_string())),
        code_description: None,
        source: Some("rustc".to_string()),
        message: "cannot find value `foo`".to_string(),
        related_information: None,
        tags: None,
        data: None,
    };

    let item = DiagnosticItem::from_lsp(&lsp_diag);
    assert_eq!(item.start_line(), 12);
    assert_eq!(item.start_col(), 4);
    assert_eq!(item.end_line(), 12);
    assert_eq!(item.end_col(), 16);
    assert_eq!(item.severity, DiagnosticSeverity::Error);
    assert_eq!(item.code.as_deref(), Some("E0425"));
    assert_eq!(item.source.as_deref(), Some("rustc"));
    assert_eq!(item.message, "cannot find value `foo`");
}

// =========================================================================
// 3. Mock JSON-RPC Client-Server Communication Tests
// =========================================================================

#[tokio::test]
async fn test_mock_lsp_client_lifecycle_and_diagnostics() {
    let (client_io, server_io) = duplex(4096);
    let (client_read, client_write) = tokio::io::split(client_io);
    let (server_read, mut server_write) = tokio::io::split(server_io);

    let store = Arc::new(DiagnosticStore::new());
    let client = LspClient::from_split(client_read, client_write, Arc::clone(&store));

    // Spawn mock language server
    let server_task = tokio::spawn(async move {
        let mut reader = LspFramedReader::new(server_read);

        // 1. Expect initialize request
        let init_raw = reader.read_message().await.unwrap().unwrap();
        let init_json: serde_json::Value = serde_json::from_str(&init_raw).unwrap();
        assert_eq!(init_json["method"], "initialize");
        let req_id = init_json["id"].as_u64().unwrap();

        // Send initialize response
        let resp = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "capabilities": {
                    "hoverProvider": true,
                    "completionProvider": {}
                }
            }
        });
        write_message(&mut server_write, &serde_json::to_string(&resp).unwrap())
            .await
            .unwrap();

        // 2. Expect initialized notification
        let initialized_raw = reader.read_message().await.unwrap().unwrap();
        let initialized_json: serde_json::Value = serde_json::from_str(&initialized_raw).unwrap();
        assert_eq!(initialized_json["method"], "initialized");

        // 3. Send textDocument/publishDiagnostics notification
        let test_uri = "file:///workspace/test.rs";
        let diag_notif = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": test_uri,
                "diagnostics": [
                    {
                        "range": {
                            "start": { "line": 3, "character": 0 },
                            "end": { "line": 3, "character": 10 }
                        },
                        "severity": 1,
                        "message": "syntax error: unexpected token"
                    }
                ]
            }
        });
        write_message(
            &mut server_write,
            &serde_json::to_string(&diag_notif).unwrap(),
        )
        .await
        .unwrap();

        // 4. Expect shutdown request
        let shutdown_raw = reader.read_message().await.unwrap().unwrap();
        let shutdown_json: serde_json::Value = serde_json::from_str(&shutdown_raw).unwrap();
        assert_eq!(shutdown_json["method"], "shutdown");
        let shutdown_id = shutdown_json["id"].as_u64().unwrap();

        let shutdown_resp = json!({
            "jsonrpc": "2.0",
            "id": shutdown_id,
            "result": null
        });
        write_message(
            &mut server_write,
            &serde_json::to_string(&shutdown_resp).unwrap(),
        )
        .await
        .unwrap();

        // 5. Expect exit notification
        let exit_raw = reader.read_message().await.unwrap().unwrap();
        let exit_json: serde_json::Value = serde_json::from_str(&exit_raw).unwrap();
        assert_eq!(exit_json["method"], "exit");
    });

    // Client flow:
    assert!(client.is_running());

    let root = path_to_uri(Path::new("/workspace")).ok();
    let init_res = client.initialize(root).await.expect("Initialize failed");
    assert!(init_res.capabilities.hover_provider.is_some());

    // Wait briefly for diagnostic notification dispatch
    tokio::time::sleep(Duration::from_millis(50)).await;

    let diags = store.get_diagnostics_for_path(Path::new("/workspace/test.rs"));
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "syntax error: unexpected token");
    assert_eq!(diags[0].severity, DiagnosticSeverity::Error);

    client.shutdown().await.expect("Shutdown failed");
    assert!(!client.is_running());

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_mock_lsp_client_completion_and_hover() {
    let (client_io, server_io) = duplex(4096);
    let (client_read, client_write) = tokio::io::split(client_io);
    let (server_read, mut server_write) = tokio::io::split(server_io);

    let store = Arc::new(DiagnosticStore::new());
    let client = LspClient::from_split(client_read, client_write, store);

    tokio::spawn(async move {
        let mut reader = LspFramedReader::new(server_read);

        while let Ok(Some(msg)) = reader.read_message().await {
            let req: serde_json::Value = serde_json::from_str(&msg).unwrap();
            let id = req["id"].as_u64().unwrap();
            let method = req["method"].as_str().unwrap();

            match method {
                "textDocument/completion" => {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": [
                            { "label": "println!", "kind": 3, "detail": "macro" },
                            { "label": "eprintln!", "kind": 3, "detail": "macro" }
                        ]
                    });
                    write_message(&mut server_write, &serde_json::to_string(&resp).unwrap())
                        .await
                        .unwrap();
                }
                "textDocument/hover" => {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "contents": {
                                "language": "rust",
                                "value": "fn main()"
                            }
                        }
                    });
                    write_message(&mut server_write, &serde_json::to_string(&resp).unwrap())
                        .await
                        .unwrap();
                }
                _ => {}
            }
        }
    });

    let uri = path_to_uri(Path::new("/src/main.rs")).unwrap();

    // Test completion request
    let completion_res = client
        .completion(uri.clone(), Position::new(5, 8))
        .await
        .unwrap()
        .expect("Expected completion response");

    match completion_res {
        CompletionResponse::Array(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].label, "println!");
            assert_eq!(items[1].label, "eprintln!");
        }
        _ => panic!("Expected array completion response"),
    }

    // Test hover request
    let hover_res = client
        .hover(uri, Position::new(1, 4))
        .await
        .unwrap()
        .expect("Expected hover response");

    match hover_res.contents {
        HoverContents::Scalar(MarkedString::LanguageString(ls)) => {
            assert_eq!(ls.language, "rust");
            assert_eq!(ls.value, "fn main()");
        }
        other => panic!("Unexpected hover contents: {:?}", other),
    }
}

#[tokio::test]
async fn test_mock_lsp_client_rpc_error() {
    let (client_io, server_io) = duplex(4096);
    let (client_read, client_write) = tokio::io::split(client_io);
    let (server_read, mut server_write) = tokio::io::split(server_io);

    let store = Arc::new(DiagnosticStore::new());
    let client = LspClient::from_split(client_read, client_write, store);

    tokio::spawn(async move {
        let mut reader = LspFramedReader::new(server_read);
        if let Ok(Some(msg)) = reader.read_message().await {
            let req: serde_json::Value = serde_json::from_str(&msg).unwrap();
            let id = req["id"].as_u64().unwrap();

            let resp = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not supported"
                }
            });
            write_message(&mut server_write, &serde_json::to_string(&resp).unwrap())
                .await
                .unwrap();
        }
    });

    let uri = path_to_uri(Path::new("/src/unknown.rs")).unwrap();
    let err = client
        .completion(uri, Position::new(0, 0))
        .await
        .unwrap_err();

    match err {
        LspError::Rpc(rpc_err) => {
            assert_eq!(rpc_err.code, -32601);
            assert_eq!(rpc_err.message, "Method not supported");
        }
        other => panic!("Expected Rpc error, got {:?}", other),
    }
}

// =========================================================================
// 4. Server Configuration & Discovery Tests
// =========================================================================

#[test]
fn test_lsp_server_configs_defaults() {
    let manager = LspManager::new();
    let configs = manager.configs();
    assert!(!configs.is_empty());

    // Rust
    let rs_cfg = manager.get_config_for_extension("rs").unwrap();
    assert_eq!(rs_cfg.language_id, "rust");
    assert_eq!(rs_cfg.executable, "rust-analyzer");

    // Python
    let py_cfg = manager.get_config_for_extension("py").unwrap();
    assert_eq!(py_cfg.language_id, "python");

    // TypeScript
    let ts_cfg = manager.get_config_for_extension("ts").unwrap();
    assert_eq!(ts_cfg.language_id, "typescript");

    // C/C++
    let c_cfg = manager.get_config_for_extension("cpp").unwrap();
    assert_eq!(c_cfg.language_id, "c");
    assert_eq!(c_cfg.executable, "clangd");

    // Go
    let go_cfg = manager.get_config_for_extension("go").unwrap();
    assert_eq!(go_cfg.language_id, "go");
    assert_eq!(go_cfg.executable, "gopls");
}

#[test]
fn test_custom_config_registration() {
    let manager = LspManager::new();

    let custom = LspServerConfig::new("zig", vec!["zig".into()], "zls", vec!["--stdio".into()]);
    manager.register_config(custom.clone());

    let found = manager.get_config_for_extension("zig").unwrap();
    assert_eq!(found.language_id, "zig");
    assert_eq!(found.executable, "zls");
}

#[test]
fn test_path_discovery() {
    // "sh" should exist on Linux/Unix systems
    #[cfg(unix)]
    {
        assert!(find_executable_on_path("sh").is_some());
        assert!(is_executable_available("sh"));
    }

    // Non-existent binary should be None
    assert!(find_executable_on_path("non_existent_binary_test_xyz_99").is_none());
    assert!(!is_executable_available("non_existent_binary_test_xyz_99"));
}
