use clap_noun_verb::error::{ActionTemplate, ErrorKind, NounVerbError, Severity, StructuredError};
use serde_json::json;

#[test]
fn test_deadline_exceeded_construction() {
    let err = StructuredError::deadline_exceeded(100, 150);
    assert_eq!(err.kind, ErrorKind::DeadlineExceeded);
    assert_eq!(err.severity, Severity::Critical);
    assert_eq!(err.message, "Deadline 100ms exceeded, took 150ms");

    let deadline_val = err.details.get("deadline_ms").unwrap();
    let actual_val = err.details.get("actual_ms").unwrap();
    assert_eq!(deadline_val, &json!(100));
    assert_eq!(actual_val, &json!(150));

    assert_eq!(
        err.action_templates[0],
        ActionTemplate::TimeoutAdjustment {
            suggested_timeout_ms: 250,
            reason: "Increase deadline budget to match observed latency".to_string()
        }
    );
}

#[test]
fn test_from_error_command_not_found() {
    let raw_err = NounVerbError::CommandNotFound {
        noun: "testcmd".to_string(),
        suggestion: ". Did you mean: \u{1b}[1m\u{1b}[33mtestcor\u{1b}[0m?".to_string(),
    };
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::CommandNotFound);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("noun").unwrap(), &json!("testcmd"));

    assert_eq!(
        err.action_templates[0],
        ActionTemplate::CommandFix {
            suggested_command: "testcor".to_string(),
            reason: "Suggested correction for misspelled command 'testcmd'".to_string()
        }
    );
}

#[test]
fn test_from_error_command_not_found_no_suggestion() {
    let raw_err =
        NounVerbError::CommandNotFound { noun: "testcmd".to_string(), suggestion: "".to_string() };
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::CommandNotFound);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("noun").unwrap(), &json!("testcmd"));
    assert!(err.action_templates.is_empty());
}

#[test]
fn test_from_error_verb_not_found() {
    let raw_err = NounVerbError::VerbNotFound {
        noun: "testcmd".to_string(),
        verb: "testverb".to_string(),
        suggestion: ". Did you mean: \u{1b}[1m\u{1b}[33mtestcor\u{1b}[0m?".to_string(),
    };
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::VerbNotFound);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("noun").unwrap(), &json!("testcmd"));
    assert_eq!(err.details.get("verb").unwrap(), &json!("testverb"));

    assert_eq!(
        err.action_templates[0],
        ActionTemplate::CommandFix {
            suggested_command: "testcmd testcor".to_string(),
            reason: "Suggested correction for misspelled verb 'testverb'".to_string()
        }
    );
}

#[test]
fn test_from_error_verb_not_found_no_suggestion() {
    let raw_err = NounVerbError::VerbNotFound {
        noun: "testcmd".to_string(),
        verb: "testverb".to_string(),
        suggestion: "".to_string(),
    };
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::VerbNotFound);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("noun").unwrap(), &json!("testcmd"));
    assert_eq!(err.details.get("verb").unwrap(), &json!("testverb"));
    assert!(err.action_templates.is_empty());
}

#[test]
fn test_from_error_invalid_structure() {
    let raw_err =
        NounVerbError::InvalidStructure { message: "invalid structure message".to_string() };
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InvalidInput);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("invalid structure message"));
}

#[test]
fn test_from_error_execution_deadline() {
    let raw_err = NounVerbError::execution_error("Operation deadline exceeded after 100ms");
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::DeadlineExceeded);
    assert_eq!(err.severity, Severity::Critical);
    assert_eq!(
        err.action_templates[0],
        ActionTemplate::TimeoutAdjustment {
            suggested_timeout_ms: 1000,
            reason: "Increase deadline budget due to execution timeout".to_string()
        }
    );
}

#[test]
fn test_from_error_execution_generic() {
    let raw_err = NounVerbError::execution_error("ordinary execution error");
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::ExecutionError);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("ordinary execution error"));
}

#[test]
fn test_from_error_argument_error() {
    let raw_err = NounVerbError::ArgumentError { message: "missing argument".to_string() };
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InvalidInput);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("missing argument"));
}

#[test]
fn test_from_error_plugin_error() {
    let raw_err = NounVerbError::PluginError("plugin failed".to_string());
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InternalError);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("plugin failed"));
}

#[test]
fn test_from_error_validation_failed() {
    let raw_err = NounVerbError::ValidationFailed("validation failed".to_string());
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InvariantBreach);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("validation failed"));
}

#[test]
fn test_from_error_middleware_error() {
    let raw_err = NounVerbError::MiddlewareError("middleware failed".to_string());
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InternalError);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("middleware failed"));
}

#[test]
fn test_from_error_telemetry_error() {
    let raw_err = NounVerbError::TelemetryError("telemetry failed".to_string());
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InternalError);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("telemetry failed"));
}

#[test]
fn test_from_error_generic() {
    let raw_err = NounVerbError::Generic("generic error".to_string());
    let err = StructuredError::from_error(&raw_err);
    assert_eq!(err.kind, ErrorKind::InternalError);
    assert_eq!(err.severity, Severity::Error);
    assert_eq!(err.details.get("message").unwrap(), &json!("generic error"));
}
