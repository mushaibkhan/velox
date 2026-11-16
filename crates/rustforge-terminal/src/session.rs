use crate::grid::TerminalGrid;
use crate::parser::GridPerformer;
use parking_lot::RwLock;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtyPair, PtySize};
use rustforge_core::SessionId;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub struct TerminalSession {
    pub id: SessionId,
    pub title: String,
    pub grid: Arc<RwLock<TerminalGrid>>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    is_alive: Arc<AtomicBool>,
}

impl TerminalSession {
    pub fn spawn(
        cols: u16,
        rows: u16,
        working_dir: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pty_system = native_pty_system();
        let pair: PtyPair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "powershell.exe".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });

        let mut cmd = CommandBuilder::new(&shell);
        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        }

        let _child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let grid = Arc::new(RwLock::new(TerminalGrid::new(cols as usize, rows as usize)));
        let is_alive = Arc::new(AtomicBool::new(true));

        // Reader background thread
        let reader_grid = Arc::clone(&grid);
        let reader_alive = Arc::clone(&is_alive);

        thread::Builder::new()
            .name("terminal-pty-reader".into())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                let mut parser = vte::Parser::new();

                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => {
                            reader_alive.store(false, Ordering::Relaxed);
                            break;
                        }
                        Ok(n) => {
                            let mut grid_guard = reader_grid.write();
                            let mut performer = GridPerformer {
                                grid: &mut grid_guard,
                            };
                            parser.advance(&mut performer, &buf[..n]);
                        }
                        Err(e) => {
                            tracing::debug!("Terminal PTY read error: {}", e);
                            reader_alive.store(false, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            })?;

        Ok(Self {
            id: SessionId::new(),
            title: shell,
            grid,
            master: pair.master,
            writer,
            is_alive,
        })
    }

    pub fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()
    }

    pub fn write_bytes(&mut self, b: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(b)?;
        self.writer.flush()
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        self.grid.write().resize(cols as usize, rows as usize);
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::Relaxed)
    }
}
