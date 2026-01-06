//! RDF graph integration for ontology-driven code generation

use crate::integration::ggen::error::{GgenError, GgenResult};

/// Wrapper around ggen-core Graph for RDF operations
///
/// Provides an ergonomic interface for loading and querying RDF graphs.
pub struct GgenGraph {
    inner: ggen_core::Graph,
}

impl GgenGraph {
    /// Create a new empty RDF graph
    pub fn new() -> GgenResult<Self> {
        let graph = ggen_core::Graph::new().map_err(|e| GgenError::Core(e))?;
        Ok(Self { inner: graph })
    }

    /// Load Turtle RDF data into the graph
    pub fn load_turtle(&self, turtle: &str) -> GgenResult<()> {
        self.inner
            .insert_turtle(turtle)
            .map_err(|e| GgenError::graph_error(format!("Failed to load Turtle: {}", e)))?;
        Ok(())
    }

    /// Execute a SPARQL query against the graph
    pub fn query(&self, sparql: &str) -> GgenResult<Vec<ggen_core::graph::QueryResult>> {
        self.inner
            .query(sparql)
            .map_err(|e| GgenError::graph_error(format!("SPARQL query failed: {}", e)))
    }

    /// Get the inner ggen-core Graph for advanced usage
    pub fn inner(&self) -> &ggen_core::Graph {
        &self.inner
    }

    /// Consume self and return the inner graph
    pub fn into_inner(self) -> ggen_core::Graph {
        self.inner
    }
}

impl Default for GgenGraph {
    fn default() -> Self {
        // Use unwrap here since this is a convenience method
        // Real error handling should use new()
        Self::new().expect("Failed to create default graph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        // Arrange & Act
        let graph = GgenGraph::new();

        // Assert
        assert!(graph.is_ok());
    }

    #[test]
    fn test_graph_load_turtle() {
        // Arrange
        let graph = GgenGraph::new().unwrap();
        let turtle = r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ;
                     ex:name "Alice" .
        "#;

        // Act
        let result = graph.load_turtle(turtle);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_query() {
        // Arrange
        let graph = GgenGraph::new().unwrap();
        let turtle = r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ;
                     ex:name "Alice" .
        "#;
        graph.load_turtle(turtle).unwrap();

        // Act
        let results = graph.query("SELECT ?s ?o WHERE { ?s ex:name ?o }");

        // Assert
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_graph_invalid_turtle() {
        // Arrange
        let graph = GgenGraph::new().unwrap();
        let invalid_turtle = "This is not valid Turtle syntax";

        // Act
        let result = graph.load_turtle(invalid_turtle);

        // Assert
        assert!(result.is_err());
    }
}
