//! ErrorHandler for Consistent Error Management
//!
//! Provides standardized error handling, logging, and recovery strategies.

use super::crud::OperationError;
use serde_json::{json, Value};
use std::fmt::Debug;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Detailed error information
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error: OperationError,
    pub severity: Severity,
    pub context: Value,
    pub recovery_hint: Option<String>,
}

impl ErrorInfo {
    /// Create new error info
    pub fn new(error: OperationError, severity: Severity) -> Self {
        Self {
            error,
            severity,
            context: json!({}),
            recovery_hint: None,
        }
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut obj) = self.context {
            obj.insert(key.into(), value);
        }
        self
    }

    /// Add recovery hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hint = Some(hint.into());
        self
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Value {
        let mut json = json!({
            "error": self.error.to_string(),
            "severity": match self.severity {
                Severity::Info => "info",
                Severity::Warning => "warning",
                Severity::Error => "error",
                Severity::Critical => "critical",
            },
            "context": self.context,
        });

        if let Some(hint) = &self.recovery_hint {
            json["recovery_hint"] = json!(hint);
        }

        json
    }
}

/// Error handler
#[derive(Debug)]
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle an operation error
    pub fn handle(error: OperationError) -> ErrorInfo {
        let severity = match error {
            OperationError::NotFound(_) => Severity::Warning,
            OperationError::InvalidInput(_) => Severity::Warning,
            OperationError::Unauthorized(_) => Severity::Error,
            OperationError::Conflict(_) => Severity::Error,
            OperationError::Timeout => Severity::Critical,
            OperationError::Failed(_) => Severity::Critical,
        };

        ErrorInfo::new(error, severity)
    }

    /// Recover from an error with retry
    pub fn recover_with_retry<T, F>(
        mut f: F,
        max_retries: usize,
    ) -> Result<T, OperationError>
    where
        F: FnMut() -> Result<T, OperationError>,
    {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());
                    if attempt < max_retries {
                        std::thread::sleep(std::time::Duration::from_millis(
                            100 * (2_u64.pow(attempt as u32)),
                        ));
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Log error
    pub fn log(error_info: &ErrorInfo) -> String {
        format!(
            "[{}] {}: {}",
            match error_info.severity {
                Severity::Info => "INFO",
                Severity::Warning => "WARN",
                Severity::Error => "ERROR",
                Severity::Critical => "CRITICAL",
            },
            error_info.error,
            error_info
                .recovery_hint
                .as_ref()
                .unwrap_or(&"no hint".to_string())
        )
    }

    /// Is error recoverable
    pub fn is_recoverable(error: &OperationError) -> bool {
        matches!(
            error,
            OperationError::Timeout | OperationError::Failed(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_info_builder() {
        let error_info = ErrorInfo::new(
            OperationError::NotFound("user:123".to_string()),
            Severity::Warning,
        )
        .with_context("attempt", json!(1))
        .with_hint("Check if the resource exists");

        assert_eq!(error_info.severity, Severity::Warning);
        assert_eq!(error_info.context["attempt"], 1);
        assert!(error_info.recovery_hint.is_some());
    }

    #[test]
    fn test_error_info_to_json() {
        let error_info = ErrorInfo::new(
            OperationError::InvalidInput("bad data".to_string()),
            Severity::Error,
        )
        .with_hint("Validate input format");

        let json = error_info.to_json();
        assert_eq!(json["severity"], "error");
        assert_eq!(json["recovery_hint"], "Validate input format");
    }

    #[test]
    fn test_error_handler_handle() {
        let error_info = ErrorHandler::handle(OperationError::NotFound("test".to_string()));
        assert_eq!(error_info.severity, Severity::Warning);

        let error_info = ErrorHandler::handle(OperationError::Unauthorized("test".to_string()));
        assert_eq!(error_info.severity, Severity::Error);

        let error_info = ErrorHandler::handle(OperationError::Timeout);
        assert_eq!(error_info.severity, Severity::Critical);
    }

    #[test]
    fn test_error_handler_log() {
        let error_info = ErrorInfo::new(
            OperationError::Failed("operation failed".to_string()),
            Severity::Critical,
        )
        .with_hint("Retry later");

        let log = ErrorHandler::log(&error_info);
        assert!(log.contains("CRITICAL"));
        assert!(log.contains("operation failed"));
    }

    #[test]
    fn test_error_handler_is_recoverable() {
        assert!(ErrorHandler::is_recoverable(&OperationError::Timeout));
        assert!(ErrorHandler::is_recoverable(&OperationError::Failed(
            "test".to_string()
        )));
        assert!(!ErrorHandler::is_recoverable(&OperationError::NotFound(
            "test".to_string()
        )));
    }

    #[test]
    fn test_error_handler_retry() {
        let mut attempt_count = 0;

        let result = ErrorHandler::recover_with_retry(
            || {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err(OperationError::Timeout)
                } else {
                    Ok("success".to_string())
                }
            },
            5,
        );

        assert!(result.is_ok());
        assert_eq!(attempt_count, 3);
    }
}
