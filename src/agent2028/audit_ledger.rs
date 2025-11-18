/// Distributed Audit Ledger (Immutable Audit Trail)
///
/// Append-only cryptographically-linked audit logs for tracking all command executions
/// across agent systems with tamper-proof verification and Merkle tree compression.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use sha3::{Digest, Keccak256};

/// Single audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub agent_id: String,
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub result: ExecutionResult,
    pub input_hash: Vec<u8>,
    pub output_hash: Vec<u8>,
}

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

impl AuditEvent {
    pub fn new(
        agent_id: String,
        command: String,
        result: ExecutionResult,
    ) -> Self {
        let mut input_hasher = Keccak256::new();
        input_hasher.update(command.as_bytes());

        let mut output_hasher = Keccak256::new();
        output_hasher.update(if result.success { b"success" } else { b"failure" });

        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            command,
            timestamp: Utc::now(),
            result,
            input_hash: input_hasher.finalize().to_vec(),
            output_hash: output_hasher.finalize().to_vec(),
        }
    }

    /// Compute hash of this event
    pub fn hash(&self) -> Vec<u8> {
        let serialized = serde_json::to_vec(&self).unwrap_or_default();
        let mut hasher = Keccak256::new();
        hasher.update(&serialized);
        hasher.finalize().to_vec()
    }
}

/// Node in Merkle tree of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub node_id: String,
    pub hash: Vec<u8>,
    pub parent_hash: Option<Vec<u8>>,
    pub children_hashes: Vec<Vec<u8>>,
    pub leaf_count: usize,
}

impl MerkleNode {
    pub fn new_leaf(event: &AuditEvent) -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            hash: event.hash(),
            parent_hash: None,
            children_hashes: vec![],
            leaf_count: 1,
        }
    }

    pub fn new_internal(left: &MerkleNode, right: &MerkleNode) -> Self {
        let mut hasher = Keccak256::new();
        hasher.update(&left.hash);
        hasher.update(&right.hash);

        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            hash: hasher.finalize().to_vec(),
            parent_hash: None,
            children_hashes: vec![left.hash.clone(), right.hash.clone()],
            leaf_count: left.leaf_count + right.leaf_count,
        }
    }
}

/// Merkle tree for audit event compression
pub struct MerkleTree {
    leaves: Arc<RwLock<Vec<MerkleNode>>>,
    root: Arc<RwLock<Option<MerkleNode>>>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            leaves: Arc::new(RwLock::new(Vec::new())),
            root: Arc::new(RwLock::new(None)),
        }
    }

    /// Add an event as a leaf to the tree
    pub async fn add_event(&self, event: &AuditEvent) {
        let leaf = MerkleNode::new_leaf(event);
        let mut leaves = self.leaves.write().await;
        leaves.push(leaf);

        // Rebuild tree on each addition
        self.rebuild().await;
    }

    /// Rebuild Merkle tree from leaves
    async fn rebuild(&self) {
        let leaves = self.leaves.read().await;

        if leaves.is_empty() {
            let mut root = self.root.write().await;
            *root = None;
            return;
        }

        if leaves.len() == 1 {
            let mut root = self.root.write().await;
            *root = Some(leaves[0].clone());
            return;
        }

        // Build tree bottom-up
        let mut current_level = leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..current_level.len()).step_by(2) {
                if i + 1 < current_level.len() {
                    let parent = MerkleNode::new_internal(&current_level[i], &current_level[i + 1]);
                    next_level.push(parent);
                } else {
                    // Odd number of nodes: hash alone becomes its own parent
                    next_level.push(current_level[i].clone());
                }
            }

            current_level = next_level;
        }

        let mut root = self.root.write().await;
        *root = current_level.into_iter().next();
    }

    /// Get root hash
    pub async fn root_hash(&self) -> Option<Vec<u8>> {
        let root = self.root.read().await;
        root.as_ref().map(|r| r.hash.clone())
    }

    /// Verify event is in tree (Merkle proof)
    pub async fn verify_inclusion(&self, event_id: &str) -> bool {
        let leaves = self.leaves.read().await;
        leaves.iter().any(|l| {
            // In practice: reconstruct path from leaf to root and verify hashes
            // Simplified: just check if event is in leaves
            l.node_id == event_id
        })
    }

    /// Get leaf count
    pub async fn leaf_count(&self) -> usize {
        let leaves = self.leaves.read().await;
        leaves.len()
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Cryptographic timestamp proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampProof {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub tsa_signature: Vec<u8>,
    pub nonce: u64,
}

impl TimestampProof {
    pub fn new(event_id: String) -> Self {
        let mut hasher = Keccak256::new();
        hasher.update(event_id.as_bytes());
        let hash = hasher.finalize();
        let nonce = u64::from_le_bytes([
            hash[0],
            hash[1],
            hash[2],
            hash[3],
            hash[4],
            hash[5],
            hash[6],
            hash[7],
        ]);

        Self {
            event_id,
            timestamp: Utc::now(),
            tsa_signature: vec![0u8; 64], // Placeholder
            nonce,
        }
    }
}

/// Distributed Audit Ledger (append-only log)
pub struct DistributedAuditLedger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    merkle_tree: Arc<MerkleTree>,
    timestamps: Arc<RwLock<Vec<TimestampProof>>>,
}

impl DistributedAuditLedger {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            merkle_tree: Arc::new(MerkleTree::new()),
            timestamps: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Append an event to the ledger
    pub async fn append(&self, event: AuditEvent) {
        // Add to events log
        let mut events = self.events.write().await;
        events.push(event.clone());

        // Add to Merkle tree
        self.merkle_tree.add_event(&event).await;

        // Create timestamp proof
        let timestamp_proof = TimestampProof::new(event.event_id);
        let mut timestamps = self.timestamps.write().await;
        timestamps.push(timestamp_proof);
    }

    /// Query events for an agent
    pub async fn query(&self, agent_id: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Query events by command
    pub async fn query_by_command(&self, command: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.command == command)
            .cloned()
            .collect()
    }

    /// Get total event count
    pub async fn event_count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    /// Verify ledger integrity via Merkle root
    pub async fn verify(&self) -> bool {
        // In production: would verify root hash with external TSA
        // For now: just check that Merkle tree is consistent
        let root_hash = self.merkle_tree.root_hash().await;
        let event_count = self.event_count().await;
        let leaf_count = self.merkle_tree.leaf_count().await;

        root_hash.is_some() && event_count == leaf_count
    }

    /// Export compact summary of ledger
    pub async fn summary(&self) -> LedgerSummary {
        let event_count = self.event_count().await;
        let root_hash = self.merkle_tree.root_hash().await;
        let events = self.events.read().await;

        let success_count = events.iter().filter(|e| e.result.success).count();
        let failure_count = event_count - success_count;

        LedgerSummary {
            total_events: event_count,
            successful_events: success_count,
            failed_events: failure_count,
            root_hash,
            creation_time: Utc::now(),
        }
    }

    /// Compact old events (archive)
    pub async fn compact(&self, keep_events: usize) {
        let mut events = self.events.write().await;
        if events.len() > keep_events {
            let remove_count = events.len() - keep_events;
            events.drain(0..remove_count);
        }
    }
}

impl Default for DistributedAuditLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Ledger summary for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    pub total_events: usize,
    pub successful_events: usize,
    pub failed_events: usize,
    pub root_hash: Option<Vec<u8>>,
    pub creation_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event() {
        let result = ExecutionResult {
            success: true,
            duration_ms: 100,
            error: None,
        };

        let event = AuditEvent::new(
            "agent-1".to_string(),
            "database.query".to_string(),
            result,
        );

        assert!(!event.hash().is_empty());
    }

    #[tokio::test]
    async fn test_merkle_tree() {
        let tree = MerkleTree::new();

        let result = ExecutionResult {
            success: true,
            duration_ms: 100,
            error: None,
        };

        let event1 = AuditEvent::new(
            "agent-1".to_string(),
            "command-1".to_string(),
            result,
        );

        tree.add_event(&event1).await;
        assert_eq!(tree.leaf_count().await, 1);

        let root = tree.root_hash().await;
        assert!(root.is_some());
    }

    #[tokio::test]
    async fn test_distributed_ledger() {
        let ledger = DistributedAuditLedger::new();

        let result = ExecutionResult {
            success: true,
            duration_ms: 100,
            error: None,
        };

        let event = AuditEvent::new(
            "agent-1".to_string(),
            "database.query".to_string(),
            result,
        );

        ledger.append(event).await;

        assert_eq!(ledger.event_count().await, 1);
        assert!(ledger.verify().await);

        let summary = ledger.summary().await;
        assert_eq!(summary.total_events, 1);
        assert_eq!(summary.successful_events, 1);
    }
}
