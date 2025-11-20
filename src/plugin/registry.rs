//! Plugin registry for managing registered plugins.

use super::{Plugin, PluginCapability, PluginState};
use std::collections::HashMap;

/// Central registry for plugin management.
pub struct PluginRegistry {
    plugins: HashMap<String, (Box<dyn Plugin>, PluginState)>,
    capability_index: HashMap<PluginCapability, Vec<String>>,
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("plugin_count", &self.plugins.len())
            .field("capabilities", &self.capability_index.len())
            .finish()
    }
}

impl PluginRegistry {
    /// Create a new plugin registry.
    pub fn new() -> Self {
        Self { plugins: HashMap::new(), capability_index: HashMap::new() }
    }

    /// Register a plugin in the registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is already registered or if loading fails.
    pub fn register(&mut self, mut plugin: Box<dyn Plugin>) -> crate::Result<()> {
        let name = plugin.name().to_string();

        // Check if already registered
        if self.plugins.contains_key(&name) {
            return Err(crate::NounVerbError::PluginError(format!(
                "Plugin '{}' is already registered",
                name
            )));
        }

        // Load the plugin
        plugin.load()?;

        // Index capabilities
        for cap in plugin.capabilities() {
            self.capability_index.entry(cap).or_insert_with(Vec::new).push(name.clone());
        }

        // Register the plugin
        self.plugins.insert(name, (plugin, PluginState::Loaded));

        Ok(())
    }

    /// Unregister a plugin from the registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not found or if unloading fails.
    pub fn unregister(&mut self, name: &str) -> crate::Result<()> {
        let (mut plugin, _) = self.plugins.remove(name).ok_or_else(|| {
            crate::NounVerbError::PluginError(format!("Plugin '{}' not found", name))
        })?;

        // Unload the plugin
        plugin.unload()?;

        // Remove from capability index
        for (_, plugins) in &mut self.capability_index {
            plugins.retain(|p| p != name);
        }

        Ok(())
    }

    /// Get a plugin by name.
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|(plugin, _)| plugin.as_ref())
    }

    /// Get the state of a plugin.
    pub fn get_state(&self, name: &str) -> Option<PluginState> {
        self.plugins.get(name).map(|(_, state)| *state)
    }

    /// Enable a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not found.
    pub fn enable(&mut self, name: &str) -> crate::Result<()> {
        let entry = self.plugins.get_mut(name).ok_or_else(|| {
            crate::NounVerbError::PluginError(format!("Plugin '{}' not found", name))
        })?;

        entry.1 = PluginState::Loaded;
        Ok(())
    }

    /// Disable a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not found.
    pub fn disable(&mut self, name: &str) -> crate::Result<()> {
        let entry = self.plugins.get_mut(name).ok_or_else(|| {
            crate::NounVerbError::PluginError(format!("Plugin '{}' not found", name))
        })?;

        entry.1 = PluginState::Disabled;
        Ok(())
    }

    /// Get all registered plugins.
    pub fn list_all(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Get all plugins with a specific capability.
    pub fn find_by_capability(&self, capability: PluginCapability) -> Vec<&str> {
        self.capability_index
            .get(&capability)
            .map(|plugins| plugins.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get the number of registered plugins.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Check if a plugin is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get all plugins in loaded state.
    pub fn loaded_plugins(&self) -> Vec<&str> {
        self.plugins
            .iter()
            .filter(|(_, (_, state))| *state == PluginState::Loaded)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Clear all plugins from the registry.
    ///
    /// # Errors
    ///
    /// Returns an error if any plugin fails to unload.
    pub fn clear(&mut self) -> crate::Result<()> {
        let plugin_names: Vec<_> = self.plugins.keys().cloned().collect();
        for name in plugin_names {
            self.unregister(&name)?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlugin {
        name: String,
    }

    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::Command]
        }

        fn load(&mut self) -> crate::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(MockPlugin { name: "test".to_string() });
        assert!(registry.register(plugin).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_plugin_registry_duplicate() {
        let mut registry = PluginRegistry::new();
        let plugin1 = Box::new(MockPlugin { name: "test".to_string() });
        let plugin2 = Box::new(MockPlugin { name: "test".to_string() });

        registry.register(plugin1).unwrap();
        assert!(registry.register(plugin2).is_err());
    }

    #[test]
    fn test_plugin_registry_get() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(MockPlugin { name: "test".to_string() });
        registry.register(plugin).unwrap();
        assert!(registry.get("test").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_plugin_registry_find_by_capability() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(MockPlugin { name: "test".to_string() });
        registry.register(plugin).unwrap();
        let found = registry.find_by_capability(PluginCapability::Command);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "test");
    }

    #[test]
    fn test_plugin_registry_list_all() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin { name: "test1".to_string() })).unwrap();
        registry.register(Box::new(MockPlugin { name: "test2".to_string() })).unwrap();
        assert_eq!(registry.list_all().len(), 2);
    }

    #[test]
    fn test_plugin_registry_enable_disable() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(MockPlugin { name: "test".to_string() });
        registry.register(plugin).unwrap();
        assert_eq!(registry.get_state("test"), Some(PluginState::Loaded));

        registry.disable("test").unwrap();
        assert_eq!(registry.get_state("test"), Some(PluginState::Disabled));

        registry.enable("test").unwrap();
        assert_eq!(registry.get_state("test"), Some(PluginState::Loaded));
    }
}
