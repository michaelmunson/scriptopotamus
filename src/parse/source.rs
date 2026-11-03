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

    /// Read a collection or quoted string through its closing delimiter.
    pub fn read_value(&mut self, first: &str) -> Result<String> {
        let start = self.line_number();
        let collection = first.starts_with('(');
        let unterminated = if collection {
            "unterminated collection literal (expected ')')"
        } else {
            "unterminated string literal (expected closing quote)"
        };
        let mut value = String::new();
        let mut depth = 0i32;
        let mut quote = None;
        let mut escaped = false;
        let mut part = first.to_string();
        loop {
            let mut previous = ' ';
            for (offset, c) in part.char_indices() {
                if escaped {
                    value.push(c);
                    escaped = false;
                } else if c == '\\' && quote != Some('\'') {
                    value.push(c);
                    escaped = true;
                } else if let Some(q) = quote {
                    value.push(c);
                    if c == q {
                        quote = None;
                        if !collection {
                            let tail = part[offset + 1..].trim();
                            if !tail.is_empty() && !tail.starts_with('#') {
                                return Err(self.error("unexpected text after string literal"));
                            }
                            return Ok(value);
                        }
                    }
                } else if c == '#' && previous.is_whitespace() {
                    break;
                } else {
                    value.push(c);
                    match c {
                        '\'' | '"' => quote = Some(c),
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                let tail = part[offset + 1..].trim();
                                if !tail.is_empty() && !tail.starts_with('#') {
                                    return Err(
                                        self.error("unexpected text after collection literal")
                                    );
                                }
                                return Ok(value);
                            }
                        }
                        _ => {}
                    }
                }
                previous = c;
            }
            value.push('\n');
            // An escaped newline is a shell line continuation.
            if escaped {
                value.pop();
                value.pop();
                escaped = false;
            }
            self.next();
            if quote.is_none() {
                if !self.peek()? {
                    return Err(CompileError::at(&self.path, start, unterminated));
                }
                part = self.current_raw_line().unwrap().to_string();
            } else if let Some(raw) = self.current_raw_line() {
                part = raw.to_string();
            } else {
                return Err(CompileError::at(&self.path, start, unterminated));
            }
        }
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

/// Whether a value still has an unmatched opening delimiter.
pub fn value_continues(text: &str) -> bool {
    let mut depth = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut previous = ' ';
    for c in text.chars() {
        if escaped {
            escaped = false;
        } else if c == '\\' && quote != Some('\'') {
            escaped = true;
        } else if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '#' && previous.is_whitespace() {
            break;
        } else {
            match c {
                '\'' | '"' => quote = Some(c),
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
        previous = c;
    }
    depth > 0 || quote.is_some()
}
