//! Generation receipts for deterministic validation
//!
//! Receipts prove that code generation was deterministic and reproducible.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[cfg(feature = "crypto")]
use blake3::Hash as Blake3Hash;

/// A receipt for a code generation operation
///
/// Receipts provide cryptographic proof that generation was deterministic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationReceipt {
    /// Hash of the template file (Blake3)
    #[cfg(feature = "crypto")]
    pub template_hash: String,

    /// Hash of the generated output (Blake3)
    #[cfg(feature = "crypto")]
    pub output_hash: String,

    /// Timestamp of generation (ISO 8601)
    #[cfg(feature = "agent2028")]
    pub timestamp: String,

    /// Variables used in generation
    pub variables: BTreeMap<String, String>,

    /// Path to generated output
    pub output_path: PathBuf,

    /// Path to template source
    pub template_path: PathBuf,
}

impl GenerationReceipt {
    /// Create a new generation receipt
    pub fn new(
        template_path: PathBuf,
        output_path: PathBuf,
        variables: BTreeMap<String, String>,
    ) -> Self {
        Self {
            #[cfg(feature = "crypto")]
            template_hash: String::new(),
            #[cfg(feature = "crypto")]
            output_hash: String::new(),
            #[cfg(feature = "agent2028")]
            timestamp: chrono::Utc::now().to_rfc3339(),
            variables,
            output_path,
            template_path,
        }
    }

    /// Set the template hash
    #[cfg(feature = "crypto")]
    pub fn with_template_hash(mut self, hash: impl Into<String>) -> Self {
        self.template_hash = hash.into();
        self
    }

    /// Set the output hash
    #[cfg(feature = "crypto")]
    pub fn with_output_hash(mut self, hash: impl Into<String>) -> Self {
        self.output_hash = hash.into();
        self
    }

    /// Verify receipt integrity (if hashes match expected)
    #[cfg(feature = "crypto")]
    pub fn verify(&self, expected_template_hash: &str, expected_output_hash: &str) -> bool {
        self.template_hash == expected_template_hash && self.output_hash == expected_output_hash
    }

    /// Get a compact summary of the receipt
    pub fn summary(&self) -> String {
        format!(
            "Generated {} from {}",
            self.output_path.display(),
            self.template_path.display()
        )
    }
}

impl std::fmt::Display for GenerationReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GenerationReceipt {{ template: {}, output: {} }}",
            self.template_path.display(),
            self.output_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_receipt_creation() {
        // Arrange
        let template_path = PathBuf::from("template.tera");
        let output_path = PathBuf::from("output.rs");
        let variables = BTreeMap::new();

        // Act
        let receipt = GenerationReceipt::new(template_path.clone(), output_path.clone(), variables);

        // Assert
        assert_eq!(receipt.template_path, template_path);
        assert_eq!(receipt.output_path, output_path);
        assert!(receipt.variables.is_empty());
    }

    #[test]
    fn test_receipt_summary() {
        // Arrange
        let receipt = GenerationReceipt::new(
            PathBuf::from("template.tera"),
            PathBuf::from("output.rs"),
            BTreeMap::new(),
        );

        // Act
        let summary = receipt.summary();

        // Assert
        assert!(summary.contains("output.rs"));
        assert!(summary.contains("template.tera"));
    }

    #[test]
    #[cfg(feature = "crypto")]
    fn test_receipt_with_hashes() {
        // Arrange
        let receipt = GenerationReceipt::new(
            PathBuf::from("template.tera"),
            PathBuf::from("output.rs"),
            BTreeMap::new(),
        )
        .with_template_hash("abc123")
        .with_output_hash("def456");

        // Act & Assert
        assert_eq!(receipt.template_hash, "abc123");
        assert_eq!(receipt.output_hash, "def456");
        assert!(receipt.verify("abc123", "def456"));
        assert!(!receipt.verify("wrong", "def456"));
    }
}
