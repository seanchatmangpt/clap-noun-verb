//! Introspection capabilities for CLI commands

use super::effects::EffectMetadata;
use super::guards::GuardConfig;
use super::planes::PlaneInteraction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command capabilities for introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCapabilities {
    /// CLI version
    pub cli_version: String,
    /// Schema version for introspection
    pub schema_version: String,
    /// Supported features
    pub features: Vec<String>,
    /// Application metadata
    pub app: AppMetadata,
}

impl CommandCapabilities {
    /// Create new command capabilities
    pub fn new(app_name: impl Into<String>, cli_version: impl Into<String>) -> Self {
        Self {
            cli_version: cli_version.into(),
            schema_version: super::SCHEMA_VERSION.to_string(),
            features: super::SUPPORTED_FEATURES.iter().map(|s| s.to_string()).collect(),
            app: AppMetadata::new(app_name),
        }
    }

    /// Set application metadata
    pub fn with_app(mut self, app: AppMetadata) -> Self {
        self.app = app;
        self
    }
}

/// Application metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    /// Application name
    pub name: String,
    /// Application version
    pub version: Option<String>,
    /// Application description
    pub about: Option<String>,
}

impl AppMetadata {
    /// Create new application metadata
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), version: None, about: None }
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set description
    pub fn with_about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }
}

/// Introspection response for the entire CLI or specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    /// Schema version
    pub schema_version: String,
    /// CLI version
    pub cli_version: String,
    /// Application metadata
    pub app: AppMetadata,
    /// Available nouns
    pub nouns: Vec<NounMetadata>,
}

impl IntrospectionResponse {
    /// Create a new introspection response
    pub fn new(cli_version: impl Into<String>, app: AppMetadata) -> Self {
        Self {
            schema_version: super::SCHEMA_VERSION.to_string(),
            cli_version: cli_version.into(),
            app,
            nouns: Vec::new(),
        }
    }

    /// Add a noun
    pub fn with_noun(mut self, noun: NounMetadata) -> Self {
        self.nouns.push(noun);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Metadata for a noun command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NounMetadata {
    /// Noun name
    pub name: String,
    /// Description
    pub about: String,
    /// Available verbs
    pub verbs: Vec<VerbMetadata>,
    /// Sub-nouns (for nested command groups)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sub_nouns: Vec<NounMetadata>,
}

impl NounMetadata {
    /// Create new noun metadata
    pub fn new(name: impl Into<String>, about: impl Into<String>) -> Self {
        Self { name: name.into(), about: about.into(), verbs: Vec::new(), sub_nouns: Vec::new() }
    }

    /// Add a verb
    pub fn with_verb(mut self, verb: VerbMetadata) -> Self {
        self.verbs.push(verb);
        self
    }

    /// Add a sub-noun
    pub fn with_sub_noun(mut self, noun: NounMetadata) -> Self {
        self.sub_nouns.push(noun);
        self
    }
}

/// Metadata for a verb command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbMetadata {
    /// Verb name
    pub name: String,
    /// Description
    pub about: String,
    /// Full command metadata
    pub command: CommandMetadata,
}

impl VerbMetadata {
    /// Create new verb metadata
    pub fn new(name: impl Into<String>, about: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            about: about.into(),
            command: CommandMetadata::default(),
        }
    }

    /// Set command metadata
    pub fn with_command(mut self, command: CommandMetadata) -> Self {
        self.command = command;
        self
    }
}

/// Complete metadata for a command
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Arguments
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<ArgumentMetadata>,
    /// Effect metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<EffectMetadata>,
    /// Plane interactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planes: Option<PlaneInteraction>,
    /// Guard configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guards: Option<GuardConfig>,
    /// Output type description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_type: Option<String>,
    /// Preconditions (commands that should be run first)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
}

impl CommandMetadata {
    /// Create new command metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an argument
    pub fn with_argument(mut self, arg: ArgumentMetadata) -> Self {
        self.arguments.push(arg);
        self
    }

    /// Set effect metadata
    pub fn with_effects(mut self, effects: EffectMetadata) -> Self {
        self.effects = Some(effects);
        self
    }

    /// Set plane interactions
    pub fn with_planes(mut self, planes: PlaneInteraction) -> Self {
        self.planes = Some(planes);
        self
    }

    /// Set guard configuration
    pub fn with_guards(mut self, guards: GuardConfig) -> Self {
        self.guards = Some(guards);
        self
    }

    /// Set output type
    pub fn with_output_type(mut self, output_type: impl Into<String>) -> Self {
        self.output_type = Some(output_type.into());
        self
    }

    /// Add a precondition
    pub fn with_precondition(mut self, precondition: impl Into<String>) -> Self {
        self.preconditions.push(precondition.into());
        self
    }
}

/// Metadata for a command argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentMetadata {
    /// Argument name
    pub name: String,
    /// Argument type (e.g., "String", "u32", "bool")
    pub arg_type: String,
    /// Whether the argument is required
    pub required: bool,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Help text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Short flag (e.g., "-v")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    /// Long flag (e.g., "--verbose")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long: Option<String>,
    /// Environment variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Positional index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    /// Possible values
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub possible_values: Vec<String>,
    /// Argument group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl ArgumentMetadata {
    /// Create new argument metadata
    pub fn new(name: impl Into<String>, arg_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arg_type: arg_type.into(),
            required: false,
            default: None,
            help: None,
            short: None,
            long: None,
            env: None,
            index: None,
            possible_values: Vec::new(),
            group: None,
        }
    }

    /// Set as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Set short flag
    pub fn with_short(mut self, short: char) -> Self {
        self.short = Some(format!("-{}", short));
        self
    }

    /// Set long flag
    pub fn with_long(mut self, long: impl Into<String>) -> Self {
        self.long = Some(format!("--{}", long.into()));
        self
    }

    /// Set environment variable
    pub fn with_env(mut self, env: impl Into<String>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// Set positional index
    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Add a possible value
    pub fn with_possible_value(mut self, value: impl Into<String>) -> Self {
        self.possible_values.push(value.into());
        self
    }

    /// Set argument group
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// Command graph for visualizing dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGraph {
    /// Graph nodes (commands)
    pub nodes: Vec<GraphNode>,
    /// Graph edges (dependencies/relations)
    pub edges: Vec<GraphEdge>,
}

impl CommandGraph {
    /// Create a new command graph
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new() }
    }

    /// Add a node
    pub fn add_node(mut self, node: GraphNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Add an edge
    pub fn add_edge(mut self, edge: GraphEdge) -> Self {
        self.edges.push(edge);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for CommandGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Node in the command graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Node ID (e.g., "services.status")
    pub id: String,
    /// Effect types
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl GraphNode {
    /// Create a new graph node
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), effects: Vec::new(), metadata: HashMap::new() }
    }

    /// Add an effect
    pub fn with_effect(mut self, effect: impl Into<String>) -> Self {
        self.effects.push(effect.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }
}

/// Edge in the command graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Relationship type
    pub relation: String,
}

impl GraphEdge {
    /// Create a new graph edge
    pub fn new(from: impl Into<String>, to: impl Into<String>, relation: impl Into<String>) -> Self {
        Self { from: from.into(), to: to.into(), relation: relation.into() }
    }
}
