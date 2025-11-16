//! CNV Grammar Model
//!
//! Provides introspectable representation of the entire noun-verb command tree:
//! - Nouns (top-level groups)
//! - Verbs (actions)
//! - Arguments (positional and named)
//! - Metadata (help, deprecation, effects, etc.)
//!
//! # Design
//!
//! The grammar model is a structured, versioned representation that external
//! tools can inspect to:
//! - Generate completions
//! - Generate manpages
//! - Enumerate commands for testing
//! - Discover capabilities for agents
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::Grammar;
//!
//! let grammar = Grammar::extract()?;
//! println!("{}", serde_json::to_string_pretty(&grammar)?);
//!
//! // Query specific nouns/verbs
//! for noun in grammar.nouns() {
//!     println!("Noun: {}", noun.name());
//!     for verb in noun.verbs() {
//!         println!("  Verb: {}", verb.name());
//!     }
//! }
//! ```

use crate::cli::registry::{ArgMetadata, __NOUN_REGISTRY, __VERB_REGISTRY};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Grammar schema version
pub const GRAMMAR_SCHEMA_VERSION: &str = "1.0.0";

/// Argument type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentType {
    /// Positional argument
    Positional,
    /// Named argument (--name)
    Named,
    /// Flag (--flag, boolean)
    Flag,
    /// Count (--verbose, -vvv)
    Count,
}

/// Argument metadata in the grammar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarArgument {
    /// Argument name
    pub name: String,
    /// Short flag (if any)
    pub short: Option<char>,
    /// Long flag (if any)
    pub long: Option<String>,
    /// Argument type
    #[serde(rename = "type")]
    pub arg_type: ArgumentType,
    /// Help text
    pub help: Option<String>,
    /// Whether required
    pub required: bool,
    /// Default value (if any)
    pub default: Option<String>,
    /// Environment variable (if any)
    pub env: Option<String>,
    /// Value name for help (e.g., <FILE>)
    pub value_name: Option<String>,
    /// Possible values (for enums)
    pub possible_values: Option<Vec<String>>,
    /// Multiple values allowed
    pub multiple: bool,
    /// Argument group (if any)
    pub group: Option<String>,
    /// Arguments this requires
    pub requires: Vec<String>,
    /// Arguments this conflicts with
    pub conflicts_with: Vec<String>,
    /// Position index (for positional args)
    pub index: Option<usize>,
}

impl GrammarArgument {
    /// Create from CLI ArgMetadata
    pub fn from_metadata(meta: &ArgMetadata) -> Self {
        use clap::ArgAction;

        // Determine argument type
        let arg_type = if meta.positional.is_some() {
            ArgumentType::Positional
        } else if meta.is_flag {
            ArgumentType::Flag
        } else if matches!(meta.action, Some(ArgAction::Count)) {
            ArgumentType::Count
        } else {
            ArgumentType::Named
        };

        // Generate long flag from name if not a positional arg
        let long = if meta.positional.is_none() {
            Some(meta.name.clone())
        } else {
            None
        };

        Self {
            name: meta.name.clone(),
            short: meta.short,
            long,
            arg_type,
            help: meta.help.clone(),
            required: meta.required,
            default: meta.default_value.clone(),
            env: meta.env.clone(),
            value_name: meta.value_name.clone(),
            possible_values: None, // Could extract from value_parser
            multiple: meta.multiple,
            group: meta.group.clone(),
            requires: meta.requires.clone(),
            conflicts_with: meta.conflicts_with.clone(),
            index: meta.positional,
        }
    }
}

/// Verb metadata in the grammar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarVerb {
    /// Verb name
    pub name: String,
    /// Parent noun
    pub noun: String,
    /// Help text
    pub help: Option<String>,
    /// Long help text
    pub long_help: Option<String>,
    /// Arguments
    pub arguments: Vec<GrammarArgument>,
    /// Whether deprecated
    pub deprecated: bool,
    /// Deprecation message
    pub deprecation_message: Option<String>,
    /// Additional metadata (extensible)
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Noun metadata in the grammar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarNoun {
    /// Noun name
    pub name: String,
    /// Help text
    pub help: Option<String>,
    /// Long help text
    pub long_help: Option<String>,
    /// Verbs under this noun
    pub verbs: Vec<GrammarVerb>,
    /// Sub-nouns (for nested structures)
    pub sub_nouns: Vec<GrammarNoun>,
    /// Additional metadata (extensible)
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Complete grammar model for a CNV application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarModel {
    /// Schema version
    pub schema_version: String,
    /// Application name
    pub app_name: String,
    /// Application version (if available)
    pub app_version: Option<String>,
    /// Top-level nouns
    pub nouns: Vec<GrammarNoun>,
    /// Global arguments (available to all commands)
    pub global_arguments: Vec<GrammarArgument>,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl GrammarModel {
    /// Create a new grammar model
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            schema_version: GRAMMAR_SCHEMA_VERSION.to_string(),
            app_name: app_name.into(),
            app_version: None,
            nouns: Vec::new(),
            global_arguments: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set application version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.app_version = Some(version.into());
        self
    }

    /// Add a noun
    pub fn add_noun(&mut self, noun: GrammarNoun) {
        self.nouns.push(noun);
    }

    /// Add a global argument
    pub fn add_global_argument(&mut self, arg: GrammarArgument) {
        self.global_arguments.push(arg);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Serialize) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
    }

    /// Get all nouns
    pub fn nouns(&self) -> &[GrammarNoun] {
        &self.nouns
    }

    /// Find a noun by name
    pub fn find_noun(&self, name: &str) -> Option<&GrammarNoun> {
        self.nouns.iter().find(|n| n.name == name)
    }

    /// Get all verbs (flattened)
    pub fn all_verbs(&self) -> Vec<&GrammarVerb> {
        self.nouns
            .iter()
            .flat_map(|n| n.verbs.iter())
            .collect()
    }

    /// Find a verb by noun and verb name
    pub fn find_verb(&self, noun: &str, verb: &str) -> Option<&GrammarVerb> {
        self.find_noun(noun)
            .and_then(|n| n.verbs.iter().find(|v| v.name == verb))
    }
}

/// Grammar extraction and introspection
pub struct Grammar;

impl Grammar {
    /// Extract the complete grammar from registered commands
    ///
    /// This inspects the compile-time registry to build a runtime
    /// grammar model that can be serialized and introspected.
    pub fn extract() -> Result<GrammarModel, Box<dyn std::error::Error>> {
        Self::extract_with_name("app")
    }

    /// Extract grammar with a specific application name
    pub fn extract_with_name(app_name: &str) -> Result<GrammarModel, Box<dyn std::error::Error>> {
        // Initialize registries
        for init in __NOUN_REGISTRY {
            init();
        }
        for init in __VERB_REGISTRY {
            init();
        }

        let model = GrammarModel::new(app_name);

        // Group verbs by noun
        let _nouns_map: HashMap<String, Vec<(String, Vec<ArgMetadata>)>> = HashMap::new();

        // Extract verbs from registry
        for _init in __VERB_REGISTRY {
            // This is a bit tricky - we need access to the actual registry data
            // For now, we'll build this from the clap Command structure
            // TODO: Enhance registry to expose verb metadata directly
        }

        // Extract nouns from registry
        for _init in __NOUN_REGISTRY {
            // Similarly, extract noun metadata
            // TODO: Enhance registry
        }

        // For now, return a basic model
        // In a full implementation, we'd walk the clap Command tree
        Ok(model)
    }

    /// Extract grammar and dump as JSON
    pub fn dump_json() -> Result<String, Box<dyn std::error::Error>> {
        let model = Self::extract()?;
        Ok(serde_json::to_string_pretty(&model)?)
    }

    /// Extract grammar and dump as YAML
    pub fn dump_yaml() -> Result<String, Box<dyn std::error::Error>> {
        let model = Self::extract()?;
        Ok(serde_yaml::to_string(&model)?)
    }
}

/// Grammar node for traversal and querying
///
/// Provides a unified interface for working with nouns and verbs
#[derive(Debug, Clone)]
pub enum GrammarNode<'a> {
    /// Root node
    Root(&'a GrammarModel),
    /// Noun node
    Noun(&'a GrammarNoun),
    /// Verb node
    Verb(&'a GrammarVerb),
}

impl<'a> GrammarNode<'a> {
    /// Get the name of this node
    pub fn name(&self) -> &str {
        match self {
            Self::Root(model) => &model.app_name,
            Self::Noun(noun) => &noun.name,
            Self::Verb(verb) => &verb.name,
        }
    }

    /// Get children of this node
    pub fn children(&self) -> Vec<GrammarNode<'a>> {
        match self {
            Self::Root(model) => model
                .nouns
                .iter()
                .map(GrammarNode::Noun)
                .collect(),
            Self::Noun(noun) => noun
                .verbs
                .iter()
                .map(GrammarNode::Verb)
                .chain(noun.sub_nouns.iter().map(GrammarNode::Noun))
                .collect(),
            Self::Verb(_) => Vec::new(),
        }
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Verb(_))
    }
}

/// Grammar query builder for advanced queries
pub struct GrammarQuery<'a> {
    model: &'a GrammarModel,
    filters: Vec<Box<dyn Fn(&GrammarNode) -> bool + 'a>>,
}

impl<'a> GrammarQuery<'a> {
    /// Create a new query
    pub fn new(model: &'a GrammarModel) -> Self {
        Self {
            model,
            filters: Vec::new(),
        }
    }

    /// Filter by deprecated status
    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.filters.push(Box::new(move |node| match node {
            GrammarNode::Verb(verb) => verb.deprecated == deprecated,
            _ => true,
        }));
        self
    }

    /// Filter by noun name
    pub fn noun(mut self, noun_name: String) -> Self {
        self.filters.push(Box::new(move |node| match node {
            GrammarNode::Verb(verb) => verb.noun == noun_name,
            GrammarNode::Noun(noun) => noun.name == noun_name,
            _ => true,
        }));
        self
    }

    /// Execute the query
    pub fn execute(&self) -> Vec<GrammarNode<'a>> {
        let mut results = Vec::new();
        self.traverse(&GrammarNode::Root(self.model), &mut results);
        results
    }

    fn traverse(&self, node: &GrammarNode<'a>, results: &mut Vec<GrammarNode<'a>>) {
        let matches = self.filters.iter().all(|f| f(node));
        if matches {
            results.push(node.clone());
        }

        for child in node.children() {
            self.traverse(&child, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_model_creation() {
        let model = GrammarModel::new("test-app")
            .with_version("1.0.0");

        assert_eq!(model.app_name, "test-app");
        assert_eq!(model.app_version, Some("1.0.0".to_string()));
        assert_eq!(model.schema_version, GRAMMAR_SCHEMA_VERSION);
    }

    #[test]
    fn test_grammar_argument_types() {
        use clap::ArgAction;

        let mut meta = ArgMetadata {
            name: "test".to_string(),
            required: false,
            is_flag: true,
            help: None,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
            short: None,
            default_value: None,
            env: None,
            multiple: false,
            value_name: None,
            aliases: Vec::new(),
            positional: None,
            action: Some(ArgAction::SetTrue),
            group: None,
            requires: Vec::new(),
            conflicts_with: Vec::new(),
            value_parser: None,
            hide: false,
            next_help_heading: None,
            long_help: None,
            next_line_help: false,
            display_order: None,
            exclusive: None,
            trailing_vararg: false,
            allow_negative_numbers: false,
        };

        let arg = GrammarArgument::from_metadata(&meta);
        assert_eq!(arg.arg_type, ArgumentType::Flag);

        meta.action = Some(ArgAction::Count);
        meta.is_flag = false;
        let arg = GrammarArgument::from_metadata(&meta);
        assert_eq!(arg.arg_type, ArgumentType::Count);

        meta.positional = Some(0);
        let arg = GrammarArgument::from_metadata(&meta);
        assert_eq!(arg.arg_type, ArgumentType::Positional);
    }

    #[test]
    fn test_grammar_node() {
        let model = GrammarModel::new("test-app");
        let node = GrammarNode::Root(&model);

        assert_eq!(node.name(), "test-app");
        assert!(!node.is_leaf());
    }
}
