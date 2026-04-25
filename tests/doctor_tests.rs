//! Tests for the diagnostic engine and pathology taxonomy

use clap_noun_verb::cli::doctor_cmd::{DiagnosticEngine, Pathology, Severity};

#[test]
fn test_pathology_creation() {
    let path = Pathology::new(
        "TestPathology",
        Severity::Warning,
        "A test diagnosis",
        "A test treatment"
    )
    .with_symptom("Symptom 1")
    .with_symptom("Symptom 2")
    .repairable(true);

    assert_eq!(path.name, "TestPathology");
    assert_eq!(path.severity, Severity::Warning);
    assert_eq!(path.diagnosis, "A test diagnosis");
    assert_eq!(path.treatment, "A test treatment");
    assert_eq!(path.symptoms.len(), 2);
    assert_eq!(path.symptoms[0], "Symptom 1");
    assert_eq!(path.symptoms[1], "Symptom 2");
    assert!(path.auto_repairable);
}

#[test]
fn test_diagnostic_engine_healthy() {
    let engine = DiagnosticEngine::new(false, true); // no fix, quiet mode
    assert!(!engine.has_errors());
}

#[test]
fn test_diagnostic_engine_with_warning() {
    let mut engine = DiagnosticEngine::new(false, true);
    
    engine.record(Pathology::new(
        "WarningPathology",
        Severity::Warning,
        "Diagnosis",
        "Treatment"
    ));

    // Warnings are not considered "errors" that fail the build
    assert!(!engine.has_errors());
}

#[test]
fn test_diagnostic_engine_with_error() {
    let mut engine = DiagnosticEngine::new(false, true);
    
    engine.record(Pathology::new(
        "ErrorPathology",
        Severity::Error,
        "Diagnosis",
        "Treatment"
    ));

    // Errors fail the build
    assert!(engine.has_errors());
}

#[test]
fn test_diagnostic_engine_with_critical() {
    let mut engine = DiagnosticEngine::new(false, true);
    
    engine.record(Pathology::new(
        "CriticalPathology",
        Severity::Critical,
        "Diagnosis",
        "Treatment"
    ));

    // Critical fails the build
    assert!(engine.has_errors());
}
