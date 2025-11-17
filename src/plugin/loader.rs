//! Plugin loader for dynamic discovery and loading from manifests.

use super::PluginRegistry;
use std::path::{Path, PathBuf};

/// Plugin manifest metadata from TOML or JSON.
#[derive(Debug, Clone)]
pub struct PluginManifest {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
    /// Plugin description
    description: String,
    /// Plugin entry point
    entry_point: String,
    /// Dependencies
    dependencies: Vec<String>,
}

impl PluginManifest {
    /// Create a new plugin manifest.
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
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a dependency.
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
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

    /// Get the plugin description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the entry point.
    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }

    /// Get the dependencies.
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

/// Plugin loader for discovering and loading plugins.
pub struct PluginLoader {
    /// Plugin manifest directory
    manifest_dir: PathBuf,
    /// Loaded manifests
    manifests: Vec<PluginManifest>,
}

impl PluginLoader {
    /// Create a new plugin loader.
    pub fn new(manifest_dir: impl AsRef<Path>) -> Self {
        Self {
            manifest_dir: manifest_dir.as_ref().to_path_buf(),
            manifests: Vec::new(),
        }
    }

    /// Discover plugins in the manifest directory.
    ///
    /// # Errors
    ///
    /// Returns an error if directory scanning fails.
    pub fn discover(&mut self) -> crate::Result<Vec<String>> {
        let mut discovered = Vec::new();

        // Check if directory exists
        if !self.manifest_dir.exists() {
            return Ok(discovered);
        }

        // Try to read directory
        let entries = std::fs::read_dir(&self.manifest_dir).map_err(|e| {
            crate::NounVerbError::PluginError(format!(
                "Failed to scan plugin directory: {}",
                e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                crate::NounVerbError::PluginError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Load TOML manifests
            if path.extension().map_or(false, |ext| ext == "toml") {
                if let Ok(name) = self.load_toml_manifest(&path) {
                    discovered.push(name);
                }
            }

            // Load JSON manifests
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(name) = self.load_json_manifest(&path) {
                    discovered.push(name);
                }
            }
        }

        Ok(discovered)
    }

    fn load_toml_manifest(&mut self, path: &Path) -> crate::Result<String> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to read manifest: {}", e))
        })?;

        // Basic TOML parsing for plugin metadata
        let mut name = String::new();
        let mut version = String::new();
        let mut entry_point = String::new();

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("name = \"") {
                name = value.trim_end_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("version = \"") {
                version = value.trim_end_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("entry_point = \"") {
                entry_point = value.trim_end_matches('"').to_string();
            }
        }

        if name.is_empty() || version.is_empty() || entry_point.is_empty() {
            return Err(crate::NounVerbError::PluginError(
                "Invalid manifest format".to_string(),
            ));
        }

        let manifest = PluginManifest::new(&name, &version, &entry_point);
        self.manifests.push(manifest);

        Ok(name)
    }

    fn load_json_manifest(&mut self, path: &Path) -> crate::Result<String> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to read manifest: {}", e))
        })?;

        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            crate::NounVerbError::PluginError(format!("Failed to parse JSON manifest: {}", e))
        })?;

        let name = json["name"]
            .as_str()
            .ok_or_else(|| crate::NounVerbError::PluginError("Missing 'name' field".to_string()))?
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

        let mut manifest = PluginManifest::new(&name, &version, &entry_point);

        if let Some(description) = json["description"].as_str() {
            manifest = manifest.with_description(description);
        }

        if let Some(deps) = json["dependencies"].as_array() {
            for dep in deps {
                if let Some(dep_str) = dep.as_str() {
                    manifest = manifest.with_dependency(dep_str);
                }
            }
        }

        self.manifests.push(manifest);
        Ok(name)
    }

    /// Get all loaded manifests.
    pub fn manifests(&self) -> &[PluginManifest] {
        &self.manifests
    }

    /// Get a manifest by name.
    pub fn get_manifest(&self, name: &str) -> Option<&PluginManifest> {
        self.manifests.iter().find(|m| m.name() == name)
    }

    /// Validate manifest dependencies against the registry.
    ///
    /// # Errors
    ///
    /// Returns an error if dependencies are not satisfied.
    pub fn validate_dependencies(
        &self,
        manifest: &PluginManifest,
        registry: &PluginRegistry,
    ) -> crate::Result<()> {
        for dep in manifest.dependencies() {
            if !registry.contains(dep) {
                return Err(crate::NounVerbError::PluginError(format!(
                    "Dependency '{}' not found for plugin '{}'",
                    dep,
                    manifest.name()
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manifest_creation() {
        let manifest = PluginManifest::new("test", "1.0.0", "lib");
        assert_eq!(manifest.name(), "test");
        assert_eq!(manifest.version(), "1.0.0");
        assert_eq!(manifest.entry_point(), "lib");
    }

    #[test]
    fn test_plugin_manifest_with_description() {
        let manifest = PluginManifest::new("test", "1.0.0", "lib")
            .with_description("Test plugin");
        assert_eq!(manifest.description(), "Test plugin");
    }

    #[test]
    fn test_plugin_manifest_with_dependencies() {
        let manifest = PluginManifest::new("test", "1.0.0", "lib")
            .with_dependency("dep1")
            .with_dependency("dep2");
        assert_eq!(manifest.dependencies().len(), 2);
    }

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new("/tmp");
        assert_eq!(loader.manifests().len(), 0);
    }
}
