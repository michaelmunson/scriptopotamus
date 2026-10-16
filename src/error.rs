use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{path}:{line}: {message}")]
pub struct CompileError {
    pub path: String,
    pub line: usize,
    pub message: String,
}

impl CompileError {
    pub fn new(path: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line,
            message: message.into(),
        }
    }

    pub fn at(path: &std::path::Path, line: usize, message: impl Into<String>) -> Self {
        Self::new(path.display().to_string(), line, message)
    }
}

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Debug, Clone, Default)]
pub struct Span {
    pub path: PathBuf,
    pub line: usize,
}
