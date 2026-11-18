pub mod action;
pub mod id;
pub mod log;
pub mod position;

pub use action::{Action, CommandItem};
pub use id::{BufferId, SessionId, TabId};
pub use log::init_logging;
pub use position::{Position, Range, Selection};

#[derive(thiserror::Error, Debug)]
pub enum ForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Buffer not found: {0}")]
    BufferNotFound(BufferId),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Terminal error: {0}")]
    TerminalError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ForgeError>;
