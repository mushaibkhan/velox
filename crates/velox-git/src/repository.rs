use std::path::{Path, PathBuf};
use git2::{IndexAddOption, Oid, Repository, Status, StatusOptions};

use crate::blame::{blame_line, BlameInfo};
use crate::diff::{compute_diff, FileDiff};

/// Git operation error classifications.
#[derive(thiserror::Error, Debug)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Repository not found")]
    NotFound,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    #[error("Other error: {0}")]
    Other(String),
}

/// The type of change for a file in git status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Typechange,
}

/// A status item representing a single changed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatusItem {
    pub path: PathBuf,
    pub status: FileChangeType,
}

/// The complete categorized working tree & staging status.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RepoStatus {
    /// Files with changes staged in index ready to be committed.
    pub staged: Vec<FileStatusItem>,
    /// Tracked files modified or deleted in working tree but not staged.
    pub unstaged: Vec<FileStatusItem>,
    /// Untracked files in working tree.
    pub untracked: Vec<PathBuf>,
    /// Conflicted files during merge or rebase.
    pub conflicted: Vec<PathBuf>,
}

impl RepoStatus {
    /// Returns true if working directory and index are completely clean.
    pub fn is_clean(&self) -> bool {
        self.staged.is_empty()
            && self.unstaged.is_empty()
            && self.untracked.is_empty()
            && self.conflicted.is_empty()
    }
}

/// Upstream sync status for current or designated branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BranchStatus {
    pub ahead: usize,
    pub behind: usize,
}

/// Detailed information about a Git branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub ahead: usize,
    pub behind: usize,
}

/// High-level wrapper around libgit2 `Repository`.
pub struct GitRepository {
    repo: Repository,
    workdir: PathBuf,
}

impl GitRepository {
    /// Discover a Git repository starting at `path` and traversing parents.
    pub fn discover(path: &Path) -> Option<Self> {
        let repo = Repository::discover(path).ok()?;
        let workdir = repo
            .workdir()
            .map(|w| w.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf());
        Some(Self { repo, workdir })
    }

    /// Open a Git repository at an exact directory path.
    pub fn open(path: &Path) -> Result<Self, GitError> {
        let repo = Repository::open(path)?;
        let workdir = repo
            .workdir()
            .map(|w| w.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf());
        Ok(Self { repo, workdir })
    }

    /// Access the underlying `git2::Repository`.
    pub fn raw(&self) -> &Repository {
        &self.repo
    }

    /// Get the working directory path.
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    /// Helper to convert absolute or relative paths to relative against `workdir`.
    pub fn relative_path<'a>(&self, path: &'a Path) -> &'a Path {
        if let Ok(rel) = path.strip_prefix(&self.workdir) {
            rel
        } else {
            path
        }
    }

    /// Scan repository working tree and index status.
    pub fn status(&self) -> Result<RepoStatus, GitError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);
        opts.renames_head_to_index(true);
        opts.renames_index_to_workdir(true);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicted = Vec::new();

        for entry in statuses.iter() {
            let s = entry.status();
            let path = PathBuf::from(entry.path().unwrap_or_default());

            if s.contains(Status::CONFLICTED) {
                conflicted.push(path.clone());
                continue;
            }

            if s.contains(Status::WT_NEW) {
                untracked.push(path.clone());
            }

            // Staged changes (Index)
            if s.contains(Status::INDEX_NEW) {
                staged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Added,
                });
            } else if s.contains(Status::INDEX_MODIFIED) {
                staged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Modified,
                });
            } else if s.contains(Status::INDEX_DELETED) {
                staged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Deleted,
                });
            } else if s.contains(Status::INDEX_RENAMED) {
                staged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Renamed,
                });
            } else if s.contains(Status::INDEX_TYPECHANGE) {
                staged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Typechange,
                });
            }

            // Unstaged changes (Worktree)
            if s.contains(Status::WT_MODIFIED) {
                unstaged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Modified,
                });
            } else if s.contains(Status::WT_DELETED) {
                unstaged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Deleted,
                });
            } else if s.contains(Status::WT_RENAMED) {
                unstaged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Renamed,
                });
            } else if s.contains(Status::WT_TYPECHANGE) {
                unstaged.push(FileStatusItem {
                    path: path.clone(),
                    status: FileChangeType::Typechange,
                });
            }
        }

        staged.sort_by(|a, b| a.path.cmp(&b.path));
        unstaged.sort_by(|a, b| a.path.cmp(&b.path));
        untracked.sort();
        conflicted.sort();

        Ok(RepoStatus {
            staged,
            unstaged,
            untracked,
            conflicted,
        })
    }

    /// Stage a file into the git index (handles creation, modification, or deletion).
    pub fn stage_file(&self, path: &Path) -> Result<(), GitError> {
        let rel = self.relative_path(path);
        let full = self.workdir.join(rel);
        let mut index = self.repo.index()?;

        if full.exists() {
            index.add_path(rel)?;
        } else {
            index.remove_path(rel)?;
        }

        index.write()?;
        Ok(())
    }

    /// Unstage a file from the index back to working tree state.
    pub fn unstage_file(&self, path: &Path) -> Result<(), GitError> {
        let rel = self.relative_path(path);
        let mut index = self.repo.index()?;

        match self.repo.head() {
            Ok(head) => {
                let commit = head.peel_to_commit()?;
                self.repo.reset_default(Some(commit.as_object()), [rel])?;
            }
            Err(_) => {
                let _ = index.remove_path(rel);
                index.write()?;
            }
        }
        Ok(())
    }

    /// Stage all tracked modifications, deletions, and untracked files.
    pub fn stage_all(&self) -> Result<(), GitError> {
        let mut index = self.repo.index()?;
        index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
        index.update_all(["*"], None)?;
        index.write()?;
        Ok(())
    }

    /// Discard unstaged changes to a file, reverting to index or HEAD state.
    pub fn discard_file(&self, path: &Path) -> Result<(), GitError> {
        let rel = self.relative_path(path);
        let full = self.workdir.join(rel);

        if let Ok(st) = self.repo.status_file(rel) {
            if st.contains(Status::WT_NEW) {
                if full.is_dir() {
                    std::fs::remove_dir_all(&full)?;
                } else if full.exists() {
                    std::fs::remove_file(&full)?;
                }
                return Ok(());
            }
        }

        let mut checkout = git2::build::CheckoutBuilder::new();
        checkout.path(rel);
        checkout.force();
        self.repo.checkout_index(None, Some(&mut checkout))?;
        Ok(())
    }

    /// Create a commit with staged index changes.
    pub fn commit(
        &self,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<Oid, GitError> {
        let mut index = self.repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;
        let sig = git2::Signature::now(author_name, author_email)?;

        let parent_commit = self
            .repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());

        let parents: Vec<&git2::Commit> = match &parent_commit {
            Some(p) => vec![p],
            None => vec![],
        };

        let oid = self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parents,
        )?;
        Ok(oid)
    }

    /// Get the current branch name (or short commit hash if detached HEAD).
    pub fn current_branch(&self) -> Result<String, GitError> {
        match self.repo.head() {
            Ok(head) => {
                if head.is_branch() {
                    Ok(head.shorthand().unwrap_or("HEAD").to_string())
                } else {
                    let oid = head
                        .target()
                        .map(|o| o.to_string())
                        .unwrap_or_else(|| "HEAD".to_string());
                    Ok(format!("HEAD ({})", &oid[..7.min(oid.len())]))
                }
            }
            Err(_) => {
                if let Ok(head_ref) = self.repo.find_reference("HEAD") {
                    if let Some(target) = head_ref.symbolic_target() {
                        let name = target.strip_prefix("refs/heads/").unwrap_or(target);
                        return Ok(name.to_string());
                    }
                }
                Ok("main".to_string())
            }
        }
    }

    /// Retrieve ahead/behind counts for current HEAD against upstream.
    pub fn branch_status(&self) -> Result<BranchStatus, GitError> {
        let head = match self.repo.head() {
            Ok(h) => h,
            Err(_) => return Ok(BranchStatus::default()),
        };
        let local_oid = match head.target() {
            Some(oid) => oid,
            None => return Ok(BranchStatus::default()),
        };

        if head.is_branch() {
            let branch_name = head.shorthand().unwrap_or("HEAD");
            if let Ok(branch) = self.repo.find_branch(branch_name, git2::BranchType::Local) {
                if let Ok(upstream) = branch.upstream() {
                    if let Some(upstream_oid) = upstream.get().target() {
                        let (ahead, behind) = self.repo.graph_ahead_behind(local_oid, upstream_oid)?;
                        return Ok(BranchStatus { ahead, behind });
                    }
                }
            }
        }
        Ok(BranchStatus::default())
    }

    /// List local branches with HEAD and upstream tracking status.
    pub fn branches(&self) -> Result<Vec<BranchInfo>, GitError> {
        let mut result = Vec::new();
        let current = self.current_branch().unwrap_or_default();
        let branches = self.repo.branches(Some(git2::BranchType::Local))?;
        for item in branches {
            let (branch, _) = item?;
            let name = branch.name()?.unwrap_or_default().to_string();
            let is_head = name == current;
            let (ahead, behind) = if let Ok(upstream) = branch.upstream() {
                if let (Some(loc_oid), Some(up_oid)) =
                    (branch.get().target(), upstream.get().target())
                {
                    self.repo.graph_ahead_behind(loc_oid, up_oid).unwrap_or((0, 0))
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };
            result.push(BranchInfo {
                name,
                is_head,
                ahead,
                behind,
            });
        }
        Ok(result)
    }

    /// Calculate diff of a file between HEAD commit and working directory.
    pub fn diff_file(&self, path: &Path) -> Result<FileDiff, GitError> {
        let rel = self.relative_path(path);
        let full = self.workdir.join(rel);

        let new_text = if full.exists() {
            std::fs::read_to_string(&full)?
        } else {
            String::new()
        };

        let old_text = match self.repo.head().and_then(|h| h.peel_to_commit()) {
            Ok(head_commit) => {
                let tree = head_commit.tree()?;
                match tree.get_path(rel) {
                    Ok(entry) => {
                        let blob = self.repo.find_blob(entry.id())?;
                        String::from_utf8_lossy(blob.content()).to_string()
                    }
                    Err(_) => String::new(),
                }
            }
            Err(_) => String::new(),
        };

        Ok(compute_diff(&old_text, &new_text))
    }

    /// Blame a single 1-based line of a file.
    pub fn blame_line(&self, path: &Path, line: usize) -> Option<BlameInfo> {
        let rel = self.relative_path(path);
        blame_line(&self.repo, rel, line)
    }
}
