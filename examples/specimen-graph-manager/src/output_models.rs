// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Output types for CLI commands
//!
//! All types implement Serialize for JSON output formatting.

use serde::{Deserialize, Serialize};

/// Result from graph load operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLoadedOutput {
    pub triples_loaded: usize,
    pub source: String,
    pub status: String,
}

impl GraphLoadedOutput {
    pub fn new(triples_loaded: usize, source: impl Into<String>) -> Self {
        Self {
            triples_loaded,
            source: source.into(),
            status: "success".to_string(),
        }
    }
}

/// Result from graph query operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultOutput {
    pub query_type: String,
    pub pattern: String,
    pub results: Vec<QueryMatch>,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMatch {
    pub index: usize,
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl QueryResultOutput {
    pub fn new(query_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            query_type: query_type.into(),
            pattern: pattern.into(),
            results: Vec::new(),
            match_count: 0,
        }
    }

    pub fn with_results(mut self, results: Vec<QueryMatch>) -> Self {
        self.match_count = results.len();
        self.results = results;
        self
    }
}

/// Result from graph validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultOutput {
    pub valid: bool,
    pub errors: Vec<ValidationErrorOutput>,
    pub total_triples: usize,
    pub valid_triples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorOutput {
    pub triple_index: usize,
    pub message: String,
}

impl ValidationResultOutput {
    pub fn new(total_triples: usize) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            total_triples,
            valid_triples: total_triples,
        }
    }

    pub fn add_error(&mut self, idx: usize, msg: impl Into<String>) {
        self.valid = false;
        self.valid_triples = self.valid_triples.saturating_sub(1);
        self.errors.push(ValidationErrorOutput {
            triple_index: idx,
            message: msg.into(),
        });
    }
}

/// Result from doctor health check operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorOutput {
    pub status: String,
    pub healthy: bool,
    pub issues: Vec<HealthIssue>,
    pub graph_triples: usize,
    pub registry_packages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub level: String,  // "error", "warning", "info"
    pub message: String,
}

impl DoctorOutput {
    pub fn new(graph_triples: usize, registry_packages: usize) -> Self {
        Self {
            status: "healthy".to_string(),
            healthy: true,
            issues: Vec::new(),
            graph_triples,
            registry_packages,
        }
    }

    pub fn add_issue(&mut self, level: impl Into<String>, msg: impl Into<String>) {
        let level_str = level.into();
        if level_str == "error" {
            self.healthy = false;
            self.status = "unhealthy".to_string();
        }
        self.issues.push(HealthIssue {
            level: level_str,
            message: msg.into(),
        });
    }
}

/// Result from pack add operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackAddedOutput {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: String,
}

impl PackAddedOutput {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            status: "added".to_string(),
        }
    }
}

/// Result from pack remove operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalStatus {
    pub removed_id: String,
    pub status: String,
    pub message: String,
}

impl RemovalStatus {
    pub fn new(removed_id: impl Into<String>) -> Self {
        Self {
            removed_id: removed_id.into(),
            status: "removed".to_string(),
            message: "Package successfully removed from registry".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_loaded_output() {
        let output = GraphLoadedOutput::new(42, "test.ttl");
        assert_eq!(output.triples_loaded, 42);
        assert_eq!(output.source, "test.ttl");
        assert_eq!(output.status, "success");
    }

    #[test]
    fn test_query_result_output() {
        let output = QueryResultOutput::new("subject_match", "ex:alice");
        assert!(output.results.is_empty());
        assert_eq!(output.match_count, 0);
    }

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

    #[test]
    fn test_doctor_output() {
        let mut output = DoctorOutput::new(100, 5);
        assert!(output.healthy);

        output.add_issue("warning", "Low memory");
        assert!(output.healthy);

        output.add_issue("error", "Database unreachable");
        assert!(!output.healthy);
        assert_eq!(output.status, "unhealthy");
    }

    #[test]
    fn test_pack_added_output() {
        let output = PackAddedOutput::new("pkg-001", "GraphUtils", "2.1.0");
        assert_eq!(output.id, "pkg-001");
        assert_eq!(output.name, "GraphUtils");
        assert_eq!(output.version, "2.1.0");
        assert_eq!(output.status, "added");
    }

    #[test]
    fn test_removal_status() {
        let output = RemovalStatus::new("pkg-001");
        assert_eq!(output.removed_id, "pkg-001");
        assert_eq!(output.status, "removed");
    }
}
