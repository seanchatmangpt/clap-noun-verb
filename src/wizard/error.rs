//! Error types for wizard module
//!
//! This module provides error handling for wizard operations with proper
//! Result<T, E> error propagation and type-safe error conversion.

use std::fmt;

/// Error type for wizard operations
///
/// Represents all possible error conditions that can occur during
/// wizard session management, prompt processing, and AI interactions.
#[derive(Debug)]
pub enum WizardError {
    /// Error from the AI client
    ClientError(String),

    /// Invalid session state transition
    InvalidStateTransition {
        /// Current state
        from: String,
        /// Attempted target state
        to: String,
    },

    /// Prompt validation failed
    InvalidPrompt(String),

    /// Session not initialized
    SessionNotInitialized,

    /// Configuration error
    ConfigError(String),

    /// I/O error during wizard operations
    IoError(std::io::Error),

    /// Serialization/deserialization error
    SerdeError(serde_json::Error),

    /// Environment variable not found
    EnvVarError(std::env::VarError),

    /// Generic error with message
    Other(String),
}

impl fmt::Display for WizardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientError(msg) => write!(f, "AI client error: {}", msg),
            Self::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition from '{}' to '{}'", from, to)
            }
            Self::InvalidPrompt(msg) => write!(f, "Invalid prompt: {}", msg),
            Self::SessionNotInitialized => write!(f, "Wizard session not initialized"),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::IoError(err) => write!(f, "I/O error: {}", err),
            Self::SerdeError(err) => write!(f, "Serialization error: {}", err),
            Self::EnvVarError(err) => write!(f, "Environment variable error: {}", err),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for WizardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            Self::SerdeError(err) => Some(err),
            Self::EnvVarError(err) => Some(err),
            _ => None,
        }
    }
}

// Error conversions for ergonomic ? operator usage
impl From<std::io::Error> for WizardError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for WizardError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err)
    }
}

impl From<std::env::VarError> for WizardError {
    fn from(err: std::env::VarError) -> Self {
        Self::EnvVarError(err)
    }
}

/// Result type for wizard operations
pub type Result<T> = std::result::Result<T, WizardError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WizardError::InvalidPrompt("empty prompt".to_string());
        assert_eq!(format!("{}", err), "Invalid prompt: empty prompt");

        let err = WizardError::InvalidStateTransition {
            from: "Init".to_string(),
            to: "Complete".to_string(),
        };
        assert_eq!(format!("{}", err), "Invalid state transition from 'Init' to 'Complete'");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let wizard_err: WizardError = io_err.into();
        assert!(matches!(wizard_err, WizardError::IoError(_)));
    }
}
