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
//! use clap_noun_verb::agent_cli::AgentCliBuilder;
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

// Define agent_builder module (semantic module version not yet available)
pub mod agent_builder {
    use clap::{Arg, Command};
    use std::collections::HashMap;
    use std::sync::Arc;
    use thiserror::Error;

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
    pub trait CommandHandler: Send + Sync {
        /// Execute the command with given arguments
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

        /// Check if a named argument exists
        pub fn contains(&self, name: &str) -> bool {
            self.values.contains_key(name)
        }

        /// Get first positional argument
        pub fn first_positional(&self) -> Option<&str> {
            self.positional.first().map(|s| s.as_str())
        }

        /// Get all positional arguments
        pub fn get_all_positional(&self) -> &[String] {
            &self.positional
        }

        /// Get total count of all arguments (named + positional)
        pub fn len(&self) -> usize {
            self.values.len() + self.positional.len()
        }

        /// Check if no arguments are present
        pub fn is_empty(&self) -> bool {
            self.values.is_empty() && self.positional.is_empty()
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

    impl std::fmt::Debug for DynamicCommand {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DynamicCommand")
                .field("metadata", &self.metadata)
                .field("handler", &"<trait object>")
                .finish()
        }
    }

    /// Agent-optimized CLI builder
    #[derive(Debug)]
    pub struct AgentCliBuilder {
        /// Application name
        name: String,

        /// Application description
        description: String,

        /// Registered commands
        commands: HashMap<String, DynamicCommand>,

        /// Version string
        pub version: Option<String>,
    }

    impl AgentCliBuilder {
        /// Create new agent CLI builder
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
        pub fn register_command(
            &mut self,
            name: impl Into<String>,
            description: impl Into<String>,
            handler: Arc<dyn CommandHandler>,
        ) -> AgentResult<&mut Self> {
            let name_str = name.into();

            if name_str.is_empty() {
                return Err(AgentBuilderError::InvalidCommandName(name_str));
            }

            if self.commands.contains_key(&name_str) {
                return Err(AgentBuilderError::DuplicateCommand(name_str));
            }

            let metadata = CommandMetadata {
                name: name_str.clone(),
                description: description.into(),
                arguments: Vec::new(),
                requires_args: false,
            };

            self.commands.insert(name_str, DynamicCommand { metadata, handler });

            Ok(self)
        }

        /// Register multiple command handlers in batch
        ///
        /// # Arguments
        ///
        /// Takes an iterator of tuples: (name, description, handler)
        ///
        /// # Returns
        ///
        /// Returns error on the first duplicate or invalid name encountered
        pub fn register_commands<I>(&mut self, commands: I) -> AgentResult<&mut Self>
        where
            I: IntoIterator<Item = (String, String, Arc<dyn CommandHandler>)>,
        {
            for (name, description, handler) in commands {
                self.register_command(name, description, handler)?;
            }
            Ok(self)
        }

        /// Build the CLI application
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
    #[derive(Debug)]
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
        pub fn execute(&self, name: &str, args: CommandArgs) -> AgentResult<serde_json::Value> {
            let command = self.commands.get(name).ok_or_else(|| {
                AgentBuilderError::HandlerFailed(format!("Command not found: {}", name))
            })?;

            command.handler.execute(&args)
        }

        /// Build clap Command for CLI integration
        pub fn build_command(&self) -> Command {
            // Use Box::leak to convert owned strings to 'static references
            // This is acceptable for CLI construction (happens once per application)
            let name: &'static str = Box::leak(self.name.clone().into_boxed_str());
            let desc: &'static str = Box::leak(self.description.clone().into_boxed_str());

            let mut cmd = Command::new(name).about(desc);

            if let Some(version) = &self.version {
                let version_str: &'static str = Box::leak(version.clone().into_boxed_str());
                cmd = cmd.version(version_str);
            }

            for (name_key, cmd_def) in &self.commands {
                let cmd_name: &'static str = Box::leak(name_key.clone().into_boxed_str());
                let cmd_desc: &'static str =
                    Box::leak(cmd_def.metadata.description.clone().into_boxed_str());

                let subcommand = Command::new(cmd_name).about(cmd_desc);

                let mut subcommand = subcommand;
                for arg_spec in &cmd_def.metadata.arguments {
                    let arg_name: &'static str = Box::leak(arg_spec.name.clone().into_boxed_str());
                    let arg_help: &'static str =
                        Box::leak(arg_spec.description.clone().into_boxed_str());

                    let arg = Arg::new(arg_name).help(arg_help).required(arg_spec.required);

                    subcommand = subcommand.arg(arg);
                }

                cmd = cmd.subcommand(subcommand);
            }

            cmd
        }

        /// Run CLI with provided arguments
        pub fn run_with_args(&self, args: Vec<String>) -> AgentResult<serde_json::Value> {
            if args.len() < 2 {
                return Err(AgentBuilderError::ValidationFailed(
                    "No command specified".to_string(),
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
}

pub use agent_builder::*;
