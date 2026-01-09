//! Dynamic CLI runtime builder from discovered capabilities

use super::capability::CapabilityMetadata;
use clap::Command;
use thiserror::Error;

/// Errors from runtime CLI building
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// No capabilities provided
    #[error("Cannot build CLI from empty capability set")]
    EmptyCapabilities,

    /// Invalid CLI configuration
    #[error("Invalid CLI configuration: {0}")]
    InvalidConfiguration(String),

    /// Duplicate capability URI
    #[error("Duplicate capability URI: {0}")]
    DuplicateUri(String),
}

/// Dynamic CLI runtime builder
///
/// Constructs clap Command structures from discovered capabilities.
///
/// Type invariants:
/// - All capabilities have unique URIs
/// - Generated CLI is valid clap configuration
pub struct RuntimeBuilder {
    /// Capabilities to include in CLI
    capabilities: Vec<&'static CapabilityMetadata>,
    /// CLI name
    name: Option<String>,
    /// CLI version
    version: Option<String>,
}

impl RuntimeBuilder {
    /// Create new runtime builder
    pub fn new() -> Self {
        Self { capabilities: Vec::new(), name: None, version: None }
    }

    /// Set CLI name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set CLI version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add single capability
    pub fn add_capability(mut self, capability: &'static CapabilityMetadata) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add multiple capabilities
    pub fn add_capabilities(mut self, capabilities: &[&'static CapabilityMetadata]) -> Self {
        self.capabilities.extend_from_slice(capabilities);
        self
    }

    /// Build clap Command from capabilities
    ///
    /// # Returns
    ///
    /// `Command` with subcommands for each capability
    ///
    /// # Errors
    ///
    /// Returns `RuntimeError` if configuration is invalid
    pub fn build(self) -> Result<Command, RuntimeError> {
        if self.capabilities.is_empty() {
            return Err(RuntimeError::EmptyCapabilities);
        }

        // Check for duplicate URIs
        let mut seen = std::collections::HashSet::new();
        for cap in &self.capabilities {
            if !seen.insert(cap.uri) {
                return Err(RuntimeError::DuplicateUri(cap.uri.to_string()));
            }
        }

        // Create base command
        let mut cmd = if let Some(name) = self.name {
            let name_str: &'static str = Box::leak(name.into_boxed_str());
            Command::new(name_str)
        } else {
            Command::new("semantic-cli")
        };

        if let Some(version) = self.version {
            let version_str: &'static str = Box::leak(version.into_boxed_str());
            cmd = cmd.version(version_str);
        }

        // Add subcommand for each capability
        for capability in &self.capabilities {
            let subcmd = Command::new(capability.function_name)
                .about(format!("Capability: {}", capability.uri));

            cmd = cmd.subcommand(subcmd);
        }

        Ok(cmd)
    }

    /// Get number of capabilities
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if builder is empty
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_capability(
        uri: &'static str,
        name: &'static str,
    ) -> &'static CapabilityMetadata {
        Box::leak(Box::new(CapabilityMetadata {
            uri,
            function_name: name,
            rdf_metadata: "",
            mcp_descriptor: "{}",
        }))
    }

    #[test]
    fn test_builder_new() {
        // Arrange & Act: Create builder
        let builder = RuntimeBuilder::new();

        // Assert: Empty builder
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_builder_name() {
        // Arrange: Builder with name
        let builder = RuntimeBuilder::new().name("test-cli");

        // Assert: Name set
        assert_eq!(builder.name, Some("test-cli".to_string()));
    }

    #[test]
    fn test_builder_version() {
        // Arrange: Builder with version
        let builder = RuntimeBuilder::new().version("1.0.0");

        // Assert: Version set
        assert_eq!(builder.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_builder_add_capability() {
        // Arrange: Builder and capability
        let capability = create_test_capability("urn:test:cap1", "cap1");

        // Act: Add capability
        let builder = RuntimeBuilder::new().add_capability(capability);

        // Assert: Capability added
        assert_eq!(builder.len(), 1);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_builder_add_capabilities() {
        // Arrange: Builder and capabilities
        let cap1 = create_test_capability("urn:test:cap1", "cap1");
        let cap2 = create_test_capability("urn:test:cap2", "cap2");
        let capabilities = vec![cap1, cap2];

        // Act: Add capabilities
        let builder = RuntimeBuilder::new().add_capabilities(&capabilities);

        // Assert: All capabilities added
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn test_build_empty_capabilities() {
        // Arrange: Empty builder
        let builder = RuntimeBuilder::new();

        // Act: Build
        let result = builder.build();

        // Assert: Fails with empty capabilities error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::EmptyCapabilities));
    }

    #[test]
    fn test_build_single_capability() {
        // Arrange: Builder with capability
        let capability = create_test_capability("urn:test:cap1", "cap1");
        let builder = RuntimeBuilder::new().add_capability(capability);

        // Act: Build
        let result = builder.build();

        // Assert: Succeeds
        assert!(result.is_ok());
        let cmd = result.expect("build should succeed");
        assert_eq!(cmd.get_name(), "semantic-cli");
    }

    #[test]
    fn test_build_multiple_capabilities() {
        // Arrange: Builder with multiple capabilities
        let cap1 = create_test_capability("urn:test:cap1", "cap1");
        let cap2 = create_test_capability("urn:test:cap2", "cap2");
        let builder = RuntimeBuilder::new().add_capability(cap1).add_capability(cap2);

        // Act: Build
        let result = builder.build();

        // Assert: Succeeds with all subcommands
        assert!(result.is_ok());
        let cmd = result.expect("build should succeed");
        let subcommands: Vec<_> = cmd.get_subcommands().collect();
        assert_eq!(subcommands.len(), 2);
    }

    #[test]
    fn test_build_duplicate_uri() {
        // Arrange: Builder with duplicate URIs
        let cap1 = create_test_capability("urn:test:dup", "cap1");
        let cap2 = create_test_capability("urn:test:dup", "cap2");
        let builder = RuntimeBuilder::new().add_capability(cap1).add_capability(cap2);

        // Act: Build
        let result = builder.build();

        // Assert: Fails with duplicate URI error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DuplicateUri(_)));
    }

    #[test]
    fn test_build_with_name_and_version() {
        // Arrange: Builder with name and version
        let capability = create_test_capability("urn:test:cap1", "cap1");
        let builder =
            RuntimeBuilder::new().name("my-cli").version("2.0.0").add_capability(capability);

        // Act: Build
        let result = builder.build();

        // Assert: CLI has correct name and version
        assert!(result.is_ok());
        let cmd = result.expect("build should succeed");
        assert_eq!(cmd.get_name(), "my-cli");
        assert_eq!(cmd.get_version(), Some("2.0.0"));
    }
}
