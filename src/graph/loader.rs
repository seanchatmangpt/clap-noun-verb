// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Load Command - Load RDF data from file

use serde::{Deserialize, Serialize};

/// Result from graph load operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLoadedOutput {
    /// Number of triples loaded from the source.
    pub triples_loaded: usize,
    /// Source path the triples were loaded from.
    pub source: String,
    /// Load status (set to "success").
    pub status: String,
}

impl GraphLoadedOutput {
    /// Create a successful load result with the loaded triple count and source.
    pub fn new(triples_loaded: usize, source: impl Into<String>) -> Self {
        Self { triples_loaded, source: source.into(), status: "success".to_string() }
    }
}

/// Load RDF graph from a Turtle file
///
/// Reads a Turtle (.ttl) format RDF file and loads triples into the graph.
/// Files must contain valid N-Triples format with subject, predicate, object URIs.
///
/// Uses the generated loader implementation for optimized parsing and validation.
///
/// # Arguments
/// * `path` - Path to the Turtle format RDF file
///
/// # Example
/// ```text
/// myapp graph load data/example.ttl
/// ```
pub fn load_graph(path: String) -> crate::Result<GraphLoadedOutput> {
    let loader = super::impl_generated::GeneratedLoader::new();
    let (_graph, triple_count, source) = loader.load_from_file(&path)?;
    Ok(GraphLoadedOutput::new(triple_count, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_graph("nonexistent.ttl".to_string());
        assert!(result.is_err());
    }
}
