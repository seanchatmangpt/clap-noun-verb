// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Validate Command - Validate RDF file syntax and structure

use serde::{Deserialize, Serialize};

/// Result from graph validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultOutput {
    /// Whether all triples passed validation.
    pub valid: bool,
    /// Validation errors found, if any.
    pub errors: Vec<ValidationErrorOutput>,
    /// Total number of triples examined.
    pub total_triples: usize,
    /// Number of triples that passed validation.
    pub valid_triples: usize,
}

/// A single validation error with its source line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorOutput {
    /// Line number (or triple index) where the error occurred.
    pub triple_index: usize,
    /// Description of the validation failure.
    pub message: String,
}

impl ValidationResultOutput {
    /// Create a result assuming all `total_triples` are valid.
    pub fn new(total_triples: usize) -> Self {
        Self { valid: true, errors: Vec::new(), total_triples, valid_triples: total_triples }
    }

    /// Record an error, marking the result invalid and decrementing valid count.
    pub fn add_error(&mut self, idx: usize, msg: impl Into<String>) {
        self.valid = false;
        self.valid_triples = self.valid_triples.saturating_sub(1);
        self.errors.push(ValidationErrorOutput { triple_index: idx, message: msg.into() });
    }
}

/// Validate RDF graph syntax and structure
///
/// Performs structural validation on an RDF file:
/// - Checks for valid Turtle syntax
/// - Validates that all triples have subject, predicate, object
/// - Ensures URI format compliance
/// - Reports all validation errors with line numbers
///
/// Uses the generated validator implementation for optimized validation.
///
/// # Arguments
/// * `path` - Path to RDF file to validate
///
/// # Example
/// ```text
/// myapp graph validate data/example.ttl
/// ```
pub fn validate_graph(path: String) -> crate::Result<ValidationResultOutput> {
    let validator = super::impl_generated::GeneratedValidator::new();
    let (total_triples, errors) = validator.validate_file(&path)?;

    let mut result = ValidationResultOutput::new(total_triples);
    for error in errors {
        result.add_error(error.triple_index, error.message);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_output() {
        let mut output = ValidationResultOutput::new(10);
        assert!(output.valid);
        assert_eq!(output.valid_triples, 10);

        output.add_error(0, "Invalid subject");
        assert!(!output.valid);
        assert_eq!(output.valid_triples, 9);
        assert_eq!(output.errors.len(), 1);
    }
}
