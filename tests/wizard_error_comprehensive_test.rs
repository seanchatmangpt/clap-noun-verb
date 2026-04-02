#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Comprehensive error handling tests for wizard module
//!
//! Tests all WizardError variants, error conversions, Display implementations,
//! Error trait compliance, and error propagation patterns.
//!
//! Chicago TDD Principles:
//! - State-based testing (verify error messages and types)
//! - Behavior verification (test error conversion and propagation)
//! - AAA pattern (Arrange-Act-Assert)

#![cfg(feature = "wizard")]

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::error::{WizardError, WizardResult};
use std::io;

// =============================================================================
// Error Variant Tests - Verify all error types can be constructed
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_client_error_variant() {
    // Arrange
    let message = "API connection failed";

    // Act
    let error = WizardError::ClientError(message.to_string());

    // Assert
    assert!(matches!(error, WizardError::ClientError(_)));
    assert!(format!("{}", error).contains("API client error"));
    assert!(format!("{}", error).contains(message));
}

#[cfg(feature = "wizard")]
#[test]
fn test_invalid_state_transition_variant() {
    // Arrange
    let from = "Init".to_string();
    let to = "Complete".to_string();

    // Act
    let error = WizardError::InvalidStateTransition { from: from.clone(), to: to.clone() };

    // Assert
    assert!(matches!(error, WizardError::InvalidStateTransition { .. }));
    let display = format!("{}", error);
    assert!(display.contains("Invalid state transition"));
    assert!(display.contains(&from));
    assert!(display.contains(&to));
}

#[cfg(feature = "wizard")]
#[test]
fn test_invalid_prompt_variant() {
    // Arrange
    let reason = "Prompt text is empty";

    // Act
    let error = WizardError::InvalidPrompt(reason.to_string());

    // Assert
    assert!(matches!(error, WizardError::InvalidPrompt(_)));
    assert!(format!("{}", error).contains("Invalid prompt"));
    assert!(format!("{}", error).contains(reason));
}

#[cfg(feature = "wizard")]
#[test]
fn test_session_not_initialized_variant() {
    // Arrange + Act
    let error = WizardError::SessionNotInitialized;

    // Assert
    assert!(matches!(error, WizardError::SessionNotInitialized));
    assert_eq!(format!("{}", error), "Wizard session not initialized");
}

#[cfg(feature = "wizard")]
#[test]
fn test_config_error_variant() {
    // Arrange
    let message = "Missing API key";

    // Act
    let error = WizardError::ConfigError(message.to_string());

    // Assert
    assert!(matches!(error, WizardError::ConfigError(_)));
    assert!(format!("{}", error).contains("Configuration error"));
    assert!(format!("{}", error).contains(message));
}

#[cfg(feature = "wizard")]
#[test]
fn test_config_short_form_variant() {
    // Arrange
    let message = "Invalid model";

    // Act
    let error = WizardError::Config(message.to_string());

    // Assert
    assert!(matches!(error, WizardError::Config(_)));
    assert!(format!("{}", error).contains("Configuration error"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_io_error_variant() {
    // Arrange
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");

    // Act
    let error = WizardError::IoError(io_err);

    // Assert
    assert!(matches!(error, WizardError::IoError(_)));
    assert!(format!("{}", error).contains("I/O error"));
    assert!(format!("{}", error).contains("file not found"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_io_short_form_variant() {
    // Arrange
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");

    // Act
    let error = WizardError::Io(io_err);

    // Assert
    assert!(matches!(error, WizardError::Io(_)));
    assert!(format!("{}", error).contains("I/O error"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_serde_error_variant() {
    // Arrange
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid json}").unwrap_err();

    // Act
    let error = WizardError::SerdeError(json_err);

    // Assert
    assert!(matches!(error, WizardError::SerdeError(_)));
    assert!(format!("{}", error).contains("Serialization error"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_json_short_form_variant() {
    // Arrange
    let json_err = serde_json::from_str::<serde_json::Value>("{bad: json}").unwrap_err();

    // Act
    let error = WizardError::Json(json_err);

    // Assert
    assert!(matches!(error, WizardError::Json(_)));
}

#[cfg(feature = "wizard")]
#[test]
fn test_env_var_error_variant() {
    // Arrange
    let env_err = std::env::var("NONEXISTENT_VAR_XYZ_123").unwrap_err();

    // Act
    let error = WizardError::EnvVarError(env_err);

    // Assert
    assert!(matches!(error, WizardError::EnvVarError(_)));
    assert!(format!("{}", error).contains("Environment variable error"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_request_error_variant() {
    // Arrange
    let message = "HTTP 503: Service Unavailable";

    // Act
    let error = WizardError::Request(message.to_string());

    // Assert
    assert!(matches!(error, WizardError::Request(_)));
    assert!(format!("{}", error).contains("API request error"));
    assert!(format!("{}", error).contains(message));
}

#[cfg(feature = "wizard")]
#[test]
fn test_parse_error_variant() {
    // Arrange
    let message = "Expected JSON, got HTML";

    // Act
    let error = WizardError::Parse(message.to_string());

    // Assert
    assert!(matches!(error, WizardError::Parse(_)));
    assert!(format!("{}", error).contains("Response parsing error"));
    assert!(format!("{}", error).contains(message));
}

#[cfg(feature = "wizard")]
#[test]
fn test_token_limit_variant() {
    // Arrange
    let requested = 100000;
    let max = 8192;

    // Act
    let error = WizardError::TokenLimit { requested, max };

    // Assert
    assert!(matches!(error, WizardError::TokenLimit { .. }));
    let display = format!("{}", error);
    assert!(display.contains("Token limit exceeded"));
    assert!(display.contains(&requested.to_string()));
    assert!(display.contains(&max.to_string()));
}

#[cfg(feature = "wizard")]
#[test]
fn test_other_error_variant() {
    // Arrange
    let message = "Unknown error occurred";

    // Act
    let error = WizardError::Other(message.to_string());

    // Assert
    assert!(matches!(error, WizardError::Other(_)));
    assert_eq!(format!("{}", error), format!("Error: {}", message));
}

// =============================================================================
// Error Conversion Tests - Test From trait implementations
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_from_io_error_conversion() {
    // Arrange
    let io_err = io::Error::new(io::ErrorKind::TimedOut, "connection timeout");

    // Act
    let wizard_err: WizardError = io_err.into();

    // Assert
    assert!(matches!(wizard_err, WizardError::IoError(_)));
}

#[cfg(feature = "wizard")]
#[test]
fn test_from_serde_json_error_conversion() {
    // Arrange
    let json_err = serde_json::from_str::<Vec<String>>("[1, 2, 3]").unwrap_err();

    // Act
    let wizard_err: WizardError = json_err.into();

    // Assert
    assert!(matches!(wizard_err, WizardError::SerdeError(_)));
}

#[cfg(feature = "wizard")]
#[test]
fn test_from_env_var_error_conversion() {
    // Arrange
    let env_err = std::env::var("DEFINITELY_NOT_SET_VAR").unwrap_err();

    // Act
    let wizard_err: WizardError = env_err.into();

    // Assert
    assert!(matches!(wizard_err, WizardError::EnvVarError(_)));
}

// =============================================================================
// Error Source Tests - Test std::error::Error::source()
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_error_source_for_io_error() {
    // Arrange
    let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
    let error = WizardError::IoError(io_err);

    // Act
    let source = std::error::Error::source(&error);

    // Assert
    assert!(source.is_some());
}

#[cfg(feature = "wizard")]
#[test]
fn test_error_source_for_serde_error() {
    // Arrange
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
    let error = WizardError::SerdeError(json_err);

    // Act
    let source = std::error::Error::source(&error);

    // Assert
    assert!(source.is_some());
}

#[cfg(feature = "wizard")]
#[test]
fn test_error_source_for_simple_variants() {
    // Arrange
    let error = WizardError::SessionNotInitialized;

    // Act
    let source = std::error::Error::source(&error);

    // Assert - simple variants have no source
    assert!(source.is_none());
}

// =============================================================================
// Display Formatting Tests - Verify error messages are user-friendly
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_display_format_consistency() {
    // Arrange
    let errors = vec![
        WizardError::ClientError("test".to_string()),
        WizardError::InvalidPrompt("test".to_string()),
        WizardError::ConfigError("test".to_string()),
        WizardError::Request("test".to_string()),
        WizardError::Parse("test".to_string()),
        WizardError::Other("test".to_string()),
    ];

    // Act & Assert - all should format without panicking
    for error in errors {
        let display = format!("{}", error);
        assert!(!display.is_empty());
        assert!(display.len() < 500); // Reasonable length
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_debug_format() {
    // Arrange
    let error = WizardError::InvalidPrompt("empty prompt".to_string());

    // Act
    let debug_str = format!("{:?}", error);

    // Assert
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("InvalidPrompt"));
}

// =============================================================================
// Result Type Tests - Test WizardResult<T> usage
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_wizard_result_ok() {
    // Arrange
    let value = 42;

    // Act
    let result: WizardResult<i32> = Ok(value);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), value);
}

#[cfg(feature = "wizard")]
#[test]
fn test_wizard_result_err() {
    // Arrange
    let error = WizardError::SessionNotInitialized;

    // Act
    let result: WizardResult<String> = Err(error);

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WizardError::SessionNotInitialized));
}

// =============================================================================
// Error Propagation Tests - Test ? operator usage
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_error_propagation_with_question_mark() {
    // Arrange
    fn inner_function() -> WizardResult<()> {
        Err(WizardError::InvalidPrompt("test".to_string()))
    }

    fn outer_function() -> WizardResult<()> {
        inner_function()?;
        Ok(())
    }

    // Act
    let result = outer_function();

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WizardError::InvalidPrompt(_)));
}

#[cfg(feature = "wizard")]
#[test]
fn test_io_error_propagation() {
    // Arrange
    fn function_with_io() -> WizardResult<()> {
        let _file = std::fs::read("/nonexistent/path/to/file.txt")?;
        Ok(())
    }

    // Act
    let result = function_with_io();

    // Assert
    assert!(result.is_err());
    // Should be converted from io::Error to WizardError::IoError
    assert!(matches!(result.unwrap_err(), WizardError::IoError(_)));
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[cfg(feature = "wizard")]
#[test]
fn test_empty_error_messages() {
    // Arrange + Act
    let errors = vec![
        WizardError::ClientError(String::new()),
        WizardError::InvalidPrompt(String::new()),
        WizardError::Request(String::new()),
    ];

    // Assert - should handle empty messages gracefully
    for error in errors {
        let display = format!("{}", error);
        assert!(!display.is_empty()); // Should at least have error type
    }
}

#[cfg(feature = "wizard")]
#[test]
fn test_very_long_error_messages() {
    // Arrange
    let long_message = "x".repeat(10000);

    // Act
    let error = WizardError::Other(long_message.clone());

    // Assert
    let display = format!("{}", error);
    assert!(display.contains(&long_message));
}

#[cfg(feature = "wizard")]
#[test]
fn test_token_limit_zero_values() {
    // Arrange + Act
    let error = WizardError::TokenLimit { requested: 0, max: 0 };

    // Assert
    let display = format!("{}", error);
    assert!(display.contains("0"));
}

#[cfg(feature = "wizard")]
#[test]
fn test_error_equality_not_implemented() {
    // Note: WizardError does not implement PartialEq by design
    // This test documents that behavior
    let error1 = WizardError::SessionNotInitialized;
    let error2 = WizardError::SessionNotInitialized;

    // Assert - These cannot be compared with ==
    // This is intentional as errors with inner io::Error can't be compared
    assert!(matches!(error1, WizardError::SessionNotInitialized));
    assert!(matches!(error2, WizardError::SessionNotInitialized));
}
