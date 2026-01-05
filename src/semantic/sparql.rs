//! SPARQL query interface for capability discovery

use thiserror::Error;

/// Errors from SPARQL query operations
#[derive(Debug, Error)]
pub enum QueryError {
    /// Invalid SPARQL syntax
    #[error("Invalid SPARQL query: {0}")]
    InvalidSyntax(String),

    /// Query execution failed
    #[error("Query execution failed: {0}")]
    ExecutionFailed(String),

    /// No results found
    #[error("Query returned no results")]
    NoResults,
}

/// SPARQL query result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    /// Result bindings
    pub bindings: Vec<Binding>,
}

/// Variable binding in SPARQL result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// Variable name
    pub variable: String,
    /// Bound value (URI or literal)
    pub value: String,
}

/// SPARQL query engine for capability discovery
///
/// Type invariants:
/// - Queries are valid SPARQL WHERE patterns
/// - Results are valid RDF terms
pub struct QueryEngine {
    /// RDF store contents
    rdf_store: String,
}

impl QueryEngine {
    /// Create new query engine with RDF store
    pub fn new(rdf_store: impl Into<String>) -> Self {
        Self { rdf_store: rdf_store.into() }
    }

    /// Execute SPARQL SELECT query
    ///
    /// Note: Full SPARQL support requires oxigraph feature.
    /// This basic implementation does simple pattern matching.
    ///
    /// # Arguments
    ///
    /// - `query`: SPARQL query string
    ///
    /// # Returns
    ///
    /// `QueryResult` with variable bindings
    ///
    /// # Errors
    ///
    /// Returns `QueryError` if query is invalid or execution fails
    pub fn query(&self, query: impl AsRef<str>) -> Result<QueryResult, QueryError> {
        let query = query.as_ref();

        // Basic validation
        if query.is_empty() {
            return Err(QueryError::InvalidSyntax("empty query".to_string()));
        }

        // FUTURE: Full SPARQL evaluation with oxigraph
        // For now, return empty results
        Ok(QueryResult { bindings: Vec::new() })
    }

    /// Execute SPARQL ASK query
    ///
    /// Returns true if pattern matches, false otherwise.
    pub fn ask(&self, pattern: impl AsRef<str>) -> Result<bool, QueryError> {
        let pattern = pattern.as_ref();

        // Simple substring match for basic patterns
        Ok(self.rdf_store.contains(pattern))
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_engine_new() {
        // Arrange & Act: Create query engine
        let engine = QueryEngine::new("test RDF data");

        // Assert: Engine created
        assert_eq!(engine.rdf_store, "test RDF data");
    }

    #[test]
    fn test_query_empty() {
        // Arrange: Engine with empty query
        let engine = QueryEngine::new("test data");

        // Act: Execute empty query
        let result = engine.query("");

        // Assert: Fails with invalid syntax
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::InvalidSyntax(_)));
    }

    #[test]
    fn test_query_valid() {
        // Arrange: Engine and valid query
        let engine = QueryEngine::new("test data");

        // Act: Execute query
        let result = engine.query("SELECT * WHERE { ?s ?p ?o }");

        // Assert: Succeeds (even if empty results)
        assert!(result.is_ok());
    }

    #[test]
    fn test_ask_pattern_match() {
        // Arrange: Engine with known data
        let engine = QueryEngine::new("cap:Capability rdf:type rdfs:Class");

        // Act: Ask if pattern exists
        let result = engine.ask("cap:Capability");

        // Assert: Returns true
        assert!(result.is_ok());
        assert!(result.expect("ask should succeed"));
    }

    #[test]
    fn test_ask_pattern_no_match() {
        // Arrange: Engine with known data
        let engine = QueryEngine::new("cap:Capability rdf:type rdfs:Class");

        // Act: Ask if non-existent pattern exists
        let result = engine.ask("cap:NonExistent");

        // Assert: Returns false
        assert!(result.is_ok());
        assert!(!result.expect("ask should succeed"));
    }

    #[test]
    fn test_query_result_equality() {
        // Arrange: Create two equal results
        let result1 = QueryResult {
            bindings: vec![Binding { variable: "s".to_string(), value: "urn:test".to_string() }],
        };
        let result2 = QueryResult {
            bindings: vec![Binding { variable: "s".to_string(), value: "urn:test".to_string() }],
        };

        // Act & Assert: Results are equal
        assert_eq!(result1, result2);
    }
}
