//! `velox-git` core engine.
//!
//! Provides Git repository operations (discovery, status, staging, unstaging, commit, branches, blame)
//! and pure Rust line-by-line / inline diffing using `similar`.

pub mod blame;
pub mod diff;
pub mod repository;

pub use git2;
pub use git2::Oid;
pub use blame::{blame_line, format_relative_time, BlameInfo};
pub use diff::{compute_diff, DiffHunk, DiffLine, FileDiff, InlineSpan, LineChange};
pub use repository::{
    BranchInfo, BranchStatus, FileChangeType, FileStatusItem, GitError, GitRepository, RepoStatus,
};

impl From<GitError> for velox_core::ForgeError {
    fn from(err: GitError) -> Self {
        velox_core::ForgeError::Internal(err.to_string())
    }
}

