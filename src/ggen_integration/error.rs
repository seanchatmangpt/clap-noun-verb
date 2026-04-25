//! Error types for ggen integration
//!
//! This module provides specialized error types for parsing and code generation
//! from Turtle/RDF specifications. All errors are designed for maximum clarity
//! and debugging support.
//!
//! ## Design Principles
//!
//! - **Explicit**: Each error variant has clear meaning
//! - **Context-rich**: Errors include relevant context
//! - **Type-safe**: Using thiserror for zero-cost error handling
//!
//! ## Examples
//!
//! ```rust
//! use clap_noun_verb::ggen_integration::error::{GgenError, GgenResult};
//!
//! fn parse_command(input: &str) -> GgenResult<String> {
//!     if input.is_empty() {
//!         return Err(GgenError::EmptyInput);
//!     }
//!     Ok(input.to_string())
//! }
//! ```

use thiserror::Error;

/// Result type for ggen operations
pub type GgenResult<T> = std::result::Result<T, GgenError>;

/// Errors that can occur during ggen integration operations
#[derive(Debug, Error)]
pub enum GgenError {
    /// Empty input provided
    #[error("Empty input: {0}")]
    EmptyInput(String),

    /// Invalid Turtle/RDF syntax
    #[error("Invalid Turtle syntax at line {line}, column {column}: {message}")]
    InvalidSyntax {
        /// Line number where error occurred
        line: usize,
        /// Column number where error occurred
        column: usize,
        /// Error message
        message: String,
    },

    /// Missing required RDF property
    #[error("Missing required property '{property}' for {entity}")]
    MissingProperty {
        /// Property name that is missing
        property: String,
        /// Entity that requires the property
        entity: String,
    },

    /// Invalid type annotation
    #[error("Invalid type annotation: {0}")]
    InvalidType(String),

    /// Code generation failed
    #[error("Code generation failed: {0}")]
    CodeGeneration(String),

    /// File I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid command structure
    #[error("Invalid command structure: {0}")]
    InvalidCommand(String),

    /// Duplicate command name
    #[error("Duplicate command name: {0}")]
    DuplicateName(String),

    /// Invalid argument configuration
    #[error("Invalid argument '{arg}': {reason}")]
    InvalidArgument {
        /// Argument name
        arg: String,
        /// Reason why it's invalid
        reason: String,
    },

    /// Invalid flag configuration
    #[error("Invalid flag '{flag}': {reason}")]
    InvalidFlag {
        /// Flag name
        flag: String,
        /// Reason why it's invalid
        reason: String,
    },

    /// SPARQL query execution failed
    #[error("SPARQL query failed: {0}")]
    SparqlError(String),

    /// RDF graph operation failed
    #[error("RDF graph error: {0}")]
    RdfError(String),

    /// Conversion error from NounVerbError
    #[error("CLI framework error: {0}")]
    CliError(String),
}

impl From<crate::error::NounVerbError> for GgenError {
    fn from(err: crate::error::NounVerbError) -> Self {
        GgenError::CliError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input_error() {
        // Arrange & Act
        let error = GgenError::EmptyInput("test input".to_string());

        // Assert
        assert_eq!(error.to_string(), "Empty input: test input");
    }

    #[test]
    fn test_invalid_syntax_error() {
        // Arrange & Act
        let error = GgenError::InvalidSyntax {
            line: 10,
            column: 5,
            message: "Unexpected token".to_string(),
        };

        // Assert
        let msg = error.to_string();
        assert!(msg.contains("line 10"));
        assert!(msg.contains("column 5"));
        assert!(msg.contains("Unexpected token"));
    }

    #[test]
    fn test_missing_property_error() {
        // Arrange & Act
        let error = GgenError::MissingProperty {
            property: "name".to_string(),
            entity: "Command".to_string(),
        };

        // Assert
        let msg = error.to_string();
        assert!(msg.contains("name"));
        assert!(msg.contains("Command"));
    }

    #[test]
    fn test_invalid_argument_error() {
        // Arrange & Act
        let error = GgenError::InvalidArgument {
            arg: "username".to_string(),
            reason: "Cannot be empty".to_string(),
        };

        // Assert
        let msg = error.to_string();
        assert!(msg.contains("username"));
        assert!(msg.contains("Cannot be empty"));
    }

    #[test]
    fn test_from_io_error() {
        // Arrange
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");

        // Act
        let ggen_error: GgenError = io_error.into();

        // Assert
        assert!(ggen_error.to_string().contains("I/O error"));
    }

    #[test]
    fn test_from_noun_verb_error() {
        // Arrange
        let nv_error = crate::error::NounVerbError::command_not_found("test");

        // Act
        let ggen_error: GgenError = nv_error.into();

        // Assert
        assert!(ggen_error.to_string().contains("CLI framework error"));
    }

    #[test]
    fn test_duplicate_name_error() {
        // Arrange & Act
        let error = GgenError::DuplicateName("user".to_string());

        // Assert
        assert_eq!(error.to_string(), "Duplicate command name: user");
    }

    #[test]
    fn test_code_generation_error() {
        // Arrange & Act
        let error = GgenError::CodeGeneration("Failed to generate struct".to_string());

        // Assert
        assert!(error.to_string().contains("Code generation failed"));
    }
}
