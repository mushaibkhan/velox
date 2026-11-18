use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_text: String,
    pub match_start: usize,
    pub match_end: usize,
}

pub struct SearchEngine;

impl SearchEngine {
    pub fn search_in_dir(
        root: impl AsRef<Path>,
        pattern: &str,
        case_sensitive: bool,
        max_results: usize,
    ) -> Vec<SearchMatch> {
        let mut results = Vec::new();
        if pattern.is_empty() {
            return results;
        }

        let regex = if case_sensitive {
            Regex::new(pattern).or_else(|_| Regex::new(&regex::escape(pattern)))
        } else {
            Regex::new(&format!("(?i){}", pattern))
                .or_else(|_| Regex::new(&format!("(?i){}", regex::escape(pattern))))
        };

        let regex = match regex {
            Ok(r) => r,
            Err(_) => return results,
        };

        for entry in WalkDir::new(root)
            .max_depth(8)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            })
            .flatten()
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for (line_idx, line_res) in reader.lines().enumerate() {
                    if let Ok(line) = line_res {
                        for m in regex.find_iter(&line) {
                            results.push(SearchMatch {
                                file_path: path.to_path_buf(),
                                line_number: line_idx + 1,
                                line_text: line.clone(),
                                match_start: m.start(),
                                match_end: m.end(),
                            });

                            if results.len() >= max_results {
                                return results;
                            }
                        }
                    }
                }
            }
        }

        results
    }

    pub fn fuzzy_match(pattern: &str, target: &str) -> Option<i32> {
        if pattern.is_empty() {
            return Some(0);
        }

        let mut score = 0;
        let pat_lower = pattern.to_lowercase();
        let target_lower = target.to_lowercase();
        let mut pattern_chars = pat_lower.chars();
        let mut current_p = pattern_chars.next()?;
        let mut prev_matched_idx = 0;

        for (idx, ch) in target_lower.chars().enumerate() {
            if ch == current_p {
                // Bonus for consecutive matches
                if idx == prev_matched_idx + 1 {
                    score += 10;
                } else {
                    score += 1;
                }
                prev_matched_idx = idx;

                if let Some(next_p) = pattern_chars.next() {
                    current_p = next_p;
                } else {
                    return Some(score);
                }
            }
        }

        None
    }
}
