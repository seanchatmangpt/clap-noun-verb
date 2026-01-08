//! Configuration types for ggen integration
//!
//! Provides builder pattern for ergonomic configuration with compile-time guarantees.

use crate::integration::ggen::error::{GgenError, GgenResult};
use crate::ggen_core;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Configuration for ggen code generation
///
/// Use [`GgenConfigBuilder`] for ergonomic construction.
#[derive(Debug, Clone)]
pub struct GgenConfig {
    /// Path to the template file
    pub(crate) template_path: PathBuf,

    /// Path to the output directory or file
    pub(crate) output_path: PathBuf,

    /// Template variables for substitution
    pub(crate) variables: BTreeMap<String, String>,

    /// Optional RDF graph for ontology-driven generation
    pub(crate) rdf_graph: Option<ggen_core::Graph>,
}

impl GgenConfig {
    /// Create a new builder for GgenConfig
    pub fn builder() -> GgenConfigBuilder {
        GgenConfigBuilder::new()
    }

    /// Get the template path
    pub fn template_path(&self) -> &Path {
        &self.template_path
    }

    /// Get the output path
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    /// Get the variables
    pub fn variables(&self) -> &BTreeMap<String, String> {
        &self.variables
    }

    /// Get the RDF graph if present
    pub fn rdf_graph(&self) -> Option<&ggen_core::Graph> {
        self.rdf_graph.as_ref()
    }
}

/// Builder for GgenConfig following the builder pattern
#[derive(Debug, Default)]
pub struct GgenConfigBuilder {
    template_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    variables: BTreeMap<String, String>,
    rdf_graph: Option<ggen_core::Graph>,
}

impl GgenConfigBuilder {
    /// Create a new config builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the template path
    pub fn template<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.template_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the output path
    pub fn output<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add a variable for template substitution
    pub fn variable<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add multiple variables at once
    pub fn variables(mut self, vars: BTreeMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Set the RDF graph for ontology-driven generation
    pub fn with_rdf_graph(mut self, graph: ggen_core::Graph) -> Self {
        self.rdf_graph = Some(graph);
        self
    }

    /// Build the configuration
    ///
    /// Returns an error if required fields are missing
    pub fn build(self) -> GgenResult<GgenConfig> {
        let template_path =
            self.template_path.ok_or_else(|| GgenError::config("Template path is required"))?;

        let output_path =
            self.output_path.ok_or_else(|| GgenError::config("Output path is required"))?;

        // Validate paths are not empty
        if template_path.as_os_str().is_empty() {
            return Err(GgenError::EmptyPath);
        }

        if output_path.as_os_str().is_empty() {
            return Err(GgenError::EmptyPath);
        }

        Ok(GgenConfig {
            template_path,
            output_path,
            variables: self.variables,
            rdf_graph: self.rdf_graph,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_basic() {
        // Arrange & Act
        let config = GgenConfigBuilder::new()
            .template("templates/test.tera")
            .output("output")
            .build()
            .expect("Failed to build config");

        // Assert
        assert_eq!(config.template_path(), Path::new("templates/test.tera"));
        assert_eq!(config.output_path(), Path::new("output"));
        assert!(config.variables().is_empty());
    }

    #[test]
    fn test_config_builder_with_variables() {
        // Arrange & Act
        let config = GgenConfigBuilder::new()
            .template("templates/test.tera")
            .output("output")
            .variable("name", "test")
            .variable("version", "1.0.0")
            .build()
            .expect("Failed to build config");

        // Assert
        assert_eq!(config.variables().get("name"), Some(&"test".to_string()));
        assert_eq!(config.variables().get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_config_builder_missing_template() {
        // Arrange
        let builder = GgenConfigBuilder::new().output("output");

        // Act
        let result = builder.build();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Template path"));
    }

    #[test]
    fn test_config_builder_missing_output() {
        // Arrange
        let builder = GgenConfigBuilder::new().template("template.tera");

        // Act
        let result = builder.build();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Output path"));
    }
}
