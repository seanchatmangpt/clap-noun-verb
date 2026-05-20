//! Configuration file support
//!
//! This module provides utilities for loading CLI arguments from configuration files.
//!
//! # Supported formats
//!
//! - YAML (.yaml, .yml)
//! - TOML (.toml)
//! - JSON (.json)
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::config::ConfigLoader;
//!
//! let config = ConfigLoader::new()
//!     .with_path("config.yaml")
//!     .load()?;
//!
//! let args: Vec<String> = config.to_cli_args();
//! ```

use crate::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration loader supporting multiple formats
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    path: Option<PathBuf>,
    default_paths: Vec<PathBuf>,
    profile: Option<String>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            path: None,
            default_paths: vec![
                PathBuf::from("clap-nv.toml"),
                PathBuf::from("clap-nv.yaml"),
                PathBuf::from(".env.yaml"),
                PathBuf::from("config.yaml"),
                PathBuf::from("config.yml"),
                PathBuf::from(".config/app.yaml"),
            ],
            profile: None,
        }
    }

    /// Set explicit config file path
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the active profile (e.g., "dev", "prod")
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Set the profile from an environment variable
    pub fn with_env_profile(mut self, env_var: &str) -> Self {
        if let Ok(val) = std::env::var(env_var) {
            self.profile = Some(val);
        }
        self
    }

    /// Add a default path to search for config file
    pub fn with_default_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.default_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Find the path to the configuration file
    pub fn find_config_path(&self) -> Option<PathBuf> {
        if let Some(ref p) = self.path {
            if p.exists() {
                return Some(p.clone());
            }
        }
        self.default_paths.iter().find(|p| p.exists()).cloned()
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<Config> {
        let path = self.find_config_path().ok_or_else(|| {
            crate::error::NounVerbError::execution_error("No configuration file found")
        })?;

        let mut config = Config::from_file(&path)?;
        if let Some(ref profile) = self.profile {
            config.apply_profile(profile);
        }
        Ok(config)
    }

    /// Try to load, returning empty config if file not found
    pub fn load_optional(&self) -> Result<Config> {
        match self.load() {
            Ok(config) => Ok(config),
            Err(_) => Ok(Config::new()),
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed configuration
#[derive(Debug, Clone)]
pub struct Config {
    data: Value,
}

impl Config {
    /// Create new empty configuration
    pub fn new() -> Self {
        Self { data: json!({}) }
    }

    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to read config file: {}",
                e
            ))
        })?;

        // Phase 1: Interpolate environment variables
        let content = Self::interpolate(&content);

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("json");

        let data = match extension {
            "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| {
                crate::error::NounVerbError::execution_error(format!("Failed to parse YAML: {}", e))
            })?,
            "toml" => {
                let toml_value: toml::Value = toml::from_str(&content).map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!(
                        "Failed to parse TOML: {}",
                        e
                    ))
                })?;
                serde_json::to_value(toml_value).map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!(
                        "Failed to convert TOML: {}",
                        e
                    ))
                })?
            }
            _ => serde_json::from_str(&content).map_err(|e| {
                crate::error::NounVerbError::execution_error(format!("Failed to parse JSON: {}", e))
            })?,
        };

        Ok(Self { data })
    }

    /// Interpolate environment variables in string using ${VAR} syntax
    fn interpolate(content: &str) -> String {
        let mut result = content.to_string();
        // Simple search and replace for ${VAR}
        let mut i = 0;
        while let Some(start) = result[i..].find("${") {
            let start = i + start;
            if let Some(end) = result[start..].find('}') {
                let end = start + end;
                let var_name = &result[start + 2..end];
                if let Ok(var_val) = std::env::var(var_name) {
                    result.replace_range(start..end + 1, &var_val);
                    i = start + var_val.len();
                } else {
                    i = end + 1;
                }
            } else {
                break;
            }
        }
        result
    }

    /// Apply profile-based merging
    pub fn apply_profile(&mut self, profile: &str) {
        if let Some(profile_data) = self.data.get(profile).cloned() {
            if profile_data.is_object() {
                Self::deep_merge_values(&mut self.data, &profile_data);
            }
        }
    }

    /// Deep merge two JSON values
    fn deep_merge_values(base: &mut Value, over: &Value) {
        match (base, over) {
            (Value::Object(ref mut base_map), Value::Object(over_map)) => {
                for (key, value) in over_map {
                    if !base_map.contains_key(key) {
                        base_map.insert(key.clone(), value.clone());
                    } else {
                        Self::deep_merge_values(base_map.get_mut(key).unwrap(), value);
                    }
                }
            }
            (base, over) => {
                *base = over.clone();
            }
        }
    }

    /// Load configuration from JSON string
    pub fn from_json(json_str: &str) -> Result<Self> {
        let data = serde_json::from_str(json_str).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!("Failed to parse JSON: {}", e))
        })?;
        Ok(Self { data })
    }

    /// Load configuration from YAML string
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        let data = serde_yaml::from_str(yaml_str).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!("Failed to parse YAML: {}", e))
        })?;
        Ok(Self { data })
    }

    /// Get value by key
    pub fn get(&self, key: &str) -> Option<Value> {
        self.data.get(key).cloned()
    }

    /// Get string value
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key)?.as_str().map(String::from)
    }

    /// Convert configuration to CLI arguments
    ///
    /// Flattens the config to command-line arguments.
    /// For example: `{ "port": 8080 }` becomes `["--port", "8080"]`
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        self.flatten_to_args(&self.data, String::new(), &mut args);
        args
    }

    /// Flatten configuration to a HashMap of key-value pairs
    pub fn to_flat_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        self.flatten_to_map(&self.data, String::new(), &mut map);
        map
    }

    /// Helper to flatten nested config to CLI args
    #[allow(clippy::only_used_in_recursion)]
    fn flatten_to_args(&self, value: &Value, prefix: String, args: &mut Vec<String>) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj.iter() {
                    let new_prefix =
                        if prefix.is_empty() { key.clone() } else { format!("{}.{}", prefix, key) };
                    self.flatten_to_args(val, new_prefix, args);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    match item {
                        Value::String(s) => {
                            args.push(format!("--{}", prefix));
                            args.push(s.clone());
                        }
                        _ => {
                            args.push(format!("--{}", prefix));
                            args.push(item.to_string());
                        }
                    }
                }
            }
            Value::String(s) => {
                args.push(format!("--{}", prefix));
                args.push(s.clone());
            }
            Value::Number(n) => {
                args.push(format!("--{}", prefix));
                args.push(n.to_string());
            }
            Value::Bool(b) => {
                if *b {
                    args.push(format!("--{}", prefix));
                }
            }
            Value::Null => {}
        }
    }

    /// Helper to flatten config to map
    #[allow(clippy::only_used_in_recursion)]
    fn flatten_to_map(&self, value: &Value, prefix: String, map: &mut HashMap<String, String>) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj.iter() {
                    let new_prefix =
                        if prefix.is_empty() { key.clone() } else { format!("{}.{}", prefix, key) };
                    self.flatten_to_map(val, new_prefix, map);
                }
            }
            Value::String(s) => {
                map.insert(prefix, s.clone());
            }
            Value::Number(n) => {
                map.insert(prefix, n.to_string());
            }
            Value::Bool(b) => {
                map.insert(prefix, b.to_string());
            }
            Value::Array(_) | Value::Null => {}
        }
    }

    /// Convert configuration to TOML string
    pub fn to_toml(&self) -> Result<String> {
        let toml_value = toml::Value::try_from(&self.data).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to convert to TOML: {}",
                e
            ))
        })?;
        Ok(toml::to_string_pretty(&toml_value).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!("Failed to serialize TOML: {}", e))
        })?)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to write config file: {}",
                e
            ))
        })
    }

    /// Update configuration from a map of key-values
    pub fn update_from_map(&mut self, map: &HashMap<String, String>) {
        for (key, value) in map {
            self.set_nested_value(key, value);
        }
    }

    fn set_nested_value(&mut self, key: &str, value: &str) {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &mut self.data;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                if let Ok(b) = value.parse::<bool>() {
                    current[part] = Value::Bool(b);
                } else if let Ok(n) = value.parse::<i64>() {
                    current[part] = Value::Number(n.into());
                } else {
                    current[part] = Value::String(value.to_string());
                }
            } else {
                // Nested part, ensure object exists
                if !current.is_object() {
                    *current = json!({});
                }
                current = &mut current[part];
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

use notify::{Event, RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};

pub struct ConfigWatcher {
    loader: ConfigLoader,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(Config) + Send + Sync>>>>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new(loader: ConfigLoader) -> Self {
        Self { loader, callbacks: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Register a callback to be called when configuration changes
    pub fn on_change<F>(&mut self, callback: F)
    where
        F: Fn(Config) + Send + Sync + 'static,
    {
        let mut cbs = self.callbacks.lock().unwrap();
        cbs.push(Box::new(callback));
    }

    /// Start watching the configuration file in a background thread
    pub fn watch(self) -> Result<notify::RecommendedWatcher> {
        let path = self.loader.find_config_path().ok_or_else(|| {
            crate::error::NounVerbError::execution_error("No config file found to watch")
        })?;

        let loader_clone = self.loader.clone();
        let callbacks_clone = self.callbacks.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                // Ignore Access/Create events without content change, mostly focusing on Modify
                if event.kind.is_modify() {
                    if let Ok(new_config) = loader_clone.load() {
                        let cbs = callbacks_clone.lock().unwrap();
                        for cb in cbs.iter() {
                            cb(new_config.clone());
                        }
                    }
                }
            }
        })
        .map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to initialize watcher: {}",
                e
            ))
        })?;

        watcher.watch(&path, RecursiveMode::NonRecursive).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!("Failed to watch config: {}", e))
        })?;

        Ok(watcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_json() {
        let json_str = r#"{"port": 8080, "host": "localhost"}"#;
        let config = Config::from_json(json_str).unwrap();
        assert_eq!(config.get_string("host"), Some("localhost".to_string()));
    }

    #[test]
    fn test_config_from_yaml() {
        let yaml_str = "port: 8080\nhost: localhost";
        let config = Config::from_yaml(yaml_str).unwrap();
        assert_eq!(config.get_string("host"), Some("localhost".to_string()));
    }

    #[test]
    fn test_to_cli_args() {
        let json_str = r#"{"port": 8080, "host": "localhost", "verbose": true}"#;
        let config = Config::from_json(json_str).unwrap();
        let args = config.to_cli_args();
        assert!(args.contains(&"--port".to_string()));
        assert!(args.contains(&"8080".to_string()));
        assert!(args.contains(&"--host".to_string()));
        assert!(args.contains(&"localhost".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
    }

    #[test]
    fn test_to_flat_map() {
        let json_str = r#"{"port": 8080, "host": "localhost"}"#;
        let config = Config::from_json(json_str).unwrap();
        let map = config.to_flat_map();
        assert_eq!(map.get("port"), Some(&"8080".to_string()));
        assert_eq!(map.get("host"), Some(&"localhost".to_string()));
    }
}
