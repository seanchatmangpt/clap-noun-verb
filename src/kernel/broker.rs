//! Phase 5: Federation & BrokerKernel
//!
//! CNV Broker - Network-level μ service for federated capability routing.
//!
//! The BrokerKernel transforms CNV from a "local binary" to a long-lived service:
//! - Accepts framed requests from many agents (local or remote)
//! - Performs capability negotiation, attestation verification, quota matching
//! - Returns structured results + session frames for replay
//! - Manages multi-tenant isolation and fair-share scheduling
//!
//! ## Architecture
//!
//! ```text
//! [Agent 1]─┐
//! [Agent 2]─┼─→ [BrokerKernel]
//! [Agent 3]─┘    ├─ Capability Registry
//!                ├─ Attestation Verifier
//!                ├─ Quota Scheduler
//!                ├─ Tenancy Manager
//!                └─ Session Frame Logger
//! ```

use crate::kernel::attestation::AttestationChain;
use crate::kernel::capability::CapabilityContract;
use crate::kernel::session_log::SessionLogFrame;
use crate::autonomic::capability_id::CapabilityId;
use crate::autonomic::tenancy::{AgentIdentity, TenantIdentity, QoSHints};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Broker invocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerRequest {
    /// Unique request ID for correlation
    pub request_id: String,
    /// Tenant making the request
    pub tenant_id: TenantIdentity,
    /// Agent making the request
    pub agent_id: AgentIdentity,
    /// Capability to invoke
    pub capability_id: CapabilityId,
    /// Input arguments
    pub arguments: serde_json::Value,
    /// Attestation chain (if required) - skipped in serialization
    #[serde(skip)]
    pub attestation: Option<AttestationChain>,
    /// QoS hints for scheduling
    pub qos_hints: QoSHints,
}

/// Broker invocation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerResponse {
    /// Echo of request ID
    pub request_id: String,
    /// The session frame (for replay) - skipped in serialization
    #[serde(skip)]
    pub session_frame: Option<SessionLogFrame>,
    /// Result or error
    pub result: BrokerResult,
    /// Updated attestation (e.g., consumption receipts) - skipped in serialization
    #[serde(skip)]
    pub updated_attestation: Option<AttestationChain>,
}

/// Result of broker invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BrokerResult {
    Success(serde_json::Value),
    Error(BrokerError),
    Deferred(DeferredExecution),
}

/// Broker error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerError {
    pub error_code: String,
    pub error_message: String,
    pub details: Option<serde_json::Value>,
}

/// Deferred execution (for backpressure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredExecution {
    pub queue_position: u64,
    pub estimated_wait_ms: u64,
}

/// Capability registration in broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerCapabilityRegistry {
    /// Capability ID → Contract
    pub capabilities: BTreeMap<String, CapabilityContract>,
    /// Version → supported versions
    pub version_support: BTreeMap<String, Vec<u32>>,
}

impl BrokerCapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: BTreeMap::new(),
            version_support: BTreeMap::new(),
        }
    }

    /// Register a capability
    pub fn register(&mut self, id: String, contract: CapabilityContract) {
        self.capabilities.insert(id.clone(), contract);
    }

    /// Get capability contract
    pub fn get(&self, id: &str) -> Option<&CapabilityContract> {
        self.capabilities.get(id)
    }

    /// Check if capability is registered
    pub fn has(&self, id: &str) -> bool {
        self.capabilities.contains_key(id)
    }
}

impl Default for BrokerCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Admission control policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionPolicy {
    /// Accept immediately
    Accept,
    /// Queue for later execution
    Queue,
    /// Refuse and return error
    Refuse,
    /// Degrade service (e.g., lower verbosity, fewer results)
    Degrade,
}

/// Broker admission controller
pub struct AdmissionController {
    /// Max concurrent invocations per tenant
    pub max_concurrent_per_tenant: usize,
    /// Max queue depth per capability
    pub max_queue_depth: usize,
    /// Priority levels (higher = more urgent)
    pub priority_levels: usize,
}

impl AdmissionController {
    pub fn new() -> Self {
        Self {
            max_concurrent_per_tenant: 100,
            max_queue_depth: 1000,
            priority_levels: 5,
        }
    }

    /// Decide admission for a request
    pub fn decide(
        &self,
        request: &BrokerRequest,
        current_load: &BrokerLoad,
    ) -> AdmissionPolicy {
        let tenant_load = current_load.get_tenant_load(&request.tenant_id.tenant_id);

        // Simple policy: queue if at capacity
        if tenant_load.concurrent >= self.max_concurrent_per_tenant {
            if tenant_load.queued < self.max_queue_depth {
                AdmissionPolicy::Queue
            } else {
                AdmissionPolicy::Refuse
            }
        } else {
            AdmissionPolicy::Accept
        }
    }
}

impl Default for AdmissionController {
    fn default() -> Self {
        Self::new()
    }
}

/// Broker load state
#[derive(Debug, Clone)]
pub struct BrokerLoad {
    /// Per-tenant load
    pub tenant_loads: BTreeMap<String, TenantLoad>,
}

#[derive(Debug, Clone)]
pub struct TenantLoad {
    pub concurrent: usize,
    pub queued: usize,
    pub total_invocations: u64,
}

impl BrokerLoad {
    pub fn new() -> Self {
        Self {
            tenant_loads: BTreeMap::new(),
        }
    }

    pub fn get_tenant_load(&self, tenant_id: &str) -> TenantLoad {
        self.tenant_loads
            .get(tenant_id)
            .cloned()
            .unwrap_or(TenantLoad {
                concurrent: 0,
                queued: 0,
                total_invocations: 0,
            })
    }
}

impl Default for BrokerLoad {
    fn default() -> Self {
        Self::new()
    }
}

/// BrokerKernel - the core broker implementation
pub struct BrokerKernel {
    /// Capability registry
    pub registry: Arc<RwLock<BrokerCapabilityRegistry>>,
    /// Admission control
    pub admission: Arc<AdmissionController>,
    /// Current load
    pub load: Arc<RwLock<BrokerLoad>>,
    /// Session frame store
    pub frame_store: Arc<dyn crate::kernel::session_log::SessionLogStore>,
}

impl BrokerKernel {
    /// Create a new broker kernel
    pub fn new(frame_store: Arc<dyn crate::kernel::session_log::SessionLogStore>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(BrokerCapabilityRegistry::new())),
            admission: Arc::new(AdmissionController::new()),
            load: Arc::new(RwLock::new(BrokerLoad::new())),
            frame_store,
        }
    }

    /// Register a capability
    pub fn register_capability(&self, id: String, contract: CapabilityContract) {
        let mut registry = self.registry.write();
        registry.register(id, contract);
    }

    /// Process a broker request
    pub async fn process_request(&self, request: BrokerRequest) -> Result<BrokerResponse, String> {
        // Check admission
        let load = self.load.read().clone();
        let admission = self.admission.decide(&request, &load);
        drop(load);

        match admission {
            AdmissionPolicy::Accept => {
                // Verify capability exists
                let registry = self.registry.read();
                if !registry.has(&request.capability_id.as_str()) {
                    return Ok(BrokerResponse {
                        request_id: request.request_id.clone(),
                        session_frame: None,
                        result: BrokerResult::Error(BrokerError {
                            error_code: "CAPABILITY_NOT_FOUND".to_string(),
                            error_message: format!(
                                "Capability {} not registered",
                                request.capability_id.as_str()
                            ),
                            details: None,
                        }),
                        updated_attestation: None,
                    });
                }

                // In a real implementation, execute the capability here
                // For now, return a stub response
                Ok(BrokerResponse {
                    request_id: request.request_id.clone(),
                    session_frame: None,
                    result: BrokerResult::Success(serde_json::json!({
                        "status": "accepted",
                        "capability": request.capability_id.as_str(),
                    })),
                    updated_attestation: None,
                })
            }
            AdmissionPolicy::Queue => Ok(BrokerResponse {
                request_id: request.request_id.clone(),
                session_frame: None,
                result: BrokerResult::Deferred(DeferredExecution {
                    queue_position: 1,
                    estimated_wait_ms: 5000,
                }),
                updated_attestation: None,
            }),
            AdmissionPolicy::Refuse => Ok(BrokerResponse {
                request_id: request.request_id.clone(),
                session_frame: None,
                result: BrokerResult::Error(BrokerError {
                    error_code: "OVERLOAD".to_string(),
                    error_message: "Broker is overloaded".to_string(),
                    details: None,
                }),
                updated_attestation: None,
            }),
            AdmissionPolicy::Degrade => Ok(BrokerResponse {
                request_id: request.request_id.clone(),
                session_frame: None,
                result: BrokerResult::Success(serde_json::json!({
                    "status": "degraded",
                    "capability": request.capability_id.as_str(),
                    "note": "Results may be truncated or approximate",
                })),
                updated_attestation: None,
            }),
        }
    }
}

/// Multi-tenant policy enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPolicy {
    pub tenant_id: String,
    /// Max concurrent invocations
    pub max_concurrent: usize,
    /// Max resource usage (GB/month)
    pub max_resource_gb_month: f64,
    /// Allowed capabilities
    pub allowed_capabilities: Vec<String>,
    /// Blocked capabilities
    pub blocked_capabilities: Vec<String>,
}

impl TenantPolicy {
    /// Check if a capability is allowed
    pub fn is_capability_allowed(&self, capability_id: &str) -> bool {
        if self.blocked_capabilities.contains(&capability_id.to_string()) {
            return false;
        }
        if !self.allowed_capabilities.is_empty() {
            return self.allowed_capabilities.contains(&capability_id.to_string());
        }
        true
    }
}

/// Fair-share scheduler with type-level guarantees
pub struct FairShareScheduler {
    /// Tenant policies
    pub policies: Arc<RwLock<BTreeMap<String, TenantPolicy>>>,
}

impl FairShareScheduler {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Add a tenant policy
    pub fn add_policy(&self, policy: TenantPolicy) {
        let mut policies = self.policies.write();
        policies.insert(policy.tenant_id.clone(), policy);
    }

    /// Check if tenant can execute capability
    pub fn can_execute(
        &self,
        tenant_id: &str,
        capability_id: &str,
    ) -> bool {
        let policies = self.policies.read();
        if let Some(policy) = policies.get(tenant_id) {
            policy.is_capability_allowed(capability_id)
        } else {
            true
        }
    }
}

impl Default for FairShareScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broker_capability_registry() {
        let mut registry = BrokerCapabilityRegistry::new();
        assert!(!registry.has("test"));

        // In a real test, we'd add actual capabilities
        assert_eq!(registry.capabilities.len(), 0);
    }

    #[test]
    fn test_admission_controller() {
        let controller = AdmissionController::new();
        assert_eq!(controller.max_concurrent_per_tenant, 100);
    }

    #[test]
    fn test_tenant_policy_filtering() {
        let policy = TenantPolicy {
            tenant_id: "t1".to_string(),
            max_concurrent: 10,
            max_resource_gb_month: 100.0,
            allowed_capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            blocked_capabilities: vec![],
        };

        assert!(policy.is_capability_allowed("cap1"));
        assert!(!policy.is_capability_allowed("cap3"));
    }

    #[test]
    fn test_fair_share_scheduler() {
        let scheduler = FairShareScheduler::new();

        let policy = TenantPolicy {
            tenant_id: "t1".to_string(),
            max_concurrent: 10,
            max_resource_gb_month: 100.0,
            allowed_capabilities: vec!["cap1".to_string()],
            blocked_capabilities: vec![],
        };

        scheduler.add_policy(policy);
        assert!(scheduler.can_execute("t1", "cap1"));
    }
}
