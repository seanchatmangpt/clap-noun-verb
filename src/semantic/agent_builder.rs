//! Agent-optimized CLI builder for dynamic command composition
//!
//! This module enables MCP agents to programmatically create, compose, and
//! execute CLI applications at runtime without compile-time macros.
//!
//! # Type-First Design
//!
//! The agent builder encodes invariants:
//! - Commands must have unique identifiers
//! - Handlers must be type-safe and deterministic
//! - Composition must be validated before execution
//! - All operations are zero-cost abstractions
//!
//! # Agent Workflow
//!
//! ```ignore
//! use clap_noun_verb::semantic::AgentCliBuilder;
//!
//! // 1. Create builder
//! let mut cli = AgentCliBuilder::new("my-agent-cli", "Agent-generated CLI");
//!
//! // 2. Register dynamic commands
//! cli.register_command("list", "List items", handle_list);
//! cli.register_command("show", "Show item details", handle_show);
//!
//! // 3. Build and execute
//! let command = cli.build()?;
//! command.run_with_args(vec!["my-agent-cli", "list"])?;
//! ```

use clap::{Command, Arg};
use std::collections::HashMap;
use thiserror::Error;
use std::sync::Arc;

/// Errors from agent CLI building
#[derive(Debug, Error)]
pub enum AgentBuilderError {
    /// Duplicate command identifier
    #[error("Duplicate command identifier: {0}")]
    DuplicateCommand(String),

    /// Invalid command name
    #[error("Invalid command name: {0}")]
    InvalidCommandName(String),

    /// No commands registered
    #[error("Cannot build CLI with no commands registered")]
    NoCommands,

    /// Handler execution failed
    #[error("Handler execution failed: {0}")]
    HandlerFailed(String),

    /// Validation failed
    #[error("Command validation failed: {0}")]
    ValidationFailed(String),
}

/// Result type for agent CLI operations
pub type AgentResult<T> = Result<T, AgentBuilderError>;

/// Command handler trait for agent-generated commands
///
/// Handlers are type-safe functions that process command arguments
/// and return structured results.
pub trait CommandHandler: Send + Sync {
    /// Execute the command with given arguments
    ///
    /// # Arguments
    ///
    /// * `args` - Parsed command arguments
    ///
    /// # Returns
    ///
    /// Command result as JSON value for serialization
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value>;

    /// Get command metadata
    fn metadata(&self) -> CommandMetadata;
}

/// Command metadata for help and discovery
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Command name
    pub name: String,

    /// Command description
    pub description: String,

    /// Expected arguments
    pub arguments: Vec<ArgumentSpec>,

    /// Whether command requires arguments
    pub requires_args: bool,
}

/// Specification for command arguments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSpec {
    /// Argument name
    pub name: String,

    /// Argument description
    pub description: String,

    /// Whether argument is required
    pub required: bool,

    /// Default value (if any)
    pub default: Option<String>,
}

/// Parsed command arguments
#[derive(Debug, Clone)]
pub struct CommandArgs {
    /// Named arguments
    pub values: HashMap<String, String>,

    /// Positional arguments
    pub positional: Vec<String>,
}

impl CommandArgs {
    /// Create new empty command arguments
    pub fn new() -> Self {
        Self { values: HashMap::new(), positional: Vec::new() }
    }

    /// Get named argument value
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|s| s.as_str())
    }

    /// Get first positional argument
    pub fn first_positional(&self) -> Option<&str> {
        self.positional.first().map(|s| s.as_str())
    }

    /// Add named argument
    pub fn with_arg(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(name.into(), value.into());
        self
    }

    /// Add positional argument
    pub fn with_positional(mut self, value: impl Into<String>) -> Self {
        self.positional.push(value.into());
        self
    }
}

impl Default for CommandArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic command wrapper
struct DynamicCommand {
    metadata: CommandMetadata,
    handler: Arc<dyn CommandHandler>,
}

/// Agent-optimized CLI builder
///
/// Enables agents to build CLIs programmatically without compile-time macros.
/// Provides type-safe command registration and execution.
///
/// # Example
///
/// ```rust,ignore
/// let mut builder = AgentCliBuilder::new("agent-cli", "CLI for agents");
/// builder.register_command("list", "List items", ListHandler::new());
/// let cli = builder.build()?;
/// ```
pub struct AgentCliBuilder {
    /// Application name
    name: String,

    /// Application description
    description: String,

    /// Registered commands
    commands: HashMap<String, DynamicCommand>,

    /// Version string
    version: Option<String>,
}

impl AgentCliBuilder {
    /// Create new agent CLI builder
    ///
    /// # Arguments
    ///
    /// * `name` - CLI application name
    /// * `description` - CLI description for help text
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = AgentCliBuilder::new("my-cli", "My CLI application");
    /// ```
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            commands: HashMap::new(),
            version: None,
        }
    }

    /// Set CLI version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Register a command handler
    ///
    /// # Arguments
    ///
    /// * `name` - Command name (must be unique)
    /// * `description` - Command description
    /// * `handler` - Command handler implementation
    ///
    /// # Errors
    ///
    /// Returns error if command name is duplicate or invalid
    pub fn register_command(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: Arc<dyn CommandHandler>,
    ) -> AgentResult<&mut Self> {
        let name_str = name.into();

        // Validate command name
        if name_str.is_empty() {
            return Err(AgentBuilderError::InvalidCommandName(name_str));
        }

        // Check for duplicates
        if self.commands.contains_key(&name_str) {
            return Err(AgentBuilderError::DuplicateCommand(name_str));
        }

        let metadata = CommandMetadata {
            name: name_str.clone(),
            description: description.into(),
            arguments: Vec::new(),
            requires_args: false,
        };

        self.commands.insert(
            name_str,
            DynamicCommand {
                metadata,
                handler,
            },
        );

        Ok(self)
    }

    /// Build the CLI application
    ///
    /// # Returns
    ///
    /// AgentCli ready for execution
    ///
    /// # Errors
    ///
    /// Returns error if no commands registered or validation fails
    pub fn build(self) -> AgentResult<AgentCli> {
        if self.commands.is_empty() {
            return Err(AgentBuilderError::NoCommands);
        }

        Ok(AgentCli {
            name: self.name,
            description: self.description,
            version: self.version,
            commands: self.commands,
        })
    }

    /// Get number of registered commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// List all registered command names
    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
}

/// Built agent CLI application
///
/// Represents a complete, validated CLI ready for execution.
pub struct AgentCli {
    /// Application name
    name: String,

    /// Application description
    description: String,

    /// Version string (optional)
    version: Option<String>,

    /// Registered commands
    commands: HashMap<String, DynamicCommand>,
}

impl AgentCli {
    /// Get CLI name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get CLI description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get all registered command names
    pub fn commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Get command metadata by name
    pub fn command_info(&self, name: &str) -> Option<CommandMetadata> {
        self.commands.get(name).map(|cmd| cmd.metadata.clone())
    }

    /// Execute a command by name with arguments
    ///
    /// # Arguments
    ///
    /// * `name` - Command name
    /// * `args` - Command arguments
    ///
    /// # Returns
    ///
    /// JSON value representing command output
    pub fn execute(&self, name: &str, args: CommandArgs) -> AgentResult<serde_json::Value> {
        let command = self.commands.get(name)
            .ok_or_else(|| AgentBuilderError::HandlerFailed(
                format!("Command not found: {}", name)
            ))?;

        command.handler.execute(&args)
    }

    /// Build clap Command for CLI integration
    ///
    /// This creates a clap-compatible Command structure for standard
    /// argument parsing and help generation.
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new(&self.name)
            .about(self.description.as_str());

        if let Some(version) = &self.version {
            cmd = cmd.version(version.as_str());
        }

        // Add subcommand for each registered command
        for (name, cmd_def) in &self.commands {
            let subcommand = Command::new(name)
                .about(cmd_def.metadata.description.as_str());

            // Add arguments from metadata
            let mut subcommand = subcommand;
            for arg_spec in &cmd_def.metadata.arguments {
                let arg = Arg::new(arg_spec.name.as_str())
                    .help(arg_spec.description.as_str())
                    .required(arg_spec.required);

                subcommand = subcommand.arg(arg);
            }

            cmd = cmd.subcommand(subcommand);
        }

        cmd
    }

    /// Run CLI with provided arguments
    ///
    /// # Arguments
    ///
    /// * `args` - Command line arguments (e.g., from `std::env::args()`)
    ///
    /// # Returns
    ///
    /// JSON output from executed command
    pub fn run_with_args(&self, args: Vec<String>) -> AgentResult<serde_json::Value> {
        if args.len() < 2 {
            return Err(AgentBuilderError::ValidationFailed(
                "No command specified".to_string()
            ));
        }

        let command_name = &args[1];
        let cmd_args = CommandArgs::new();

        self.execute(command_name, cmd_args)
    }

    /// Get help text for all commands
    pub fn help(&self) -> String {
        let mut help = format!("{} - {}\n\n", self.name, self.description);
        help.push_str("Commands:\n");

        for (name, cmd) in &self.commands {
            help.push_str(&format!("  {}  {}\n", name, cmd.metadata.description));
        }

        help
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test command handler implementation
    struct TestHandler {
        metadata: CommandMetadata,
    }

    impl TestHandler {
        fn new(name: &str, description: &str) -> Arc<Self> {
            Arc::new(Self {
                metadata: CommandMetadata {
                    name: name.to_string(),
                    description: description.to_string(),
                    arguments: Vec::new(),
                    requires_args: false,
                },
            })
        }
    }

    impl CommandHandler for TestHandler {
        fn execute(&self, _args: &CommandArgs) -> AgentResult<serde_json::Value> {
            Ok(serde_json::json!({
                "status": "success",
                "message": "Test command executed"
            }))
        }

        fn metadata(&self) -> CommandMetadata {
            self.metadata.clone()
        }
    }

    #[test]
    fn test_builder_new() {
        // Arrange & Act
        let builder = AgentCliBuilder::new("test-cli", "Test CLI");

        // Assert
        assert_eq!(builder.name, "test-cli");
        assert_eq!(builder.description, "Test CLI");
        assert_eq!(builder.command_count(), 0);
    }

    #[test]
    fn test_builder_version() {
        // Arrange & Act
        let builder = AgentCliBuilder::new("test", "Test").version("1.0.0");

        // Assert
        assert_eq!(builder.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_register_command_success() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("list", "List items");

        // Act
        let result = builder.register_command("list", "List items", handler);

        // Assert
        assert!(result.is_ok());
        assert_eq!(builder.command_count(), 1);
        assert!(builder.list_commands().contains(&"list".to_string()));
    }

    #[test]
    fn test_register_command_duplicate() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("list", "List items");
        builder.register_command("list", "List items", handler.clone()).ok();

        // Act
        let result = builder.register_command("list", "List items again", handler);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentBuilderError::DuplicateCommand(_)));
    }

    #[test]
    fn test_register_command_empty_name() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("test", "Test");

        // Act
        let result = builder.register_command("", "Empty name", handler);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentBuilderError::InvalidCommandName(_)));
    }

    #[test]
    fn test_build_no_commands() {
        // Arrange
        let builder = AgentCliBuilder::new("test", "Test");

        // Act
        let result = builder.build();

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentBuilderError::NoCommands));
    }

    #[test]
    fn test_build_success() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("list", "List items");
        builder.register_command("list", "List items", handler).ok();

        // Act
        let result = builder.build();

        // Assert
        assert!(result.is_ok());
        let cli = result.unwrap();
        assert_eq!(cli.name(), "test");
        assert_eq!(cli.commands().len(), 1);
    }

    #[test]
    fn test_command_args_get() {
        // Arrange
        let args = CommandArgs::new().with_arg("name", "value");

        // Act
        let result = args.get("name");

        // Assert
        assert_eq!(result, Some("value"));
    }

    #[test]
    fn test_command_args_missing() {
        // Arrange
        let args = CommandArgs::new();

        // Act
        let result = args.get("missing");

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn test_execute_command() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("test", "Test command");
        builder.register_command("test", "Test command", handler).ok();
        let cli = builder.build().unwrap();

        // Act
        let result = cli.execute("test", CommandArgs::new());

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output["status"], "success");
    }

    #[test]
    fn test_execute_nonexistent_command() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("list", "List");
        builder.register_command("list", "List", handler).ok();
        let cli = builder.build().unwrap();

        // Act
        let result = cli.execute("missing", CommandArgs::new());

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_help() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test CLI");
        let handler = TestHandler::new("list", "List items");
        builder.register_command("list", "List items", handler).ok();
        let cli = builder.build().unwrap();

        // Act
        let help = cli.help();

        // Assert
        assert!(help.contains("test"));
        assert!(help.contains("Test CLI"));
        assert!(help.contains("list"));
    }

    #[test]
    fn test_command_info() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let handler = TestHandler::new("list", "List items");
        builder.register_command("list", "List items", handler).ok();
        let cli = builder.build().unwrap();

        // Act
        let info = cli.command_info("list");

        // Assert
        assert!(info.is_some());
        let metadata = info.unwrap();
        assert_eq!(metadata.name, "list");
        assert_eq!(metadata.description, "List items");
    }

    #[test]
    fn test_multiple_commands() {
        // Arrange
        let mut builder = AgentCliBuilder::new("test", "Test");
        let h1 = TestHandler::new("list", "List");
        let h2 = TestHandler::new("show", "Show");
        let h3 = TestHandler::new("delete", "Delete");

        // Act
        builder.register_command("list", "List", h1).ok();
        builder.register_command("show", "Show", h2).ok();
        builder.register_command("delete", "Delete", h3).ok();
        let cli = builder.build();

        // Assert
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.commands().len(), 3);
        assert!(cli.commands().contains(&"list"));
        assert!(cli.commands().contains(&"show"));
        assert!(cli.commands().contains(&"delete"));
    }
}
