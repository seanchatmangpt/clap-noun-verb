//! I/O Error Types for clap-noun-verb
//!
//! Provides comprehensive error handling for I/O operations, supporting both
//! the clio ecosystem and internal error patterns.
//!
//! # Error Types
//!
//! - `IoError`: Core I/O error type wrapping std::io::Error
//! - `PathError`: Path validation and access errors
//! - `FormatError`: File format and encoding errors
//!
//! # Examples
//!
//! ```rust,ignore
//! use clap_noun_verb::io::Result;
//!
//! fn process_file(path: &str) -> Result<String> {
//!     let content = std::fs::read_to_string(path)?;
//!     Ok(content)
//! }
//! ```

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type for I/O operations
pub type Result<T> = std::result::Result<T, IoError>;

/// Comprehensive I/O error type
#[derive(Debug)]
pub enum IoError {
    /// Standard I/O error
    Io(io::Error),

    /// Path validation error
    Path {
        path: PathBuf,
        reason: String,
    },

    /// File format error
    Format {
        path: PathBuf,
        reason: String,
    },

    /// Encoding error (UTF-8, etc.)
    Encoding {
        path: PathBuf,
        expected: String,
        found: String,
    },

    /// Permission denied
    PermissionDenied {
        path: PathBuf,
        operation: String,
    },

    /// File not found
    NotFound(PathBuf),

    /// Custom error with context
    Custom {
        message: String,
        context: Option<String>,
    },
}

impl IoError {
    /// Create a new custom error
    pub fn custom<S: Into<String>>(message: S) -> Self {
        Self::Custom {
            message: message.into(),
            context: None,
        }
    }

    /// Add context to error
    pub fn with_context<S: Into<String>>(mut self, context: S) -> Self {
        match &mut self {
            Self::Custom { context: ctx, .. } => {
                *ctx = Some(context.into());
            }
            _ => {}
        }
        self
    }

    /// Get error path if available
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Path { path, .. }
            | Self::Format { path, .. }
            | Self::Encoding { path, .. }
            | Self::PermissionDenied { path, .. }
            | Self::NotFound(path) => Some(path),
            _ => None,
        }
    }

    /// Get error reason/explanation
    pub fn reason(&self) -> String {
        match self {
            Self::Io(e) => e.to_string(),
            Self::Path { reason, .. } => reason.clone(),
            Self::Format { reason, .. } => reason.clone(),
            Self::Encoding { expected, found, .. } => {
                format!("encoding mismatch: expected {}, found {}", expected, found)
            }
            Self::PermissionDenied { operation, .. } => {
                format!("permission denied for operation: {}", operation)
            }
            Self::NotFound(_) => "file not found".to_string(),
            Self::Custom { message, .. } => message.clone(),
        }
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Path { path, reason } => {
                write!(f, "path error '{}': {}", path.display(), reason)
            }
            Self::Format { path, reason } => {
                write!(f, "format error in '{}': {}", path.display(), reason)
            }
            Self::Encoding { path, expected, found } => {
                write!(
                    f,
                    "encoding error in '{}': expected {}, found {}",
                    path.display(),
                    expected,
                    found
                )
            }
            Self::PermissionDenied { path, operation } => {
                write!(
                    f,
                    "permission denied for {} on '{}'",
                    operation,
                    path.display()
                )
            }
            Self::NotFound(path) => write!(f, "file not found: '{}'", path.display()),
            Self::Custom { message, context } => {
                if let Some(ctx) = context {
                    write!(f, "{}: {}", message, ctx)
                } else {
                    write!(f, "{}", message)
                }
            }
        }
    }
}

impl std::error::Error for IoError {}

// Conversion from std::io::Error
impl From<io::Error> for IoError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

// Conversion from PathBuf for convenience
impl From<PathBuf> for IoError {
    fn from(path: PathBuf) -> Self {
        Self::NotFound(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = IoError::NotFound(PathBuf::from("test.txt"));
        assert!(err.to_string().contains("test.txt"));
    }

    #[test]
    fn test_error_path_extraction() {
        let path = PathBuf::from("/tmp/test.txt");
        let err = IoError::Path {
            path: path.clone(),
            reason: "test reason".to_string(),
        };
        assert_eq!(err.path(), Some(&path));
    }

    #[test]
    fn test_custom_error_with_context() {
        let err = IoError::custom("base error").with_context("additional context");
        assert!(err.to_string().contains("base error"));
        assert!(err.to_string().contains("additional context"));
    }

    #[test]
    fn test_io_error_from_std() {
        let std_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err: IoError = std_err.into();
        assert!(matches!(err, IoError::Io(_)));
    }
}
