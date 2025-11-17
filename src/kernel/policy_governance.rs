//! Policy Adapter Governance Guardrails (Task 5)
//!
//! Typed policy deltas with legal transition enforcement and policy snapshots

use serde::{Deserialize, Serialize};

/// Typed policy delta with strong semantics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyDelta {
    /// Tighten quotas or add restrictions
    Tighten {
        capability_id: String,
        reason: String,
    },
    /// Relax quotas or remove restrictions
    Relax {
        capability_id: String,
        reason: String,
    },
    /// Mark as deprecated (transition to ApprovalRequired)
    Deprecate {
        capability_id: String,
        deadline: String,
        replacement: Option<String>,
    },
    /// Enable a capability
    Enable {
        capability_id: String,
        tenant_id: Option<String>,
    },
    /// Disable a capability (requires approval workflow for re-enable)
    Disable {
        capability_id: String,
        reason: String,
    },
}

impl PolicyDelta {
    pub fn capability_id(&self) -> &str {
        match self {
            Self::Tighten { capability_id, .. } => capability_id,
            Self::Relax { capability_id, .. } => capability_id,
            Self::Deprecate { capability_id, .. } => capability_id,
            Self::Enable { capability_id, .. } => capability_id,
            Self::Disable { capability_id, .. } => capability_id,
        }
    }
}

/// Policy state for legal transition tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyState {
    /// Capability is enabled and active
    Enabled,
    /// Capability is deprecated, pending removal
    Deprecated,
    /// Capability requires explicit approval for re-enabling
    ApprovalRequired,
    /// Capability is disabled
    Disabled,
}

impl PolicyState {
    /// Check if transition from this state to another is legal
    pub fn can_transition_to(&self, target: PolicyState, delta: &PolicyDelta) -> bool {
        match (self, target, delta) {
            // Enable from disabled (requires ApprovalRequired first)
            (Self::Disabled, Self::Enabled, PolicyDelta::Enable { .. }) => false,
            // Disabled → ApprovalRequired → Enabled (legal path)
            (Self::Disabled, Self::ApprovalRequired, PolicyDelta::Disable { .. }) => true,
            (Self::ApprovalRequired, Self::Enabled, PolicyDelta::Enable { .. }) => true,
            // Enabled → Deprecated (legal)
            (Self::Enabled, Self::Deprecated, PolicyDelta::Deprecate { .. }) => true,
            // Any tighten/relax from enabled (legal)
            (Self::Enabled, Self::Enabled, PolicyDelta::Tighten { .. }) => true,
            (Self::Enabled, Self::Enabled, PolicyDelta::Relax { .. }) => true,
            // Disable from enabled (legal)
            (Self::Enabled, Self::Disabled, PolicyDelta::Disable { .. }) => true,
            // Other transitions not allowed
            _ => false,
        }
    }
}

/// Policy snapshot captured at invocation time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub policy_version: u32,
    pub policy_hash: String,
    pub timestamp_ns: u64,
    pub enabled_capabilities: Vec<String>,
    pub deprecated_capabilities: Vec<String>,
    pub capability_state: String,  // Serialized state map
}

impl PolicySnapshot {
    /// Create a snapshot with version and hash
    pub fn new(
        policy_version: u32,
        policy_hash: String,
        enabled_capabilities: Vec<String>,
        deprecated_capabilities: Vec<String>,
    ) -> Self {
        Self {
            policy_version,
            policy_hash,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            enabled_capabilities,
            deprecated_capabilities,
            capability_state: String::new(),
        }
    }
}

/// Policy transition validator
pub struct PolicyTransitionValidator {
    current_state: PolicyState,
}

impl PolicyTransitionValidator {
    pub fn new(current_state: PolicyState) -> Self {
        Self { current_state }
    }

    /// Validate a policy delta against current state
    pub fn validate_transition(&self, delta: &PolicyDelta, target_state: PolicyState) -> Result<(), TransitionError> {
        if !self.current_state.can_transition_to(target_state, delta) {
            return Err(TransitionError::IllegalTransition {
                from: self.current_state,
                to: target_state,
                attempted_delta: format!("{:?}", delta),
            });
        }

        // Check for conflicting deltas
        match delta {
            PolicyDelta::Enable { .. } if self.current_state == PolicyState::Disabled => {
                return Err(TransitionError::DisabledRequiresApproval {
                    capability_id: delta.capability_id().to_string(),
                });
            }
            _ => {}
        }

        Ok(())
    }
}

/// Policy transition error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionError {
    IllegalTransition {
        from: PolicyState,
        to: PolicyState,
        attempted_delta: String,
    },
    DisabledRequiresApproval {
        capability_id: String,
    },
    ConflictingRules {
        rules: Vec<String>,
    },
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalTransition {
                from,
                to,
                attempted_delta,
            } => {
                write!(
                    f,
                    "Illegal policy transition: {:?} -> {:?} (attempted: {})",
                    from, to, attempted_delta
                )
            }
            Self::DisabledRequiresApproval { capability_id } => {
                write!(
                    f,
                    "Cannot re-enable {} without passing through ApprovalRequired state",
                    capability_id
                )
            }
            Self::ConflictingRules { rules } => {
                write!(f, "Conflicting policy rules: {:?}", rules)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_delta_tighten() {
        let delta = PolicyDelta::Tighten {
            capability_id: "test.cap".to_string(),
            reason: "Security hardening".to_string(),
        };
        assert_eq!(delta.capability_id(), "test.cap");
    }

    #[test]
    fn test_policy_state_valid_transitions() {
        let validator = PolicyTransitionValidator::new(PolicyState::Enabled);
        let delta = PolicyDelta::Deprecate {
            capability_id: "test.cap".to_string(),
            deadline: "2025-12-31".to_string(),
            replacement: None,
        };
        let result = validator.validate_transition(&delta, PolicyState::Deprecated);
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_state_illegal_transition() {
        let validator = PolicyTransitionValidator::new(PolicyState::Disabled);
        let delta = PolicyDelta::Enable {
            capability_id: "test.cap".to_string(),
            tenant_id: None,
        };
        let result = validator.validate_transition(&delta, PolicyState::Enabled);
        assert!(result.is_err());
    }

    #[test]
    fn test_policy_state_approval_required_path() {
        // Disabled → ApprovalRequired
        let validator = PolicyTransitionValidator::new(PolicyState::Disabled);
        let delta = PolicyDelta::Disable {
            capability_id: "test.cap".to_string(),
            reason: "Manual disable".to_string(),
        };
        let result = validator.validate_transition(&delta, PolicyState::ApprovalRequired);
        assert!(result.is_ok());

        // ApprovalRequired → Enabled
        let validator = PolicyTransitionValidator::new(PolicyState::ApprovalRequired);
        let delta = PolicyDelta::Enable {
            capability_id: "test.cap".to_string(),
            tenant_id: None,
        };
        let result = validator.validate_transition(&delta, PolicyState::Enabled);
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_snapshot_creation() {
        let snapshot = PolicySnapshot::new(
            1,
            "hash_abc123".to_string(),
            vec!["cap1".to_string(), "cap2".to_string()],
            vec![],
        );
        assert_eq!(snapshot.policy_version, 1);
        assert_eq!(snapshot.enabled_capabilities.len(), 2);
    }

    #[test]
    fn test_transition_error_display() {
        let error = TransitionError::DisabledRequiresApproval {
            capability_id: "test.cap".to_string(),
        };
        assert!(error.to_string().contains("ApprovalRequired"));
    }
}
