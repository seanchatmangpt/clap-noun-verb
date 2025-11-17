//! Input/output schemas and capability composition for swarm-native CLIs
//!
//! Enables agents to understand data flow between commands and compose
//! multi-step workflows by matching outputs to inputs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema for a data type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TypeSchema {
    /// Primitive type (string, number, boolean, null)
    Primitive { primitive_type: PrimitiveType },
    /// Array of items
    Array { items: Box<TypeSchema> },
    /// Object with properties
    Object { properties: HashMap<String, TypeSchema> },
    /// Reference to a named type
    Reference { type_ref: String },
    /// Union of multiple types
    Union { options: Vec<TypeSchema> },
}

impl TypeSchema {
    /// Create a primitive type schema
    pub fn primitive(primitive_type: PrimitiveType) -> Self {
        TypeSchema::Primitive { primitive_type }
    }

    /// Create a string type schema
    pub fn string() -> Self {
        TypeSchema::primitive(PrimitiveType::String)
    }

    /// Create a number type schema
    pub fn number() -> Self {
        TypeSchema::primitive(PrimitiveType::Number)
    }

    /// Create a boolean type schema
    pub fn boolean() -> Self {
        TypeSchema::primitive(PrimitiveType::Boolean)
    }

    /// Create an array type schema
    pub fn array(items: TypeSchema) -> Self {
        TypeSchema::Array { items: Box::new(items) }
    }

    /// Create an object type schema
    pub fn object(properties: HashMap<String, TypeSchema>) -> Self {
        TypeSchema::Object { properties }
    }

    /// Create a type reference
    pub fn reference(type_ref: impl Into<String>) -> Self {
        TypeSchema::Reference { type_ref: type_ref.into() }
    }

    /// Create a union type schema
    pub fn union(options: Vec<TypeSchema>) -> Self {
        TypeSchema::Union { options }
    }

    /// Check if this schema is compatible with another (for output -> input matching)
    pub fn is_compatible_with(&self, other: &TypeSchema) -> bool {
        match (self, other) {
            (TypeSchema::Primitive { primitive_type: a }, TypeSchema::Primitive { primitive_type: b }) => {
                a == b
            }
            (TypeSchema::Array { items: a }, TypeSchema::Array { items: b }) => {
                a.is_compatible_with(b)
            }
            (TypeSchema::Reference { type_ref: a }, TypeSchema::Reference { type_ref: b }) => a == b,
            (TypeSchema::Union { options }, other) => {
                options.iter().any(|opt| opt.is_compatible_with(other))
            }
            (other, TypeSchema::Union { options }) => {
                options.iter().any(|opt| other.is_compatible_with(opt))
            }
            _ => false,
        }
    }
}

/// Primitive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveType {
    String,
    Number,
    Boolean,
    Null,
}

/// Input schema for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSchema {
    /// Required inputs
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub required: HashMap<String, TypeSchema>,
    /// Optional inputs
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub optional: HashMap<String, TypeSchema>,
    /// Whether this command can accept piped input
    pub accepts_stdin: bool,
    /// Schema for stdin input if accepted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_schema: Option<TypeSchema>,
}

impl InputSchema {
    /// Create a new input schema
    pub fn new() -> Self {
        Self {
            required: HashMap::new(),
            optional: HashMap::new(),
            accepts_stdin: false,
            stdin_schema: None,
        }
    }

    /// Add a required input
    pub fn with_required(mut self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.required.insert(name.into(), schema);
        self
    }

    /// Add an optional input
    pub fn with_optional(mut self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.optional.insert(name.into(), schema);
        self
    }

    /// Mark as accepting stdin with schema
    pub fn with_stdin(mut self, schema: TypeSchema) -> Self {
        self.accepts_stdin = true;
        self.stdin_schema = Some(schema);
        self
    }
}

impl Default for InputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Output schema for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    /// Success output schema
    pub success: TypeSchema,
    /// Error output schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TypeSchema>,
    /// Whether this command outputs to stdout
    pub outputs_stdout: bool,
    /// Named outputs (for multiple output channels)
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub named_outputs: HashMap<String, TypeSchema>,
}

impl OutputSchema {
    /// Create a new output schema
    pub fn new(success: TypeSchema) -> Self {
        Self {
            success,
            error: None,
            outputs_stdout: true,
            named_outputs: HashMap::new(),
        }
    }

    /// Set error schema
    pub fn with_error(mut self, error: TypeSchema) -> Self {
        self.error = Some(error);
        self
    }

    /// Disable stdout output
    pub fn no_stdout(mut self) -> Self {
        self.outputs_stdout = false;
        self
    }

    /// Add a named output
    pub fn with_named_output(mut self, name: impl Into<String>, schema: TypeSchema) -> Self {
        self.named_outputs.insert(name.into(), schema);
        self
    }

    /// Get the primary output schema (success or first named output)
    pub fn primary_output(&self) -> &TypeSchema {
        &self.success
    }
}

/// Resource that a command consumes or produces
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type (e.g., "file", "network", "database")
    pub resource_type: String,
    /// Resource identifier or pattern
    pub identifier: String,
    /// Schema for the resource data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<TypeSchema>,
}

impl Resource {
    /// Create a new resource
    pub fn new(resource_type: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            identifier: identifier.into(),
            schema: None,
        }
    }

    /// Set schema
    pub fn with_schema(mut self, schema: TypeSchema) -> Self {
        self.schema = Some(schema);
        self
    }
}

/// Capability composition metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionMetadata {
    /// Input schema
    pub inputs: InputSchema,
    /// Output schema
    pub outputs: OutputSchema,
    /// Resources this command consumes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub consumes: Vec<Resource>,
    /// Resources this command produces
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub produces: Vec<Resource>,
}

impl CompositionMetadata {
    /// Create new composition metadata
    pub fn new(inputs: InputSchema, outputs: OutputSchema) -> Self {
        Self {
            inputs,
            outputs,
            consumes: Vec::new(),
            produces: Vec::new(),
        }
    }

    /// Add a consumed resource
    pub fn consumes(mut self, resource: Resource) -> Self {
        self.consumes.push(resource);
        self
    }

    /// Add a produced resource
    pub fn produces(mut self, resource: Resource) -> Self {
        self.produces.push(resource);
        self
    }

    /// Check if this command's output is compatible with another command's input
    pub fn can_pipe_to(&self, other: &CompositionMetadata) -> bool {
        if !self.outputs.outputs_stdout || !other.inputs.accepts_stdin {
            return false;
        }

        if let Some(ref stdin_schema) = other.inputs.stdin_schema {
            return self.outputs.success.is_compatible_with(stdin_schema);
        }

        false
    }

    /// Find matching resources between this command's products and another's consumers
    pub fn find_matching_resources(&self, other: &CompositionMetadata) -> Vec<(Resource, Resource)> {
        let mut matches = Vec::new();

        for produced in &self.produces {
            for consumed in &other.consumes {
                if produced.resource_type == consumed.resource_type {
                    // Could add more sophisticated matching here
                    matches.push((produced.clone(), consumed.clone()));
                }
            }
        }

        matches
    }
}

/// Equivalence class for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceClass {
    /// Class identifier
    pub class_id: String,
    /// Commands in this equivalence class
    pub commands: Vec<CommandReference>,
    /// Relationship type
    pub relationship: EquivalenceRelationship,
}

impl EquivalenceClass {
    /// Create a new equivalence class
    pub fn new(class_id: impl Into<String>, relationship: EquivalenceRelationship) -> Self {
        Self {
            class_id: class_id.into(),
            commands: Vec::new(),
            relationship,
        }
    }

    /// Add a command to the class
    pub fn add_command(mut self, command: CommandReference) -> Self {
        self.commands.push(command);
        self
    }
}

/// Reference to a command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandReference {
    /// CLI identifier (optional, for cross-CLI references)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    /// Noun
    pub noun: String,
    /// Verb
    pub verb: String,
    /// Capability ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
}

impl CommandReference {
    /// Create a new command reference
    pub fn new(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self {
            cli: None,
            noun: noun.into(),
            verb: verb.into(),
            capability_id: None,
        }
    }

    /// Set CLI
    pub fn with_cli(mut self, cli: impl Into<String>) -> Self {
        self.cli = Some(cli.into());
        self
    }

    /// Set capability ID
    pub fn with_capability_id(mut self, id: impl Into<String>) -> Self {
        self.capability_id = Some(id.into());
        self
    }

    /// Get fully qualified name
    pub fn fqn(&self) -> String {
        if let Some(ref cli) = self.cli {
            format!("{}::{}.{}", cli, self.noun, self.verb)
        } else {
            format!("{}.{}", self.noun, self.verb)
        }
    }
}

/// Equivalence relationship between commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquivalenceRelationship {
    /// Commands are functionally equivalent
    Equivalent,
    /// One command is a superset of another
    Superset,
    /// One command is a subset of another
    Subset,
    /// Commands are similar but not equivalent
    Similar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_schema_compatibility() {
        let string_schema = TypeSchema::string();
        let number_schema = TypeSchema::number();

        assert!(string_schema.is_compatible_with(&TypeSchema::string()));
        assert!(!string_schema.is_compatible_with(&number_schema));

        let union = TypeSchema::union(vec![TypeSchema::string(), TypeSchema::number()]);
        assert!(union.is_compatible_with(&string_schema));
        assert!(union.is_compatible_with(&number_schema));
    }

    #[test]
    fn test_composition_metadata_piping() {
        let cmd1_outputs = OutputSchema::new(TypeSchema::string());
        let cmd1 = CompositionMetadata::new(InputSchema::new(), cmd1_outputs);

        let cmd2_inputs = InputSchema::new().with_stdin(TypeSchema::string());
        let cmd2 = CompositionMetadata::new(cmd2_inputs, OutputSchema::new(TypeSchema::string()));

        assert!(cmd1.can_pipe_to(&cmd2));
    }

    #[test]
    fn test_command_reference() {
        let cmd_ref = CommandReference::new("services", "status")
            .with_cli("myapp")
            .with_capability_id("cap_123");

        assert_eq!(cmd_ref.fqn(), "myapp::services.status");
        assert_eq!(cmd_ref.capability_id, Some("cap_123".to_string()));
    }
}
