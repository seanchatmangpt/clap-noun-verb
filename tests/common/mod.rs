//! Common test utilities for clap-noun-verb tests

/// Capture stdout for testing
pub struct OutputCapture {
    // Will implement stdout capture for JSON output testing
}

impl OutputCapture {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn capture<F>(f: F) -> String
    where
        F: FnOnce() -> (),
    {
        // Placeholder - will implement actual stdout capture
        f();
        String::new()
    }
}

/// Assert JSON output matches expected value
pub fn assert_json_eq<T>(actual: &T, expected: &T)
where
    T: serde::Serialize + PartialEq + std::fmt::Debug,
{
    let actual_json = serde_json::to_string(actual).unwrap();
    let expected_json = serde_json::to_string(expected).unwrap();
    assert_eq!(actual_json, expected_json, "JSON output mismatch");
}

