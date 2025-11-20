//! CLI integration for autonomic features
//!
//! This module provides the runtime integration between the autonomic layer
//! and the clap CLI building infrastructure.

use super::introspection::{
    AppMetadata, CommandCapabilities, CommandMetadata, IntrospectionResponse, NounMetadata,
    VerbMetadata,
};
use crate::error::Result;
use crate::noun::NounCommand;
use crate::registry::CommandRegistry;

/// Autonomic CLI handler that wraps a registry with autonomic capabilities
pub struct AutonomicCli {
    registry: CommandRegistry,
    cli_version: String,
    app_metadata: AppMetadata,
}

impl AutonomicCli {
    /// Create a new autonomic CLI from a registry
    pub fn new(
        registry: CommandRegistry,
        cli_version: impl Into<String>,
        app_metadata: AppMetadata,
    ) -> Self {
        Self { registry, cli_version: cli_version.into(), app_metadata }
    }

    /// Create from registry with default app metadata
    pub fn from_registry(registry: CommandRegistry, cli_version: impl Into<String>) -> Self {
        let app_metadata = AppMetadata::new("cli");
        Self { registry, cli_version: cli_version.into(), app_metadata }
    }

    /// Get command capabilities
    pub fn capabilities(&self) -> CommandCapabilities {
        CommandCapabilities::new(&self.app_metadata.name, &self.cli_version)
            .with_app(self.app_metadata.clone())
    }

    /// Get full introspection response
    pub fn introspect(&self) -> IntrospectionResponse {
        let mut response = IntrospectionResponse::new(&self.cli_version, self.app_metadata.clone());

        for noun in self.registry.nouns() {
            response = response.with_noun(self.introspect_noun(noun));
        }

        response
    }

    /// Introspect a specific noun
    pub fn introspect_noun(&self, noun: &dyn NounCommand) -> NounMetadata {
        let mut noun_meta = NounMetadata::new(noun.name(), noun.about());

        for verb in noun.verbs() {
            let verb_meta = VerbMetadata::new(verb.name(), verb.about())
                .with_command(self.introspect_verb_command(verb.as_ref()));

            noun_meta = noun_meta.with_verb(verb_meta);
        }

        for sub_noun in noun.sub_nouns() {
            noun_meta = noun_meta.with_sub_noun(self.introspect_noun(sub_noun.as_ref()));
        }

        noun_meta
    }

    /// Introspect a verb command (can be overridden by trait extensions)
    fn introspect_verb_command(&self, _verb: &dyn crate::verb::VerbCommand) -> CommandMetadata {
        // Default implementation - can be extended by implementing AutonomicVerbCommand trait
        CommandMetadata::default()
    }

    /// Handle introspection request for a specific noun
    pub fn introspect_noun_by_name(&self, noun_name: &str) -> Option<NounMetadata> {
        self.registry.get_noun(noun_name).map(|noun| self.introspect_noun(noun))
    }

    /// Handle introspection request for a specific verb
    pub fn introspect_verb(&self, noun_name: &str, verb_name: &str) -> Option<VerbMetadata> {
        self.registry.get_noun(noun_name).and_then(|noun| {
            noun.verbs().into_iter().find(|v| v.name() == verb_name).map(|verb| {
                VerbMetadata::new(verb.name(), verb.about())
                    .with_command(self.introspect_verb_command(verb.as_ref()))
            })
        })
    }

    /// Get the underlying registry
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Convert to JSON capabilities string
    pub fn capabilities_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.capabilities())
            .map_err(|e| crate::error::NounVerbError::execution_error(e.to_string()))
    }

    /// Convert to JSON introspection string
    pub fn introspect_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.introspect())
            .map_err(|e| crate::error::NounVerbError::execution_error(e.to_string()))
    }
}

/// Trait for verbs that provide autonomic metadata
pub trait AutonomicVerbCommand: crate::verb::VerbCommand {
    /// Get command metadata for introspection
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::default()
    }
}

/// Trait for nouns that provide autonomic metadata
pub trait AutonomicNounCommand: crate::noun::NounCommand {
    /// Get additional metadata for the noun
    fn noun_metadata(&self) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        None
    }
}
