use clap_noun_verb::error::{StructuredError, ErrorKind, Severity, ActionTemplate, NounVerbError};
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
