//! Error types for clap-noun-verb

use thiserror::Error;

/// Errors that can occur in the noun-verb CLI framework
#[derive(Error, Debug)]
pub enum NounVerbError {
    /// Command not found
    #[error("Command '{noun}' not found")]
    CommandNotFound { noun: String },

    /// Verb not found for a given noun
    #[error("Verb '{verb}' not found for noun '{noun}'")]
    VerbNotFound { noun: String, verb: String },

    /// Invalid command structure
    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    /// Command execution error
    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },

    /// Argument parsing error
    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },

    /// Plugin-related error
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Middleware error
    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    /// Telemetry error
    #[error("Telemetry error: {0}")]
    TelemetryError(String),

    /// Generic error wrapper
    #[error("Error: {0}")]
    Generic(String),
}

impl NounVerbError {
    /// Enhance error with recovery suggestions from RDF guard validation
    ///
    /// Attempts to provide helpful suggestions using the RDF ontology and SPARQL queries.
    pub fn with_recovery_suggestions(self) -> String {
        let msg = self.to_string();

        // Try to load middleware and get suggestions
        #[cfg(feature = "rdf-control")]
        {
            use crate::rdf::{GuardValidationMiddleware, recover_from_error};
            let middleware = GuardValidationMiddleware::global();
            if let Ok(Some(suggestion)) = recover_from_error(&self, &middleware) {
                return format!("{}\n\n{}", msg, suggestion);
            }
        }

        msg
    }

    /// Create a command not found error
    pub fn command_not_found(noun: impl Into<String>) -> Self {
        Self::CommandNotFound { noun: noun.into() }
    }

    /// Create a verb not found error
    pub fn verb_not_found(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self::VerbNotFound { noun: noun.into(), verb: verb.into() }
    }

    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure { message: message.into() }
    }

    /// Create an execution error
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::ExecutionError { message: message.into() }
    }

    /// Create an argument error
    pub fn argument_error(message: impl Into<String>) -> Self {
        Self::ArgumentError { message: message.into() }
    }

    /// Create a missing argument error (helper for common case)
    pub fn missing_argument(name: impl Into<String>) -> Self {
        Self::ArgumentError { message: format!("Required argument '{}' is missing", name.into()) }
    }

    /// Create a validation error with constraints
    pub fn validation_error(
        name: impl Into<String>,
        value: impl Into<String>,
        constraints: Option<&str>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        if let Some(constraints) = constraints {
            Self::ArgumentError {
                message: format!(
                    "Invalid value '{}' for argument '{}'. {}",
                    value, name, constraints
                ),
            }
        } else {
            Self::ArgumentError {
                message: format!("Invalid value '{}' for argument '{}'", value, name),
            }
        }
    }

    /// Create a validation error with range constraints
    pub fn validation_range_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<&str>,
        max: Option<&str>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        let constraint_msg = match (min, max) {
            (Some(min), Some(max)) => format!("Must be between {} and {}", min, max),
            (Some(min), None) => format!("Must be >= {}", min),
            (None, Some(max)) => format!("Must be <= {}", max),
            (None, None) => "Invalid value".to_string(),
        };
        Self::validation_error(name, value, Some(&constraint_msg))
    }

    /// Create a validation error with length constraints
    pub fn validation_length_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        let constraint_msg = match (min, max) {
            (Some(min), Some(max)) => {
                format!("Length must be between {} and {} characters", min, max)
            }
            (Some(min), None) => format!("Length must be at least {} characters", min),
            (None, Some(max)) => format!("Length must be at most {} characters", max),
            (None, None) => "Invalid length".to_string(),
        };
        Self::validation_error(name, value, Some(&constraint_msg))
    }
}

/// Result type alias for noun-verb operations
pub type Result<T> = std::result::Result<T, NounVerbError>;
