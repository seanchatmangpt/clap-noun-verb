//! Provenance tracking with blake3 hashing for KGC lockchain

use crate::rdf::invocation::ParsedInvocation;
use crate::rdf::types::{RdfTriple, RdfValue};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Execution receipt with provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// blake3 hash of invocation
    pub invocation_hash: String,
    /// Exit code
    pub exit_code: i32,
    /// blake3 hash of stdout
    pub stdout_hash: String,
    /// blake3 hash of stderr
    pub stderr_hash: String,
    /// ISO8601 timestamp
    pub timestamp: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Builder for creating receipts
pub struct ReceiptGenerator {
    invocation: Option<String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
    timestamp: Option<String>,
    duration_ms: u64,
    metadata: BTreeMap<String, serde_json::Value>,
}

/// Receipt errors
#[derive(Debug, Error)]
pub enum ReceiptError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid timestamp format: {0}")]
    InvalidTimestamp(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl ReceiptGenerator {
    /// Create a new receipt generator
    pub fn new() -> Self {
        Self {
            invocation: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: 0,
            timestamp: None,
            duration_ms: 0,
            metadata: BTreeMap::new(),
        }
    }

    /// Set invocation
    pub fn with_invocation(mut self, inv: &ParsedInvocation) -> Self {
        self.invocation = Some(format!("{:?}", inv));
        self
    }

    /// Set stdout
    pub fn with_stdout(mut self, data: Vec<u8>) -> Self {
        self.stdout = data;
        self
    }

    /// Set stderr
    pub fn with_stderr(mut self, data: Vec<u8>) -> Self {
        self.stderr = data;
        self
    }

    /// Set exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    /// Set timestamp to current time
    pub fn with_timestamp(mut self) -> Self {
        self.timestamp = Some(chrono::Utc::now().to_rfc3339());
        self
    }

    /// Set custom timestamp
    pub fn with_custom_timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = Some(ts.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the receipt
    pub fn build(self) -> std::result::Result<Receipt, ReceiptError> {
        let invocation = self
            .invocation
            .ok_or_else(|| ReceiptError::MissingField("invocation".to_string()))?;

        let timestamp = self
            .timestamp
            .ok_or_else(|| ReceiptError::MissingField("timestamp".to_string()))?;

        // Compute blake3 hashes
        let invocation_hash = Self::compute_hash(invocation.as_bytes());
        let stdout_hash = Self::compute_hash(&self.stdout);
        let stderr_hash = Self::compute_hash(&self.stderr);

        Ok(Receipt {
            invocation_hash,
            exit_code: self.exit_code,
            stdout_hash,
            stderr_hash,
            timestamp,
            duration_ms: self.duration_ms,
            metadata: serde_json::Value::Object(
                self.metadata
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect(),
            ),
        })
    }

    /// Compute blake3 hash of data
    fn compute_hash(data: &[u8]) -> String {
        // Use blake3 for hashing
        let hash = blake3::hash(data);
        hash.to_hex().to_string()
    }

    /// Convert receipt to RDF triples
    pub fn to_rdf_triples(receipt: &Receipt) -> std::result::Result<Vec<RdfTriple>, ReceiptError> {
        let receipt_uri = format!(
            "{}Receipt-{}",
            crate::rdf::CNV_NAMESPACE,
            uuid::Uuid::new_v4()
        );
        let rdf_type = format!("{}type", crate::rdf::RDF_NS);
        let receipt_class = format!("{}Receipt", crate::rdf::CNV_NAMESPACE);

        let mut triples = Vec::new();

        // Receipt is a cnv:Receipt
        triples.push(RdfTriple::new(
            &receipt_uri,
            &rdf_type,
            RdfValue::uri(&receipt_class),
        ));

        // Add invocation hash
        triples.push(RdfTriple::new(
            &receipt_uri,
            format!("{}invocationHash", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(&receipt.invocation_hash),
        ));

        // Add exit code
        triples.push(RdfTriple::new(
            &receipt_uri,
            format!("{}exitCode", crate::rdf::CNV_NAMESPACE),
            RdfValue::typed_literal(
                receipt.exit_code.to_string(),
                format!("{}integer", crate::rdf::XSD_NS),
            ),
        ));

        // Add stdout hash
        triples.push(RdfTriple::new(
            &receipt_uri,
            format!("{}stdoutHash", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(&receipt.stdout_hash),
        ));

        // Add stderr hash
        triples.push(RdfTriple::new(
            &receipt_uri,
            format!("{}stderrHash", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(&receipt.stderr_hash),
        ));

        // Add timestamp
        triples.push(RdfTriple::new(
            &receipt_uri,
            format!("{}timestamp", crate::rdf::CNV_NAMESPACE),
            RdfValue::typed_literal(
                &receipt.timestamp,
                format!("{}dateTime", crate::rdf::XSD_NS),
            ),
        ));

        // Add duration
        triples.push(RdfTriple::new(
            &receipt_uri,
            format!("{}durationMs", crate::rdf::CNV_NAMESPACE),
            RdfValue::typed_literal(
                receipt.duration_ms.to_string(),
                format!("{}long", crate::rdf::XSD_NS),
            ),
        ));

        Ok(triples)
    }

    /// Convert receipt to Turtle format
    pub fn to_turtle(receipt: &Receipt) -> std::result::Result<String, ReceiptError> {
        let triples = Self::to_rdf_triples(receipt)?;
        let mut output = String::new();

        for triple in triples {
            output.push_str(&triple.to_turtle());
            output.push('\n');
        }

        Ok(output)
    }
}

impl Default for ReceiptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Receipt {
    /// Convert to JSON
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "invocation_hash": self.invocation_hash,
            "exit_code": self.exit_code,
            "stdout_hash": self.stdout_hash,
            "stderr_hash": self.stderr_hash,
            "timestamp": self.timestamp,
            "duration_ms": self.duration_ms,
            "metadata": self.metadata,
        })
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_invocation() -> ParsedInvocation {
        ParsedInvocation {
            command: "test-run".to_string(),
            args: BTreeMap::from([("pattern".to_string(), "*.rs".to_string())]),
            output_format: Some("json".to_string()),
        }
    }

    #[test]
    fn test_receipt_generator_creation() {
        let gen = ReceiptGenerator::new();
        assert_eq!(gen.exit_code, 0);
        assert_eq!(gen.duration_ms, 0);
    }

    #[test]
    fn test_build_receipt() {
        let inv = create_test_invocation();
        let receipt = ReceiptGenerator::new()
            .with_invocation(&inv)
            .with_stdout(b"output".to_vec())
            .with_stderr(b"".to_vec())
            .with_exit_code(0)
            .with_timestamp()
            .with_duration(123)
            .build()
            .expect("build receipt");

        assert_eq!(receipt.exit_code, 0);
        assert_eq!(receipt.duration_ms, 123);
        assert!(!receipt.invocation_hash.is_empty());
        assert!(!receipt.stdout_hash.is_empty());
        assert!(receipt.is_success());
    }

    #[test]
    fn test_build_receipt_missing_invocation() {
        let result = ReceiptGenerator::new()
            .with_timestamp()
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_build_receipt_missing_timestamp() {
        let inv = create_test_invocation();
        let result = ReceiptGenerator::new()
            .with_invocation(&inv)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_receipt_to_json() {
        let inv = create_test_invocation();
        let receipt = ReceiptGenerator::new()
            .with_invocation(&inv)
            .with_timestamp()
            .build()
            .expect("build");

        let json = receipt.as_json();
        assert!(json.get("invocation_hash").is_some());
        assert!(json.get("exit_code").is_some());
    }

    #[test]
    fn test_receipt_to_rdf() {
        let inv = create_test_invocation();
        let receipt = ReceiptGenerator::new()
            .with_invocation(&inv)
            .with_timestamp()
            .build()
            .expect("build");

        let triples = ReceiptGenerator::to_rdf_triples(&receipt).expect("to rdf");
        assert!(!triples.is_empty());
    }

    #[test]
    fn test_receipt_to_turtle() {
        let inv = create_test_invocation();
        let receipt = ReceiptGenerator::new()
            .with_invocation(&inv)
            .with_timestamp()
            .build()
            .expect("build");

        let turtle = ReceiptGenerator::to_turtle(&receipt).expect("to turtle");
        assert!(turtle.contains("Receipt"));
        assert!(turtle.contains("invocationHash"));
    }

    #[test]
    fn test_blake3_hashing() {
        let data = b"test data";
        let hash1 = ReceiptGenerator::compute_hash(data);
        let hash2 = ReceiptGenerator::compute_hash(data);
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_duration_secs() {
        let inv = create_test_invocation();
        let receipt = ReceiptGenerator::new()
            .with_invocation(&inv)
            .with_timestamp()
            .with_duration(2500)
            .build()
            .expect("build");

        assert_eq!(receipt.duration_secs(), 2.5);
    }

    #[test]
    fn test_receipt_metadata() {
        let inv = create_test_invocation();
        let receipt = ReceiptGenerator::new()
            .with_invocation(&inv)
            .with_timestamp()
            .with_metadata("user", serde_json::json!("agent-007"))
            .with_metadata("priority", serde_json::json!("high"))
            .build()
            .expect("build");

        let json = receipt.as_json();
        assert!(json["metadata"]["user"].is_string());
        assert_eq!(json["metadata"]["user"], "agent-007");
    }
}
