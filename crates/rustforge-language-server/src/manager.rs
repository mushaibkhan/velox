//! LSP server discovery, configuration, and multi-language manager.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::client::LspClient;
use crate::diagnostics::DiagnosticStore;
use crate::LspError;

/// Configuration defining an LSP server executable and supported file types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspServerConfig {
    pub language_id: String,
    pub extensions: Vec<String>,
    pub executable: String,
    pub args: Vec<String>,
}

impl LspServerConfig {
    /// Creates a new server configuration.
    pub fn new(
        language_id: impl Into<String>,
        extensions: Vec<String>,
        executable: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        Self {
            language_id: language_id.into(),
            extensions,
            executable: executable.into(),
            args,
        }
    }

    /// Default LSP server configurations for standard languages.
    pub fn default_configs() -> Vec<Self> {
        vec![
            Self::new("rust", vec!["rs".into()], "rust-analyzer", vec![]),
            Self::new(
                "python",
                vec!["py".into(), "pyi".into()],
                "pyright-langserver",
                vec!["--stdio".into()],
            ),
            Self::new(
                "python",
                vec!["py".into(), "pyi".into()],
                "pylsp",
                vec![],
            ),
            Self::new(
                "typescript",
                vec![
                    "ts".into(),
                    "tsx".into(),
                    "js".into(),
                    "jsx".into(),
                    "mjs".into(),
                    "cjs".into(),
                ],
                "typescript-language-server",
                vec!["--stdio".into()],
            ),
            Self::new(
                "c",
                vec![
                    "c".into(),
                    "cpp".into(),
                    "cc".into(),
                    "cxx".into(),
                    "h".into(),
                    "hpp".into(),
                ],
                "clangd",
                vec![],
            ),
            Self::new("go", vec!["go".into()], "gopls", vec![]),
        ]
    }
}

/// Checks the system `PATH` to find the absolute path of an executable.
pub fn find_executable_on_path(executable: &str) -> Option<PathBuf> {
    let p = Path::new(executable);
    if p.components().count() > 1 {
        if p.is_file() {
            return Some(p.to_path_buf());
        }
        return None;
    }

    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(executable);
            #[cfg(windows)]
            let candidate = if candidate.extension().is_none() {
                candidate.with_extension("exe")
            } else {
                candidate
            };
            if candidate.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = candidate.metadata() {
                        if meta.permissions().mode() & 0o111 != 0 {
                            return Some(candidate);
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// Returns whether an executable exists and is executable on `PATH`.
pub fn is_executable_available(executable: &str) -> bool {
    find_executable_on_path(executable).is_some()
}

/// Central manager orchestrating LSP clients, configurations, and the shared diagnostic store.
pub struct LspManager {
    diagnostic_store: Arc<DiagnosticStore>,
    configs: RwLock<Vec<LspServerConfig>>,
    clients: RwLock<HashMap<String, Arc<LspClient>>>,
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LspManager {
    /// Creates a new `LspManager` with default server configurations.
    pub fn new() -> Self {
        Self {
            diagnostic_store: Arc::new(DiagnosticStore::new()),
            configs: RwLock::new(LspServerConfig::default_configs()),
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// Creates an `LspManager` with a provided shared diagnostic store.
    pub fn with_store(diagnostic_store: Arc<DiagnosticStore>) -> Self {
        Self {
            diagnostic_store,
            configs: RwLock::new(LspServerConfig::default_configs()),
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// Access the shared diagnostic store.
    pub fn diagnostic_store(&self) -> Arc<DiagnosticStore> {
        Arc::clone(&self.diagnostic_store)
    }

    /// Registers a custom or override server configuration.
    pub fn register_config(&self, config: LspServerConfig) {
        let mut configs = self.configs.write();
        // Insert at the front so custom registrations take precedence
        configs.insert(0, config);
    }

    /// Returns a snapshot of all registered configurations.
    pub fn configs(&self) -> Vec<LspServerConfig> {
        self.configs.read().clone()
    }

    /// Resolves the first server configuration matching a file extension.
    pub fn get_config_for_extension(&self, ext: &str) -> Option<LspServerConfig> {
        let clean_ext = ext.trim_start_matches('.').to_ascii_lowercase();
        self.configs
            .read()
            .iter()
            .find(|cfg| {
                cfg.extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(&clean_ext))
            })
            .cloned()
    }

    /// Resolves the first available (installed on `PATH`) server configuration for an extension.
    pub fn find_available_config_for_extension(&self, ext: &str) -> Option<LspServerConfig> {
        let clean_ext = ext.trim_start_matches('.').to_ascii_lowercase();
        self.configs
            .read()
            .iter()
            .find(|cfg| {
                cfg.extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(&clean_ext))
                    && is_executable_available(&cfg.executable)
            })
            .cloned()
    }

    /// Resolves the primary configuration for a given language ID.
    pub fn get_config_for_language(&self, language_id: &str) -> Option<LspServerConfig> {
        self.configs
            .read()
            .iter()
            .find(|cfg| cfg.language_id.eq_ignore_ascii_case(language_id))
            .cloned()
    }

    /// Retrieves an active client for a language ID, if one is running.
    pub fn get_client(&self, language_id: &str) -> Option<Arc<LspClient>> {
        self.clients.read().get(language_id).cloned()
    }

    /// Retrieves an active client for a given file path based on its extension.
    pub fn get_client_for_path(&self, path: &Path) -> Option<Arc<LspClient>> {
        let ext = path.extension()?.to_str()?;
        let cfg = self.get_config_for_extension(ext)?;
        self.get_client(&cfg.language_id)
    }

    /// Registers an existing client (e.g. mock client or in-memory transport for testing).
    pub fn register_client(&self, language_id: impl Into<String>, client: Arc<LspClient>) {
        self.clients.write().insert(language_id.into(), client);
    }

    /// Starts or returns an existing LSP client for a given language ID.
    pub async fn start_client(
        &self,
        language_id: &str,
        root_uri: Option<lsp_types::Uri>,
    ) -> Result<Arc<LspClient>, LspError> {
        // Return existing active client if available
        if let Some(client) = self.get_client(language_id) {
            if client.is_running() {
                return Ok(client);
            }
        }

        // Find available config
        let configs = self.configs.read().clone();
        let cfg = configs
            .into_iter()
            .find(|c| c.language_id.eq_ignore_ascii_case(language_id) && is_executable_available(&c.executable))
            .or_else(|| self.get_config_for_language(language_id))
            .ok_or_else(|| LspError::ServerNotFound(format!("No LSP config found for language '{language_id}'")))?;

        if !is_executable_available(&cfg.executable) {
            return Err(LspError::ServerNotFound(format!(
                "Executable '{}' for language '{}' not found on PATH",
                cfg.executable, language_id
            )));
        }

        let client = LspClient::spawn_process(
            &cfg.executable,
            &cfg.args,
            Arc::clone(&self.diagnostic_store),
        )?;

        client.initialize(root_uri).await?;

        let client_arc = Arc::new(client);
        self.clients
            .write()
            .insert(cfg.language_id.clone(), Arc::clone(&client_arc));

        Ok(client_arc)
    }

    /// Starts or returns an existing LSP client suitable for a given file path.
    pub async fn start_client_for_path(
        &self,
        path: &Path,
        root_uri: Option<lsp_types::Uri>,
    ) -> Result<Arc<LspClient>, LspError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| LspError::ServerNotFound("File has no extension".into()))?;

        let cfg = self
            .find_available_config_for_extension(ext)
            .or_else(|| self.get_config_for_extension(ext))
            .ok_or_else(|| {
                LspError::ServerNotFound(format!("No LSP config found for extension '.{ext}'"))
            })?;

        self.start_client(&cfg.language_id, root_uri).await
    }

    /// Gracefully stops an active client for a language ID.
    pub async fn stop_client(&self, language_id: &str) -> Result<(), LspError> {
        let client = self.clients.write().remove(language_id);
        if let Some(c) = client {
            c.shutdown().await?;
        }
        Ok(())
    }

    /// Gracefully stops all active language server clients.
    pub async fn stop_all(&self) -> Result<(), LspError> {
        let clients: Vec<Arc<LspClient>> = self.clients.write().drain().map(|(_, c)| c).collect();
        for client in clients {
            let _ = client.shutdown().await;
        }
        Ok(())
    }
}
