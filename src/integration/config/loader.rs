//! Plugin manifest loader for TOML and JSON formats.

use super::PluginConfig;
use std::path::Path;

/// Loader for plugin manifest files (TOML or JSON).
pub struct PluginManifestLoader;

impl PluginManifestLoader {
    /// Load a plugin configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_toml(path: impl AsRef<Path>) -> crate::Result<PluginConfig> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to read manifest: {}", e))
        })?;

        Self::parse_toml(&content)
    }

    /// Load a plugin configuration from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_json(path: impl AsRef<Path>) -> crate::Result<PluginConfig> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to read manifest: {}", e))
        })?;

        Self::parse_json(&content)
    }

    /// Parse a TOML manifest string.
    pub fn parse_toml(content: &str) -> crate::Result<PluginConfig> {
        let table: toml::Table = toml::from_str(content).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to parse TOML: {}", e))
        })?;

        let name = table
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::NounVerbError::PluginError("Missing 'name' field".to_string())
            })?
            .to_string();

        let version = table
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::NounVerbError::PluginError("Missing 'version' field".to_string())
            })?
            .to_string();

        let entry_point = table
            .get("entry_point")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::NounVerbError::PluginError("Missing 'entry_point' field".to_string())
            })?
            .to_string();

        let description = table
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let dependencies: Vec<String> = table
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut config = PluginConfig::new(&name, &version, &entry_point);
        config.description = description;
        config.dependencies = dependencies;

        config.validate()?;
        Ok(config)
    }

    /// Parse a JSON manifest string.
    pub fn parse_json(content: &str) -> crate::Result<PluginConfig> {
        let json: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to parse JSON: {}", e))
        })?;

        let name = json["name"]
            .as_str()
            .ok_or_else(|| {
                crate::NounVerbError::PluginError("Missing 'name' field".to_string())
            })?
            .to_string();

        let version = json["version"]
            .as_str()
            .ok_or_else(|| {
                crate::NounVerbError::PluginError("Missing 'version' field".to_string())
            })?
            .to_string();

        let entry_point = json["entry_point"]
            .as_str()
            .ok_or_else(|| {
                crate::NounVerbError::PluginError("Missing 'entry_point' field".to_string())
            })?
            .to_string();

        let description = json["description"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let dependencies: Vec<String> = json["dependencies"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut config = PluginConfig::new(&name, &version, &entry_point);
        config.description = description;
        config.dependencies = dependencies;

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_toml_manifest() {
        let toml = r#"
name = "test-plugin"
version = "1.0.0"
entry_point = "lib.so"
description = "Test plugin"
dependencies = ["dep1", "dep2"]
"#;
        let config = PluginManifestLoader::parse_toml(toml);
        assert!(config.is_ok());
        let cfg = config.unwrap();
        assert_eq!(cfg.name, "test-plugin");
        assert_eq!(cfg.dependencies.len(), 2);
    }

    #[test]
    fn test_parse_json_manifest() {
        let json = r#"
{
  "name": "test-plugin",
  "version": "1.0.0",
  "entry_point": "lib.so",
  "description": "Test plugin",
  "dependencies": ["dep1"]
}
"#;
        let config = PluginManifestLoader::parse_json(json);
        assert!(config.is_ok());
        let cfg = config.unwrap();
        assert_eq!(cfg.name, "test-plugin");
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = r#"name = "test""#; // Missing required fields
        let config = PluginManifestLoader::parse_toml(toml);
        assert!(config.is_err());
    }
}
