//! KGC-compatible lockchain for receipt audit trail
//!
//! Provides:
//! - Atomic append operations
//! - Chain integrity verification
//! - Blake3 hash chaining
//! - Immutable audit trail

use crate::rdf::{Blake3Hash, LockchainReceipt};
use anyhow::{Context, Result};
use std::sync::Mutex;

/// Lockchain entry linking receipts with Blake3 hashes
#[derive(Debug, Clone)]
pub struct LockchainEntry {
    pub receipt: LockchainReceipt,
    pub prev_hash: Option<Blake3Hash>,
    pub chain_hash: Blake3Hash,
    pub timestamp: u64,
    pub index: u64,
}

/// KGC-compatible lockchain for immutable audit trail
pub struct Lockchain {
    entries: Mutex<Vec<LockchainEntry>>,
    head: Mutex<Option<Blake3Hash>>,
}

impl Lockchain {
    /// Create new empty lockchain
    pub fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()), head: Mutex::new(None) }
    }

    /// Append receipt to chain (atomic operation)
    ///
    /// Returns the chain hash of the new entry
    pub fn append(&self, receipt: LockchainReceipt) -> Result<Blake3Hash> {
        let mut entries =
            self.entries.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut head = self.head.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        let index = entries.len() as u64;
        let chain_hash = self.compute_chain_hash(&receipt, head.as_ref());

        let entry = LockchainEntry {
            receipt,
            prev_hash: *head,
            chain_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .context("Failed to get system time")?
                .as_secs(),
            index,
        };

        entries.push(entry);
        *head = Some(chain_hash);

        Ok(chain_hash)
    }

    /// Verify chain integrity
    ///
    /// Returns true if all chain hashes are correctly computed
    /// and all prev_hash links are valid
    pub fn verify(&self) -> bool {
        let entries = match self.entries.lock() {
            Ok(e) => e,
            Err(_) => return false,
        };

        let mut prev_hash = None;
        for entry in entries.iter() {
            // Verify chain_hash matches hash(receipt || prev_hash)
            let expected = self.compute_chain_hash(&entry.receipt, prev_hash.as_ref());
            if expected != entry.chain_hash {
                return false;
            }

            // Verify prev_hash links correctly
            if entry.prev_hash != prev_hash {
                return false;
            }

            prev_hash = Some(entry.chain_hash);
        }

        true
    }

    /// Get all entries (cloned)
    pub fn entries(&self) -> Vec<LockchainEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// Get entry by index
    pub fn get_entry(&self, index: u64) -> Option<LockchainEntry> {
        let entries = self.entries.lock().unwrap();
        entries.get(index as usize).cloned()
    }

    /// Get latest entry
    pub fn latest(&self) -> Option<LockchainEntry> {
        let entries = self.entries.lock().unwrap();
        entries.last().cloned()
    }

    /// Get current head hash
    pub fn head(&self) -> Option<Blake3Hash> {
        *self.head.lock().unwrap()
    }

    /// Get chain length
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Compute chain hash from receipt and previous hash
    ///
    /// chain_hash = blake3(invocation_hash || result_hash || prev_hash)
    fn compute_chain_hash(
        &self,
        receipt: &LockchainReceipt,
        prev: Option<&Blake3Hash>,
    ) -> Blake3Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&receipt.invocation_hash.0);
        hasher.update(&receipt.result_hash.0);
        if let Some(prev_hash) = prev {
            hasher.update(&prev_hash.0);
        }
        Blake3Hash(*hasher.finalize().as_bytes())
    }
}

impl Default for Lockchain {
    fn default() -> Self {
        Self::new()
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
    fn test_lockchain_empty() {
        // Arrange
        let chain = Lockchain::new();

        // Act & Assert
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());
        assert!(chain.head().is_none());
        assert!(chain.latest().is_none());
    }

    #[test]
    fn test_lockchain_append_single() {
        // Arrange
        let chain = Lockchain::new();
        let receipt = create_test_receipt(1);

        // Act
        let chain_hash = chain.append(receipt.clone()).unwrap();

        // Assert
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
        assert_eq!(chain.head(), Some(chain_hash));

        let entry = chain.latest().unwrap();
        assert_eq!(entry.index, 0);
        assert_eq!(entry.prev_hash, None);
        assert_eq!(entry.chain_hash, chain_hash);
        assert_eq!(entry.receipt.invocation_hash, receipt.invocation_hash);
    }

    #[test]
    fn test_lockchain_append_multiple() {
        // Arrange
        let chain = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);
        let receipt3 = create_test_receipt(3);

        // Act
        let hash1 = chain.append(receipt1).unwrap();
        let hash2 = chain.append(receipt2).unwrap();
        let hash3 = chain.append(receipt3).unwrap();

        // Assert
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.head(), Some(hash3));

        let entries = chain.entries();
        assert_eq!(entries.len(), 3);

        // Verify first entry
        assert_eq!(entries[0].index, 0);
        assert_eq!(entries[0].prev_hash, None);
        assert_eq!(entries[0].chain_hash, hash1);

        // Verify second entry
        assert_eq!(entries[1].index, 1);
        assert_eq!(entries[1].prev_hash, Some(hash1));
        assert_eq!(entries[1].chain_hash, hash2);

        // Verify third entry
        assert_eq!(entries[2].index, 2);
        assert_eq!(entries[2].prev_hash, Some(hash2));
        assert_eq!(entries[2].chain_hash, hash3);
    }

    #[test]
    fn test_lockchain_verify_valid() {
        // Arrange
        let chain = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);
        let receipt3 = create_test_receipt(3);

        // Act
        chain.append(receipt1).unwrap();
        chain.append(receipt2).unwrap();
        chain.append(receipt3).unwrap();

        // Assert
        assert!(chain.verify());
    }

    #[test]
    fn test_lockchain_verify_tampered() {
        // Arrange
        let chain = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        chain.append(receipt1).unwrap();
        chain.append(receipt2).unwrap();

        // Act - tamper with chain by modifying entry
        {
            let mut entries = chain.entries.lock().unwrap();
            entries[1].chain_hash = Blake3Hash([99u8; 32]); // Invalid hash
        }

        // Assert
        assert!(!chain.verify());
    }

    #[test]
    fn test_lockchain_get_entry() {
        // Arrange
        let chain = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        chain.append(receipt1.clone()).unwrap();
        chain.append(receipt2.clone()).unwrap();

        // Act & Assert
        let entry0 = chain.get_entry(0).unwrap();
        assert_eq!(entry0.index, 0);
        assert_eq!(entry0.receipt.invocation_hash, receipt1.invocation_hash);

        let entry1 = chain.get_entry(1).unwrap();
        assert_eq!(entry1.index, 1);
        assert_eq!(entry1.receipt.invocation_hash, receipt2.invocation_hash);

        assert!(chain.get_entry(2).is_none());
    }

    #[test]
    fn test_lockchain_deterministic_hashing() {
        // Arrange
        let chain1 = Lockchain::new();
        let chain2 = Lockchain::new();
        let receipt = create_test_receipt(42);

        // Act
        let hash1 = chain1.append(receipt.clone()).unwrap();
        let hash2 = chain2.append(receipt).unwrap();

        // Assert - same receipt produces same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_lockchain_different_receipts_different_hashes() {
        // Arrange
        let chain = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        // Act
        let hash1 = chain.append(receipt1).unwrap();
        let hash2 = chain.append(receipt2).unwrap();

        // Assert - different receipts produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_lockchain_chain_hash_includes_prev() {
        // Arrange
        let chain1 = Lockchain::new();
        let chain2 = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        // Act - append in different orders
        chain1.append(receipt1.clone()).unwrap();
        let hash1 = chain1.append(receipt2.clone()).unwrap();

        chain2.append(receipt2).unwrap();
        let hash2 = chain2.append(receipt1).unwrap();

        // Assert - different order produces different hashes
        // (because prev_hash is included in chain_hash computation)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_lockchain_timestamps_ascending() {
        // Arrange
        let chain = Lockchain::new();
        let receipt1 = create_test_receipt(1);
        let receipt2 = create_test_receipt(2);

        // Act
        chain.append(receipt1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        chain.append(receipt2).unwrap();

        // Assert
        let entries = chain.entries();
        assert!(entries[0].timestamp <= entries[1].timestamp);
    }
}
