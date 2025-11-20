//! Plugin/Extension System (Feature 1 - v4.3)
//!
//! This module provides a comprehensive plugin system for dynamic command registration,
//! extension loading, and capability-based sandboxing.
//!
//! # Architecture
//!
//! - **Plugin trait** - Base interface for all plugins
//! - **PluginRegistry** - Central registry for plugin discovery and management
//! - **PluginLoader** - Dynamic loading from manifests
//! - **Built-in plugins** - Help, history, and alias plugins
//!
//! # Example
//!
//! ```ignore
//! use clap_noun_verb::plugin::{Plugin, PluginRegistry, PluginCapability};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!     fn version(&self) -> &str { "1.0.0" }
//!     fn capabilities(&self) -> Vec<PluginCapability> {
//!         vec![PluginCapability::Command]
//!     }
//!     fn load(&mut self) -> Result<()> { Ok(()) }
//! }
//!
//! let mut registry = PluginRegistry::new();
//! registry.register(Box::new(MyPlugin))?;
//! ```

pub mod builtin;
pub mod loader;
pub mod registry;

use std::fmt;

pub use builtin::{AliasPlugin, HelpPlugin, HistoryPlugin};
pub use loader::PluginLoader;
pub use registry::PluginRegistry;

/// Plugin capability enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// Can register new commands
    Command,
    /// Can hook into execution pipeline
    Hook,
    /// Can intercept and modify command processing
    Middleware,
    /// Can provide custom validation
    Validator,
    /// Can generate completions
    Completion,
}

impl fmt::Display for PluginCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command => write!(f, "command"),
            Self::Hook => write!(f, "hook"),
            Self::Middleware => write!(f, "middleware"),
            Self::Validator => write!(f, "validator"),
            Self::Completion => write!(f, "completion"),
        }
    }
}

/// Plugin metadata for version and dependency tracking.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
    /// Plugin author
    author: String,
    /// Plugin description
    description: String,
    /// Dependencies on other plugins
    dependencies: Vec<String>,
    /// Minimum required API version
    min_api_version: String,
}

impl PluginMetadata {
    /// Create new plugin metadata.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            author: String::new(),
            description: String::new(),
            dependencies: Vec::new(),
            min_api_version: "4.3.0".to_string(),
        }
    }

    /// Set the plugin author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Set the plugin description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a plugin dependency.
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set the minimum API version.
    pub fn with_min_api_version(mut self, version: impl Into<String>) -> Self {
        self.min_api_version = version.into();
        self
    }

    /// Get the plugin name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the plugin version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the plugin author.
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Get the plugin description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get plugin dependencies.
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Get the minimum API version.
    pub fn min_api_version(&self) -> &str {
        &self.min_api_version
    }
}

/// Core plugin trait.
///
/// All plugins must implement this trait to be loadable and executable.
pub trait Plugin: Send + Sync {
    /// Get the plugin name.
    fn name(&self) -> &str;

    /// Get the plugin version.
    fn version(&self) -> &str;

    /// Get the plugin metadata.
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
    }

    /// Get the capabilities this plugin provides.
    fn capabilities(&self) -> Vec<PluginCapability>;

    /// Check if this plugin has a specific capability.
    fn has_capability(&self, cap: PluginCapability) -> bool {
        self.capabilities().contains(&cap)
    }

    /// Load the plugin (initialize resources).
    ///
    /// Called when the plugin is registered.
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails.
    fn load(&mut self) -> crate::Result<()>;

    /// Unload the plugin (cleanup resources).
    ///
    /// Called when the plugin is unregistered.
    ///
    /// # Errors
    ///
    /// Returns an error if unloading fails.
    fn unload(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Validate that dependencies are satisfied.
    ///
    /// # Errors
    ///
    /// Returns an error if dependencies are missing.
    fn validate_dependencies(&self, _registry: &PluginRegistry) -> crate::Result<()> {
        Ok(())
    }

    /// Get a human-readable status of this plugin.
    fn status(&self) -> String {
        format!("{} v{}", self.name(), self.version())
    }
}

/// Plugin state tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is registered but not loaded
    Registered,
    /// Plugin is currently loaded
    Loaded,
    /// Plugin failed to load
    Failed,
    /// Plugin is disabled
    Disabled,
}

impl fmt::Display for PluginState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered => write!(f, "registered"),
            Self::Loaded => write!(f, "loaded"),
            Self::Failed => write!(f, "failed"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// Plugin configuration.
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Enable plugin auto-discovery
    auto_discover: bool,
    /// Plugin manifest directory
    manifest_dir: String,
    /// Enable plugin caching
    enable_cache: bool,
    /// Sandbox plugins (capability-based restrictions)
    sandbox: bool,
}

impl PluginConfig {
    /// Create new plugin configuration.
    pub fn new() -> Self {
        Self {
            auto_discover: true,
            manifest_dir: "./plugins".to_string(),
            enable_cache: true,
            sandbox: true,
        }
    }

    /// Enable plugin auto-discovery.
    pub fn with_auto_discover(mut self, discover: bool) -> Self {
        self.auto_discover = discover;
        self
    }

    /// Set the manifest directory.
    pub fn with_manifest_dir(mut self, dir: impl Into<String>) -> Self {
        self.manifest_dir = dir.into();
        self
    }

    /// Enable plugin result caching.
    pub fn with_cache(mut self, cache: bool) -> Self {
        self.enable_cache = cache;
        self
    }

    /// Enable plugin sandboxing.
    pub fn with_sandbox(mut self, sandbox: bool) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Check if auto-discover is enabled.
    pub fn is_auto_discover_enabled(&self) -> bool {
        self.auto_discover
    }

    /// Get the manifest directory.
    pub fn manifest_dir(&self) -> &str {
        &self.manifest_dir
    }

    /// Check if caching is enabled.
    pub fn is_cache_enabled(&self) -> bool {
        self.enable_cache
    }

    /// Check if sandboxing is enabled.
    pub fn is_sandbox_enabled(&self) -> bool {
        self.sandbox
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_capability_display() {
        assert_eq!(PluginCapability::Command.to_string(), "command");
        assert_eq!(PluginCapability::Hook.to_string(), "hook");
        assert_eq!(PluginCapability::Middleware.to_string(), "middleware");
    }

    #[test]
    fn test_plugin_metadata_creation() {
        let meta = PluginMetadata::new("test", "1.0.0");
        assert_eq!(meta.name(), "test");
        assert_eq!(meta.version(), "1.0.0");
    }

    #[test]
    fn test_plugin_metadata_with_author() {
        let meta = PluginMetadata::new("test", "1.0.0").with_author("Author");
        assert_eq!(meta.author(), "Author");
    }

    #[test]
    fn test_plugin_state_display() {
        assert_eq!(PluginState::Loaded.to_string(), "loaded");
        assert_eq!(PluginState::Failed.to_string(), "failed");
    }

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.is_auto_discover_enabled());
        assert!(config.is_cache_enabled());
        assert!(config.is_sandbox_enabled());
    }

    #[test]
    fn test_plugin_config_with_options() {
        let config = PluginConfig::new().with_auto_discover(false).with_manifest_dir("/custom");
        assert!(!config.is_auto_discover_enabled());
        assert_eq!(config.manifest_dir(), "/custom");
    }
}
