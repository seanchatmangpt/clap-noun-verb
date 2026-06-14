// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Query Command - Query RDF data with simple patterns

use serde::{Deserialize, Serialize};

/// Result from graph query operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultOutput {
    /// Query type used ("subject", "predicate", "object", or "all").
    pub query_type: String,
    /// Pattern matched against triple components.
    pub pattern: String,
    /// Matching triples.
    pub results: Vec<QueryMatch>,
    /// Number of matches found.
    pub match_count: usize,
}

/// A single triple matched by a graph query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMatch {
    /// Index of the match within the result set.
    pub index: usize,
    /// Subject of the matched triple.
    pub subject: String,
    /// Predicate of the matched triple.
    pub predicate: String,
    /// Object of the matched triple.
    pub object: String,
}

impl QueryResultOutput {
    /// Create an empty result for the given query type and pattern.
    pub fn new(query_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            query_type: query_type.into(),
            pattern: pattern.into(),
            results: Vec::new(),
            match_count: 0,
        }
    }

    /// Attach matches, updating `match_count` to the result length.
    pub fn with_results(mut self, results: Vec<QueryMatch>) -> Self {
        self.match_count = results.len();
        self.results = results;
        self
    }
}

fn execute_query(query_type: &str, pattern: &str, graph: &super::Graph) -> Vec<QueryMatch> {
    let triples = match query_type {
        "subject" => graph.query_by_subject(pattern),
        "predicate" => graph.query_by_predicate(pattern),
        "object" => {
            graph.triples().iter().filter(|t| t.object.contains(pattern)).cloned().collect()
        }
        _ => graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.contains(pattern)
                    || t.predicate.contains(pattern)
                    || t.object.contains(pattern)
            })
            .cloned()
            .collect(),
    };
    triples
        .into_iter()
        .enumerate()
        .map(|(i, t)| QueryMatch {
            index: i,
            subject: t.subject,
            predicate: t.predicate,
            object: t.object,
        })
        .collect()
}

/// Domain logic: Parse query string
fn parse_query_string(query_string: &str) -> crate::Result<(String, String)> {
    let parts: Vec<&str> = query_string.split(':').collect();

    if parts.is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Invalid query format".to_string(),
        ));
    }

    let query_type = if parts.len() > 1 { parts[0].to_string() } else { "all".to_string() };
    let pattern = if parts.len() > 1 { parts[1..].join(":") } else { query_string.to_string() };

    if pattern.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Query pattern cannot be empty".to_string(),
        ));
    }

    Ok((query_type, pattern))
}

/// Query the graph using simple pattern matching
///
/// Supports basic SPARQL-like queries with pattern matching on
/// subject, predicate, or object. Patterns use simple substring matching.
///
/// Uses the generated query planner for optimized execution and result formatting.
///
/// # Arguments
/// * `query_string` - Query pattern (e.g., "subject:ex:alice" or "predicate:rdf:type")
///
/// # Example
/// ```text
/// myapp graph query "subject:ex:alice"
/// ```
pub fn query_graph(query_string: String) -> crate::Result<QueryResultOutput> {
    // There is no global graph store in this codebase — graph state is not shared
    // across verb handler calls. Returning zero results from an empty graph would
    // silently mislead the user. Return a clear error instead.
    Err(crate::error::NounVerbError::execution_error(
        "No graph loaded — use 'graph load' first to load RDF data before querying. \
         Query string was: "
            .to_string()
            + &query_string,
    ))
}

/// Query the graph using generated planner
///
/// This is the preferred implementation using the generated QueryPlanner.
/// It provides better performance and SPARQL-like semantics.
pub fn query_graph_generated(
    query_string: String,
    graph: &super::Graph,
) -> crate::Result<QueryResultOutput> {
    let mut planner = super::impl_generated::QueryPlanner::new();
    planner.execute(&query_string, graph)?;

    let (query_type, pattern) = parse_query_string(&query_string)?;
    let results = execute_query(&query_type, &pattern, graph);

    Ok(QueryResultOutput::new(query_type, pattern).with_results(results))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_subject() {
        let (qtype, pattern) = parse_query_string("subject:ex:alice").unwrap();
        assert_eq!(qtype, "subject");
        assert_eq!(pattern, "ex:alice");
    }

    #[test]
    fn test_parse_query_empty_pattern() {
        let result = parse_query_string(":");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_query_subject() {
        let mut graph = super::super::Graph::new();
        graph.add_triple(super::super::Triple::new("ex:test", "rdf:type", "ex:Entity")).unwrap();
        graph.add_triple(super::super::Triple::new("ex:test", "foaf:name", "Alice")).unwrap();
        let results = execute_query("subject", "ex:test", &graph);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].subject, "ex:test");
    }
}
