//! Command aliasing plugin for shortcuts.

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::HashMap;

/// Plugin for command aliases and shortcuts.
#[derive(Debug)]
pub struct AliasPlugin {
    loaded: bool,
    aliases: HashMap<String, String>,
}

impl AliasPlugin {
    /// Create a new alias plugin.
    pub fn new() -> Self {
        Self { loaded: false, aliases: HashMap::new() }
    }

    /// Add a command alias.
    pub fn add_alias(&mut self, alias: impl Into<String>, command: impl Into<String>) {
        self.aliases.insert(alias.into(), command.into());
    }

    /// Get a command by alias.
    pub fn get_alias(&self, alias: &str) -> Option<&str> {
        self.aliases.get(alias).map(|s| s.as_str())
    }

    /// Remove an alias.
    pub fn remove_alias(&mut self, alias: &str) -> Option<String> {
        self.aliases.remove(alias)
    }

    /// Get all aliases.
    pub fn list_aliases(&self) -> Vec<(&str, &str)> {
        self.aliases.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    }

    /// Get the number of aliases.
    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }

    /// Clear all aliases.
    pub fn clear_aliases(&mut self) {
        self.aliases.clear();
    }
}

impl Default for AliasPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AliasPlugin {
    fn name(&self) -> &str {
        "alias"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_author("clap-noun-verb")
            .with_description("Command aliasing and shortcuts")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Hook, PluginCapability::Middleware]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }

    fn unload(&mut self) -> crate::Result<()> {
        self.loaded = false;
        self.clear_aliases();
        Ok(())
    }

    fn status(&self) -> String {
        format!(
            "AliasPlugin v{} ({} aliases, {})",
            self.version(),
            self.alias_count(),
            if self.loaded { "loaded" } else { "unloaded" }
        )
    }
}
