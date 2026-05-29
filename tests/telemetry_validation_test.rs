use clap_noun_verb::NounVerbError;
use std::time::{Duration, Instant};

/// A real, functional trace span for validation testing
pub struct TestSpan {
    pub name: String,
    pub start_time: Instant,
    pub attributes: std::collections::HashMap<String, String>,
}

impl TestSpan {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start_time: Instant::now(),
            attributes: std::collections::HashMap::new(),
        }
    }
}

#[test]
fn test_telemetry_error_formatting() {
    let err = NounVerbError::TelemetryError("Span validation failed".to_string());
    assert_eq!(err.to_string(), "Telemetry error: Span validation failed");
}

#[test]
fn test_telemetry_span_lifecycle_simulation() {
    let mut span = TestSpan::new("cli_execution");
    span.attributes.insert("command".to_string(), "user create".to_string());

    // Simulate execution time
    std::thread::sleep(Duration::from_millis(10));
    let duration = span.start_time.elapsed();

    assert_eq!(span.name, "cli_execution");
    assert_eq!(span.attributes.get("command").unwrap(), "user create");
    assert!(duration >= Duration::from_millis(10));
}
