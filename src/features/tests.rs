//! Integration Tests for Feature Modules
//!
//! Tests that verify the complete noun-verb command flow including
//! CRUD operations, middleware processing, routing, and error handling.

use super::async_runner::AsyncRunner;
use super::builder::CommandBuilder;
use super::crud::{CrudOperation, NounVerb, OperationResult};
use super::error_handler::{ErrorHandler, Severity};
use super::executor::{ExecutionContext, VerbExecutor};
use super::middleware_chain::{Middleware, MiddlewareChain, MiddlewareMessage};
use super::route_registry::{RouteInfo, RouteRegistry};
use super::test_runner::{TestCase, TestFixture, TestResult, TestRunner};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

// ===== Mock Implementations =====

#[derive(Debug)]
struct MockNoun;

#[async_trait::async_trait]
impl NounVerb for MockNoun {
    fn noun_name(&self) -> &str {
        "Test"
    }

    async fn create(&self, data: Value) -> OperationResult<Value> {
        Ok(json!({"id": "1", "data": data}))
    }

    async fn read(&self, id: &str) -> OperationResult<Value> {
        Ok(json!({"id": id, "name": "test"}))
    }

    async fn update(&self, id: &str, data: Value) -> OperationResult<Value> {
        Ok(json!({"id": id, "data": data}))
    }

    async fn delete(&self, _id: &str) -> OperationResult<()> {
        Ok(())
    }

    async fn list(&self) -> OperationResult<Vec<Value>> {
        Ok(vec![json!({"id": "1"}), json!({"id": "2"})])
    }

    async fn execute(&self, operation: &str, data: Value) -> OperationResult<Value> {
        Ok(json!({"operation": operation, "data": data}))
    }
}

#[derive(Debug)]
struct CountingMiddleware {
    name: String,
}

#[async_trait::async_trait]
impl Middleware for CountingMiddleware {
    async fn process(&self, mut message: MiddlewareMessage) -> Result<MiddlewareMessage, String> {
        message.request_data[&self.name] = json!(true);
        Ok(message)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ===== Integration Tests =====

#[test]
fn test_command_builder_integration() {
    let cmd = CommandBuilder::new("user", "create")
        .with_arg("alice")
        .with_arg("--email")
        .with_arg("alice@example.com")
        .build();

    assert_eq!(cmd.noun, "user");
    assert_eq!(cmd.verb, "create");
    assert_eq!(cmd.args.len(), 3);

    let json = cmd.to_json();
    assert_eq!(json["noun"], "user");
    assert_eq!(json["verb"], "create");

    let cli_str = cmd.to_cli_string();
    assert!(cli_str.contains("user create"));
    assert!(cli_str.contains("alice"));
}

#[tokio::test]
async fn test_execution_context_flow() {
    let context = ExecutionContext::new("product", "update")
        .with_id("prod-123")
        .with_data(json!({"price": 99.99}))
        .with_metadata("version", json!("1.0"));

    assert_eq!(context.noun, "product");
    assert_eq!(context.verb, "update");
    assert_eq!(context.resource_id, Some("prod-123".to_string()));
    assert_eq!(context.data["price"], 99.99);
    assert_eq!(context.metadata["version"], "1.0");
}

#[tokio::test]
async fn test_verb_executor_with_mock_noun() {
    let mock = MockNoun;
    let context = ExecutionContext::new("test", "create").with_data(json!({"value": "test"}));

    let result = VerbExecutor::execute(context, &mock).await.unwrap();

    assert_eq!(result["noun"], "test");
    assert_eq!(result["verb"], "create");
    assert_eq!(result["result"]["data"]["value"], "test");
}

#[tokio::test]
async fn test_crud_operations_flow() {
    let mock = MockNoun;

    // Test Create
    let create_result = mock
        .create(json!({"name": "Alice"}))
        .await
        .unwrap();
    assert_eq!(create_result["id"], "1");

    // Test Read
    let read_result = mock.read("1").await.unwrap();
    assert_eq!(read_result["id"], "1");
    assert_eq!(read_result["name"], "test");

    // Test Update
    let update_result = mock
        .update("1", json!({"name": "Bob"}))
        .await
        .unwrap();
    assert_eq!(update_result["id"], "1");
    assert_eq!(update_result["data"]["name"], "Bob");

    // Test List
    let list_result = mock.list().await.unwrap();
    assert_eq!(list_result.len(), 2);

    // Test Delete
    assert!(mock.delete("1").await.is_ok());

    // Test Execute
    let exec_result = mock
        .execute("custom_op", json!({"key": "value"}))
        .await
        .unwrap();
    assert_eq!(exec_result["operation"], "custom_op");
}

#[tokio::test]
async fn test_error_handling_flow() {
    let error = super::crud::OperationError::NotFound("resource:123".to_string());
    let error_info = ErrorHandler::handle(error);

    assert_eq!(error_info.severity, Severity::Warning);

    let with_context = error_info
        .with_context("attempt", json!(1))
        .with_hint("Check if resource exists");

    assert_eq!(with_context.context["attempt"], 1);
    assert!(with_context.recovery_hint.is_some());

    let log = ErrorHandler::log(&with_context);
    assert!(log.contains("WARN"));
}

#[tokio::test]
async fn test_middleware_chain_integration() {
    let middleware1 = Arc::new(CountingMiddleware {
        name: "first".to_string(),
    });
    let middleware2 = Arc::new(CountingMiddleware {
        name: "second".to_string(),
    });

    let chain = MiddlewareChain::new()
        .add(middleware1)
        .add(middleware2);

    let context = ExecutionContext::new("user", "create");
    let msg = MiddlewareMessage::new(context, json!({}));

    let result = chain.process(msg).await.unwrap();
    assert_eq!(result.request_data["first"], true);
    assert_eq!(result.request_data["second"], true);
}

#[test]
fn test_route_registry_integration() {
    let mock = Arc::new(MockNoun);

    let route_info = RouteInfo::new("test")
        .with_verb("create")
        .with_verb("read")
        .with_verb("update")
        .with_verb("delete")
        .with_verb("list")
        .with_description("Test noun implementation");

    let registry = RouteRegistry::new().register("test", mock, route_info);

    // Verify registration
    assert!(registry.has_noun("test"));
    assert!(registry.supports_verb("test", "create"));
    assert!(registry.supports_verb("test", "read"));

    // Get handler
    let handler = registry.get_handler("test").unwrap();
    assert_eq!(handler.noun_name(), "Test");

    // Get verbs
    let verbs = registry.get_verbs("test");
    assert_eq!(verbs.len(), 5);

    // Get route info
    let info = registry.get_route_info("test").unwrap();
    assert!(info.description.is_some());
}

#[tokio::test]
async fn test_async_runner_with_operations() {
    let runner = AsyncRunner::new(Duration::from_secs(5), 3);

    let future1 = async { Ok::<i32, super::crud::OperationError>(1) };
    let future2 = async { Ok::<i32, super::crud::OperationError>(2) };
    let future3 = async { Ok::<i32, super::crud::OperationError>(3) };

    let result1 = runner.run(future1).await.unwrap();
    let result2 = runner.run(future2).await.unwrap();
    let result3 = runner.run(future3).await.unwrap();

    assert_eq!(result1, 1);
    assert_eq!(result2, 2);
    assert_eq!(result3, 3);
}

#[test]
fn test_test_runner_framework() {
    let fixture = TestFixture::new("setup").with_data(json!({"initialized": true}));

    let test_case = TestCase::new("create_user")
        .with_description("Create a new user")
        .with_input(json!({"name": "alice", "email": "alice@example.com"}))
        .with_expected(json!({"id": "1", "name": "alice"}))
        .with_timeout(Duration::from_secs(5));

    let runner = TestRunner::new().add_fixture(fixture);

    // Run assertions
    let assertions = vec![
        TestRunner::assert_eq(json!("alice"), json!("alice")),
        TestRunner::assert_ne(json!(1), json!(2)),
        TestRunner::assert_true(true, "This should pass"),
        TestRunner::assert_contains("hello world", "world"),
    ];

    let result = TestResult::success(&test_case.name, assertions, Duration::from_millis(100));
    assert!(result.passed);
    assert_eq!(result.assertion_pass_rate(), 1.0);
}

#[tokio::test]
async fn test_complete_noun_verb_flow() {
    // 1. Build command
    let cmd = CommandBuilder::new("user", "create")
        .with_arg("test-user")
        .build();

    assert!(cmd.validate().is_ok());

    // 2. Create execution context
    let context = ExecutionContext::new(cmd.noun.clone(), cmd.verb.clone())
        .with_data(json!({"name": "Alice"}));

    // 3. Execute with mock noun
    let mock = MockNoun;
    let result = VerbExecutor::execute(context, &mock).await.unwrap();

    // 4. Verify result
    assert_eq!(result["noun"], "user");
    assert_eq!(result["verb"], "create");
    assert!(result["result"]["id"].is_string());

    // 5. Handle and log any errors (in this case, success)
    let log = ErrorHandler::log(
        &ErrorHandler::handle(super::crud::OperationError::NotFound("test".to_string())),
    );
    assert!(log.contains("WARN"));
}

#[test]
fn test_multiple_nouns_in_registry() {
    let user_info = RouteInfo::new("user")
        .with_verb("create")
        .with_verb("read")
        .with_verb("delete");

    let product_info = RouteInfo::new("product")
        .with_verb("create")
        .with_verb("update");

    let order_info = RouteInfo::new("order").with_verb("list");

    let registry = RouteRegistry::new()
        .register("user", Arc::new(MockNoun), user_info)
        .register("product", Arc::new(MockNoun), product_info)
        .register("order", Arc::new(MockNoun), order_info);

    // Verify all nouns registered
    let nouns = registry.get_nouns();
    assert_eq!(nouns.len(), 3);

    // Verify verb isolation
    assert!(registry.supports_verb("user", "delete"));
    assert!(!registry.supports_verb("product", "delete"));
    assert!(!registry.supports_verb("order", "create"));

    // Find routes by verb
    let create_routes = registry.find_by_verb("create");
    assert_eq!(create_routes.len(), 2);
}

#[tokio::test]
async fn test_middleware_with_routing() {
    // Create middleware
    let middleware = Arc::new(CountingMiddleware {
        name: "auth".to_string(),
    });
    let chain = MiddlewareChain::new().add(middleware);

    // Create registry
    let registry = RouteRegistry::new().register(
        "user",
        Arc::new(MockNoun),
        RouteInfo::new("user").with_verb("create"),
    );

    // Create and process message through middleware
    let context = ExecutionContext::new("user", "create");
    let msg = MiddlewareMessage::new(context.clone(), json!({"name": "test"}));

    let processed = chain.process(msg).await.unwrap();

    // Verify middleware ran
    assert_eq!(processed.request_data["auth"], true);

    // Verify routing would work
    assert!(registry.supports_verb("user", "create"));
}

#[tokio::test]
async fn test_error_recovery_flow() {
    let mut attempts = 0;

    let result = ErrorHandler::recover_with_retry(
        || {
            attempts += 1;
            if attempts < 3 {
                Err(super::crud::OperationError::Timeout)
            } else {
                Ok("recovered".to_string())
            }
        },
        5,
    );

    assert!(result.is_ok());
    assert_eq!(attempts, 3);
}

#[test]
fn test_command_validation_integration() {
    // Valid command
    let valid_cmd = CommandBuilder::new("user", "create")
        .with_arg("alice")
        .build();
    assert!(valid_cmd.validate().is_ok());

    // Invalid - no noun
    let invalid_cmd1 = CommandBuilder::new("", "create").build();
    assert!(invalid_cmd1.validate().is_err());

    // Invalid - no verb
    let invalid_cmd2 = CommandBuilder::new("user", "").build();
    assert!(invalid_cmd2.validate().is_err());

    // Valid - with options and metadata
    let complex_cmd = CommandBuilder::new("product", "update")
        .with_arg("prod-123")
        .with_option("force", json!(true))
        .with_metadata("timestamp", json!("2024-11-20"))
        .build();
    assert!(complex_cmd.validate().is_ok());
    assert_eq!(complex_cmd.options["force"], true);
}
