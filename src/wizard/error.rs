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

    /// Configuration error (short form alias)
    Config(String),

    /// I/O error during wizard operations
    IoError(std::io::Error),

    /// I/O error (short form alias)
    Io(std::io::Error),

    /// Serialization/deserialization error
    SerdeError(serde_json::Error),

    /// JSON error (short form alias)
    Json(serde_json::Error),

    /// Environment variable not found
    EnvVarError(std::env::VarError),

    /// API request error
    Request(String),

    /// Response parsing error
    Parse(String),

    /// Token limit exceeded
    TokenLimit {
        /// Requested tokens
        requested: usize,
        /// Maximum allowed tokens
        max: usize,
    },

    /// Authentication error
    Auth(String),

    /// Rate limit exceeded
    RateLimit(String),

    /// Request timeout
    Timeout(String),

    /// Network error
    Network(String),

    /// All models in fallback chain failed
    Fallback(String),

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
            Self::ConfigError(msg) | Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::IoError(err) | Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::SerdeError(err) | Self::Json(err) => write!(f, "Serialization error: {}", err),
            Self::EnvVarError(err) => write!(f, "Environment variable error: {}", err),
            Self::Request(msg) => write!(f, "API request error: {}", msg),
            Self::Parse(msg) => write!(f, "Response parsing error: {}", msg),
            Self::TokenLimit { requested, max } => {
                write!(f, "Token limit exceeded: requested {} but max is {}", requested, max)
            }
            Self::Auth(msg) => write!(f, "Authentication error: {}", msg),
            Self::RateLimit(msg) => write!(f, "Rate limit exceeded: {}", msg),
            Self::Timeout(msg) => write!(f, "Request timeout: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Fallback(msg) => write!(f, "Fallback chain failed: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for WizardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(err) | Self::Io(err) => Some(err),
            Self::SerdeError(err) | Self::Json(err) => Some(err),
            Self::EnvVarError(err) => Some(err),
            _ => None,
        }
    }
}

// Error conversions for ergonomic ? operator usage
impl From<std::io::Error> for WizardError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for WizardError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<std::env::VarError> for WizardError {
    fn from(err: std::env::VarError) -> Self {
        Self::EnvVarError(err)
    }
}

/// Result type for wizard operations
pub type Result<T> = std::result::Result<T, WizardError>;

/// Result type alias for wizard operations (alternative name)
pub type WizardResult<T> = std::result::Result<T, WizardError>;

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

        let err = WizardError::TokenLimit { requested: 10000, max: 8192 };
        assert_eq!(format!("{}", err), "Token limit exceeded: requested 10000 but max is 8192");

        let err = WizardError::RateLimit("too many requests".to_string());
        assert_eq!(format!("{}", err), "Rate limit exceeded: too many requests");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let wizard_err: WizardError = io_err.into();
        assert!(matches!(wizard_err, WizardError::Io(_)));
    }
}
