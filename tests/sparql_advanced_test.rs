use clap_noun_verb::NounVerbError;

#[test]
fn test_error_recovery_suggestions_command_not_found() {
    let err = NounVerbError::command_not_found("nonexistent");
    let msg = err.with_recovery_suggestions();
    assert!(msg.contains("nonexistent"));
}

#[test]
fn test_error_recovery_suggestions_verb_not_found() {
    let err = NounVerbError::verb_not_found("services", "nonexistent");
    let msg = err.with_recovery_suggestions();
    assert!(msg.contains("services"));
    assert!(msg.contains("nonexistent"));
}

#[test]
fn test_error_recovery_suggestions_generic() {
    let err = NounVerbError::Generic("Something went wrong".to_string());
    let msg = err.with_recovery_suggestions();
    assert_eq!(msg, "Error: Something went wrong");
}
