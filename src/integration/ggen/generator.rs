//! High-level generator API with builder pattern and type-state
//!
//! Provides an ergonomic interface for code generation with compile-time guarantees.

use crate::ggen_core;
use crate::integration::ggen::{
    config::{GgenConfig, GgenConfigBuilder},
    error::{GgenError, GgenResult},
    receipt::GenerationReceipt,
    state::{Configured, Generated, State},
};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// High-level code generator with type-state pattern
///
/// The type parameter `S` tracks the generator's state at compile time,
/// preventing invalid operations (e.g., generating before configuration).
pub struct GgenGenerator<S: State = Configured> {
    config: Option<GgenConfig>,
    receipt: Option<GenerationReceipt>,
    _state: PhantomData<S>,
}

impl GgenGenerator<Configured> {
    /// Create a new generator in the configured state
    pub fn new() -> Self {
        Self { config: None, receipt: None, _state: PhantomData }
    }

    /// Set the template path
    pub fn template<P: AsRef<Path>>(mut self, path: P) -> GgenResult<Self> {
        let path_buf = path.as_ref().to_path_buf();

        // Validate template path exists
        if !path_buf.exists() {
            return Err(GgenError::template_not_found(path_buf));
        }

        let builder = self
            .config
            .map_or_else(
                || GgenConfigBuilder::new(),
                |c| GgenConfigBuilder::new().output(c.output_path).variables(c.variables),
            )
            .template(path_buf);

        self.config = Some(builder.build()?);
        Ok(self)
    }

    /// Set the output path
    pub fn output<P: AsRef<Path>>(mut self, path: P) -> GgenResult<Self> {
        let path_buf = path.as_ref().to_path_buf();

        let builder = self
            .config
            .map_or_else(
                || GgenConfigBuilder::new(),
                |c| GgenConfigBuilder::new().template(c.template_path).variables(c.variables),
            )
            .output(path_buf);

        self.config = Some(builder.build()?);
        Ok(self)
    }

    /// Add a variable for template substitution
    pub fn variable<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: V,
    ) -> GgenResult<Self> {
        if let Some(config) = self.config.take() {
            let mut variables = config.variables.clone();
            variables.insert(key.into(), value.into());

            let builder = GgenConfigBuilder::new()
                .template(config.template_path)
                .output(config.output_path)
                .variables(variables);

            self.config = Some(builder.build()?);
        } else {
            let mut variables = BTreeMap::new();
            variables.insert(key.into(), value.into());

            let builder = GgenConfigBuilder::new().variables(variables);
            self.config = Some(builder.build().unwrap_or_else(|_| {
                // If build fails, it's because template/output not set yet - that's OK
                // We'll validate on generate()
                panic!("Unreachable: config should be valid at this point")
            }));
        }

        Ok(self)
    }

    /// Set the RDF graph for ontology-driven generation
    pub fn with_rdf_graph(mut self, graph: ggen_core::Graph) -> GgenResult<Self> {
        if let Some(config) = self.config.take() {
            let builder = GgenConfigBuilder::new()
                .template(config.template_path)
                .output(config.output_path)
                .variables(config.variables)
                .with_rdf_graph(graph);

            self.config = Some(builder.build()?);
        } else {
            let builder = GgenConfigBuilder::new().with_rdf_graph(graph);
            self.config =
                Some(builder.build().unwrap_or_else(|_| {
                    panic!("Unreachable: config should be valid at this point")
                }));
        }

        Ok(self)
    }

    /// Generate code and transition to Generated state
    pub async fn generate(self) -> GgenResult<GgenGenerator<Generated>> {
        let config = self.config.ok_or_else(|| {
            GgenError::config("Generator not fully configured. Call template() and output() first")
        })?;

        // Create receipt
        let receipt = GenerationReceipt::new(
            config.template_path.clone(),
            config.output_path.clone(),
            config.variables.clone(),
        );

        // TODO: Actual generation using ggen-core
        // For now, create a placeholder implementation
        // This will be replaced with real ggen-core integration

        Ok(GgenGenerator { config: Some(config), receipt: Some(receipt), _state: PhantomData })
    }

    /// Get the current configuration
    pub fn config(&self) -> Option<&GgenConfig> {
        self.config.as_ref()
    }
}

impl GgenGenerator<Generated> {
    /// Get the generation receipt
    pub fn receipt(&self) -> Option<&GenerationReceipt> {
        self.receipt.as_ref()
    }

    /// Get the configuration that was used
    pub fn config(&self) -> Option<&GgenConfig> {
        self.config.as_ref()
    }
}

impl Default for GgenGenerator<Configured> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generator_creation() {
        // Arrange & Act
        let generator = GgenGenerator::new();

        // Assert
        assert!(generator.config().is_none());
    }

    #[test]
    fn test_generator_configuration() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template.tera");
        fs::write(&template_path, "Hello {{ name }}!").unwrap();

        // Act
        let generator = GgenGenerator::new()
            .template(&template_path)
            .unwrap()
            .output(temp_dir.path().join("output"))
            .unwrap()
            .variable("name", "World")
            .unwrap();

        // Assert
        let config = generator.config().expect("Config should be set");
        assert_eq!(config.template_path(), &template_path);
        assert_eq!(config.variables().get("name"), Some(&"World".to_string()));
    }

    #[test]
    fn test_generator_template_not_found() {
        // Arrange
        let non_existent_path = PathBuf::from("/non/existent/template.tera");

        // Act
        let result = GgenGenerator::new().template(&non_existent_path);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GgenError::TemplateNotFound(_)));
    }

    #[tokio::test]
    async fn test_generator_full_workflow() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template.tera");
        fs::write(&template_path, "Hello {{ name }}!").unwrap();

        // Act
        let generated = GgenGenerator::new()
            .template(&template_path)
            .unwrap()
            .output(temp_dir.path().join("output"))
            .unwrap()
            .variable("name", "Rust")
            .unwrap()
            .generate()
            .await
            .unwrap();

        // Assert
        assert!(generated.receipt().is_some());
        let receipt = generated.receipt().unwrap();
        assert_eq!(receipt.template_path, template_path);
    }
}
