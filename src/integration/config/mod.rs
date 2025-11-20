//! Plugin configuration system with dependency resolution.
//!
//! Features:
//! - TOML manifest parsing
//! - Dependency graph resolution
//! - Topological sorting for load order
//! - Manifest validation

mod graph;
mod loader;

pub use graph::PluginDependencyGraph;
pub use loader::PluginManifestLoader;

/// Plugin configuration from manifest file.
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin entry point
    pub entry_point: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Plugin configuration
    pub config: std::collections::HashMap<String, String>,
}

impl PluginConfig {
    /// Create a new plugin configuration.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        entry_point: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            entry_point: entry_point.into(),
            dependencies: Vec::new(),
            config: std::collections::HashMap::new(),
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::NounVerbError::ValidationFailed(
                "Plugin name cannot be empty".to_string(),
            ));
        }
        if self.version.is_empty() {
            return Err(crate::NounVerbError::ValidationFailed(
                "Plugin version cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config_creation() {
        let config = PluginConfig::new("test", "1.0.0", "lib.so");
        assert_eq!(config.name, "test");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_plugin_config_validation() {
        let config = PluginConfig::new("", "1.0.0", "lib.so");
        assert!(config.validate().is_err());
    }
}
