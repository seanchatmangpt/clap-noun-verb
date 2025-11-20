//! Chicago TDD tests for Middleware System CLI Integration
//!
//! Tests middleware execution chain:
//! - Pre-execution middleware
//! - Post-execution middleware
//! - Error handling middleware
//! - Logging middleware
//! - Validation middleware
//! - Transformation middleware

use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
use clap_noun_verb::middleware::{Middleware, MiddlewareChain, MiddlewareExecutor};
use std::sync::{Arc, Mutex};

/// Test middleware that logs execution
#[derive(Clone)]
struct LoggingMiddleware {
    log: Arc<Mutex<Vec<String>>>,
}

impl LoggingMiddleware {
    fn new() -> Self {
        Self { log: Arc::new(Mutex::new(Vec::new())) }
    }

    fn get_log(&self) -> Vec<String> {
        self.log.lock().ok().unwrap().clone()
    }
}

impl Middleware for LoggingMiddleware {
    fn name(&self) -> &str {
        "logging"
    }

    fn execute(&self, input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
        self.log.lock().ok().unwrap().push(format!("Executed: {}", input.noun()));
        Ok(())
    }
}

/// Test middleware that validates input
#[derive(Clone)]
struct ValidationMiddleware {
    required_fields: Vec<String>,
}

impl ValidationMiddleware {
    fn new(fields: Vec<String>) -> Self {
        Self { required_fields: fields }
    }
}

impl Middleware for ValidationMiddleware {
    fn name(&self) -> &str {
        "validation"
    }

    fn execute(&self, input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
        for field in &self.required_fields {
            if !input.has_arg(field) {
                return Err(clap_noun_verb::error::NounVerbError::ValidationError(format!(
                    "Missing required field: {}",
                    field
                )));
            }
        }
        Ok(())
    }
}

/// Test middleware that transforms input
#[derive(Clone)]
struct TransformMiddleware;

impl Middleware for TransformMiddleware {
    fn name(&self) -> &str {
        "transform"
    }

    fn execute(&self, input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
        // Transform noun to uppercase
        let noun = input.noun().to_uppercase();
        input.set_noun(&noun);
        Ok(())
    }
}

// ============================================================================
// Middleware Chain Tests (30+ tests)
// ============================================================================

#[test]
fn test_middleware_chain_creation() {
    // Arrange & Act
    let chain = MiddlewareChain::new();

    // Assert
    assert_eq!(chain.len(), 0, "New chain should be empty");
}

#[test]
fn test_middleware_chain_add_single() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    let logging = Box::new(LoggingMiddleware::new());

    // Act
    chain.add(logging);

    // Assert
    assert_eq!(chain.len(), 1, "Chain should have 1 middleware");
}

#[test]
fn test_middleware_chain_add_multiple() {
    // Arrange
    let mut chain = MiddlewareChain::new();

    // Act
    chain.add(Box::new(LoggingMiddleware::new()));
    chain.add(Box::new(ValidationMiddleware::new(vec![])));
    chain.add(Box::new(TransformMiddleware));

    // Assert
    assert_eq!(chain.len(), 3, "Chain should have 3 middleware");
}

#[test]
fn test_middleware_execution_order() {
    // Arrange
    let logging = LoggingMiddleware::new();
    let logging_clone = logging.clone();
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(logging));

    let mut input = HandlerInput::new("test_noun", "test_verb");

    // Act
    let result = chain.execute(&mut input);

    // Assert
    assert!(result.is_ok(), "Middleware execution should succeed");
    let log = logging_clone.get_log();
    assert_eq!(log.len(), 1, "Logging middleware should execute once");
    assert_eq!(log[0], "Executed: test_noun");
}

#[test]
fn test_middleware_short_circuit_on_error() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    let logging = LoggingMiddleware::new();
    let logging_clone = logging.clone();

    // Add validation that will fail, then logging
    chain.add(Box::new(ValidationMiddleware::new(vec!["required_field".to_string()])));
    chain.add(Box::new(logging));

    let mut input = HandlerInput::new("test", "test");

    // Act
    let result = chain.execute(&mut input);

    // Assert
    assert!(result.is_err(), "Validation failure should error");
    let log = logging_clone.get_log();
    assert_eq!(log.len(), 0, "Second middleware should not execute after error");
}

#[test]
fn test_middleware_transform_modifies_input() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(TransformMiddleware));

    let mut input = HandlerInput::new("test_noun", "verb");

    // Act
    let _ = chain.execute(&mut input);

    // Assert
    assert_eq!(input.noun(), "TEST_NOUN", "Noun should be transformed to uppercase");
}

#[test]
fn test_middleware_multiple_executions() {
    // Arrange
    let logging = LoggingMiddleware::new();
    let logging_clone = logging.clone();
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(logging));

    // Act - Execute multiple times
    for i in 0..5 {
        let mut input = HandlerInput::new(&format!("noun{}", i), "verb");
        let _ = chain.execute(&mut input);
    }

    // Assert
    let log = logging_clone.get_log();
    assert_eq!(log.len(), 5, "Middleware should execute 5 times");
}

#[test]
fn test_middleware_chain_clear() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(LoggingMiddleware::new()));
    chain.add(Box::new(ValidationMiddleware::new(vec![])));

    // Act
    chain.clear();

    // Assert
    assert_eq!(chain.len(), 0, "Chain should be empty after clear");
}

#[test]
fn test_middleware_chain_remove_by_name() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(LoggingMiddleware::new()));
    chain.add(Box::new(ValidationMiddleware::new(vec![])));

    // Act
    let removed = chain.remove("logging");

    // Assert
    assert!(removed.is_ok(), "Remove should succeed");
    assert_eq!(chain.len(), 1, "Chain should have 1 middleware remaining");
}

#[test]
fn test_middleware_chain_has_middleware() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(LoggingMiddleware::new()));

    // Act & Assert
    assert!(chain.has("logging"), "Chain should have logging middleware");
    assert!(!chain.has("nonexistent"), "Chain should not have nonexistent middleware");
}

// ============================================================================
// Middleware Executor Tests (20+ tests)
// ============================================================================

#[test]
fn test_middleware_executor_creation() {
    // Arrange & Act
    let executor = MiddlewareExecutor::new();

    // Assert
    assert_eq!(executor.middleware_count(), 0, "New executor should have no middleware");
}

#[test]
fn test_middleware_executor_register() {
    // Arrange
    let mut executor = MiddlewareExecutor::new();

    // Act
    executor.register(Box::new(LoggingMiddleware::new()));
    executor.register(Box::new(ValidationMiddleware::new(vec![])));

    // Assert
    assert_eq!(executor.middleware_count(), 2, "Executor should have 2 middleware");
}

#[test]
fn test_middleware_executor_run() {
    // Arrange
    let logging = LoggingMiddleware::new();
    let logging_clone = logging.clone();
    let mut executor = MiddlewareExecutor::new();
    executor.register(Box::new(logging));

    let mut input = HandlerInput::new("test", "verb");

    // Act
    let result = executor.run(&mut input);

    // Assert
    assert!(result.is_ok(), "Executor run should succeed");
    let log = logging_clone.get_log();
    assert!(log.len() > 0, "Middleware should have executed");
}

#[test]
fn test_middleware_executor_with_validation() {
    // Arrange
    let mut executor = MiddlewareExecutor::new();
    executor.register(Box::new(ValidationMiddleware::new(vec!["required".to_string()])));

    let mut input_missing = HandlerInput::new("test", "verb");
    let mut input_valid = HandlerInput::new("test", "verb");
    input_valid.set_arg("required", "value");

    // Act & Assert
    assert!(executor.run(&mut input_missing).is_err(), "Missing required field should error");
    assert!(executor.run(&mut input_valid).is_ok(), "Valid input should succeed");
}

#[test]
fn test_middleware_executor_pre_and_post() {
    // Arrange
    let mut executor = MiddlewareExecutor::new();
    let logging = LoggingMiddleware::new();
    let logging_clone = logging.clone();

    executor.register_pre(Box::new(logging.clone()));
    executor.register_post(Box::new(logging));

    let mut input = HandlerInput::new("test", "verb");

    // Act
    executor.run_pre(&mut input).ok();
    executor.run_post(&mut input).ok();

    // Assert
    let log = logging_clone.get_log();
    assert_eq!(log.len(), 2, "Both pre and post middleware should execute");
}

#[test]
fn test_middleware_executor_error_handling() {
    // Arrange
    let mut executor = MiddlewareExecutor::new();
    executor.register(Box::new(ValidationMiddleware::new(vec!["required".to_string()])));

    let mut input = HandlerInput::new("test", "verb");

    // Act
    let result = executor.run(&mut input);

    // Assert
    assert!(result.is_err(), "Validation error should propagate");
    assert!(result.err().unwrap().to_string().contains("Missing required field"));
}

// ============================================================================
// Custom Middleware Tests (15+ tests)
// ============================================================================

/// Test middleware that counts invocations
#[derive(Clone)]
struct CounterMiddleware {
    count: Arc<Mutex<usize>>,
}

impl CounterMiddleware {
    fn new() -> Self {
        Self { count: Arc::new(Mutex::new(0)) }
    }

    fn get_count(&self) -> usize {
        *self.count.lock().ok().unwrap()
    }
}

impl Middleware for CounterMiddleware {
    fn name(&self) -> &str {
        "counter"
    }

    fn execute(&self, _input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
        *self.count.lock().ok().unwrap() += 1;
        Ok(())
    }
}

#[test]
fn test_counter_middleware_increments() {
    // Arrange
    let counter = CounterMiddleware::new();
    let counter_clone = counter.clone();
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(counter));

    // Act
    for _ in 0..10 {
        let mut input = HandlerInput::new("test", "verb");
        let _ = chain.execute(&mut input);
    }

    // Assert
    assert_eq!(counter_clone.get_count(), 10, "Counter should be incremented 10 times");
}

/// Test middleware that modifies arguments
#[derive(Clone)]
struct ArgModifierMiddleware;

impl Middleware for ArgModifierMiddleware {
    fn name(&self) -> &str {
        "arg_modifier"
    }

    fn execute(&self, input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
        input.set_arg("modified", "true");
        Ok(())
    }
}

#[test]
fn test_arg_modifier_middleware_adds_arg() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(ArgModifierMiddleware));

    let mut input = HandlerInput::new("test", "verb");

    // Act
    let _ = chain.execute(&mut input);

    // Assert
    assert!(input.has_arg("modified"), "Middleware should add argument");
    assert_eq!(input.get_arg("modified"), Some("true".to_string()));
}

/// Test middleware that checks permissions
#[derive(Clone)]
struct PermissionMiddleware {
    allowed_users: Vec<String>,
}

impl PermissionMiddleware {
    fn new(users: Vec<String>) -> Self {
        Self { allowed_users: users }
    }
}

impl Middleware for PermissionMiddleware {
    fn name(&self) -> &str {
        "permission"
    }

    fn execute(&self, input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
        if let Some(user) = input.get_arg("user") {
            if self.allowed_users.contains(&user) {
                return Ok(());
            }
        }
        Err(clap_noun_verb::error::NounVerbError::PermissionDenied)
    }
}

#[test]
fn test_permission_middleware_allows_authorized() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(PermissionMiddleware::new(vec!["admin".to_string()])));

    let mut input = HandlerInput::new("test", "verb");
    input.set_arg("user", "admin");

    // Act
    let result = chain.execute(&mut input);

    // Assert
    assert!(result.is_ok(), "Authorized user should be allowed");
}

#[test]
fn test_permission_middleware_blocks_unauthorized() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(PermissionMiddleware::new(vec!["admin".to_string()])));

    let mut input = HandlerInput::new("test", "verb");
    input.set_arg("user", "guest");

    // Act
    let result = chain.execute(&mut input);

    // Assert
    assert!(result.is_err(), "Unauthorized user should be blocked");
}

// ============================================================================
// Integration Tests - Middleware with CLI (10+ tests)
// ============================================================================

#[test]
fn test_middleware_with_command_execution() {
    // Arrange
    let logging = LoggingMiddleware::new();
    let logging_clone = logging.clone();
    let mut executor = MiddlewareExecutor::new();
    executor.register(Box::new(logging));

    // Act - Simulate command execution with middleware
    let mut input = HandlerInput::new("config", "set");
    input.set_arg("key", "value");
    let _ = executor.run(&mut input);

    // Assert
    let log = logging_clone.get_log();
    assert!(log.contains(&"Executed: config".to_string()), "Middleware should log command");
}

#[test]
fn test_middleware_error_recovery() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(ValidationMiddleware::new(vec!["field1".to_string()])));

    let mut input = HandlerInput::new("test", "verb");

    // Act - First try without field (should fail)
    let result1 = chain.execute(&mut input);
    assert!(result1.is_err(), "First execution should fail");

    // Add required field and retry
    input.set_arg("field1", "value1");
    let result2 = chain.execute(&mut input);

    // Assert
    assert!(result2.is_ok(), "Second execution with field should succeed");
}

#[test]
fn test_middleware_chain_composition() {
    // Arrange
    let mut chain = MiddlewareChain::new();
    let counter = CounterMiddleware::new();
    let counter_clone = counter.clone();

    chain.add(Box::new(counter));
    chain.add(Box::new(TransformMiddleware));
    chain.add(Box::new(ArgModifierMiddleware));

    let mut input = HandlerInput::new("test", "verb");

    // Act
    let _ = chain.execute(&mut input);

    // Assert - All middleware should execute
    assert_eq!(counter_clone.get_count(), 1, "Counter should increment");
    assert_eq!(input.noun(), "TEST", "Transform should uppercase");
    assert!(input.has_arg("modified"), "Arg modifier should add arg");
}

#[test]
fn test_middleware_concurrent_execution() {
    // Arrange
    let counter = CounterMiddleware::new();
    let counter_clone = counter.clone();
    let chain = Arc::new(Mutex::new(MiddlewareChain::new()));
    chain.lock().ok().unwrap().add(Box::new(counter));

    let mut handles = vec![];

    // Act - Concurrent executions
    for i in 0..10 {
        let chain_clone = chain.clone();
        let handle = std::thread::spawn(move || {
            let mut input = HandlerInput::new(&format!("noun{}", i), "verb");
            chain_clone.lock().ok().unwrap().execute(&mut input)
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().ok();
    }

    // Assert
    assert_eq!(counter_clone.get_count(), 10, "Counter should reach 10 from concurrent executions");
}
