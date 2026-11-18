use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Represents a filesystem event relevant to the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEvent {
    Modified(PathBuf),
    Created(PathBuf),
    Deleted(PathBuf),
}

impl FileEvent {
    pub fn path(&self) -> &Path {
        match self {
            FileEvent::Modified(p) | FileEvent::Created(p) | FileEvent::Deleted(p) => p,
        }
    }

    pub fn is_modified(&self) -> bool {
        matches!(self, FileEvent::Modified(_))
    }

    pub fn is_created(&self) -> bool {
        matches!(self, FileEvent::Created(_))
    }

    pub fn is_deleted(&self) -> bool {
        matches!(self, FileEvent::Deleted(_))
    }
}

/// Errors produced during file watching.
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Watcher already stopped")]
    AlreadyStopped,
}

impl From<WatcherError> for std::io::Error {
    fn from(err: WatcherError) -> Self {
        match err {
            WatcherError::Io(e) => e,
            WatcherError::Notify(e) => std::io::Error::other(e),
            WatcherError::AlreadyStopped => std::io::Error::other("Watcher already stopped"),
        }
    }
}

/// Determines whether a given path should be ignored by the watcher.
/// Ignores `.git`, `target`, and `node_modules` folders.
pub fn should_ignore_path(path: &Path) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            let s = name.to_string_lossy();
            if s == ".git" || s == "target" || s == "node_modules" {
                return true;
            }
        }
    }
    false
}

/// A filesystem watcher with debounced event emissions.
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    stop_tx: Option<mpsc::Sender<()>>,
    debounce_delay: Duration,
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileWatcher {
    /// Creates a new `FileWatcher` with a default debounce delay of 50ms.
    pub fn new() -> Self {
        Self {
            watcher: None,
            stop_tx: None,
            debounce_delay: Duration::from_millis(50),
        }
    }

    /// Creates a new `FileWatcher` with a custom debounce delay.
    pub fn with_debounce(debounce_delay: Duration) -> Self {
        Self {
            watcher: None,
            stop_tx: None,
            debounce_delay,
        }
    }

    /// Returns whether the watcher is currently active.
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Returns the configured debounce delay.
    pub fn debounce_delay(&self) -> Duration {
        self.debounce_delay
    }

    /// Starts watching `path` recursively for filesystem changes, invoking `callback`
    /// with debounced `FileEvent`s.
    pub fn start_watching<F: Fn(FileEvent) + Send + 'static>(
        &mut self,
        path: &Path,
        callback: F,
    ) -> Result<(), WatcherError> {
        self.stop_watching();

        if !path.exists() {
            return Err(WatcherError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            )));
        }

        let (raw_tx, raw_rx) = mpsc::channel::<(PathBuf, notify::EventKind)>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        let raw_sender = raw_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    for p in event.paths {
                        if !should_ignore_path(&p) {
                            let _ = raw_sender.send((p, event.kind));
                        }
                    }
                }
            },
            Config::default(),
        )?;

        let recursive_mode = if path.is_file() {
            RecursiveMode::NonRecursive
        } else {
            RecursiveMode::Recursive
        };
        watcher.watch(path, recursive_mode)?;

        let debounce_delay = self.debounce_delay;
        std::thread::Builder::new()
            .name("velox-file-watcher-debounce".into())
            .spawn(move || {
                let mut pending: HashMap<PathBuf, (FileEvent, Instant)> = HashMap::new();

                loop {
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }

                    let now = Instant::now();
                    let mut ready_keys = Vec::new();
                    let mut next_wake = None;

                    for (p, (_ev, time)) in &pending {
                        let elapsed = now.duration_since(*time);
                        if elapsed >= debounce_delay {
                            ready_keys.push(p.clone());
                        } else {
                            let remaining = debounce_delay - elapsed;
                            match next_wake {
                                None => next_wake = Some(remaining),
                                Some(prev) if remaining < prev => next_wake = Some(remaining),
                                _ => {}
                            }
                        }
                    }

                    for key in ready_keys {
                        if let Some((event, _)) = pending.remove(&key) {
                            callback(event);
                        }
                    }

                    let timeout = next_wake.unwrap_or(Duration::from_millis(100));

                    match raw_rx.recv_timeout(timeout) {
                        Ok((path, kind)) => {
                            let event_opt = match kind {
                                notify::EventKind::Create(_) => {
                                    Some(FileEvent::Created(path.clone()))
                                }
                                notify::EventKind::Modify(_) => {
                                    Some(FileEvent::Modified(path.clone()))
                                }
                                notify::EventKind::Remove(_) => {
                                    Some(FileEvent::Deleted(path.clone()))
                                }
                                _ => None,
                            };

                            if let Some(new_event) = event_opt {
                                let now = Instant::now();
                                let should_remove = matches!(
                                    (pending.get(&path), &new_event),
                                    (Some((FileEvent::Created(_), _)), FileEvent::Deleted(_))
                                );

                                if should_remove {
                                    pending.remove(&path);
                                } else if let Some((existing, timestamp)) = pending.get_mut(&path) {
                                    if matches!(
                                        (&existing, &new_event),
                                        (FileEvent::Created(_), FileEvent::Modified(_))
                                    ) {
                                        *timestamp = now;
                                    } else {
                                        *existing = new_event;
                                        *timestamp = now;
                                    }
                                } else {
                                    pending.insert(path, (new_event, now));
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Loop around to expire any ready items
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            // Watcher stopped or raw_tx dropped, flush remaining events
                            for (_p, (event, _)) in pending.drain() {
                                callback(event);
                            }
                            break;
                        }
                    }
                }
            })
            .map_err(WatcherError::Io)?;

        self.watcher = Some(watcher);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }

    /// Stops the active watcher and debounce thread.
    pub fn stop_watching(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        self.watcher = None;
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}
