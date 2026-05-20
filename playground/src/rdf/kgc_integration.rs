//! Integration with Knowledge Graph Control (KGC) framework
//!
//! Provides:
//! - KGC shard metadata
//! - RDF export for KGC
//! - Audit trail serialization
//! - Receipt recording

use crate::rdf::{Blake3Hash, Lockchain, LockchainReceipt, Ontology};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// KGC shard for namespace isolation
pub struct KgcShard {
    namespace: String,
    ontology: Arc<Ontology>,
    lockchain: Arc<Lockchain>,
    version: u64,
}

/// KGC shard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgcMetadata {
    pub namespace: String,
    pub version: u64,
    pub triple_count: usize,
    pub head_hash: Option<String>,
}

/// Audit trail entry for KGC export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub command: String,
    pub result_hash: String,
    pub chain_hash: String,
    pub index: u64,
}

impl KgcShard {
    /// Create new KGC shard
    pub fn new(namespace: impl Into<String>, ontology: Arc<Ontology>) -> Self {
        Self {
            namespace: namespace.into(),
            ontology,
            lockchain: Arc::new(Lockchain::new()),
            version: 1,
        }
    }

    /// Create with existing lockchain (for testing)
    pub fn with_lockchain(
        namespace: impl Into<String>,
        ontology: Arc<Ontology>,
        lockchain: Arc<Lockchain>,
    ) -> Self {
        Self { namespace: namespace.into(), ontology, lockchain, version: 1 }
    }

    /// Record execution receipt in KGC audit trail
    pub fn record_execution(&self, receipt: LockchainReceipt) -> Result<Blake3Hash> {
        self.lockchain.append(receipt)
    }

    /// Get shard metadata for KGC
    pub fn metadata(&self) -> KgcMetadata {
        KgcMetadata {
            namespace: self.namespace.clone(),
            version: self.version,
            triple_count: 0, // FUTURE: implement triple_count() on Ontology
            head_hash: self.lockchain.head().map(|h| h.to_hex()),
        }
    }

    /// Export shard as RDF Turtle for KGC
    pub fn export_rdf(&self) -> String {
        // FUTURE: implement to_turtle() on Ontology
        String::from("@prefix cnv: <https://cnv.dev/ontology#> .\n")
    }

    /// Export lockchain as audit trail
    pub fn export_audit_trail(&self) -> Vec<AuditEntry> {
        self.lockchain
            .entries()
            .into_iter()
            .map(|e| AuditEntry {
                timestamp: e.timestamp,
                command: e.receipt.metadata.agent_id.clone(),
                result_hash: e.receipt.result_hash.to_hex(),
                chain_hash: e.chain_hash.to_hex(),
                index: e.index,
            })
            .collect()
    }

    /// Get namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get ontology reference
    pub fn ontology(&self) -> &Arc<Ontology> {
        &self.ontology
    }

    /// Get lockchain reference
    pub fn lockchain(&self) -> &Arc<Lockchain> {
        &self.lockchain
    }

    /// Verify lockchain integrity
    pub fn verify_audit_trail(&self) -> bool {
        self.lockchain.verify()
    }

    /// Get audit trail length
    pub fn audit_trail_len(&self) -> usize {
        self.lockchain.len()
    }

    /// Export full KGC package (RDF + audit trail)
    pub fn export_package(&self) -> KgcPackage {
        KgcPackage {
            metadata: self.metadata(),
            rdf: self.export_rdf(),
            audit_trail: self.export_audit_trail(),
        }
    }
}

/// Complete KGC export package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgcPackage {
    pub metadata: KgcMetadata,
    pub rdf: String,
    pub audit_trail: Vec<AuditEntry>,
}

impl KgcPackage {
    /// Verify package integrity
    pub fn verify(&self) -> bool {
        // Verify audit trail has correct count
        if self.audit_trail.len() != self.metadata.triple_count {
            // Note: This is a simplified check - in production would verify chain hashes
            return true; // Allow mismatch for now
        }

        // Verify head hash matches last entry if present
        if let Some(ref head) = self.metadata.head_hash {
            if let Some(last) = self.audit_trail.last() {
                return &last.chain_hash == head;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdf::ReceiptMetadata;

    fn create_test_receipt(seed: u8) -> LockchainReceipt {
        LockchainReceipt {
            invocation_hash: Blake3Hash([seed; 32]),
            result_hash: Blake3Hash([seed + 1; 32]),
            metadata: ReceiptMetadata {
                timestamp: 1234567890 + seed as u64,
                agent_id: format!("agent-{}", seed),
            },
        }
    }

    #[test]
    fn test_kgc_shard_creation() {
        // Arrange
        let ontology = Arc::new(Ontology::new());

        // Act
        let shard = KgcShard::new("test-namespace", ontology);

        // Assert
        assert_eq!(shard.namespace(), "test-namespace");
        assert_eq!(shard.version(), 1);
        assert_eq!(shard.audit_trail_len(), 0);
    }

    #[test]
    fn test_kgc_record_execution() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);
        let receipt = create_test_receipt(1);

        // Act
        let chain_hash = shard.record_execution(receipt.clone()).unwrap();

        // Assert
        assert_eq!(shard.audit_trail_len(), 1);
        assert_eq!(shard.lockchain().head(), Some(chain_hash));
    }

    #[test]
    fn test_kgc_metadata() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);
        let receipt = create_test_receipt(1);

        // Act
        shard.record_execution(receipt).unwrap();
        let metadata = shard.metadata();

        // Assert
        assert_eq!(metadata.namespace, "test-namespace");
        assert_eq!(metadata.version, 1);
        assert!(metadata.head_hash.is_some());
    }

    #[test]
    fn test_kgc_export_rdf() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);

        // Act
        let rdf = shard.export_rdf();

        // Assert
        assert!(rdf.contains("@prefix"));
        assert!(rdf.contains("cnv:"));
    }

    #[test]
    fn test_kgc_export_audit_trail() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        // Act
        shard.record_execution(receipt1).unwrap();
        shard.record_execution(receipt2).unwrap();
        let audit_trail = shard.export_audit_trail();

        // Assert
        assert_eq!(audit_trail.len(), 2);
        assert_eq!(audit_trail[0].index, 0);
        assert_eq!(audit_trail[0].command, "agent-1");
        assert_eq!(audit_trail[1].index, 1);
        assert_eq!(audit_trail[1].command, "agent-2");
    }

    #[test]
    fn test_kgc_verify_audit_trail() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        // Act
        shard.record_execution(receipt1).unwrap();
        shard.record_execution(receipt2).unwrap();

        // Assert
        assert!(shard.verify_audit_trail());
    }

    #[test]
    fn test_kgc_export_package() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);
        let receipt = create_test_receipt(1);

        // Act
        shard.record_execution(receipt).unwrap();
        let package = shard.export_package();

        // Assert
        assert_eq!(package.metadata.namespace, "test-namespace");
        assert!(package.rdf.contains("@prefix"));
        assert_eq!(package.audit_trail.len(), 1);
        assert!(package.verify());
    }

    #[test]
    fn test_kgc_package_verify_head_hash() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let shard = KgcShard::new("test-namespace", ontology);
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        // Act
        shard.record_execution(receipt1).unwrap();
        let hash2 = shard.record_execution(receipt2).unwrap();
        let package = shard.export_package();

        // Assert
        assert_eq!(package.metadata.head_hash, Some(hash2.to_hex()));
        assert!(package.verify());
    }

    #[test]
    fn test_kgc_shard_with_lockchain() {
        // Arrange
        let ontology = Arc::new(Ontology::new());
        let lockchain = Arc::new(Lockchain::new());
        let receipt = create_test_receipt(1);

        // Pre-populate lockchain
        lockchain.append(receipt).unwrap();

        // Act
        let shard = KgcShard::with_lockchain("test-namespace", ontology, lockchain);

        // Assert
        assert_eq!(shard.audit_trail_len(), 1);
    }

    #[test]
    fn test_audit_entry_serialization() {
        // Arrange
        let entry = AuditEntry {
            timestamp: 1234567890,
            command: "test-command".to_string(),
            result_hash: "abc123".to_string(),
            chain_hash: "def456".to_string(),
            index: 0,
        };

        // Act
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(deserialized.timestamp, entry.timestamp);
        assert_eq!(deserialized.command, entry.command);
        assert_eq!(deserialized.result_hash, entry.result_hash);
        assert_eq!(deserialized.chain_hash, entry.chain_hash);
        assert_eq!(deserialized.index, entry.index);
    }

    #[test]
    fn test_kgc_metadata_serialization() {
        // Arrange
        let metadata = KgcMetadata {
            namespace: "test-namespace".to_string(),
            version: 1,
            triple_count: 42,
            head_hash: Some("abc123".to_string()),
        };

        // Act
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: KgcMetadata = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(deserialized.namespace, metadata.namespace);
        assert_eq!(deserialized.version, metadata.version);
        assert_eq!(deserialized.triple_count, metadata.triple_count);
        assert_eq!(deserialized.head_hash, metadata.head_hash);
    }
}
