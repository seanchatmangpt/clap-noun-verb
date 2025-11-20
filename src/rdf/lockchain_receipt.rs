//! Lockchain-specific receipt types for KGC integration

use crate::rdf::Blake3Hash;
use serde::{Deserialize, Serialize};

/// Lockchain receipt with blake3 hashes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockchainReceipt {
    pub invocation_hash: Blake3Hash,
    pub result_hash: Blake3Hash,
    pub metadata: ReceiptMetadata,
}

/// Receipt metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    pub timestamp: u64,
    pub agent_id: String,
}

impl LockchainReceipt {
    /// Create new lockchain receipt
    pub fn new(
        invocation_hash: Blake3Hash,
        result_hash: Blake3Hash,
        timestamp: u64,
        agent_id: impl Into<String>,
    ) -> Self {
        Self {
            invocation_hash,
            result_hash,
            metadata: ReceiptMetadata { timestamp, agent_id: agent_id.into() },
        }
    }

    /// Create from invocation and result data
    pub fn from_data(invocation: &[u8], result: &[u8], agent_id: impl Into<String>) -> Self {
        let invocation_hash = Blake3Hash::hash(invocation);
        let result_hash = Blake3Hash::hash(result);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self::new(invocation_hash, result_hash, timestamp, agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockchain_receipt_creation() {
        let inv_hash = Blake3Hash::hash(b"invocation");
        let res_hash = Blake3Hash::hash(b"result");
        let receipt = LockchainReceipt::new(inv_hash, res_hash, 1234567890, "agent-1");

        assert_eq!(receipt.invocation_hash, inv_hash);
        assert_eq!(receipt.result_hash, res_hash);
        assert_eq!(receipt.metadata.timestamp, 1234567890);
        assert_eq!(receipt.metadata.agent_id, "agent-1");
    }

    #[test]
    fn test_lockchain_receipt_from_data() {
        let receipt = LockchainReceipt::from_data(b"test invocation", b"test result", "agent-007");

        assert_eq!(receipt.metadata.agent_id, "agent-007");
        assert_ne!(receipt.invocation_hash.0, [0u8; 32]);
        assert_ne!(receipt.result_hash.0, [0u8; 32]);
    }

    #[test]
    fn test_receipt_metadata_serialization() {
        let metadata =
            ReceiptMetadata { timestamp: 1234567890, agent_id: "test-agent".to_string() };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ReceiptMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timestamp, metadata.timestamp);
        assert_eq!(deserialized.agent_id, metadata.agent_id);
    }
}
