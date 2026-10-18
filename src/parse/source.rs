use crate::error::{CompileError, Result};
use std::path::{Path, PathBuf};

pub struct Source {
    pub path: PathBuf,
    lines: Vec<String>,
    idx: usize,
    pub text: String,
    pub indent: i32,
}

impl Source {
    pub fn new(path: &Path, text: &str) -> Self {
        Self {
            path: path.to_path_buf(),
            lines: text.lines().map(|l| l.to_string()).collect(),
            idx: 0,
            text: String::new(),
            indent: 0,
        }
    }

    pub fn line_number(&self) -> usize {
        self.idx + 1
    }

    pub fn error(&self, message: impl Into<String>) -> CompileError {
        CompileError::at(&self.path, self.line_number(), message)
    }

    pub fn peek(&mut self) -> Result<bool> {
        while self.idx < self.lines.len() {
            let line = &self.lines[self.idx];
            let trimmed = line.trim();
            let indent = (line.len() - line.trim_start().len()) as i32;
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.idx += 1;
                continue;
            }
            self.text = trimmed.to_string();
            self.indent = indent;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn next(&mut self) {
        self.idx += 1;
    }

    pub fn usage_comments(&self) -> Vec<String> {
        let mut comments = Vec::new();
        for line in self.lines[..self.idx].iter().rev() {
            if let Some(comment) = line.trim().strip_prefix("#?") {
                comments.push(comment.trim().to_string());
            } else {
                break;
            }
        }
        comments.reverse();
        comments
    }

    pub fn current_raw_line(&self) -> Option<&str> {
        self.lines.get(self.idx).map(|s| s.as_str())
    }
}
