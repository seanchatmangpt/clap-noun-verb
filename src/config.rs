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
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            path: None,
            default_paths: vec![
                PathBuf::from(".env.yaml"),
                PathBuf::from("config.yaml"),
                PathBuf::from("config.yml"),
                PathBuf::from(".config/app.yaml"),
            ],
        }
    }

    /// Set explicit config file path
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add a default path to search for config file
    pub fn with_default_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.default_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<Config> {
        let path = if let Some(ref p) = self.path {
            p.clone()
        } else {
            // Try default paths
            self.default_paths
                .iter()
                .find(|p| p.exists())
                .cloned()
                .ok_or_else(|| {
                    crate::error::NounVerbError::execution_error(
                        "No configuration file found",
                    )
                })?
        };

        Config::from_file(&path)
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
        Self {
            data: json!({}),
        }
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

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("json");

        let data = match extension {
            "yaml" | "yml" => {
                serde_yaml::from_str(&content).map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!(
                        "Failed to parse YAML: {}",
                        e
                    ))
                })?
            }
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
            _ => {
                serde_json::from_str(&content).map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!(
                        "Failed to parse JSON: {}",
                        e
                    ))
                })?
            }
        };

        Ok(Self { data })
    }

    /// Load configuration from JSON string
    pub fn from_json(json_str: &str) -> Result<Self> {
        let data = serde_json::from_str(json_str).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to parse JSON: {}",
                e
            ))
        })?;
        Ok(Self { data })
    }

    /// Load configuration from YAML string
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        let data = serde_yaml::from_str(yaml_str).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to parse YAML: {}",
                e
            ))
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
    fn flatten_to_args(&self, value: &Value, prefix: String, args: &mut Vec<String>) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj.iter() {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
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
    fn flatten_to_map(&self, value: &Value, prefix: String, map: &mut HashMap<String, String>) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj.iter() {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
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
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
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
