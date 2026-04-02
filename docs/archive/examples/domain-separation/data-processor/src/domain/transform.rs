//! Domain logic for data transformation - ZERO CLI dependencies
//!
//! This module contains pure business logic with no knowledge of:
//! - Command-line arguments
//! - File paths
//! - User interaction
//! - Specific I/O implementations

use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use thiserror::Error;

/// Domain error types - business logic failures only
#[derive(Debug, Error, PartialEq)]
pub enum TransformError {
    #[error("Invalid record at line {line}: {reason}")]
    InvalidRecord { line: usize, reason: String },

    #[error("Transformation failed: {0}")]
    TransformFailed(String),

    #[error("I/O error: {0}")]
    IoError(String),
}

/// Input record - domain model
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Record {
    pub id: u64,
    pub name: String,
    pub value: f64,
    pub category: String,
}

/// Output record - transformed domain model
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransformedRecord {
    pub id: u64,
    pub normalized_name: String,
    pub scaled_value: f64,
    pub category: String,
    pub computed_score: f64,
}

/// Transformation configuration - domain rules
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub scale_factor: f64,
    pub score_multiplier: f64,
    pub normalize_names: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            score_multiplier: 10.0,
            normalize_names: true,
        }
    }
}

/// Core domain logic - transform a single record
///
/// Type-first design: Takes pure data, returns Result
/// Zero dependencies on CLI or I/O
pub fn transform_record(
    record: Record,
    config: &TransformConfig,
) -> Result<TransformedRecord, TransformError> {
    // Validate business rules
    if record.value < 0.0 {
        return Err(TransformError::TransformFailed(
            format!("Negative value not allowed: {}", record.value)
        ));
    }

    // Apply transformations
    let normalized_name = if config.normalize_names {
        record.name.trim().to_lowercase()
    } else {
        record.name
    };

    let scaled_value = record.value * config.scale_factor;
    let computed_score = scaled_value * config.score_multiplier;

    Ok(TransformedRecord {
        id: record.id,
        normalized_name,
        scaled_value,
        category: record.category,
        computed_score,
    })
}

/// Streaming processor - handles large datasets efficiently
///
/// Generic over readers/writers - testable with in-memory buffers
pub fn process_stream<R, W>(
    reader: R,
    writer: &mut W,
    config: &TransformConfig,
) -> Result<ProcessingStats, TransformError>
where
    R: BufRead,
    W: Write,
{
    let mut stats = ProcessingStats::default();
    let mut csv_reader = csv::Reader::from_reader(reader);
    let mut csv_writer = csv::Writer::from_writer(writer);

    for (line_num, result) in csv_reader.deserialize::<Record>().enumerate() {
        let line = line_num + 2; // Account for header

        match result {
            Ok(record) => {
                match transform_record(record, config) {
                    Ok(transformed) => {
                        csv_writer.serialize(&transformed)
                            .map_err(|e| TransformError::IoError(e.to_string()))?;
                        stats.processed += 1;
                    }
                    Err(e) => {
                        stats.failed += 1;
                        stats.errors.push((line, e.to_string()));
                    }
                }
            }
            Err(e) => {
                stats.failed += 1;
                stats.errors.push((line, format!("Parse error: {}", e)));
            }
        }
    }

    csv_writer.flush()
        .map_err(|e| TransformError::IoError(e.to_string()))?;

    Ok(stats)
}

/// Processing statistics - observable output for testing
#[derive(Debug, Default, PartialEq)]
pub struct ProcessingStats {
    pub processed: usize,
    pub failed: usize,
    pub errors: Vec<(usize, String)>,
}

impl ProcessingStats {
    pub fn success_rate(&self) -> f64 {
        let total = self.processed + self.failed;
        if total == 0 {
            return 0.0;
        }
        (self.processed as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_record_success() {
        // Arrange
        let record = Record {
            id: 1,
            name: "  Test Item  ".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        let config = TransformConfig::default();

        // Act
        let result = transform_record(record, &config);

        // Assert
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.normalized_name, "test item");
        assert_eq!(transformed.scaled_value, 10.0);
        assert_eq!(transformed.computed_score, 100.0);
    }

    #[test]
    fn test_transform_record_negative_value_fails() {
        // Arrange
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: -5.0,
            category: "A".to_string(),
        };
        let config = TransformConfig::default();

        // Act
        let result = transform_record(record, &config);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            TransformError::TransformFailed("Negative value not allowed: -5".to_string())
        );
    }

    #[test]
    fn test_process_stream_success() {
        // Arrange
        let input = "id,name,value,category\n1,Test,10.0,A\n2,Example,20.0,B\n";
        let reader = input.as_bytes();
        let mut writer = Vec::new();
        let config = TransformConfig::default();

        // Act
        let stats = process_stream(reader, &mut writer, &config).unwrap();

        // Assert
        assert_eq!(stats.processed, 2);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.success_rate(), 100.0);

        // Verify output contains expected data
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("test"));
        assert!(output.contains("example"));
    }

    #[test]
    fn test_process_stream_with_errors() {
        // Arrange
        let input = "id,name,value,category\n1,Valid,10.0,A\n2,Invalid,-5.0,B\n3,Good,15.0,C\n";
        let reader = input.as_bytes();
        let mut writer = Vec::new();
        let config = TransformConfig::default();

        // Act
        let stats = process_stream(reader, &mut writer, &config).unwrap();

        // Assert
        assert_eq!(stats.processed, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].1.contains("Negative value"));
    }
}
