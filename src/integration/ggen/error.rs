//! Error types for ggen integration
//!
//! Following Rust best practices and clap-noun-verb conventions:
//! - Use `thiserror` for error definitions
//! - Provide Result type alias
//! - No panic or unwrap in production code

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for ggen operations
pub type GgenResult<T> = Result<T, GgenError>;

/// Error types for ggen integration operations
#[derive(Debug, Error)]
pub enum GgenError {
    /// Template file not found at specified path
    #[error("Template not found: {0}")]
    TemplateNotFound(PathBuf),

    /// Output path is invalid or cannot be created
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(PathBuf),

    /// RDF graph operation failed
    #[error("RDF graph error: {0}")]
    GraphError(String),

    /// Template rendering failed
    #[error("Template rendering failed: {0}")]
    RenderingFailed(String),

    /// Variable substitution failed
    #[error("Variable substitution failed: {0}")]
    VariableError(String),

    /// Code generation failed
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from ggen-core
    #[error("Core error: {0}")]
    Core(#[from] ggen_core::Error),

    /// Error from ggen-domain
    #[error("Domain error: {0}")]
    Domain(#[from] ggen_domain::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Path cannot be empty
    #[error("Path cannot be empty")]
    EmptyPath,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl GgenError {
    /// Create a new template not found error
    pub fn template_not_found(path: impl Into<PathBuf>) -> Self {
        Self::TemplateNotFound(path.into())
    }

    /// Create a new invalid output path error
    pub fn invalid_output_path(path: impl Into<PathBuf>) -> Self {
        Self::InvalidOutputPath(path.into())
    }

    /// Create a new graph error
    pub fn graph_error(msg: impl Into<String>) -> Self {
        Self::GraphError(msg.into())
    }

    /// Create a new rendering failed error
    pub fn rendering_failed(msg: impl Into<String>) -> Self {
        Self::RenderingFailed(msg.into())
    }

    /// Create a new variable error
    pub fn variable_error(msg: impl Into<String>) -> Self {
        Self::VariableError(msg.into())
    }

    /// Create a new generation failed error
    pub fn generation_failed(msg: impl Into<String>) -> Self {
        Self::GenerationFailed(msg.into())
    }

    /// Create a new config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new invalid config error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // Arrange & Act
        let error = GgenError::template_not_found("/path/to/template.tera");

        // Assert
        assert_eq!(
            error.to_string(),
            "Template not found: /path/to/template.tera"
        );
    }

    #[test]
    fn test_error_conversion() {
        // Arrange
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

        // Act
        let ggen_error: GgenError = io_error.into();

        // Assert
        assert!(matches!(ggen_error, GgenError::Io(_)));
    }
}
