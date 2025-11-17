//! Configuration Manager Plugin - Load and manage configuration
//! See PLUGIN_IMPLEMENTATION_GUIDE.md for full specification

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configuration entry with type information
#[derive(Clone, Debug)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<String>),
}

impl ConfigValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Configuration Manager Plugin - Load, store, and manage configuration
#[derive(Clone)]
pub struct ConfigManagerPlugin {
    config: Arc<Mutex<HashMap<String, ConfigValue>>>,
    defaults: Arc<Mutex<HashMap<String, ConfigValue>>>,
    loaded: bool,
}

impl ConfigManagerPlugin {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            defaults: Arc::new(Mutex::new(HashMap::new())),
            loaded: false,
        }
    }

    /// Set a configuration value
    pub fn set(&self, key: &str, value: ConfigValue) -> crate::Result<()> {
        let mut config = self.config.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Config lock failed".to_string())
        })?;
        config.insert(key.to_string(), value);
        Ok(())
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> crate::Result<Option<ConfigValue>> {
        let config = self.config.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Config lock failed".to_string())
        })?;

        Ok(config.get(key).cloned().or_else(|| {
            // Fall back to defaults
            let defaults = self.defaults.lock().ok()?;
            defaults.get(key).cloned()
        }))
    }

    /// Get string value with default
    pub fn get_string(&self, key: &str, default: &str) -> crate::Result<String> {
        let value = self.get(key)?;
        match value {
            Some(ConfigValue::String(s)) => Ok(s),
            _ => Ok(default.to_string()),
        }
    }

    /// Get integer value with default
    pub fn get_integer(&self, key: &str, default: i64) -> crate::Result<i64> {
        let value = self.get(key)?;
        match value {
            Some(ConfigValue::Integer(i)) => Ok(i),
            _ => Ok(default),
        }
    }

    /// Get boolean value with default
    pub fn get_boolean(&self, key: &str, default: bool) -> crate::Result<bool> {
        let value = self.get(key)?;
        match value {
            Some(ConfigValue::Boolean(b)) => Ok(b),
            _ => Ok(default),
        }
    }

    /// Set default value for a key
    pub fn set_default(&self, key: &str, value: ConfigValue) -> crate::Result<()> {
        let mut defaults = self.defaults.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Defaults lock failed".to_string())
        })?;
        defaults.insert(key.to_string(), value);
        Ok(())
    }

    /// Clear all configuration (keep defaults)
    pub fn clear(&self) -> crate::Result<()> {
        let mut config = self.config.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Config lock failed".to_string())
        })?;
        config.clear();
        Ok(())
    }

    /// Get all configuration keys
    pub fn keys(&self) -> crate::Result<Vec<String>> {
        let config = self.config.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Config lock failed".to_string())
        })?;
        Ok(config.keys().cloned().collect())
    }

    /// Check if key exists
    pub fn has_key(&self, key: &str) -> crate::Result<bool> {
        let config = self.config.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Config lock failed".to_string())
        })?;
        Ok(config.contains_key(key))
    }
}

impl Default for ConfigManagerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ConfigManagerPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigManagerPlugin").finish()
    }
}

impl Plugin for ConfigManagerPlugin {
    fn name(&self) -> &str {
        "config-manager"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_description("Configuration management with defaults")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Middleware]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chicago-TDD: Integration tests with real config manager
    #[test]
    fn test_config_manager_set_and_get_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("app_name", ConfigValue::String("MyApp".to_string())).unwrap();
        plugin.set("port", ConfigValue::Integer(8080)).unwrap();

        let name = plugin.get("app_name").unwrap();
        assert_eq!(name.as_ref().and_then(|v| v.as_string()), Some("MyApp"));

        let port = plugin.get("port").unwrap();
        assert_eq!(port.as_ref().and_then(|v| v.as_integer()), Some(8080));
    }

    #[test]
    fn test_config_manager_defaults_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set_default("timeout", ConfigValue::Integer(30)).unwrap();
        plugin.set_default("retry", ConfigValue::Boolean(true)).unwrap();

        // Should get default when not set
        let timeout = plugin.get_integer("timeout", 0).unwrap();
        assert_eq!(timeout, 30);

        let retry = plugin.get_boolean("retry", false).unwrap();
        assert!(retry);
    }

    #[test]
    fn test_config_manager_override_defaults_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set_default("workers", ConfigValue::Integer(4)).unwrap();

        // Get default
        let workers = plugin.get_integer("workers", 0).unwrap();
        assert_eq!(workers, 4);

        // Override with specific value
        plugin.set("workers", ConfigValue::Integer(8)).unwrap();

        let workers = plugin.get_integer("workers", 0).unwrap();
        assert_eq!(workers, 8);
    }

    #[test]
    fn test_config_manager_string_values_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set_default("database_url", ConfigValue::String("localhost".to_string())).unwrap();
        plugin.set_default("api_key", ConfigValue::String("secret123".to_string())).unwrap();

        let db = plugin.get_string("database_url", "").unwrap();
        assert_eq!(db, "localhost");

        let key = plugin.get_string("api_key", "").unwrap();
        assert_eq!(key, "secret123");
    }

    #[test]
    fn test_config_manager_boolean_values_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("debug", ConfigValue::Boolean(true)).unwrap();
        plugin.set("ssl_verify", ConfigValue::Boolean(false)).unwrap();

        let debug = plugin.get_boolean("debug", false).unwrap();
        assert!(debug);

        let ssl = plugin.get_boolean("ssl_verify", true).unwrap();
        assert!(!ssl);
    }

    #[test]
    fn test_config_manager_clear_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("key1", ConfigValue::String("value1".to_string())).unwrap();
        plugin.set("key2", ConfigValue::Integer(42)).unwrap();

        // Verify set
        assert!(plugin.has_key("key1").unwrap());
        assert!(plugin.has_key("key2").unwrap());

        // Clear
        plugin.clear().unwrap();

        // Defaults should still exist
        plugin.set_default("default_key", ConfigValue::String("default_value".to_string())).unwrap();
        let val = plugin.get("default_key").unwrap();
        assert!(val.is_some());
    }

    #[test]
    fn test_config_manager_keys_listing_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("key1", ConfigValue::String("v1".to_string())).unwrap();
        plugin.set("key2", ConfigValue::String("v2".to_string())).unwrap();
        plugin.set("key3", ConfigValue::String("v3".to_string())).unwrap();

        let keys = plugin.keys().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[test]
    fn test_config_manager_multiple_types_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("string_val", ConfigValue::String("hello".to_string())).unwrap();
        plugin.set("int_val", ConfigValue::Integer(123)).unwrap();
        plugin.set("float_val", ConfigValue::Float(45.67)).unwrap();
        plugin.set("bool_val", ConfigValue::Boolean(true)).unwrap();
        plugin.set("list_val", ConfigValue::List(vec!["a".to_string(), "b".to_string()])).unwrap();

        let keys = plugin.keys().unwrap();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn test_config_manager_concurrent_access_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        let plugin = Arc::new(plugin);
        let mut handles = vec![];

        // Spawn threads that write and read config
        for i in 0..5 {
            let p = Arc::clone(&plugin);
            let handle = std::thread::spawn(move || {
                let key = format!("thread_key_{}", i);
                p.set(&key, ConfigValue::Integer(i as i64)).unwrap();
                let val = p.get(&key).unwrap();
                assert_eq!(val.as_ref().and_then(|v| v.as_integer()), Some(i as i64));
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all keys exist
        let keys = plugin.keys().unwrap();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn test_config_manager_missing_key_fallback_workflow() {
        let mut plugin = ConfigManagerPlugin::new();
        plugin.load().unwrap();

        // Key doesn't exist, should return default
        let val = plugin.get_string("nonexistent", "default_value").unwrap();
        assert_eq!(val, "default_value");

        // Integer default
        let num = plugin.get_integer("no_number", 999).unwrap();
        assert_eq!(num, 999);
    }
}
