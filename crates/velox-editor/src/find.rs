use regex::Regex;
use std::ops::Range;

#[derive(Debug, Clone, Default)]
pub struct FindState {
    pub query: String,
    pub replacement: String,
    pub is_regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub matches: Vec<Range<usize>>, // char offset ranges
    pub current_match_idx: Option<usize>,
}

impl FindState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_search(&mut self, text: &str) {
        self.matches.clear();
        self.current_match_idx = None;

        if self.query.is_empty() {
            return;
        }

        let regex_res = if self.is_regex {
            Regex::new(&self.query)
        } else {
            let escaped = regex::escape(&self.query);
            let pattern = if self.whole_word {
                format!(r"\b{}\b", escaped)
            } else {
                escaped
            };
            if self.case_sensitive {
                Regex::new(&pattern)
            } else {
                Regex::new(&format!("(?i){}", pattern))
            }
        };

        if let Ok(re) = regex_res {
            for m in re.find_iter(text) {
                // Convert byte offsets to char offsets
                let char_start = text[..m.start()].chars().count();
                let char_end = char_start + text[m.start()..m.end()].chars().count();
                self.matches.push(char_start..char_end);
            }
        }

        if !self.matches.is_empty() {
            self.current_match_idx = Some(0);
        }
    }

    pub fn next_match(&mut self) -> Option<Range<usize>> {
        if self.matches.is_empty() {
            return None;
        }
        let next_idx = match self.current_match_idx {
            Some(idx) => (idx + 1) % self.matches.len(),
            None => 0,
        };
        self.current_match_idx = Some(next_idx);
        Some(self.matches[next_idx].clone())
    }

    pub fn prev_match(&mut self) -> Option<Range<usize>> {
        if self.matches.is_empty() {
            return None;
        }
        let prev_idx = match self.current_match_idx {
            Some(idx) => {
                if idx == 0 {
                    self.matches.len() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
        };
        self.current_match_idx = Some(prev_idx);
        Some(self.matches[prev_idx].clone())
    }
}
