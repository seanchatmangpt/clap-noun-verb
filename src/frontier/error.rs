//! Error types for frontier package integration
//!
//! This module provides comprehensive error handling for RDF/Semantic operations
//! following Rust best practices with thiserror.

use thiserror::Error;

/// Result type alias for frontier operations
pub type Result<T> = std::result::Result<T, FrontierError>;

/// Comprehensive error type for frontier package operations
#[derive(Error, Debug)]
pub enum FrontierError {
    /// RDF parsing or validation error
    #[error("RDF error: {0}")]
    Rdf(String),

    /// SPARQL query parsing or execution error
    #[error("SPARQL error: {0}")]
    Sparql(String),

    /// JSON-LD processing error
    #[error("JSON-LD error: {0}")]
    JsonLd(String),

    /// Type erasure or serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Capability discovery error
    #[error("Discovery error: {0}")]
    Discovery(String),

    /// Invalid IRI or namespace error
    #[error("Invalid IRI: {0}")]
    InvalidIri(String),

    /// Graph operation error
    #[error("Graph error: {0}")]
    Graph(String),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error for unexpected conditions
    #[error("Internal error: {0}")]
    Internal(String),
}

// Convert from oxrdf errors
#[cfg(feature = "rdf-composition")]
impl From<oxrdf::IriParseError> for FrontierError {
    fn from(err: oxrdf::IriParseError) -> Self {
        FrontierError::InvalidIri(err.to_string())
    }
}

// Convert from serde_json errors
impl From<serde_json::Error> for FrontierError {
    fn from(err: serde_json::Error) -> Self {
        FrontierError::Serialization(err.to_string())
    }
}

// Convert from oxigraph errors
#[cfg(feature = "rdf-composition")]
impl From<oxigraph::store::StorageError> for FrontierError {
    fn from(err: oxigraph::store::StorageError) -> Self {
        FrontierError::Graph(err.to_string())
    }
}

#[cfg(feature = "rdf-composition")]
impl From<oxigraph::sparql::QueryParseError> for FrontierError {
    fn from(err: oxigraph::sparql::QueryParseError) -> Self {
        FrontierError::Sparql(err.to_string())
    }
}

#[cfg(feature = "rdf-composition")]
impl From<oxigraph::sparql::EvaluationError> for FrontierError {
    fn from(err: oxigraph::sparql::EvaluationError) -> Self {
        FrontierError::Sparql(err.to_string())
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // Arrange: Create various errors
        let rdf_err = FrontierError::Rdf("invalid triple".to_string());
        let sparql_err = FrontierError::Sparql("syntax error".to_string());
        let iri_err = FrontierError::InvalidIri("malformed IRI".to_string());

        // Act: Convert to string
        let rdf_msg = rdf_err.to_string();
        let sparql_msg = sparql_err.to_string();
        let iri_msg = iri_err.to_string();

        // Assert: Error messages are descriptive
        assert!(rdf_msg.contains("RDF error"));
        assert!(rdf_msg.contains("invalid triple"));
        assert!(sparql_msg.contains("SPARQL error"));
        assert!(sparql_msg.contains("syntax error"));
        assert!(iri_msg.contains("Invalid IRI"));
        assert!(iri_msg.contains("malformed IRI"));
    }

    #[test]
    fn test_result_type_alias() {
        // Arrange: Create result types
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(FrontierError::Internal("test error".to_string()));

        // Act: Pattern match
        let success_value = success.is_ok();
        let failure_value = failure.is_err();

        // Assert: Type alias works correctly
        assert!(success_value);
        assert!(failure_value);
    }

    #[test]
    fn test_error_conversion_from_serde_json() {
        // Arrange: Create a malformed JSON string
        let json_str = "{invalid json}";

        // Act: Parse and convert error
        let result: Result<serde_json::Value> =
            serde_json::from_str(json_str).map_err(FrontierError::from);

        // Assert: Conversion produces serialization error
        match result {
            Err(FrontierError::Serialization(msg)) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected serialization error"),
        }
    }
}
