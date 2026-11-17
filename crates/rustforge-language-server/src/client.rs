//! Asynchronous JSON-RPC 2.0 client for Language Server Protocol communication.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::diagnostics::DiagnosticStore;
use crate::transport::{write_message, LspFramedReader};
use crate::LspError;

/// Standard JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("LSP RPC Error (code {code}): {message}")]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// An incoming or outgoing LSP notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct OutgoingRequest<'a, P> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

#[derive(Serialize)]
struct OutgoingNotification<'a, P> {
    jsonrpc: &'static str,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

#[derive(Serialize)]
struct ServerResponse<'a> {
    jsonrpc: &'static str,
    id: &'a serde_json::Value,
    result: serde_json::Value,
}

type ResponseMap = HashMap<u64, oneshot::Sender<Result<serde_json::Value, JsonRpcError>>>;

/// An active LSP client communicating via JSON-RPC 2.0 over standard I/O or in-memory streams.
pub struct LspClient {
    next_request_id: Arc<AtomicU64>,
    outgoing_tx: mpsc::Sender<String>,
    pending_requests: Arc<Mutex<ResponseMap>>,
    diagnostic_store: Arc<DiagnosticStore>,
    is_running: Arc<AtomicBool>,
    child_process: Option<Arc<tokio::sync::Mutex<tokio::process::Child>>>,
    notification_tx: broadcast::Sender<NotificationMessage>,
}

impl fmt::Debug for LspClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LspClient")
            .field("is_running", &self.is_running.load(Ordering::SeqCst))
            .field(
                "pending_requests_count",
                &self.pending_requests.lock().len(),
            )
            .finish()
    }
}

impl LspClient {
    /// Creates an `LspClient` from split async read and write streams (e.g. duplex stream for testing).
    pub fn from_split<R, W>(
        reader: R,
        mut writer: W,
        diagnostic_store: Arc<DiagnosticStore>,
    ) -> Self
    where
        R: AsyncRead + Send + Unpin + 'static,
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let next_request_id = Arc::new(AtomicU64::new(1));
        let pending_requests: Arc<Mutex<ResponseMap>> = Arc::new(Mutex::new(HashMap::new()));
        let is_running = Arc::new(AtomicBool::new(true));
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(256);
        let (notification_tx, _) = broadcast::channel(128);

        // Spawn writer background task
        let writer_running = Arc::clone(&is_running);
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Err(e) = write_message(&mut writer, &msg).await {
                    tracing::error!("Failed to write message to LSP transport: {:?}", e);
                    break;
                }
            }
            writer_running.store(false, Ordering::SeqCst);
        });

        // Spawn reader background task
        let reader_pending = Arc::clone(&pending_requests);
        let reader_store = Arc::clone(&diagnostic_store);
        let reader_running = Arc::clone(&is_running);
        let reader_reply_tx = outgoing_tx.clone();
        let reader_notify_tx = notification_tx.clone();

        tokio::spawn(async move {
            let mut framed_reader = LspFramedReader::new(reader);

            while let Ok(Some(payload)) = framed_reader.read_message().await {
                let parsed: serde_json::Value = match serde_json::from_str(&payload) {
                    Ok(val) => val,
                    Err(e) => {
                        tracing::warn!("Failed to deserialize incoming LSP JSON: {:?}", e);
                        continue;
                    }
                };

                // Case 1: Server request to client (has both "id" and "method")
                if let (Some(id), Some(method)) = (parsed.get("id"), parsed.get("method")) {
                    let method_str = method.as_str().unwrap_or_default();
                    tracing::debug!(
                        "Received server request: method={}, id={:?}",
                        method_str,
                        id
                    );

                    let response = ServerResponse {
                        jsonrpc: "2.0",
                        id,
                        result: serde_json::Value::Null,
                    };
                    if let Ok(resp_str) = serde_json::to_string(&response) {
                        let _ = reader_reply_tx.send(resp_str).await;
                    }
                    continue;
                }

                // Case 2: Server response to client request (has "id" and ("result" or "error"))
                if let Some(id_val) = parsed.get("id") {
                    if let Some(id) = id_val.as_u64() {
                        let maybe_tx = reader_pending.lock().remove(&id);
                        if let Some(tx) = maybe_tx {
                            if let Some(err_val) = parsed.get("error") {
                                match serde_json::from_value::<JsonRpcError>(err_val.clone()) {
                                    Ok(rpc_err) => {
                                        let _ = tx.send(Err(rpc_err));
                                    }
                                    Err(_) => {
                                        let _ = tx.send(Err(JsonRpcError {
                                            code: -32603,
                                            message: format!("Unrecognized error format: {err_val}"),
                                            data: None,
                                        }));
                                    }
                                }
                            } else {
                                let res = parsed
                                    .get("result")
                                    .cloned()
                                    .unwrap_or(serde_json::Value::Null);
                                let _ = tx.send(Ok(res));
                            }
                        }
                    }
                    continue;
                }

                // Case 3: Server notification (has "method", no "id")
                if let Some(method_val) = parsed.get("method") {
                    let method_name = method_val.as_str().unwrap_or_default().to_string();
                    let params = parsed.get("params").cloned();

                    // Handle publishDiagnostics
                    if method_name == "textDocument/publishDiagnostics" {
                        if let Some(ref p) = params {
                            if let Ok(diag_params) =
                                serde_json::from_value::<lsp_types::PublishDiagnosticsParams>(
                                    p.clone(),
                                )
                            {
                                reader_store.update_from_lsp(
                                    &diag_params.uri,
                                    &diag_params.diagnostics,
                                );
                            }
                        }
                    }

                    let _ = reader_notify_tx.send(NotificationMessage {
                        method: method_name,
                        params,
                    });
                }
            }

            // Reader exited: connection closed
            reader_running.store(false, Ordering::SeqCst);
            let mut pending = reader_pending.lock();
            for (_, tx) in pending.drain() {
                let _ = tx.send(Err(JsonRpcError {
                    code: -32000,
                    message: "LSP server connection closed".to_string(),
                    data: None,
                }));
            }
        });

        Self {
            next_request_id,
            outgoing_tx,
            pending_requests,
            diagnostic_store,
            is_running,
            child_process: None,
            notification_tx,
        }
    }

    /// Spawns a child language server process and connects via piped stdin and stdout.
    pub fn spawn_process(
        command: &str,
        args: &[String],
        diagnostic_store: Arc<DiagnosticStore>,
    ) -> Result<Self, LspError> {
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::Client("Failed to capture child process stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::Client("Failed to capture child process stdout".into()))?;

        // Optional stderr drain task
        if let Some(mut stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 1024];
                while let Ok(n) = stderr.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                        tracing::debug!("[LSP stderr] {}", s.trim());
                    }
                }
            });
        }

        let mut client = Self::from_split(stdout, stdin, diagnostic_store);
        client.child_process = Some(Arc::new(tokio::sync::Mutex::new(child)));
        Ok(client)
    }

    /// Returns whether the LSP client is currently connected and running.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Access the shared diagnostic store.
    pub fn diagnostic_store(&self) -> Arc<DiagnosticStore> {
        Arc::clone(&self.diagnostic_store)
    }

    /// Subscribes to incoming LSP notifications.
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<NotificationMessage> {
        self.notification_tx.subscribe()
    }

    /// Sends a JSON-RPC request and awaits the deserialized result.
    pub async fn request<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, LspError> {
        if !self.is_running() {
            return Err(LspError::Disconnected);
        }

        let id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        self.pending_requests.lock().insert(id, tx);

        let req = OutgoingRequest {
            jsonrpc: "2.0",
            id,
            method,
            params: Some(params),
        };

        let json_str = serde_json::to_string(&req)?;
        if self.outgoing_tx.send(json_str).await.is_err() {
            self.pending_requests.lock().remove(&id);
            return Err(LspError::Disconnected);
        }

        match rx.await {
            Ok(Ok(val)) => {
                let res = serde_json::from_value(val)?;
                Ok(res)
            }
            Ok(Err(rpc_err)) => Err(LspError::Rpc(rpc_err)),
            Err(_) => Err(LspError::Disconnected),
        }
    }

    /// Sends a JSON-RPC notification.
    pub async fn notify<P: Serialize>(&self, method: &str, params: P) -> Result<(), LspError> {
        if !self.is_running() {
            return Err(LspError::Disconnected);
        }

        let notif = OutgoingNotification {
            jsonrpc: "2.0",
            method,
            params: Some(params),
        };

        let json_str = serde_json::to_string(&notif)?;
        self.outgoing_tx
            .send(json_str)
            .await
            .map_err(|_| LspError::Disconnected)?;

        Ok(())
    }

    /// Sends the `initialize` request and subsequent `initialized` notification.
    pub async fn initialize(
        &self,
        root_uri: Option<lsp_types::Uri>,
    ) -> Result<lsp_types::InitializeResult, LspError> {
        #[allow(deprecated)]
        let params = lsp_types::InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: root_uri.clone(),
            capabilities: lsp_types::ClientCapabilities::default(),
            initialization_options: None,
            trace: None,
            workspace_folders: root_uri.map(|u| {
                vec![lsp_types::WorkspaceFolder {
                    uri: u,
                    name: "workspace".to_string(),
                }]
            }),
            client_info: Some(lsp_types::ClientInfo {
                name: "RustForge".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            locale: None,
            root_path: None,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
        };

        let result: lsp_types::InitializeResult = self.request("initialize", params).await?;

        // Conform to LSP specification: send "initialized" notification immediately after response
        let init_notif = lsp_types::InitializedParams {};
        let _ = self.notify("initialized", init_notif).await;

        Ok(result)
    }

    /// Notifies the language server that a document was opened.
    pub async fn did_open(
        &self,
        uri: lsp_types::Uri,
        language_id: String,
        version: i32,
        text: String,
    ) -> Result<(), LspError> {
        let params = lsp_types::DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri,
                language_id,
                version,
                text,
            },
        };
        self.notify("textDocument/didOpen", params).await
    }

    /// Notifies the language server of document changes (full-buffer sync).
    pub async fn did_change(
        &self,
        uri: lsp_types::Uri,
        version: i32,
        text: String,
    ) -> Result<(), LspError> {
        let params = lsp_types::DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier { uri, version },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text,
            }],
        };
        self.notify("textDocument/didChange", params).await
    }

    /// Notifies the language server that a document was saved.
    pub async fn did_save(
        &self,
        uri: lsp_types::Uri,
        text: Option<String>,
    ) -> Result<(), LspError> {
        let params = lsp_types::DidSaveTextDocumentParams {
            text_document: lsp_types::TextDocumentIdentifier { uri },
            text,
        };
        self.notify("textDocument/didSave", params).await
    }

    /// Requests code completion at a given line and column position.
    pub async fn completion(
        &self,
        uri: lsp_types::Uri,
        position: rustforge_core::Position,
    ) -> Result<Option<lsp_types::CompletionResponse>, LspError> {
        let params = lsp_types::CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri },
                position: lsp_types::Position {
                    line: position.line as u32,
                    character: position.column as u32,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        self.request("textDocument/completion", params).await
    }

    /// Requests hover tooltip information at a given position.
    pub async fn hover(
        &self,
        uri: lsp_types::Uri,
        position: rustforge_core::Position,
    ) -> Result<Option<lsp_types::Hover>, LspError> {
        let params = lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri },
                position: lsp_types::Position {
                    line: position.line as u32,
                    character: position.column as u32,
                },
            },
            work_done_progress_params: Default::default(),
        };
        self.request("textDocument/hover", params).await
    }

    /// Requests go-to definition locations.
    pub async fn goto_definition(
        &self,
        uri: lsp_types::Uri,
        position: rustforge_core::Position,
    ) -> Result<Option<lsp_types::GotoDefinitionResponse>, LspError> {
        let params = lsp_types::GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri },
                position: lsp_types::Position {
                    line: position.line as u32,
                    character: position.column as u32,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        self.request("textDocument/definition", params).await
    }

    /// Requests document formatting edits.
    pub async fn formatting(
        &self,
        uri: lsp_types::Uri,
        tab_size: u32,
        insert_spaces: bool,
    ) -> Result<Option<Vec<lsp_types::TextEdit>>, LspError> {
        let params = lsp_types::DocumentFormattingParams {
            text_document: lsp_types::TextDocumentIdentifier { uri },
            options: lsp_types::FormattingOptions {
                tab_size,
                insert_spaces,
                properties: Default::default(),
                trim_trailing_whitespace: Some(true),
                insert_final_newline: Some(true),
                trim_final_newlines: Some(true),
            },
            work_done_progress_params: Default::default(),
        };
        self.request("textDocument/formatting", params).await
    }

    /// Gracefully shuts down the LSP server session and terminates any child process.
    pub async fn shutdown(&self) -> Result<(), LspError> {
        let shutdown_res: Result<serde_json::Value, _> =
            self.request("shutdown", serde_json::Value::Null).await;
        let _ = self.notify("exit", serde_json::Value::Null).await;

        self.is_running.store(false, Ordering::SeqCst);

        if let Some(ref child_lock) = self.child_process {
            let mut child = child_lock.lock().await;
            let wait_timeout =
                tokio::time::timeout(std::time::Duration::from_millis(500), child.wait()).await;
            if wait_timeout.is_err() {
                let _ = child.kill().await;
            }
        }

        let _ = shutdown_res;
        Ok(())
    }
}
