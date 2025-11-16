//! Phase 6.3: Attested Execution Receipts
//!
//! Every CNV invocation can emit a **CapabilityExecutionReceipt**:
//! - Capability ID + version
//! - Invocation attestation
//! - Tenant + agent identity
//! - Quota tier + actual usage
//! - Policy ID that allowed/rewrote the request
//! - Outcome + effect summary
//!
//! These receipts are:
//! - Signed by the broker
//! - Linkable into AHI's global receipt graph (Γ)
//! - Queryable for analytics and compliance

use crate::kernel::session_log::{ExitCodeClass, QuotaFootprint};
use crate::autonomic::capability_id::CapabilityId;
use crate::autonomic::tenancy::{AgentIdentity, TenantIdentity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Attested execution receipt - proof of invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExecutionReceipt {
    /// Receipt ID
    pub receipt_id: String,

    /// Capability that was invoked
    pub capability_id: CapabilityId,
    pub capability_version: u32,

    /// Invocation context
    pub tenant_id: TenantIdentity,
    pub agent_id: AgentIdentity,

    /// Attestation chain hash used for this invocation
    pub invocation_attestation_hash: Option<String>,

    /// Quota information
    pub quota_tier: String,
    pub quota_footprint: QuotaFootprint,

    /// Policy that was applied
    pub policy_id: String,
    pub policy_version: u32,

    /// Outcome
    pub exit_code: ExitCodeClass,
    pub success: bool,

    /// Effect summary
    pub effect_summary: EffectSummary,

    /// Signature (signed by broker)
    pub signature: Option<String>,

    /// Parent receipt hash (for causal chains)
    pub parent_receipt_hash: Option<String>,

    /// Metadata tags
    pub tags: BTreeMap<String, String>,

    /// Timestamp (nanoseconds since epoch)
    pub timestamp_ns: u64,
}

/// Summary of effects from invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSummary {
    /// Files created/modified
    pub files_affected: Vec<String>,

    /// Processes spawned
    pub processes_spawned: Vec<String>,

    /// Network connections made
    pub network_connections: Vec<NetworkConnection>,

    /// Environment variables modified
    pub env_vars_modified: Vec<String>,

    /// Data classification of inputs/outputs
    pub data_classification: Option<String>,
}

/// Network connection record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: String,
    pub peer_address: String,
    pub bytes_transferred: u64,
    pub duration_ms: u64,
}

impl CapabilityExecutionReceipt {
    /// Create a new execution receipt
    pub fn new(
        receipt_id: String,
        capability_id: CapabilityId,
        capability_version: u32,
        tenant_id: TenantIdentity,
        agent_id: AgentIdentity,
        quota_tier: String,
        quota_footprint: QuotaFootprint,
        policy_id: String,
        exit_code: ExitCodeClass,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            receipt_id,
            capability_id,
            capability_version,
            tenant_id,
            agent_id,
            invocation_attestation: None,
            quota_tier,
            quota_footprint,
            policy_id,
            policy_version: 1,
            exit_code,
            success: exit_code == ExitCodeClass::Success,
            effect_summary: EffectSummary {
                files_affected: vec![],
                processes_spawned: vec![],
                network_connections: vec![],
                env_vars_modified: vec![],
                data_classification: None,
            },
            signature: None,
            parent_receipt_hash: None,
            tags: BTreeMap::new(),
            timestamp_ns,
        }
    }

    /// Add a file effect
    pub fn add_file_effect(&mut self, file_path: String) {
        if !self.effect_summary.files_affected.contains(&file_path) {
            self.effect_summary.files_affected.push(file_path);
        }
    }

    /// Add a network connection
    pub fn add_network_connection(&mut self, connection: NetworkConnection) {
        self.effect_summary.network_connections.push(connection);
    }

    /// Compute receipt hash (for chaining)
    pub fn compute_hash(&self) -> Result<String, serde_json::Error> {
        use sha2::{Sha256, Digest};

        let json = serde_json::to_string(&self)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Sign the receipt (mock implementation)
    pub fn sign(&mut self, _signing_key: &str) -> Result<(), String> {
        // In a real implementation, sign with the broker's private key
        self.signature = Some("mock-signature".to_string());
        Ok(())
    }

    /// Verify receipt signature (mock implementation)
    pub fn verify_signature(&self, _verifying_key: &str) -> bool {
        // In a real implementation, verify with the broker's public key
        self.signature.is_some()
    }

    /// Get receipt metadata for filing
    pub fn get_metadata(&self) -> ReceiptMetadata {
        ReceiptMetadata {
            receipt_id: self.receipt_id.clone(),
            capability_id: self.capability_id.as_str().to_string(),
            tenant_id: self.tenant_id.tenant_id.clone(),
            agent_id: self.agent_id.agent_id.clone(),
            timestamp_ns: self.timestamp_ns,
            success: self.success,
            quota_footprint: self.quota_footprint.clone(),
        }
    }
}

/// Receipt metadata for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    pub receipt_id: String,
    pub capability_id: String,
    pub tenant_id: String,
    pub agent_id: String,
    pub timestamp_ns: u64,
    pub success: bool,
    pub quota_footprint: QuotaFootprint,
}

/// Receipt store trait
pub trait ReceiptStore: Send + Sync {
    /// Store a receipt
    fn store_receipt(&self, receipt: CapabilityExecutionReceipt) -> Result<(), String>;

    /// Retrieve a receipt by ID
    fn get_receipt(&self, receipt_id: &str) -> Result<Option<CapabilityExecutionReceipt>, String>;

    /// Query receipts by predicate
    fn query_receipts(
        &self,
        predicate: &dyn Fn(&CapabilityExecutionReceipt) -> bool,
    ) -> Result<Vec<CapabilityExecutionReceipt>, String>;

    /// Get receipts for tenant
    fn get_tenant_receipts(&self, tenant_id: &str) -> Result<Vec<CapabilityExecutionReceipt>, String>;

    /// Get receipts for capability
    fn get_capability_receipts(&self, capability_id: &str)
        -> Result<Vec<CapabilityExecutionReceipt>, String>;

    /// Get usage statistics
    fn get_usage_stats(&self, tenant_id: &str) -> Result<UsageStatistics, String>;
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub tenant_id: String,
    pub total_invocations: u64,
    pub successful_invocations: u64,
    pub failed_invocations: u64,
    pub total_runtime_ms: u64,
    pub total_memory_gb: f64,
    pub total_io_operations: u64,
    pub total_network_bytes: u64,
    pub average_runtime_ms: f64,
}

/// In-memory receipt store
pub struct InMemoryReceiptStore {
    receipts: parking_lot::RwLock<BTreeMap<String, CapabilityExecutionReceipt>>,
}

impl InMemoryReceiptStore {
    pub fn new() -> Self {
        Self {
            receipts: parking_lot::RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for InMemoryReceiptStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceiptStore for InMemoryReceiptStore {
    fn store_receipt(&self, receipt: CapabilityExecutionReceipt) -> Result<(), String> {
        let mut receipts = self.receipts.write();
        receipts.insert(receipt.receipt_id.clone(), receipt);
        Ok(())
    }

    fn get_receipt(&self, receipt_id: &str) -> Result<Option<CapabilityExecutionReceipt>, String> {
        let receipts = self.receipts.read();
        Ok(receipts.get(receipt_id).cloned())
    }

    fn query_receipts(
        &self,
        predicate: &dyn Fn(&CapabilityExecutionReceipt) -> bool,
    ) -> Result<Vec<CapabilityExecutionReceipt>, String> {
        let receipts = self.receipts.read();
        Ok(receipts.values().filter(|r| predicate(r)).cloned().collect())
    }

    fn get_tenant_receipts(&self, tenant_id: &str) -> Result<Vec<CapabilityExecutionReceipt>, String> {
        let receipts = self.receipts.read();
        Ok(receipts
            .values()
            .filter(|r| r.tenant_id.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    fn get_capability_receipts(
        &self,
        capability_id: &str,
    ) -> Result<Vec<CapabilityExecutionReceipt>, String> {
        let receipts = self.receipts.read();
        Ok(receipts
            .values()
            .filter(|r| r.capability_id.as_str() == capability_id)
            .cloned()
            .collect())
    }

    fn get_usage_stats(&self, tenant_id: &str) -> Result<UsageStatistics, String> {
        let receipts = self.get_tenant_receipts(tenant_id)?;

        if receipts.is_empty() {
            return Ok(UsageStatistics {
                tenant_id: tenant_id.to_string(),
                total_invocations: 0,
                successful_invocations: 0,
                failed_invocations: 0,
                total_runtime_ms: 0,
                total_memory_gb: 0.0,
                total_io_operations: 0,
                total_network_bytes: 0,
                average_runtime_ms: 0.0,
            });
        }

        let total_invocations = receipts.len() as u64;
        let successful = receipts.iter().filter(|r| r.success).count() as u64;
        let failed = total_invocations - successful;
        let total_runtime: u64 = receipts.iter().map(|r| r.quota_footprint.runtime_ms).sum();
        let total_memory: u64 = receipts.iter().map(|r| r.quota_footprint.peak_memory_bytes).sum();
        let total_io: u64 = receipts.iter().map(|r| r.quota_footprint.io_operations).sum();
        let total_network: u64 = receipts.iter().map(|r| r.quota_footprint.network_bytes).sum();

        Ok(UsageStatistics {
            tenant_id: tenant_id.to_string(),
            total_invocations,
            successful_invocations: successful,
            failed_invocations: failed,
            total_runtime_ms: total_runtime,
            total_memory_gb: total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            total_io_operations: total_io,
            total_network_bytes: total_network,
            average_runtime_ms: if total_invocations > 0 {
                total_runtime as f64 / total_invocations as f64
            } else {
                0.0
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_receipt_creation() {
        let receipt = CapabilityExecutionReceipt::new(
            "r1".to_string(),
            CapabilityId {
                id: "cap1".to_string(),
                version: 1,
            },
            1,
            TenantIdentity {
                id: "t1".to_string(),
            },
            AgentIdentity {
                id: "a1".to_string(),
            },
            "standard".to_string(),
            QuotaFootprint::zero(),
            "policy1".to_string(),
            ExitCodeClass::Success,
            1000,
        );

        assert_eq!(receipt.receipt_id, "r1");
        assert!(receipt.success);
    }

    #[test]
    fn test_in_memory_receipt_store() {
        let store = InMemoryReceiptStore::new();

        let receipt = CapabilityExecutionReceipt::new(
            "r1".to_string(),
            CapabilityId {
                id: "cap1".to_string(),
                version: 1,
            },
            1,
            TenantIdentity {
                id: "t1".to_string(),
            },
            AgentIdentity {
                id: "a1".to_string(),
            },
            "standard".to_string(),
            QuotaFootprint::zero(),
            "policy1".to_string(),
            ExitCodeClass::Success,
            1000,
        );

        assert!(store.store_receipt(receipt.clone()).is_ok());
        assert_eq!(store.get_receipt("r1").unwrap().unwrap().receipt_id, "r1");
    }

    #[test]
    fn test_usage_statistics() {
        let store = InMemoryReceiptStore::new();

        let receipt = CapabilityExecutionReceipt::new(
            "r1".to_string(),
            CapabilityId {
                id: "cap1".to_string(),
                version: 1,
            },
            1,
            TenantIdentity {
                id: "t1".to_string(),
            },
            AgentIdentity {
                id: "a1".to_string(),
            },
            "standard".to_string(),
            QuotaFootprint {
                runtime_ms: 100,
                peak_memory_bytes: 1024,
                io_operations: 10,
                network_bytes: 512,
                cpu_cycles: None,
            },
            "policy1".to_string(),
            ExitCodeClass::Success,
            1000,
        );

        store.store_receipt(receipt).unwrap();
        let stats = store.get_usage_stats("t1").unwrap();
        assert_eq!(stats.total_invocations, 1);
        assert_eq!(stats.successful_invocations, 1);
        assert_eq!(stats.total_runtime_ms, 100);
    }
}
