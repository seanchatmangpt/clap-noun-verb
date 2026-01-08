//! Integration tests for agent CLI composition
//!
//! Tests verify that MCP agents can:
//! - Create CLIs without macros
//! - Register dynamic commands
//! - Execute commands and get structured results
//! - Discover and introspect CLI capabilities

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, AgentResult, CommandArgs, CommandHandler, CommandMetadata,
};
use std::sync::Arc;

// ============================================================================
// Test Command Handlers
// ============================================================================

/// Simple test handler
struct EchoHandler {
    message: String,
}

impl EchoHandler {
    fn new(message: &str) -> Arc<Self> {
        Arc::new(Self { message: message.to_string() })
    }
}

impl CommandHandler for EchoHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        let input = args.get("input").unwrap_or(&self.message);
        Ok(serde_json::json!({
            "output": input,
            "echoed": true
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "echo".to_string(),
            description: "Echo the input".to_string(),
            arguments: vec![],
            requires_args: false,
        }
    }
}

/// Counter handler
struct CounterHandler;

impl CommandHandler for CounterHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        let count = args.get("count").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        Ok(serde_json::json!({
            "count": count,
            "counted": true
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "count".to_string(),
            description: "Count something".to_string(),
            arguments: vec![],
            requires_args: false,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_agent_cli_builder_new() {
    // Arrange & Act
    let builder = AgentCliBuilder::new("test-cli", "Test CLI");

    // Assert
    assert_eq!(builder.command_count(), 0);
}

#[test]
fn test_agent_cli_register_single_command() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    let handler = EchoHandler::new("test");

    // Act
    let result = builder.register_command("echo", "Echo command", handler);

    // Assert
    assert!(result.is_ok());
    assert_eq!(builder.command_count(), 1);
}

#[test]
fn test_agent_cli_register_multiple_commands() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    let h1 = EchoHandler::new("test1");
    let h2 = EchoHandler::new("test2");
    let h3 = CounterHandler;

    // Act
    builder.register_command("echo1", "Echo 1", h1).ok();
    builder.register_command("echo2", "Echo 2", h2).ok();
    builder.register_command("counter", "Counter", Arc::new(h3)).ok();

    // Assert
    assert_eq!(builder.command_count(), 3);
    let commands = builder.list_commands();
    assert!(commands.contains(&"echo1".to_string()));
    assert!(commands.contains(&"echo2".to_string()));
    assert!(commands.contains(&"counter".to_string()));
}

#[test]
fn test_agent_cli_build_success() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.name(), "test-cli");
    assert_eq!(cli.description(), "Test CLI");
}

#[test]
fn test_agent_cli_execute_command() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("echo", "Echo", EchoHandler::new("hello")).ok();
    let cli = builder.build().unwrap();

    // Act
    let result = cli.execute("echo", CommandArgs::new());

    // Assert
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["output"], "hello");
    assert_eq!(output["echoed"], true);
}

#[test]
fn test_agent_cli_execute_with_args() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("echo", "Echo", EchoHandler::new("default")).ok();
    let cli = builder.build().unwrap();

    // Act
    let args = CommandArgs::new().with_arg("input", "custom");
    let result = cli.execute("echo", args);

    // Assert
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["output"], "custom");
}

#[test]
fn test_agent_cli_command_discovery() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();
    builder.register_command("counter", "Counter", Arc::new(CounterHandler)).ok();
    let cli = builder.build().unwrap();

    // Act
    let commands = cli.commands();

    // Assert
    assert_eq!(commands.len(), 2);
    assert!(commands.contains(&"echo"));
    assert!(commands.contains(&"counter"));
}

#[test]
fn test_agent_cli_command_metadata() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("echo", "Echo command description", EchoHandler::new("test")).ok();
    let cli = builder.build().unwrap();

    // Act
    let metadata = cli.command_info("echo");

    // Assert
    assert!(metadata.is_some());
    let meta = metadata.unwrap();
    assert_eq!(meta.name, "echo");
    assert_eq!(meta.description, "Echo command description");
}

#[test]
fn test_agent_cli_version() {
    // Arrange
    let builder = AgentCliBuilder::new("test-cli", "Test").version("2.0.0");

    // Assert - version is set internally
    assert!(builder.version.is_some());
}

#[test]
fn test_agent_cli_help_text() {
    // Arrange
    let mut builder = AgentCliBuilder::new("myapp", "My CLI");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();
    builder.register_command("counter", "Counter", Arc::new(CounterHandler)).ok();
    let cli = builder.build().unwrap();

    // Act
    let help = cli.help();

    // Assert
    assert!(help.contains("myapp"));
    assert!(help.contains("My CLI"));
    assert!(help.contains("echo"));
    assert!(help.contains("counter"));
}

#[test]
fn test_agent_cli_execute_nonexistent_command() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();
    let cli = builder.build().unwrap();

    // Act
    let result = cli.execute("missing", CommandArgs::new());

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_agent_cli_json_output() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test");
    builder.register_command("counter", "Counter", Arc::new(CounterHandler)).ok();
    let cli = builder.build().unwrap();

    // Act
    let args = CommandArgs::new().with_arg("count", "42");
    let result = cli.execute("counter", args);

    // Assert
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_object());
    assert_eq!(output["count"], 42);
    assert_eq!(output["counted"], true);
}

#[test]
fn test_agent_cli_composition_workflow() {
    // Arrange: Agent discovers it needs file operations
    let mut builder = AgentCliBuilder::new("agent-cli", "Dynamic agent CLI");

    // Act: Agent dynamically composes CLI from available handlers
    builder.register_command("echo", "Echo input", EchoHandler::new("Agent speaking")).ok();
    builder.register_command("count", "Count items", Arc::new(CounterHandler)).ok();

    // Build and execute
    let cli = builder.build().unwrap();

    // Assert: Can discover and execute commands
    assert_eq!(cli.commands().len(), 2);

    // Execute both commands
    let echo_result = cli.execute("echo", CommandArgs::new()).unwrap();
    assert_eq!(echo_result["output"], "Agent speaking");

    let count_result = cli.execute("count", CommandArgs::new().with_arg("count", "99")).unwrap();
    assert_eq!(count_result["count"], 99);
}

#[test]
fn test_agent_cli_introspection_for_mcp() {
    // Arrange: Simulate MCP agent introspecting a generated CLI
    let mut builder = AgentCliBuilder::new("agent-generated-cli", "CLI built by MCP agent");
    builder.register_command("list", "List resources", EchoHandler::new("[]")).ok();
    builder.register_command("read", "Read resource", EchoHandler::new("{}")).ok();
    let cli = builder.build().unwrap();

    // Act: MCP agent discovers what commands are available
    let available_commands = cli.commands();

    // Assert: MCP can introspect and plan execution
    assert_eq!(available_commands.len(), 2);

    for cmd_name in available_commands {
        let metadata = cli.command_info(cmd_name);
        assert!(metadata.is_some());
    }
}

#[test]
fn test_agent_cli_deterministic_output() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test");
    builder.register_command("echo", "Echo", EchoHandler::new("fixed")).ok();
    let cli = builder.build().unwrap();

    // Act: Execute same command multiple times
    let result1 = cli.execute("echo", CommandArgs::new()).unwrap();
    let result2 = cli.execute("echo", CommandArgs::new()).unwrap();

    // Assert: Output is deterministic
    assert_eq!(result1, result2);
}

#[test]
fn test_agent_cli_can_chain_commands() {
    // Arrange: Simulate chaining where output of one becomes input of another
    let mut builder = AgentCliBuilder::new("chain-cli", "CLI for chaining");
    builder.register_command("produce", "Produce data", EchoHandler::new("data")).ok();
    builder.register_command("process", "Process data", EchoHandler::new("processed")).ok();
    let cli = builder.build().unwrap();

    // Act: First command produces, second command processes
    let produce_result = cli.execute("produce", CommandArgs::new()).unwrap();
    assert_eq!(produce_result["output"], "data");

    // Pass output from first command as input to second command
    let process_args =
        CommandArgs::new().with_arg("input", produce_result["output"].as_str().unwrap());
    let process_result = cli.execute("process", process_args).unwrap();

    // Assert: Chaining works - the echo handler echoes back the input provided
    assert_eq!(process_result["output"], "data");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_register_duplicate_command_error() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test", "Test");
    let handler = EchoHandler::new("test");
    builder.register_command("echo", "Echo", handler.clone()).ok();

    // Act
    let result = builder.register_command("echo", "Echo again", handler);

    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap_noun_verb::agent_cli::AgentBuilderError::DuplicateCommand(_)
    ));
}

#[test]
fn test_register_empty_command_name_error() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test", "Test");
    let handler = EchoHandler::new("test");

    // Act
    let result = builder.register_command("", "Empty name", handler);

    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap_noun_verb::agent_cli::AgentBuilderError::InvalidCommandName(_)
    ));
}

#[test]
fn test_build_without_commands_error() {
    // Arrange
    let builder = AgentCliBuilder::new("test", "Test");

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap_noun_verb::agent_cli::AgentBuilderError::NoCommands
    ));
}

#[test]
fn test_execute_nonexistent_command_error() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test", "Test");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();
    let cli = builder.build().unwrap();

    // Act
    let result = cli.execute("missing", CommandArgs::new());

    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap_noun_verb::agent_cli::AgentBuilderError::HandlerFailed(_)
    ));
}

#[test]
fn test_run_with_args_no_command_error() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test", "Test");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();
    let cli = builder.build().unwrap();

    // Act: No command specified in args
    let result = cli.run_with_args(vec!["test-cli".to_string()]);

    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap_noun_verb::agent_cli::AgentBuilderError::ValidationFailed(_)
    ));
}

// ============================================================================
// Convenience Method Tests
// ============================================================================

#[test]
fn test_command_args_contains() {
    // Arrange
    let args = CommandArgs::new().with_arg("key1", "value1").with_arg("key2", "value2");

    // Assert
    assert!(args.contains("key1"));
    assert!(args.contains("key2"));
    assert!(!args.contains("key3"));
}

#[test]
fn test_command_args_get_all_positional() {
    // Arrange
    let args =
        CommandArgs::new().with_positional("arg1").with_positional("arg2").with_positional("arg3");

    // Act
    let all = args.get_all_positional();

    // Assert
    assert_eq!(all.len(), 3);
    assert_eq!(all[0], "arg1");
    assert_eq!(all[1], "arg2");
    assert_eq!(all[2], "arg3");
}

#[test]
fn test_command_args_len() {
    // Arrange
    let args = CommandArgs::new()
        .with_arg("named1", "value1")
        .with_arg("named2", "value2")
        .with_positional("pos1")
        .with_positional("pos2");

    // Act
    let len = args.len();

    // Assert
    assert_eq!(len, 4); // 2 named + 2 positional
}

#[test]
fn test_command_args_is_empty() {
    // Arrange
    let empty_args = CommandArgs::new();
    let args_with_named = CommandArgs::new().with_arg("key", "value");
    let args_with_positional = CommandArgs::new().with_positional("arg");

    // Assert
    assert!(empty_args.is_empty());
    assert!(!args_with_named.is_empty());
    assert!(!args_with_positional.is_empty());
}

#[test]
fn test_command_args_len_empty() {
    // Arrange
    let args = CommandArgs::new();

    // Act
    let len = args.len();

    // Assert
    assert_eq!(len, 0);
}

// ============================================================================
// Batch Registration Tests
// ============================================================================

#[test]
fn test_batch_command_registration() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    let h1 = EchoHandler::new("handler1");
    let h2 = EchoHandler::new("handler2");
    let h3 = CounterHandler;

    let commands = vec![
        ("cmd1".to_string(), "Command 1".to_string(), h1 as Arc<dyn CommandHandler>),
        ("cmd2".to_string(), "Command 2".to_string(), h2 as Arc<dyn CommandHandler>),
        ("cmd3".to_string(), "Command 3".to_string(), Arc::new(h3) as Arc<dyn CommandHandler>),
    ];

    // Act
    let result = builder.register_commands(commands);

    // Assert
    assert!(result.is_ok());
    assert_eq!(builder.command_count(), 3);
    assert_eq!(builder.list_commands().len(), 3);
}

#[test]
fn test_batch_registration_stops_on_error() {
    // Arrange
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("cmd1", "Command 1", EchoHandler::new("test")).ok();

    let h1 = EchoHandler::new("test");
    let h2 = CounterHandler;

    let commands = vec![
        ("cmd1".to_string(), "Duplicate".to_string(), h1 as Arc<dyn CommandHandler>),
        ("cmd2".to_string(), "Command 2".to_string(), Arc::new(h2) as Arc<dyn CommandHandler>),
    ];

    // Act
    let result = builder.register_commands(commands);

    // Assert - should fail on first duplicate
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap_noun_verb::agent_cli::AgentBuilderError::DuplicateCommand(_)
    ));
    // Only the first command (cmd1) was registered before failure
    assert_eq!(builder.command_count(), 1);
}

#[test]
fn test_batch_registration_mixed_workflow() {
    // Arrange: Register some commands individually, then batch
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    builder.register_command("echo", "Echo", EchoHandler::new("test")).ok();

    let h1 = CounterHandler;
    let h2 = EchoHandler::new("batch1");

    let batch_commands = vec![
        ("count".to_string(), "Count".to_string(), Arc::new(h1) as Arc<dyn CommandHandler>),
        ("batch".to_string(), "Batch".to_string(), h2 as Arc<dyn CommandHandler>),
    ];

    // Act
    let result = builder.register_commands(batch_commands);
    assert!(result.is_ok()); // Check result first before moving builder

    let cli = builder.build();

    // Assert
    assert!(cli.is_ok());
    assert_eq!(cli.unwrap().commands().len(), 3);
}
