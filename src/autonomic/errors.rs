//! Structured error model for autonomic CLI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kind of error that occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ErrorKind {
    /// Invalid input or arguments
    InvalidInput,
    /// Permission denied
    PermissionDenied,
    /// Invariant breach (Q-level constraint violated)
    InvariantBreach,
    /// Deadline exceeded
    DeadlineExceeded,
    /// Guard budget exceeded
    GuardExceeded,
    /// Command not found
    CommandNotFound,
    /// Verb not found
    VerbNotFound,
    /// Invalid command structure
    InvalidStructure,
    /// Execution error
    ExecutionError,
    /// Internal error
    InternalError,
}

impl ErrorKind {
    /// Get a short description of the error kind
    pub fn description(&self) -> &'static str {
        match self {
            ErrorKind::InvalidInput => "Invalid input or arguments",
            ErrorKind::PermissionDenied => "Permission denied",
            ErrorKind::InvariantBreach => "Invariant constraint violated",
            ErrorKind::DeadlineExceeded => "Deadline exceeded",
            ErrorKind::GuardExceeded => "Guard budget exceeded",
            ErrorKind::CommandNotFound => "Command not found",
            ErrorKind::VerbNotFound => "Verb not found",
            ErrorKind::InvalidStructure => "Invalid command structure",
            ErrorKind::ExecutionError => "Execution failed",
            ErrorKind::InternalError => "Internal error",
        }
    }
}

/// Structured error with machine-readable fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredError {
    /// Kind of error
    pub kind: ErrorKind,
    /// Human-readable message
    pub message: String,
    /// Additional structured details
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub details: HashMap<String, serde_json::Value>,
}

impl StructuredError {
    /// Create a new structured error
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into(), details: HashMap::new() }
    }

    /// Add a detail field
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.details.insert(key.into(), json_value);
        }
        self
    }

    /// Create an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, message)
    }

    /// Create a permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::PermissionDenied, message)
    }

    /// Create an invariant breach error
    pub fn invariant_breach(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvariantBreach, message)
    }

    /// Create a deadline exceeded error
    pub fn deadline_exceeded(deadline_ms: u64, actual_ms: u64) -> Self {
        Self::new(
            ErrorKind::DeadlineExceeded,
            format!("Deadline {}ms exceeded, took {}ms", deadline_ms, actual_ms),
        )
        .with_detail("deadline_ms", deadline_ms)
        .with_detail("actual_ms", actual_ms)
    }

    /// Create a guard exceeded error
    pub fn guard_exceeded(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::GuardExceeded, message)
    }

    /// Create a command not found error
    pub fn command_not_found(command: impl Into<String>) -> Self {
        Self::new(ErrorKind::CommandNotFound, format!("Command '{}' not found", command.into()))
    }

    /// Create a verb not found error
    pub fn verb_not_found(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        let noun = noun.into();
        let verb = verb.into();
        Self::new(ErrorKind::VerbNotFound, format!("Verb '{}' not found for noun '{}'", verb, noun))
            .with_detail("noun", noun)
            .with_detail("verb", verb)
    }

    /// Create an execution error
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ExecutionError, message)
    }

    /// Convert from NounVerbError
    pub fn from_noun_verb_error(error: &crate::error::NounVerbError) -> Self {
        match error {
            crate::error::NounVerbError::CommandNotFound { noun } => Self::command_not_found(noun),
            crate::error::NounVerbError::VerbNotFound { noun, verb } => {
                Self::verb_not_found(noun, verb)
            }
            crate::error::NounVerbError::InvalidStructure { message } => {
                Self::new(ErrorKind::InvalidStructure, message)
            }
            crate::error::NounVerbError::ExecutionError { message } => {
                Self::execution_error(message)
            }
            crate::error::NounVerbError::ArgumentError { message } => Self::invalid_input(message),
            crate::error::NounVerbError::PluginError(message) => {
                Self::new(ErrorKind::InternalError, message)
            }
            crate::error::NounVerbError::ValidationFailed(message) => Self::invalid_input(message),
            crate::error::NounVerbError::MiddlewareError(message) => {
                Self::new(ErrorKind::InternalError, message)
            }
            crate::error::NounVerbError::TelemetryError(message) => {
                Self::new(ErrorKind::InternalError, message)
            }
            crate::error::NounVerbError::Generic(message) => {
                Self::new(ErrorKind::InternalError, message)
            }
        }
    }
}

/// Wrapper for JSON error output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The structured error
    pub error: StructuredError,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: StructuredError) -> Self {
        Self { error }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
