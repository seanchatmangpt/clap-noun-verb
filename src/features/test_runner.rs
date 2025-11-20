//! TestRunner for Testing Capabilities
//!
//! Provides a testing framework for executing and validating noun-verb commands
//! with support for fixtures, assertions, and reporting.

use serde_json::{json, Value};
use std::time::Duration;

/// Test assertion result
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub passed: bool,
    pub message: String,
}

impl AssertionResult {
    /// Create successful assertion
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            passed: true,
            message: message.into(),
        }
    }

    /// Create failed assertion
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            passed: false,
            message: message.into(),
        }
    }
}

/// Test fixture for setup/teardown
#[derive(Debug, Clone)]
pub struct TestFixture {
    pub name: String,
    pub setup_data: Value,
    pub teardown_required: bool,
}

impl TestFixture {
    /// Create new test fixture
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            setup_data: json!({}),
            teardown_required: false,
        }
    }

    /// Set up data
    pub fn with_data(mut self, data: Value) -> Self {
        self.setup_data = data;
        self
    }

    /// Mark teardown required
    pub fn with_teardown(mut self) -> Self {
        self.teardown_required = true;
        self
    }
}

/// Test case
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub input: Value,
    pub expected_output: Value,
    pub timeout: Duration,
    pub should_fail: bool,
}

impl TestCase {
    /// Create new test case
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            input: json!({}),
            expected_output: json!({}),
            timeout: Duration::from_secs(5),
            should_fail: false,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set input data
    pub fn with_input(mut self, input: Value) -> Self {
        self.input = input;
        self
    }

    /// Set expected output
    pub fn with_expected(mut self, expected: Value) -> Self {
        self.expected_output = expected;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Mark as expected to fail
    pub fn should_fail(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub assertions: Vec<AssertionResult>,
    pub duration: Duration,
    pub error: Option<String>,
}

impl TestResult {
    /// Create successful test result
    pub fn success(name: impl Into<String>, assertions: Vec<AssertionResult>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            passed: true,
            assertions,
            duration,
            error: None,
        }
    }

    /// Create failed test result
    pub fn failure(name: impl Into<String>, error: String, duration: Duration) -> Self {
        Self {
            name: name.into(),
            passed: false,
            assertions: vec![],
            duration,
            error: Some(error),
        }
    }

    /// Check if all assertions passed
    pub fn all_assertions_passed(&self) -> bool {
        self.assertions.iter().all(|a| a.passed)
    }

    /// Get assertion pass rate
    pub fn assertion_pass_rate(&self) -> f64 {
        if self.assertions.is_empty() {
            return 1.0;
        }
        let passed = self.assertions.iter().filter(|a| a.passed).count();
        passed as f64 / self.assertions.len() as f64
    }
}

/// Test runner for executing test cases
#[derive(Debug)]
pub struct TestRunner {
    fixtures: Vec<TestFixture>,
}

impl TestRunner {
    /// Create new test runner
    pub fn new() -> Self {
        Self {
            fixtures: Vec::new(),
        }
    }

    /// Add fixture
    pub fn add_fixture(mut self, fixture: TestFixture) -> Self {
        self.fixtures.push(fixture);
        self
    }

    /// Get fixture by name
    pub fn get_fixture(&self, name: &str) -> Option<&TestFixture> {
        self.fixtures.iter().find(|f| f.name == name)
    }

    /// Assert equality
    pub fn assert_eq(actual: Value, expected: Value) -> AssertionResult {
        if actual == expected {
            AssertionResult::success(format!("{} == {}", actual, expected))
        } else {
            AssertionResult::failure(format!("Expected {}, got {}", expected, actual))
        }
    }

    /// Assert not equal
    pub fn assert_ne(actual: Value, expected: Value) -> AssertionResult {
        if actual != expected {
            AssertionResult::success(format!("{} != {}", actual, expected))
        } else {
            AssertionResult::failure(format!("Expected values to differ, but both are {}", actual))
        }
    }

    /// Assert true
    pub fn assert_true(condition: bool, message: impl Into<String>) -> AssertionResult {
        if condition {
            AssertionResult::success(message)
        } else {
            AssertionResult::failure(message)
        }
    }

    /// Assert false
    pub fn assert_false(condition: bool, message: impl Into<String>) -> AssertionResult {
        if !condition {
            AssertionResult::success(message)
        } else {
            AssertionResult::failure(message)
        }
    }

    /// Assert contains
    pub fn assert_contains(container: &str, substring: &str) -> AssertionResult {
        if container.contains(substring) {
            AssertionResult::success(format!("'{}' contains '{}'", container, substring))
        } else {
            AssertionResult::failure(format!("'{}' does not contain '{}'", container, substring))
        }
    }

    /// Assert not contains
    pub fn assert_not_contains(container: &str, substring: &str) -> AssertionResult {
        if !container.contains(substring) {
            AssertionResult::success(format!("'{}' does not contain '{}'", container, substring))
        } else {
            AssertionResult::failure(format!("'{}' contains '{}'", container, substring))
        }
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertion_result_success() {
        let result = AssertionResult::success("Test passed");
        assert!(result.passed);
        assert_eq!(result.message, "Test passed");
    }

    #[test]
    fn test_assertion_result_failure() {
        let result = AssertionResult::failure("Test failed");
        assert!(!result.passed);
        assert_eq!(result.message, "Test failed");
    }

    #[test]
    fn test_test_fixture_new() {
        let fixture = TestFixture::new("setup");
        assert_eq!(fixture.name, "setup");
        assert!(!fixture.teardown_required);
    }

    #[test]
    fn test_test_fixture_with_data() {
        let fixture = TestFixture::new("user_fixture")
            .with_data(json!({"id": "123", "name": "alice"}));

        assert_eq!(fixture.setup_data["id"], "123");
        assert_eq!(fixture.setup_data["name"], "alice");
    }

    #[test]
    fn test_test_fixture_with_teardown() {
        let fixture = TestFixture::new("cleanup").with_teardown();
        assert!(fixture.teardown_required);
    }

    #[test]
    fn test_test_case_new() {
        let test_case = TestCase::new("create_user");
        assert_eq!(test_case.name, "create_user");
        assert_eq!(test_case.timeout, Duration::from_secs(5));
        assert!(!test_case.should_fail);
    }

    #[test]
    fn test_test_case_builder() {
        let test_case = TestCase::new("update_product")
            .with_description("Update product price")
            .with_input(json!({"price": 99.99}))
            .with_expected(json!({"success": true}))
            .with_timeout(Duration::from_secs(10));

        assert_eq!(test_case.description, "Update product price");
        assert_eq!(test_case.input["price"], 99.99);
        assert_eq!(test_case.expected_output["success"], true);
        assert_eq!(test_case.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_test_case_should_fail() {
        let test_case = TestCase::new("invalid_operation").should_fail();
        assert!(test_case.should_fail);
    }

    #[test]
    fn test_test_result_success() {
        let assertions = vec![
            AssertionResult::success("First check"),
            AssertionResult::success("Second check"),
        ];
        let result = TestResult::success("test1", assertions, Duration::from_millis(100));

        assert!(result.passed);
        assert_eq!(result.assertions.len(), 2);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_test_result_failure() {
        let result = TestResult::failure("test2", "Operation timed out".to_string(), Duration::from_millis(5000));

        assert!(!result.passed);
        assert_eq!(result.error, Some("Operation timed out".to_string()));
    }

    #[test]
    fn test_test_result_all_assertions_passed() {
        let assertions = vec![
            AssertionResult::success("First"),
            AssertionResult::success("Second"),
        ];
        let result = TestResult::success("test3", assertions, Duration::from_millis(50));
        assert!(result.all_assertions_passed());

        let assertions_mixed = vec![
            AssertionResult::success("First"),
            AssertionResult::failure("Failed check"),
        ];
        let result_mixed = TestResult::success("test4", assertions_mixed, Duration::from_millis(50));
        assert!(!result_mixed.all_assertions_passed());
    }

    #[test]
    fn test_test_result_assertion_pass_rate() {
        let assertions = vec![
            AssertionResult::success("Pass 1"),
            AssertionResult::success("Pass 2"),
            AssertionResult::failure("Fail 1"),
        ];
        let result = TestResult::success("test5", assertions, Duration::from_millis(50));
        assert_eq!(result.assertion_pass_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_test_runner_new() {
        let runner = TestRunner::new();
        assert!(runner.fixtures.is_empty());
    }

    #[test]
    fn test_test_runner_add_fixture() {
        let fixture = TestFixture::new("setup");
        let runner = TestRunner::new().add_fixture(fixture);
        assert_eq!(runner.fixtures.len(), 1);
    }

    #[test]
    fn test_test_runner_get_fixture() {
        let fixture = TestFixture::new("db_setup").with_data(json!({"db": "test"}));
        let runner = TestRunner::new().add_fixture(fixture);

        let found = runner.get_fixture("db_setup");
        assert!(found.is_some());
        assert_eq!(found.unwrap().setup_data["db"], "test");
    }

    #[test]
    fn test_assert_eq() {
        let assertion = TestRunner::assert_eq(json!("hello"), json!("hello"));
        assert!(assertion.passed);

        let assertion = TestRunner::assert_eq(json!(42), json!(43));
        assert!(!assertion.passed);
    }

    #[test]
    fn test_assert_ne() {
        let assertion = TestRunner::assert_ne(json!(1), json!(2));
        assert!(assertion.passed);

        let assertion = TestRunner::assert_ne(json!("a"), json!("a"));
        assert!(!assertion.passed);
    }

    #[test]
    fn test_assert_true() {
        let assertion = TestRunner::assert_true(true, "Should pass");
        assert!(assertion.passed);

        let assertion = TestRunner::assert_true(false, "Should fail");
        assert!(!assertion.passed);
    }

    #[test]
    fn test_assert_false() {
        let assertion = TestRunner::assert_false(false, "Should pass");
        assert!(assertion.passed);

        let assertion = TestRunner::assert_false(true, "Should fail");
        assert!(!assertion.passed);
    }

    #[test]
    fn test_assert_contains() {
        let assertion = TestRunner::assert_contains("hello world", "world");
        assert!(assertion.passed);

        let assertion = TestRunner::assert_contains("foo", "bar");
        assert!(!assertion.passed);
    }

    #[test]
    fn test_assert_not_contains() {
        let assertion = TestRunner::assert_not_contains("hello", "xyz");
        assert!(assertion.passed);

        let assertion = TestRunner::assert_not_contains("hello", "ell");
        assert!(!assertion.passed);
    }
}
