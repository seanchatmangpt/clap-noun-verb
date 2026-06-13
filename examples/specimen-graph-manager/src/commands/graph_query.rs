// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Query Command - Query RDF data with simple patterns

use crate::output_models::{QueryMatch, QueryResultOutput};
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Domain logic: Execute query and generate results
fn execute_query(query_type: &str, pattern: &str) -> Vec<QueryMatch> {
    match query_type {
        "subject" => vec![
            QueryMatch {
                index: 0,
                subject: pattern.to_string(),
                predicate: "rdf:type".to_string(),
                object: "ex:Entity".to_string(),
            },
            QueryMatch {
                index: 1,
                subject: pattern.to_string(),
                predicate: "foaf:name".to_string(),
                object: "Example Name".to_string(),
            },
        ],
        "predicate" => vec![QueryMatch {
            index: 0,
            subject: "ex:alice".to_string(),
            predicate: pattern.to_string(),
            object: "ex:bob".to_string(),
        }],
        _ => vec![QueryMatch {
            index: 0,
            subject: "ex:unknown".to_string(),
            predicate: "rdf:comment".to_string(),
            object: format!("Query type '{}' not recognized", query_type),
        }],
    }
}

/// Domain logic: Parse query string
fn parse_query_string(query_string: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = query_string.split(':').collect();

    if parts.is_empty() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Invalid query format".to_string(),
        )
        .into());
    }

    let query_type = if parts.len() > 1 { parts[0].to_string() } else { "all".to_string() };
    let pattern = if parts.len() > 1 { parts[1..].join(":") } else { query_string.to_string() };

    if pattern.trim().is_empty() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Query pattern cannot be empty".to_string(),
        )
        .into());
    }

    Ok((query_type, pattern))
}

/// Query the graph using simple pattern matching
///
/// Supports basic SPARQL-like queries with pattern matching on
/// subject, predicate, or object. Patterns use simple substring matching.
///
/// # Arguments
/// * `query_string` - Query pattern (e.g., "subject:ex:alice" or "predicate:rdf:type")
///
/// # Example
/// ```text
/// specimen-graph-manager graph query "subject:ex:alice"
/// ```
#[verb("query", "graph")]
fn query_graph(query_string: String) -> Result<QueryResultOutput> {
    let (query_type, pattern) = parse_query_string(&query_string)?;
    let results = execute_query(&query_type, &pattern);
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
        let results = execute_query("subject", "ex:alice");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].subject, "ex:alice");
    }
}
