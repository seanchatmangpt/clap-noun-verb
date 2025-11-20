//! CommandBuilder for Constructing Noun-Verb Commands
//!
//! Provides a builder pattern for constructing noun-verb CLI commands with
//! fluent API and compile-time safety.

use serde_json::{json, Value};

/// Builder for constructing noun-verb commands
#[derive(Debug, Clone)]
pub struct CommandBuilder {
    noun: String,
    verb: String,
    args: Vec<String>,
    options: Value,
    metadata: Value,
}

impl CommandBuilder {
    /// Create a new command builder
    pub fn new(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self {
            noun: noun.into(),
            verb: verb.into(),
            args: Vec::new(),
            options: json!({}),
            metadata: json!({}),
        }
    }

    /// Add a positional argument
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple positional arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args.extend(args);
        self
    }

    /// Set an option value
    pub fn with_option(mut self, key: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut obj) = self.options {
            obj.insert(key.into(), value);
        }
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut obj) = self.metadata {
            obj.insert(key.into(), value);
        }
        self
    }

    /// Build the command
    pub fn build(self) -> Command {
        Command {
            noun: self.noun,
            verb: self.verb,
            args: self.args,
            options: self.options,
            metadata: self.metadata,
        }
    }

    /// Get the noun
    pub fn noun(&self) -> &str {
        &self.noun
    }

    /// Get the verb
    pub fn verb(&self) -> &str {
        &self.verb
    }

    /// Get the arguments
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Built command ready for execution
#[derive(Debug, Clone)]
pub struct Command {
    pub noun: String,
    pub verb: String,
    pub args: Vec<String>,
    pub options: Value,
    pub metadata: Value,
}

impl Command {
    /// Get command as JSON representation
    pub fn to_json(&self) -> Value {
        json!({
            "noun": self.noun,
            "verb": self.verb,
            "args": self.args,
            "options": self.options,
            "metadata": self.metadata
        })
    }

    /// Get command as command line string
    pub fn to_cli_string(&self) -> String {
        let mut cmd = format!("{} {}", self.noun, self.verb);
        for arg in &self.args {
            cmd.push(' ');
            cmd.push_str(arg);
        }
        cmd
    }

    /// Validate command
    pub fn validate(&self) -> Result<(), String> {
        if self.noun.is_empty() {
            return Err("Noun cannot be empty".to_string());
        }
        if self.verb.is_empty() {
            return Err("Verb cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_builder_basic() {
        let cmd = CommandBuilder::new("user", "create")
            .with_arg("alice")
            .build();

        assert_eq!(cmd.noun, "user");
        assert_eq!(cmd.verb, "create");
        assert_eq!(cmd.args, vec!["alice"]);
    }

    #[test]
    fn test_command_builder_fluent() {
        let cmd = CommandBuilder::new("product", "update")
            .with_arg("123")
            .with_arg("--name")
            .with_arg("Widget")
            .with_option("force", json!(true))
            .with_metadata("timestamp", json!("2024-11-20"))
            .build();

        assert_eq!(cmd.noun, "product");
        assert_eq!(cmd.verb, "update");
        assert_eq!(cmd.args.len(), 3);
        assert_eq!(cmd.options["force"], true);
        assert_eq!(cmd.metadata["timestamp"], "2024-11-20");
    }

    #[test]
    fn test_command_builder_multiple_args() {
        let args = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];
        let cmd = CommandBuilder::new("order", "list").with_args(args).build();

        assert_eq!(cmd.args.len(), 3);
    }

    #[test]
    fn test_command_to_json() {
        let cmd = CommandBuilder::new("user", "delete")
            .with_arg("123")
            .with_option("force", json!(true))
            .build();

        let json = cmd.to_json();
        assert_eq!(json["noun"], "user");
        assert_eq!(json["verb"], "delete");
        assert_eq!(json["args"][0], "123");
        assert_eq!(json["options"]["force"], true);
    }

    #[test]
    fn test_command_to_cli_string() {
        let cmd = CommandBuilder::new("user", "create")
            .with_arg("alice")
            .with_arg("--email")
            .with_arg("alice@example.com")
            .build();

        let cli = cmd.to_cli_string();
        assert!(cli.contains("user create"));
        assert!(cli.contains("alice"));
        assert!(cli.contains("alice@example.com"));
    }

    #[test]
    fn test_command_validate() {
        let cmd = CommandBuilder::new("user", "create").build();
        assert!(cmd.validate().is_ok());

        let cmd = CommandBuilder::new("", "create").build();
        assert!(cmd.validate().is_err());

        let cmd = CommandBuilder::new("user", "").build();
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_command_builder_getters() {
        let builder = CommandBuilder::new("service", "list")
            .with_arg("--filter")
            .with_arg("active");

        assert_eq!(builder.noun(), "service");
        assert_eq!(builder.verb(), "list");
        assert_eq!(builder.args(), &["--filter".to_string(), "active".to_string()]);
    }

    #[test]
    fn test_command_builder_multiple_options() {
        let cmd = CommandBuilder::new("resource", "execute")
            .with_option("timeout", json!(5000))
            .with_option("retry", json!(3))
            .with_option("verbose", json!(true))
            .build();

        assert_eq!(cmd.options["timeout"], 5000);
        assert_eq!(cmd.options["retry"], 3);
        assert_eq!(cmd.options["verbose"], true);
    }
}
