//! Integration tests for agent CLI composition
//!
//! Tests verify that MCP agents can:
//! - Create CLIs without macros
//! - Register dynamic commands
//! - Execute commands and get structured results
//! - Discover and introspect CLI capabilities

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, CommandArgs, CommandHandler, CommandMetadata, AgentResult,
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
    let builder = AgentCliBuilder::new("test-cli", "Test")
        .version("2.0.0");

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
    let process_args = CommandArgs::new().with_arg("input", produce_result["output"].as_str().unwrap());
    let process_result = cli.execute("process", process_args).unwrap();

    // Assert: Chaining works - the echo handler echoes back the input provided
    assert_eq!(process_result["output"], "data");
}
