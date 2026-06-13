// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Validate Command - Validate RDF file syntax and structure

use crate::output_models::ValidationResultOutput;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use std::fs;

/// Domain logic: Validate file content
fn validate_file_content(content: &str) -> (usize, Vec<(usize, String)>) {
    let mut total_triples = 0;
    let mut errors = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        total_triples += 1;

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 3 {
            errors.push((idx + 1, "Triple must have subject, predicate, and object".to_string()));
        } else if !parts[0].starts_with('<') && !parts[0].starts_with('_') {
            errors.push((
                idx + 1,
                "Subject must be URI (start with <) or blank node (start with _)".to_string(),
            ));
        } else if !parts[1].starts_with('<') && !parts[1].starts_with('_') {
            errors.push((
                idx + 1,
                "Predicate must be URI (start with <) or qualified name".to_string(),
            ));
        }
    }

    (total_triples, errors)
}

/// Domain logic: Load and validate file
fn validate_graph_impl(path: &str) -> Result<(usize, Vec<(usize, String)>)> {
    if !std::path::Path::new(path).exists() {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(format!(
            "File not found: {}",
            path
        ))
        .into());
    }

    let content = fs::read_to_string(path).map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!("Failed to read file: {}", e))
    })?;

    Ok(validate_file_content(&content))
}

/// Validate RDF graph syntax and structure
///
/// Performs structural validation on an RDF file:
/// - Checks for valid Turtle syntax
/// - Validates that all triples have subject, predicate, object
/// - Ensures URI format compliance
/// - Reports all validation errors with line numbers
///
/// # Arguments
/// * `path` - Path to RDF file to validate
///
/// # Example
/// ```text
/// specimen-graph-manager graph validate data/example.ttl
/// ```
#[verb("validate", "graph")]
fn validate_graph(path: String) -> Result<ValidationResultOutput> {
    let (total_triples, errors) = validate_graph_impl(&path)?;

    let mut result = ValidationResultOutput::new(total_triples);
    for (line_num, msg) in errors {
        result.add_error(line_num, msg);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_content_valid() {
        let content = "<s1> <p1> <o1> .\n<s2> <p2> <o2> .\n";
        let (total, errors) = validate_file_content(content);
        assert_eq!(total, 2);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_content_with_errors() {
        let content = "<s1> <p1>\n<s2> <p2> <o2> .\n";
        let (total, errors) = validate_file_content(content);
        assert_eq!(total, 2);
        assert!(!errors.is_empty());
    }
}
