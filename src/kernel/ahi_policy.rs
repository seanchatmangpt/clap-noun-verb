//! Phase 6.2: AHI Policy Hook Integration
//!
//! Connects CNV to the AHI (Autonomic Heuristic Integration) system.
//!
//! AHI feeds ΔΣ (policy changes) into CNV broker:
//! - Enable/disable capabilities
//! - Update quotas
//! - Change tenancy policies
//! - Route policy decisions
//!
//! This makes CNV a fully autonomous, policy-governed service.

use crate::kernel::broker::BrokerKernel;
use crate::kernel::capability_contracts::CapabilityContractV2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Policy decision from AHI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    /// Enable a capability for a tenant
    EnableCapability {
        capability_id: String,
        tenant_id: String,
        until_timestamp: Option<u64>,
    },
    /// Disable a capability for a tenant
    DisableCapability {
        capability_id: String,
        tenant_id: String,
        reason: String,
    },
    /// Tighten quotas
    TightenQuota {
        capability_id: String,
        quota_type: String,
        new_limit: u64,
    },
    /// Relax quotas
    RelaxQuota {
        capability_id: String,
        quota_type: String,
        new_limit: u64,
    },
    /// Deprecate capability
    Deprecate {
        capability_id: String,
        replacement: Option<String>,
        deadline: u64,
    },
    /// Risk-based action
    RiskAction {
        capability_id: String,
        risk_class: String,
        action: RiskActionType,
    },
}

/// Risk-based policy action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskActionType {
    /// Allow (baseline)
    Allow,
    /// Require human approval
    RequireApproval,
    /// Log all executions
    LogExecution,
    /// Degrade service
    Degrade,
    /// Block execution
    Block,
}

/// Policy update (ΔΣ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdate {
    pub update_id: String,
    pub timestamp: u64,
    pub decisions: Vec<PolicyDecision>,
    pub priority: u32,
    pub expires_at: Option<u64>,
}

impl PolicyUpdate {
    /// Check if update is still valid
    pub fn is_valid(&self, current_timestamp: u64) -> bool {
        if let Some(expires) = self.expires_at {
            current_timestamp < expires
        } else {
            true
        }
    }
}

/// Policy state - applied policies with versioning
#[derive(Debug, Clone)]
pub struct PolicyState {
    pub enabled_capabilities: BTreeMap<String, CapabilityPolicy>,
    pub disabled_capabilities: BTreeMap<String, DisabledCapabilityInfo>,
    pub deprecations: BTreeMap<String, DeprecationInfo>,
    pub risk_policies: BTreeMap<String, RiskPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    pub capability_id: String,
    pub allowed_tenants: Vec<String>,
    pub forbidden_tenants: Vec<String>,
    pub quota_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabledCapabilityInfo {
    pub capability_id: String,
    pub disabled_at: u64,
    pub reason: String,
    pub tenant_exceptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    pub capability_id: String,
    pub replacement: Option<String>,
    pub deadline: u64,
    pub warnings_sent: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPolicy {
    pub capability_id: String,
    pub risk_class: String,
    pub action: RiskActionType,
    pub requires_audit: bool,
}

/// AHI Policy Adapter - bridges CNV to AHI
pub struct AhiPolicyAdapter {
    /// Reference to broker kernel
    pub broker: Arc<BrokerKernel>,
    /// Current policy state
    pub policy_state: parking_lot::RwLock<PolicyState>,
    /// Update history
    pub update_history: parking_lot::RwLock<Vec<PolicyUpdate>>,
}

impl AhiPolicyAdapter {
    /// Create a new AHI policy adapter
    pub fn new(broker: Arc<BrokerKernel>) -> Self {
        Self {
            broker,
            policy_state: parking_lot::RwLock::new(PolicyState {
                enabled_capabilities: BTreeMap::new(),
                disabled_capabilities: BTreeMap::new(),
                deprecations: BTreeMap::new(),
                risk_policies: BTreeMap::new(),
            }),
            update_history: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// Apply a policy update (ΔΣ from AHI)
    pub fn apply_policy_update(&self, update: PolicyUpdate) -> Result<(), String> {
        let mut state = self.policy_state.write();

        for decision in &update.decisions {
            match decision {
                PolicyDecision::EnableCapability {
                    capability_id,
                    tenant_id,
                    until_timestamp: _,
                } => {
                    let policy = state
                        .enabled_capabilities
                        .entry(capability_id.clone())
                        .or_insert_with(|| CapabilityPolicy {
                            capability_id: capability_id.clone(),
                            allowed_tenants: vec![],
                            forbidden_tenants: vec![],
                            quota_multiplier: 1.0,
                        });

                    if !policy.allowed_tenants.contains(tenant_id) {
                        policy.allowed_tenants.push(tenant_id.clone());
                    }

                    // Remove from disabled if present
                    state.disabled_capabilities.remove(capability_id);
                }

                PolicyDecision::DisableCapability {
                    capability_id,
                    tenant_id: _,
                    reason,
                } => {
                    let info = DisabledCapabilityInfo {
                        capability_id: capability_id.clone(),
                        disabled_at: update.timestamp,
                        reason: reason.clone(),
                        tenant_exceptions: vec![],
                    };

                    state
                        .disabled_capabilities
                        .insert(capability_id.clone(), info);
                }

                PolicyDecision::TightenQuota {
                    capability_id,
                    quota_type: _,
                    new_limit: _,
                } => {
                    // In a real implementation, update quotas
                    if let Some(policy) = state.enabled_capabilities.get_mut(capability_id) {
                        policy.quota_multiplier = 0.8; // Example: reduce by 20%
                    }
                }

                PolicyDecision::RelaxQuota {
                    capability_id,
                    quota_type: _,
                    new_limit: _,
                } => {
                    // In a real implementation, update quotas
                    if let Some(policy) = state.enabled_capabilities.get_mut(capability_id) {
                        policy.quota_multiplier = 1.2; // Example: increase by 20%
                    }
                }

                PolicyDecision::Deprecate {
                    capability_id,
                    replacement,
                    deadline,
                } => {
                    let info = DeprecationInfo {
                        capability_id: capability_id.clone(),
                        replacement: replacement.clone(),
                        deadline: *deadline,
                        warnings_sent: 0,
                    };

                    state
                        .deprecations
                        .insert(capability_id.clone(), info);
                }

                PolicyDecision::RiskAction {
                    capability_id,
                    risk_class,
                    action,
                } => {
                    let policy = RiskPolicy {
                        capability_id: capability_id.clone(),
                        risk_class: risk_class.clone(),
                        action: *action,
                        requires_audit: *action != RiskActionType::Allow,
                    };

                    state.risk_policies.insert(capability_id.clone(), policy);
                }
            }
        }

        // Record update
        let mut history = self.update_history.write();
        history.push(update);

        Ok(())
    }

    /// Check if capability is enabled for tenant
    pub fn is_enabled(&self, capability_id: &str, tenant_id: &str) -> bool {
        let state = self.policy_state.read();

        // Check if explicitly disabled
        if state.disabled_capabilities.contains_key(capability_id) {
            return false;
        }

        // Check if enabled
        if let Some(policy) = state.enabled_capabilities.get(capability_id) {
            return policy.allowed_tenants.is_empty()
                || policy.allowed_tenants.contains(&tenant_id.to_string());
        }

        // Default to enabled
        true
    }

    /// Get risk action for capability
    pub fn get_risk_action(&self, capability_id: &str) -> Option<RiskActionType> {
        let state = self.policy_state.read();
        state
            .risk_policies
            .get(capability_id)
            .map(|p| p.action)
    }

    /// Get deprecation info
    pub fn get_deprecation_info(&self, capability_id: &str) -> Option<DeprecationInfo> {
        let state = self.policy_state.read();
        state.deprecations.get(capability_id).cloned()
    }

    /// Check if capability requires approval
    pub fn requires_approval(&self, capability_id: &str) -> bool {
        if let Some(action) = self.get_risk_action(capability_id) {
            matches!(action, RiskActionType::RequireApproval)
        } else {
            false
        }
    }

    /// Audit log for policy decision
    pub fn audit_policy_decision(
        &self,
        capability_id: &str,
        tenant_id: &str,
        decision: &str,
    ) -> Result<(), String> {
        // In a real implementation, log to audit system
        println!(
            "[AUDIT] Policy decision for {}/{}: {}",
            capability_id, tenant_id, decision
        );
        Ok(())
    }
}

/// Policy validator - checks policy consistency
pub struct PolicyValidator;

impl PolicyValidator {
    /// Validate a policy update
    pub fn validate(update: &PolicyUpdate) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for conflicting decisions
        let mut enabled_capabilities = Vec::new();
        let mut disabled_capabilities = Vec::new();

        for decision in &update.decisions {
            match decision {
                PolicyDecision::EnableCapability {
                    capability_id, ..
                } => enabled_capabilities.push(capability_id.clone()),
                PolicyDecision::DisableCapability {
                    capability_id, ..
                } => disabled_capabilities.push(capability_id.clone()),
                _ => {}
            }
        }

        // Check for enable/disable conflicts
        for cap in &enabled_capabilities {
            if disabled_capabilities.contains(cap) {
                errors.push(format!(
                    "Conflicting enable/disable for capability {}",
                    cap
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::session_log::InMemorySessionLogStore;

    #[test]
    fn test_policy_update_validity() {
        let update = PolicyUpdate {
            update_id: "u1".to_string(),
            timestamp: 1000,
            decisions: vec![],
            priority: 1,
            expires_at: Some(2000),
        };

        assert!(update.is_valid(1500));
        assert!(!update.is_valid(2500));
    }

    #[test]
    fn test_policy_validator() {
        let update = PolicyUpdate {
            update_id: "u1".to_string(),
            timestamp: 1000,
            decisions: vec![
                PolicyDecision::EnableCapability {
                    capability_id: "cap1".to_string(),
                    tenant_id: "t1".to_string(),
                    until_timestamp: None,
                },
                PolicyDecision::DisableCapability {
                    capability_id: "cap1".to_string(),
                    tenant_id: "t1".to_string(),
                    reason: "test".to_string(),
                },
            ],
            priority: 1,
            expires_at: None,
        };

        let result = PolicyValidator::validate(&update);
        assert!(result.is_err());
    }

    #[test]
    fn test_ahi_adapter_enable_capability() {
        let store = Arc::new(InMemorySessionLogStore::new());
        let broker = Arc::new(BrokerKernel::new(store));
        let adapter = AhiPolicyAdapter::new(broker);

        let update = PolicyUpdate {
            update_id: "u1".to_string(),
            timestamp: 1000,
            decisions: vec![PolicyDecision::EnableCapability {
                capability_id: "cap1".to_string(),
                tenant_id: "t1".to_string(),
                until_timestamp: None,
            }],
            priority: 1,
            expires_at: None,
        };

        assert!(adapter.apply_policy_update(update).is_ok());
        assert!(adapter.is_enabled("cap1", "t1"));
    }
}
