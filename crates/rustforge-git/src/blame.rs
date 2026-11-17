use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use git2::{BlameOptions, Repository};

/// Line-level blame metadata for a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameInfo {
    /// Commit hash (full SHA).
    pub commit_id: String,
    /// Author name from commit signature.
    pub author: String,
    /// Short summary / first line of commit message.
    pub summary: String,
    /// Relative time string (e.g. "2 hours ago", "just now").
    pub time: String,
}

/// Retrieve blame metadata for a specific 1-based line of a file.
pub fn blame_line(repo: &Repository, path: &Path, line: usize) -> Option<BlameInfo> {
    if line == 0 {
        return None;
    }

    let mut opts = BlameOptions::new();
    let blame = repo.blame_file(path, Some(&mut opts)).ok()?;
    let hunk = blame.get_line(line)?;

    let commit_id = hunk.final_commit_id();
    let author = hunk.final_signature().name().unwrap_or("Unknown").to_string();
    let commit = repo.find_commit(commit_id).ok();
    let summary = commit.as_ref().and_then(|c| c.summary()).unwrap_or("").to_string();
    let time_sec = hunk.final_signature().when().seconds();
    let time = format_relative_time(time_sec);

    Some(BlameInfo {
        commit_id: commit_id.to_string(),
        author,
        summary,
        time,
    })
}

/// Convert unix epoch timestamp in seconds to human readable relative time.
pub fn format_relative_time(timestamp_secs: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp_secs);
    if diff <= 0 || diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        if mins == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{mins} minutes ago")
        }
    } else if diff < 86400 {
        let hours = diff / 3600;
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{hours} hours ago")
        }
    } else if diff < 2592000 {
        let days = diff / 86400;
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{days} days ago")
        }
    } else if diff < 31536000 {
        let months = diff / 2592000;
        if months == 1 {
            "1 month ago".to_string()
        } else {
            format!("{months} months ago")
        }
    } else {
        let years = diff / 31536000;
        if years == 1 {
            "1 year ago".to_string()
        } else {
            format!("{years} years ago")
        }
    }
}
