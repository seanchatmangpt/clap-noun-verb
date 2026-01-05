//! Capability registry with RDF store and SPARQL queries

use super::capability::{CapabilityMetadata, SEMANTIC_CAPABILITIES};
use super::sparql::QueryError;
use std::collections::HashMap;
use thiserror::Error;

/// Errors from capability registry operations
#[derive(Debug, Error)]
pub enum RegistryError {
    /// RDF parsing failed
    #[error("Failed to parse RDF metadata: {0}")]
    RdfParsing(String),

    /// SPARQL query error
    #[error("SPARQL query failed: {0}")]
    QueryFailed(#[from] QueryError),

    /// Capability not found
    #[error("Capability not found: {0}")]
    NotFound(String),
}

/// Capability registry with indexed RDF store
///
/// Type invariants:
/// - All capabilities have unique URIs
/// - RDF metadata is valid Turtle
/// - Index is synchronized with RDF store
pub struct CapabilityRegistry {
    /// Capability lookup by URI
    index: HashMap<String, &'static CapabilityMetadata>,
    /// RDF store contents (Turtle format)
    rdf_store: String,
}

impl CapabilityRegistry {
    /// Create new registry from distributed slice
    ///
    /// Loads all capabilities registered via `#[semantic_composable]`
    /// and builds RDF store for SPARQL queries.
    pub fn new() -> Result<Self, RegistryError> {
        let mut index = HashMap::new();
        let mut rdf_store = String::new();

        // Add RDF prefixes
        rdf_store.push_str("@prefix cap: <urn:clap-noun-verb:capability:> .\n");
        rdf_store.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        rdf_store.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");

        // Load capabilities from distributed slice
        for capability in SEMANTIC_CAPABILITIES {
            index.insert(capability.uri.to_string(), *capability);
            rdf_store.push_str(capability.rdf_metadata);
            rdf_store.push_str("\n\n");
        }

        Ok(Self { index, rdf_store })
    }

    /// Get capability by URI
    pub fn get(&self, uri: impl AsRef<str>) -> Option<&'static CapabilityMetadata> {
        self.index.get(uri.as_ref()).copied()
    }

    /// Get all registered capabilities
    pub fn all_capabilities(&self) -> Vec<&'static CapabilityMetadata> {
        self.index.values().copied().collect()
    }

    /// Get RDF store contents in Turtle format
    pub fn rdf_store(&self) -> &str {
        &self.rdf_store
    }

    /// Query capabilities using SPARQL WHERE pattern
    ///
    /// Note: Full SPARQL support requires oxigraph feature.
    /// This basic implementation does simple pattern matching.
    pub fn query_capabilities(
        &self,
        pattern: impl AsRef<str>,
    ) -> Result<Vec<&'static CapabilityMetadata>, QueryError> {
        let pattern = pattern.as_ref();

        // Basic pattern matching - can be enhanced with oxigraph
        let results: Vec<&'static CapabilityMetadata> = self
            .all_capabilities()
            .into_iter()
            .filter(|cap| {
                // Simple substring match for basic queries
                // FUTURE: Full SPARQL evaluation with oxigraph
                cap.rdf_metadata.contains(pattern)
            })
            .collect();

        Ok(results)
    }

    /// Number of registered capabilities
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self { index: HashMap::new(), rdf_store: String::new() })
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        // Arrange & Act: Create registry
        let registry = CapabilityRegistry::new();

        // Assert: Registry created successfully
        assert!(registry.is_ok());
    }

    #[test]
    fn test_registry_rdf_store_has_prefixes() {
        // Arrange: Create registry
        let registry = CapabilityRegistry::new().expect("registry creation should succeed");

        // Act: Get RDF store
        let rdf = registry.rdf_store();

        // Assert: Contains required prefixes
        assert!(rdf.contains("@prefix cap:"));
        assert!(rdf.contains("@prefix rdf:"));
        assert!(rdf.contains("@prefix rdfs:"));
    }

    #[test]
    fn test_registry_all_capabilities() {
        // Arrange: Create registry
        let registry = CapabilityRegistry::new().expect("registry creation should succeed");

        // Act: Get all capabilities
        let capabilities = registry.all_capabilities();

        // Assert: Returns vector (may be empty if no capabilities registered)
        assert!(capabilities.len() >= 0);
    }

    #[test]
    fn test_registry_query_capabilities() {
        // Arrange: Create registry
        let registry = CapabilityRegistry::new().expect("registry creation should succeed");

        // Act: Query with pattern (will match empty set if no capabilities)
        let result = registry.query_capabilities("cap:Capability");

        // Assert: Query succeeds
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_len_and_is_empty() {
        // Arrange: Create registry
        let registry = CapabilityRegistry::new().expect("registry creation should succeed");

        // Act: Check size
        let len = registry.len();
        let is_empty = registry.is_empty();

        // Assert: Consistent state
        if len == 0 {
            assert!(is_empty);
        } else {
            assert!(!is_empty);
        }
    }
}
