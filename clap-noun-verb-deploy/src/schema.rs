use crate::Invocation;
use clap_noun_verb::{Arg, ArgAction, Command};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use thiserror::Error;

/// JSON-facing argument shape inferred from Clap metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentKind {
    String,
    Boolean,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSchema {
    pub name: String,
    pub about: Option<String>,
    pub commands: Vec<CommandSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSchema {
    pub path: Vec<String>,
    pub about: Option<String>,
    pub arguments: Vec<ArgumentSchema>,
    pub callable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentSchema {
    pub id: String,
    pub long: Option<String>,
    pub short: Option<char>,
    pub required: bool,
    pub positional: bool,
    pub kind: ArgumentKind,
}

/// Protocol-neutral tool description derived from one callable command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvocationBuildError {
    #[error("unknown tool '{0}'")]
    UnknownTool(String),
    #[error("tool '{0}' is not callable")]
    NotCallable(String),
    #[error("missing required argument '{argument}' for tool '{tool}'")]
    MissingRequired { tool: String, argument: String },
    #[error("unknown argument '{argument}' for tool '{tool}'")]
    UnknownArgument { tool: String, argument: String },
    #[error("argument '{argument}' for tool '{tool}' expected {expected}")]
    InvalidType {
        tool: String,
        argument: String,
        expected: &'static str,
    },
}

impl CliSchema {
    #[must_use]
    pub fn from_command(command: &Command) -> Self {
        let mut commands = Vec::new();
        collect(command, &mut Vec::new(), &mut commands);
        Self {
            name: command.get_name().to_owned(),
            about: command.get_about().map(ToString::to_string),
            commands,
        }
    }

    /// Return only callable leaf commands as protocol-neutral tools.
    #[must_use]
    pub fn tools(&self) -> Vec<ToolSchema> {
        self.commands
            .iter()
            .filter(|command| command.callable)
            .map(CommandSchema::as_tool)
            .collect()
    }

    /// Construct argv from a tool name and validated JSON arguments.
    pub fn build_invocation(
        &self,
        tool_name: &str,
        arguments: &Map<String, Value>,
    ) -> Result<Invocation, InvocationBuildError> {
        let command = self
            .commands
            .iter()
            .find(|command| command.tool_name() == tool_name)
            .ok_or_else(|| InvocationBuildError::UnknownTool(tool_name.to_owned()))?;

        if !command.callable {
            return Err(InvocationBuildError::NotCallable(tool_name.to_owned()));
        }

        command.build_invocation(arguments)
    }
}

impl CommandSchema {
    #[must_use]
    pub fn tool_name(&self) -> String {
        self.path.join("__")
    }

    #[must_use]
    pub fn as_tool(&self) -> ToolSchema {
        let properties = self
            .arguments
            .iter()
            .map(|argument| (argument.external_name().to_owned(), argument.json_schema()))
            .collect::<Map<_, _>>();
        let required = self
            .arguments
            .iter()
            .filter(|argument| argument.required)
            .map(|argument| Value::String(argument.external_name().to_owned()))
            .collect::<Vec<_>>();

        ToolSchema {
            name: self.tool_name(),
            description: self.about.clone(),
            input_schema: json!({
                "type": "object",
                "properties": properties,
                "required": required,
                "additionalProperties": false
            }),
        }
    }

    fn build_invocation(
        &self,
        arguments: &Map<String, Value>,
    ) -> Result<Invocation, InvocationBuildError> {
        let tool = self.tool_name();
        let admitted = self
            .arguments
            .iter()
            .map(ArgumentSchema::external_name)
            .collect::<BTreeSet<_>>();

        for key in arguments.keys() {
            if !admitted.contains(key.as_str()) {
                return Err(InvocationBuildError::UnknownArgument {
                    tool: tool.clone(),
                    argument: key.clone(),
                });
            }
        }

        let mut argv = self.path.clone();
        for argument in &self.arguments {
            let key = argument.external_name();
            let value = arguments.get(key);
            if argument.required && value.is_none() {
                return Err(InvocationBuildError::MissingRequired {
                    tool: tool.clone(),
                    argument: key.to_owned(),
                });
            }
            let Some(value) = value else {
                continue;
            };
            argument.push_argv(&tool, value, &mut argv)?;
        }
        Ok(Invocation::new(argv))
    }
}

impl ArgumentSchema {
    fn external_name(&self) -> &str {
        self.long.as_deref().unwrap_or(&self.id)
    }

    fn json_schema(&self) -> Value {
        match self.kind {
            ArgumentKind::String => json!({"type": "string"}),
            ArgumentKind::Boolean => json!({"type": "boolean"}),
            ArgumentKind::Array => json!({"type": "array", "items": {"type": "string"}}),
        }
    }

    fn push_argv(
        &self,
        tool: &str,
        value: &Value,
        argv: &mut Vec<String>,
    ) -> Result<(), InvocationBuildError> {
        match self.kind {
            ArgumentKind::Boolean => {
                let Some(enabled) = value.as_bool() else {
                    return Err(self.invalid_type(tool, "boolean"));
                };
                if enabled {
                    self.push_switch(argv);
                }
            }
            ArgumentKind::String => {
                let Some(text) = value.as_str() else {
                    return Err(self.invalid_type(tool, "string"));
                };
                self.push_value(argv, text);
            }
            ArgumentKind::Array => {
                let Some(values) = value.as_array() else {
                    return Err(self.invalid_type(tool, "array of strings"));
                };
                for value in values {
                    let Some(text) = value.as_str() else {
                        return Err(self.invalid_type(tool, "array of strings"));
                    };
                    self.push_value(argv, text);
                }
            }
        }
        Ok(())
    }

    fn push_switch(&self, argv: &mut Vec<String>) {
        if let Some(long) = &self.long {
            argv.push(format!("--{long}"));
        } else if let Some(short) = self.short {
            argv.push(format!("-{short}"));
        }
    }

    fn push_value(&self, argv: &mut Vec<String>, value: &str) {
        if self.positional {
            argv.push(value.to_owned());
            return;
        }
        if let Some(long) = &self.long {
            argv.push(format!("--{long}"));
        } else if let Some(short) = self.short {
            argv.push(format!("-{short}"));
        }
        argv.push(value.to_owned());
    }

    fn invalid_type(&self, tool: &str, expected: &'static str) -> InvocationBuildError {
        InvocationBuildError::InvalidType {
            tool: tool.to_owned(),
            argument: self.external_name().to_owned(),
            expected,
        }
    }
}

fn collect(command: &Command, parent: &mut Vec<String>, output: &mut Vec<CommandSchema>) {
    for subcommand in command.get_subcommands() {
        parent.push(subcommand.get_name().to_owned());
        let has_children = subcommand.get_subcommands().next().is_some();
        let arguments = subcommand
            .get_arguments()
            .filter(|argument| argument.get_id().as_str() != "introspect")
            .map(argument_schema)
            .collect();
        output.push(CommandSchema {
            path: parent.clone(),
            about: subcommand.get_about().map(ToString::to_string),
            arguments,
            callable: !has_children,
        });
        collect(subcommand, parent, output);
        parent.pop();
    }
}

fn argument_schema(argument: &Arg) -> ArgumentSchema {
    let action = argument.get_action();
    let kind = if matches!(action, ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count) {
        ArgumentKind::Boolean
    } else if matches!(action, ArgAction::Append) {
        ArgumentKind::Array
    } else {
        ArgumentKind::String
    };

    ArgumentSchema {
        id: argument.get_id().to_string(),
        long: argument.get_long().map(str::to_owned),
        short: argument.get_short(),
        required: argument.is_required_set(),
        positional: argument.get_long().is_none() && argument.get_short().is_none(),
        kind,
    }
}
