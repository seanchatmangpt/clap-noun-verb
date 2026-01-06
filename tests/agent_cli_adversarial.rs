//! Adversarial Chicago TDD Quality Assurance Tests
//! Tests validate Agent CLI Builder capabilities under stress and edge cases
//! Chicago TDD: State-based testing, real collaborators, behavior verification (AAA)

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, AgentResult, CommandArgs, CommandHandler, CommandMetadata,
    AgentBuilderError,
};
use std::sync::{Arc, Mutex};

// Real test collaborators for stateful behavior verification
#[derive(Clone)]
struct CountingHandler {
    counter: Arc<Mutex<u32>>,
}

impl CountingHandler {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    fn count(&self) -> u32 {
        *self.counter.lock().unwrap()
    }
}

impl CommandHandler for CountingHandler {
    fn execute(&self, _args: &CommandArgs) -> AgentResult<serde_json::Value> {
        let mut c = self.counter.lock().unwrap();
        *c += 1;
        Ok(serde_json::json!({ "count": *c }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "counting".to_string(),
            description: "Counting test handler".to_string(),
            arguments: Vec::new(),
            requires_args: false,
        }
    }
}

#[derive(Clone)]
struct ConditionalHandler {
    should_fail: bool,
}

impl CommandHandler for ConditionalHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        if self.should_fail {
            Err(AgentBuilderError::HandlerFailed("Conditional failure".to_string()))
        } else {
            Ok(serde_json::json!({
                "status": "success",
                "args": args.len()
            }))
        }
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "conditional".to_string(),
            description: "Conditional test handler".to_string(),
            arguments: Vec::new(),
            requires_args: false,
        }
    }
}

// ==================== CORE CAPABILITY TESTS ====================

#[test]
fn test_large_command_registry_stress() {
    // Arrange: Create CLI with 100 commands
    let mut builder = AgentCliBuilder::new("scale-test", "Scale test CLI");
    let handler = Arc::new(CountingHandler::new());

    // Register 100 commands
    for i in 0..100 {
        let cmd_name = format!("cmd-{:03}", i);
        builder.register_command(&cmd_name, &cmd_name, handler.clone()).expect("Register failed");
    }

    // Act
    let cli = builder.build().expect("Build failed");
    let commands = cli.commands();

    // Assert: All commands registered and discoverable
    assert_eq!(commands.len(), 100, "Should have 100 commands");
    assert!(commands.contains(&"cmd-000"), "Should contain first command");
    assert!(commands.contains(&"cmd-099"), "Should contain last command");
}

#[test]
fn test_execute_commands_from_large_registry() {
    // Arrange
    let mut builder = AgentCliBuilder::new("execute-scale", "Execute scale test");
    let handler = Arc::new(CountingHandler::new());

    for i in 0..50 {
        let cmd_name = format!("exec-{:02}", i);
        builder.register_command(&cmd_name, &cmd_name, handler.clone()).expect("Register failed");
    }

    let cli = builder.build().expect("Build failed");

    // Act: Execute different commands
    let result1 = cli.execute("exec-00", CommandArgs::new());
    let result2 = cli.execute("exec-25", CommandArgs::new());
    let result3 = cli.execute("exec-49", CommandArgs::new());

    // Assert: Shared handler tracks calls
    assert!(result1.is_ok(), "exec-00 should succeed");
    assert!(result2.is_ok(), "exec-25 should succeed");
    assert!(result3.is_ok(), "exec-49 should succeed");
    assert_eq!(handler.count(), 3, "Handler should be called 3 times");
}

#[test]
fn test_edge_case_command_names() {
    // Arrange
    let mut builder = AgentCliBuilder::new("edge-test", "Edge case test");
    let handler = Arc::new(CountingHandler::new());

    let names = vec![
        "cmd-with-hyphens",
        "cmd123numbers",
        "cmd_with_underscores",
        "cmd-123-mixed",
    ];

    // Act: Register edge case names
    for name in &names {
        builder.register_command(*name, *name, handler.clone()).expect("Register failed");
    }
    let cli = builder.build().expect("Build failed");
    let commands = cli.commands();

    // Assert: All edge case names work
    for name in &names {
        assert!(commands.contains(&name), "Should contain command: {}", name);
    }
}

#[test]
fn test_positional_arguments_structure() {
    // Arrange
    let handler = Arc::new(ConditionalHandler { should_fail: false });
    let mut builder = AgentCliBuilder::new("pos-test", "Positional test");
    builder.register_command("cmd", "Test command", handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act: Create args with positional values
    let args = CommandArgs::new()
        .with_positional("first")
        .with_positional("second")
        .with_positional("third");

    let result = cli.execute("cmd", args.clone());

    // Assert: Positional args preserved
    assert!(result.is_ok(), "Execution should succeed");
    let positional = args.get_all_positional();
    assert_eq!(positional.len(), 3, "Should have 3 positional args");
    assert_eq!(positional[0], "first", "First arg should be correct");
    assert_eq!(positional[2], "third", "Last arg should be correct");
}

// ==================== ERROR HANDLING ====================

#[test]
fn test_handler_error_isolation() {
    // Arrange
    let failing = Arc::new(ConditionalHandler { should_fail: true });
    let ok_handler = Arc::new(ConditionalHandler { should_fail: false });

    let mut builder = AgentCliBuilder::new("error-test", "Error test");
    builder.register_command("fail", "Failing", failing).expect("Register failed");
    builder.register_command("ok", "OK", ok_handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act: Execute both commands
    let fail_result = cli.execute("fail", CommandArgs::new());
    let ok_result = cli.execute("ok", CommandArgs::new());

    // Assert: Failure doesn't affect subsequent commands
    assert!(fail_result.is_err(), "Failing command should error");
    assert!(ok_result.is_ok(), "OK command should succeed");
}

#[test]
fn test_nonexistent_command_error() {
    // Arrange
    let handler = Arc::new(CountingHandler::new());
    let mut builder = AgentCliBuilder::new("nonexist", "Nonexist test");
    builder.register_command("exists", "Exists", handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act
    let result = cli.execute("does_not_exist", CommandArgs::new());

    // Assert: Proper error
    assert!(result.is_err(), "Should error for nonexistent command");
}

#[test]
fn test_duplicate_command_rejected() {
    // Arrange
    let handler = Arc::new(CountingHandler::new());
    let mut builder = AgentCliBuilder::new("dup-test", "Duplicate test");

    // Act: Try to register duplicate
    builder.register_command("dup", "First", handler.clone()).expect("First should succeed");
    let result = builder.register_command("dup", "Second", handler);

    // Assert: Duplicate rejected
    assert!(result.is_err(), "Should reject duplicate command");
}

#[test]
fn test_empty_cli_build_fails() {
    // Arrange & Act: Try to build CLI with no commands
    let builder = AgentCliBuilder::new("empty", "Empty test");
    let result = builder.build();

    // Assert: Should fail
    assert!(result.is_err(), "Should error when no commands registered");
}

// ==================== STATE ISOLATION ====================

#[test]
fn test_independent_handler_instances() {
    // Arrange: Two separate handler instances
    let handler1 = Arc::new(CountingHandler::new());
    let handler2 = Arc::new(CountingHandler::new());

    let mut builder = AgentCliBuilder::new("isolation", "Isolation test");
    builder.register_command("cmd1", "Command 1", handler1.clone()).expect("Register failed");
    builder.register_command("cmd2", "Command 2", handler2.clone()).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act: Execute both commands
    cli.execute("cmd1", CommandArgs::new()).ok();
    cli.execute("cmd2", CommandArgs::new()).ok();

    // Assert: Each handler has independent state
    assert_eq!(handler1.count(), 1, "Handler1 called once");
    assert_eq!(handler2.count(), 1, "Handler2 called once");
}

#[test]
fn test_shared_handler_accumulates_state() {
    // Arrange: Shared handler instance
    let shared = Arc::new(CountingHandler::new());

    let mut builder = AgentCliBuilder::new("shared", "Shared state test");
    builder.register_command("cmd-a", "A", shared.clone()).expect("Register failed");
    builder.register_command("cmd-b", "B", shared.clone()).expect("Register failed");
    builder.register_command("cmd-c", "C", shared.clone()).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act: Execute all (same handler)
    cli.execute("cmd-a", CommandArgs::new()).ok();
    cli.execute("cmd-b", CommandArgs::new()).ok();
    cli.execute("cmd-c", CommandArgs::new()).ok();

    // Assert: Shared state accumulates
    assert_eq!(shared.count(), 3, "Shared handler should count 3 calls total");
}

// ==================== METADATA & DISCOVERY ====================

#[test]
fn test_command_discovery() {
    // Arrange
    let handler = Arc::new(CountingHandler::new());
    let mut builder = AgentCliBuilder::new("discover", "Discover test");

    for name in &["list", "show", "create", "delete"] {
        builder.register_command(*name, *name, handler.clone()).expect("Register failed");
    }

    // Act
    let cli = builder.build().expect("Build failed");
    let commands = cli.commands();

    // Assert: All commands discoverable
    assert_eq!(commands.len(), 4, "Should discover 4 commands");
    assert!(commands.contains(&"list"), "Should contain list");
    assert!(commands.contains(&"show"), "Should contain show");
}

#[test]
fn test_command_metadata_retrieval() {
    // Arrange
    let handler = Arc::new(CountingHandler::new());
    let mut builder = AgentCliBuilder::new("metadata", "Metadata test");
    builder.register_command("test-cmd", "Test description", handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act
    let info = cli.command_info("test-cmd");

    // Assert: Metadata available
    assert!(info.is_some(), "Should return command info");
    let meta = info.unwrap();
    assert_eq!(meta.name, "test-cmd", "Name should match");
    assert_eq!(meta.description, "Test description", "Description should match");
}

// ==================== ARGUMENT HANDLING ====================

#[test]
fn test_empty_arguments_execution() {
    // Arrange
    let handler = Arc::new(ConditionalHandler { should_fail: false });
    let mut builder = AgentCliBuilder::new("empty-args", "Empty args test");
    builder.register_command("cmd", "Test", handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act
    let result = cli.execute("cmd", CommandArgs::new());

    // Assert: Works with empty args
    assert!(result.is_ok(), "Should handle empty arguments");
}

#[test]
fn test_named_arguments() {
    // Arrange
    let handler = Arc::new(ConditionalHandler { should_fail: false });
    let mut builder = AgentCliBuilder::new("named-args", "Named args test");
    builder.register_command("cmd", "Test", handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act: Build args with named values
    let args = CommandArgs::new()
        .with_arg("key1", "value1")
        .with_arg("key2", "value2");

    let result = cli.execute("cmd", args.clone());

    // Assert: Args stored correctly
    assert!(result.is_ok(), "Should execute successfully");
    assert_eq!(args.get("key1"), Some("value1"), "Should retrieve key1");
    assert_eq!(args.get("key2"), Some("value2"), "Should retrieve key2");
}

// ==================== JSON SERIALIZATION ====================

#[test]
fn test_execution_returns_json() {
    // Arrange
    let handler = Arc::new(ConditionalHandler { should_fail: false });
    let mut builder = AgentCliBuilder::new("json", "JSON test");
    builder.register_command("cmd", "Test", handler).expect("Register failed");
    let cli = builder.build().expect("Build failed");

    // Act
    let result = cli.execute("cmd", CommandArgs::new());

    // Assert: Result is valid JSON object
    assert!(result.is_ok(), "Should execute");
    let json = result.unwrap();
    assert!(json.is_object(), "Should be JSON object");
    assert!(json.get("status").is_some(), "Should have status field");
}
