//! Receipt storage integration layer
//!
//! Handles storing and retrieving cryptographic receipts with signature
//! verification and indexing for fast lookups.

use crate::domain::receipt::{Receipt, ReceiptVerifier, VerificationResult};
use crate::integration::workspace::WorkspaceDetector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Receipt storage manager with verification and indexing
pub struct ReceiptStore {
    receipts_dir: PathBuf,
    verifier: ReceiptVerifier,
    index: Arc<RwLock<ReceiptIndex>>,
}

/// In-memory index for fast receipt lookups
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct ReceiptIndex {
    by_id: HashMap<String, IndexedReceipt>,
    by_timestamp: Vec<String>,
    by_agent: HashMap<String, Vec<String>>,
}

/// Indexed receipt metadata for fast lookups
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct IndexedReceipt {
    id: String,
    timestamp: String,
    agent_id: String,
    artifact_count: usize,
    operations_count: usize,
    content_hash: String,
}

impl ReceiptStore {
    /// Default receipts directory name
    pub const DIRNAME: &'static str = "receipts";

    /// Index filename
    pub const INDEX_FILENAME: &'static str = "receipts.index";

    /// Create new receipt store by detecting workspace root
    pub fn detect() -> Result<Self, String> {
        let workspace_root = WorkspaceDetector::find_workspace_root()?;
        Self::new(workspace_root)
    }

    /// Create new receipt store for workspace
    pub fn new(workspace_root: PathBuf) -> Result<Self, String> {
        let receipts_dir = workspace_root.join(Self::DIRNAME);

        // Create receipts directory if it doesn't exist
        if !receipts_dir.exists() {
            std::fs::create_dir_all(&receipts_dir)
                .map_err(|e| format!("Failed to create receipts directory: {}", e))?;
        }

        // Load or create index
        let index = Self::load_index(&receipts_dir)?;

        Ok(Self {
            receipts_dir,
            verifier: ReceiptVerifier::new(),
            index: Arc::new(RwLock::new(index)),
        })
    }

    /// Get path to receipt file
    pub fn receipt_path(&self, receipt_id: &str) -> PathBuf {
        self.receipts_dir.join(format!("{}.json", receipt_id))
    }

    /// Store receipt with verification and indexing
    pub fn store(&self, receipt: &Receipt) -> Result<(), String> {
        // Verify receipt before storing
        let verification = self.verifier.verify(receipt)?;
        if !verification.valid {
            return Err(format!(
                "Cannot store invalid receipt: {:?}",
                verification.warnings
            ));
        }

        // Save receipt
        let path = self.receipt_path(&receipt.id);
        receipt.save(&path)?;

        // Update index
        self.update_index(receipt)?;

        Ok(())
    }

    /// Load receipt by ID with verification
    pub fn load(&self, receipt_id: &str) -> Result<Receipt, String> {
        let path = self.receipt_path(receipt_id);
        let receipt = Receipt::from_file(&path)?;

        // Verify loaded receipt
        let verification = self.verifier.verify(&receipt)?;
        if !verification.valid {
            return Err(format!(
                "Loaded receipt is invalid: {:?}",
                verification.warnings
            ));
        }

        Ok(receipt)
    }

    /// Load receipt without verification (for audit/debugging)
    pub fn load_unverified(&self, receipt_id: &str) -> Result<Receipt, String> {
        let path = self.receipt_path(receipt_id);
        Receipt::from_file(&path)
    }

    /// List all receipt IDs (most recent first)
    pub fn list_receipts(&self) -> Result<Vec<String>, String> {
        let index = self.index.read()
            .map_err(|e| format!("Failed to acquire index lock: {}", e))?;

        Ok(index.by_timestamp.clone())
    }

    /// Find receipts by agent ID
    pub fn find_by_agent(&self, agent_id: &str) -> Result<Vec<String>, String> {
        let index = self.index.read()
            .map_err(|e| format!("Failed to acquire index lock: {}", e))?;

        Ok(index.by_agent.get(agent_id).cloned().unwrap_or_default())
    }

    /// Verify receipt and return detailed result
    pub fn verify(&self, receipt_id: &str) -> Result<VerificationResult, String> {
        let receipt = self.load(receipt_id)?;
        self.verifier.verify(&receipt)
    }

    /// Verify receipt chain (receipt pointing to previous receipts)
    pub fn verify_chain(&self, receipt_id: &str) -> Result<VerificationResult, String> {
        let receipt = self.load(receipt_id)?;
        self.verifier.verify_chain(&receipt)
    }

    /// Get receipt metadata from index without loading full receipt
    pub fn get_metadata(&self, receipt_id: &str) -> Result<Option<IndexedReceipt>, String> {
        let index = self.index.read()
            .map_err(|e| format!("Failed to acquire index lock: {}", e))?;

        Ok(index.by_id.get(receipt_id).cloned())
    }

    /// Search receipts by timestamp range
    pub fn search_by_timestamp(
        &self,
        start: &str,
        end: &str
    ) -> Result<Vec<String>, String> {
        let index = self.index.read()
            .map_err(|e| format!("Failed to acquire index lock: {}", e))?;

        let results: Vec<String> = index.by_timestamp.iter()
            .filter(|id| {
                if let Some(meta) = index.by_id.get(*id) {
                    meta.timestamp.as_str() >= start && meta.timestamp.as_str() <= end
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// Get receipts directory path
    pub fn receipts_dir(&self) -> &Path {
        &self.receipts_dir
    }

    /// Rebuild index from disk
    pub fn rebuild_index(&self) -> Result<(), String> {
        let mut index = ReceiptIndex::default();

        for entry in std::fs::read_dir(&self.receipts_dir)
            .map_err(|e| format!("Failed to read receipts directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    // Load receipt metadata without full verification
                    if let Ok(receipt) = self.load_unverified(name) {
                        let indexed = IndexedReceipt {
                            id: receipt.id.clone(),
                            timestamp: receipt.timestamp.clone(),
                            agent_id: receipt.agent.agent_id.clone(),
                            artifact_count: receipt.artifacts.len(),
                            operations_count: receipt.operations.len(),
                            content_hash: receipt.content_hash.clone(),
                        };

                        index.by_id.insert(receipt.id.clone(), indexed);
                        index.by_timestamp.push(receipt.id.clone());
                        index.by_agent
                            .entry(receipt.agent.agent_id.clone())
                            .or_insert_with(Vec::new)
                            .push(receipt.id.clone());
                    }
                }
            }
        }

        // Sort by timestamp (most recent first)
        // Collect into a separate vec to avoid borrow conflict between
        // index.by_id (immutable) and index.by_timestamp.sort_by (mutable)
        let mut timestamp_pairs: Vec<(String, String)> = index
            .by_timestamp
            .iter()
            .filter_map(|id| {
                index.by_id.get(id).map(|m| (id.clone(), m.timestamp.clone()))
            })
            .collect();
        timestamp_pairs.sort_by(|a, b| b.1.cmp(&a.1));
        index.by_timestamp = timestamp_pairs.into_iter().map(|(id, _)| id).collect();

        // Save updated index
        self.save_index(&index)?;

        // Update in-memory index
        let mut index_guard = self.index.write()
            .map_err(|e| format!("Failed to acquire index write lock: {}", e))?;
        *index_guard = index;

        Ok(())
    }

    /// Get index statistics
    pub fn index_stats(&self) -> Result<ReceiptIndexStats, String> {
        let index = self.index.read()
            .map_err(|e| format!("Failed to acquire index lock: {}", e))?;

        Ok(ReceiptIndexStats {
            total_receipts: index.by_id.len(),
            unique_agents: index.by_agent.len(),
            oldest_timestamp: index.by_timestamp.last()
                .and_then(|id| index.by_id.get(id))
                .map(|meta| meta.timestamp.clone()),
            newest_timestamp: index.by_timestamp.first()
                .and_then(|id| index.by_id.get(id))
                .map(|meta| meta.timestamp.clone()),
        })
    }

    /// Update index with new receipt
    fn update_index(&self, receipt: &Receipt) -> Result<(), String> {
        let mut index = self.index.write()
            .map_err(|e| format!("Failed to acquire index write lock: {}", e))?;

        let indexed = IndexedReceipt {
            id: receipt.id.clone(),
            timestamp: receipt.timestamp.clone(),
            agent_id: receipt.agent.agent_id.clone(),
            artifact_count: receipt.artifacts.len(),
            operations_count: receipt.operations.len(),
            content_hash: receipt.content_hash.clone(),
        };

        // Add to index
        index.by_id.insert(receipt.id.clone(), indexed);
        index.by_timestamp.push(receipt.id.clone());
        index.by_agent
            .entry(receipt.agent.agent_id.clone())
            .or_insert_with(Vec::new)
            .push(receipt.id.clone());

        // Sort by timestamp (most recent first)
        // Collect into a separate vec to avoid borrow conflict between
        // index.by_id (immutable) and index.by_timestamp.sort_by (mutable)
        let mut timestamp_pairs: Vec<(String, String)> = index
            .by_timestamp
            .iter()
            .filter_map(|id| {
                index.by_id.get(id).map(|m| (id.clone(), m.timestamp.clone()))
            })
            .collect();
        timestamp_pairs.sort_by(|a, b| b.1.cmp(&a.1));
        index.by_timestamp = timestamp_pairs.into_iter().map(|(id, _)| id).collect();

        // Persist index
        self.save_index(&index)?;

        Ok(())
    }

    /// Load index from disk
    fn load_index(receipts_dir: &Path) -> Result<ReceiptIndex, String> {
        let index_path = receipts_dir.join(Self::INDEX_FILENAME);

        if !index_path.exists() {
            return Ok(ReceiptIndex::default());
        }

        let content = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Failed to read index file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse index file: {}", e))
    }

    /// Save index to disk
    fn save_index(&self, index: &ReceiptIndex) -> Result<(), String> {
        let index_path = self.receipts_dir.join(Self::INDEX_FILENAME);

        let content = serde_json::to_string_pretty(index)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;

        std::fs::write(&index_path, content)
            .map_err(|e| format!("Failed to write index file: {}", e))?;

        Ok(())
    }
}

/// Receipt index statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReceiptIndexStats {
    pub total_receipts: usize,
    pub unique_agents: usize,
    pub oldest_timestamp: Option<String>,
    pub newest_timestamp: Option<String>,
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::detect().unwrap_or_else(|_| {
            Self {
                receipts_dir: PathBuf::from(".receipts"),
                verifier: ReceiptVerifier::new(),
                index: Arc::new(RwLock::new(ReceiptIndex::default())),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::receipt::{ReceiptAgent, ReceiptOperation, OperationResult};
    use tempfile::TempDir;

    fn create_test_receipt(agent_id: &str) -> Receipt {
        let agent = ReceiptAgent {
            agent_type: "test".to_string(),
            agent_id: agent_id.to_string(),
            version: "1.0.0".to_string(),
        };

        let mut receipt = Receipt::new(agent);
        receipt.add_operation(ReceiptOperation {
            operation_type: "test_op".to_string(),
            target: "test_target".to_string(),
            result: OperationResult {
                success: true,
                output: "test output".to_string(),
                duration_ms: 100,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[0u8; 32]);
        receipt.finalize(&signing_key).unwrap();
        receipt
    }

    #[test]
    fn test_receipt_store_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(store.receipts_dir().exists());
        assert!(store.receipts_dir().ends_with("receipts"));
    }

    #[test]
    fn test_receipt_store_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt = create_test_receipt("agent-1");
        store.store(&receipt).unwrap();

        let loaded = store.load(&receipt.id).unwrap();
        assert_eq!(loaded.id, receipt.id);
        assert_eq!(loaded.agent.agent_id, "agent-1");
    }

    #[test]
    fn test_receipt_list() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt1 = create_test_receipt("agent-1");
        let receipt2 = create_test_receipt("agent-2");

        store.store(&receipt1).unwrap();
        store.store(&receipt2).unwrap();

        let receipts = store.list_receipts().unwrap();
        assert_eq!(receipts.len(), 2);
    }

    #[test]
    fn test_receipt_find_by_agent() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt1 = create_test_receipt("agent-1");
        let receipt2 = create_test_receipt("agent-2");

        store.store(&receipt1).unwrap();
        store.store(&receipt2).unwrap();

        let agent1_receipts = store.find_by_agent("agent-1").unwrap();
        assert_eq!(agent1_receipts.len(), 1);
        assert_eq!(agent1_receipts[0], receipt1.id);
    }

    #[test]
    fn test_receipt_verify() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt = create_test_receipt("agent-1");
        store.store(&receipt).unwrap();

        let verification = store.verify(&receipt.id).unwrap();
        assert!(verification.valid);
    }

    #[test]
    fn test_receipt_get_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt = create_test_receipt("agent-1");
        store.store(&receipt).unwrap();

        let metadata = store.get_metadata(&receipt.id).unwrap();
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert_eq!(meta.id, receipt.id);
        assert_eq!(meta.agent_id, "agent-1");
    }

    #[test]
    fn test_receipt_index_stats() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt1 = create_test_receipt("agent-1");
        let receipt2 = create_test_receipt("agent-2");

        store.store(&receipt1).unwrap();
        store.store(&receipt2).unwrap();

        let stats = store.index_stats().unwrap();
        assert_eq!(stats.total_receipts, 2);
        assert_eq!(stats.unique_agents, 2);
    }

    #[test]
    fn test_receipt_rebuild_index() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt = create_test_receipt("agent-1");
        store.store(&receipt).unwrap();

        // Rebuild index
        store.rebuild_index().unwrap();

        // Verify receipt is still accessible
        let loaded = store.load(&receipt.id).unwrap();
        assert_eq!(loaded.id, receipt.id);
    }

    #[test]
    fn test_receipt_search_by_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let store = ReceiptStore::new(temp_dir.path().to_path_buf()).unwrap();

        let receipt = create_test_receipt("agent-1");
        store.store(&receipt).unwrap();

        let now = chrono::Utc::now();
        let start = (now - chrono::Duration::hours(1)).to_rfc3339();
        let end = (now + chrono::Duration::hours(1)).to_rfc3339();

        let results = store.search_by_timestamp(&start, &end).unwrap();
        assert_eq!(results.len(), 1);
    }
}
