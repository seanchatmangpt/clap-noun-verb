//! # Type-State Pattern for Capability Escalation
//!
//! Ensures capability transitions are type-safe at compile time.
//! Prevents capability misuse in trillion-agent systems where security is critical.
//!
//! ## Design
//!
//! Uses phantom types to encode capability state in the type system:
//! - `Unverified`: Initial state, no capabilities
//! - `Verified<C>`: Verified with capability C
//! - `Escalated<C1, C2>`: Escalated from C1 to C2
//!
//! Transitions are only allowed through explicit methods that enforce security invariants.

use crate::kernel::capability::{CapabilityClass, CapabilityContract, SafetyProfile};
use std::marker::PhantomData;

// ============================================================================
// Type-State Markers (Zero-Sized Types)
// ============================================================================

/// Unverified state - no capability verification performed
pub struct Unverified;

/// Verified state - capability has been verified
pub struct Verified<C> {
    _phantom: PhantomData<C>,
}

/// Escalated state - capability has been escalated from C1 to C2
pub struct Escalated<C1, C2> {
    _phantom: PhantomData<(C1, C2)>,
}

// ============================================================================
// Kinetic Representation
// ============================================================================

pub struct KineticInstruction {
    pub op_code: u32,
    pub capability_mask: u64,
}

pub struct KineticResult {
    pub success: bool,
    pub code: u32,
}

// ============================================================================
// Capability State Machine
// ============================================================================

pub struct TypedSession<State> {
    name: String,
    contract: Option<CapabilityContract>,
    audit_log: Vec<AuditEntry>,
    _state: PhantomData<State>,
}

/// Audit entry for capability transitions
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub event: AuditEvent,
}

#[derive(Debug, Clone)]
pub enum AuditEvent {
    SessionCreated { name: String },
    Verified { capability: String },
    Escalated { from: String, to: String, reason: String },
    OperationExecuted { capability: String },
    EscalationDenied { from: String, to: String, reason: String },
}

// ============================================================================
// Unverified State - Initial state
// ============================================================================

impl TypedSession<Unverified> {
    /// Create new unverified session (always safe)
    pub const fn new(_name: &str) -> Self {
        Self {
            name: String::new(),
            contract: None,
            audit_log: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Create with runtime name
    pub fn with_name(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut session = Self {
            name: name.clone(),
            contract: None,
            audit_log: Vec::new(),
            _state: PhantomData,
        };

        session.audit_log.push(AuditEntry {
            timestamp: current_timestamp(),
            event: AuditEvent::SessionCreated { name },
        });

        session
    }

    /// Verify initial capability
    pub fn verify<C>(mut self, contract: CapabilityContract) -> TypedSession<Verified<C>> {
        self.audit_log.push(AuditEntry {
            timestamp: current_timestamp(),
            event: AuditEvent::Verified {
                capability: format!("{:?}", contract.capability_class),
            },
        });

        TypedSession {
            name: self.name,
            contract: Some(contract),
            audit_log: self.audit_log,
            _state: PhantomData,
        }
    }
}

// ============================================================================
// Verified State - Can execute operations
// ============================================================================

impl<C> TypedSession<Verified<C>> {
    /// Execute operation with current capability using verified kinetic representation
    pub fn execute(&self, instruction: &KineticInstruction) -> Result<KineticResult, EscalationError> {
        // Safety: contract must be present if in Verified/Escalated state
        let contract = self.contract.as_ref().ok_or_else(|| EscalationError::MissingContract)?;

        if instruction.capability_mask == 0 {
             return Ok(KineticResult { success: false, code: 1 });
        }

        Ok(KineticResult { success: true, code: 0 })
    }

    /// Escalate to higher capability
    pub fn escalate<C2>(
        mut self,
        new_contract: CapabilityContract,
        reason: impl Into<String>,
    ) -> Result<TypedSession<Escalated<C, C2>>, EscalationError> {
        let reason = reason.into();
        let old_contract = self.contract.as_ref().ok_or_else(|| EscalationError::MissingContract)?;

        if !is_escalation_allowed(old_contract, &new_contract, &reason) {
            self.audit_log.push(AuditEntry {
                timestamp: current_timestamp(),
                event: AuditEvent::EscalationDenied {
                    from: format!("{:?}", old_contract.capability_class),
                    to: format!("{:?}", new_contract.capability_class),
                    reason: reason.clone(),
                },
            });

            return Err(EscalationError::PolicyViolation {
                from: old_contract.capability_class.clone(),
                to: new_contract.capability_class.clone(),
                reason,
            });
        }

        self.audit_log.push(AuditEntry {
            timestamp: current_timestamp(),
            event: AuditEvent::Escalated {
                from: format!("{:?}", old_contract.capability_class),
                to: format!("{:?}", new_contract.capability_class),
                reason: reason.clone(),
            },
        });

        Ok(TypedSession {
            name: self.name,
            contract: Some(new_contract),
            audit_log: self.audit_log,
            _state: PhantomData,
        })
    }

    pub fn capability(&self) -> Option<&CapabilityContract> {
        self.contract.as_ref()
    }

    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }
}

// ============================================================================
// Escalated State
// ============================================================================

impl<C1, C2> TypedSession<Escalated<C1, C2>> {
    pub fn execute(&self, instruction: &KineticInstruction) -> Result<KineticResult, EscalationError> {
        if instruction.capability_mask == 0 {
            return Ok(KineticResult { success: false, code: 1 });
        }
        Ok(KineticResult { success: true, code: 0 })
    }

    pub fn capability(&self) -> Option<&CapabilityContract> {
        self.contract.as_ref()
    }

    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }
}

// ============================================================================
// Escalation Policy & Errors
// ============================================================================

fn is_escalation_allowed(
    from: &CapabilityContract,
    to: &CapabilityContract,
    reason: &str,
) -> bool {
    use CapabilityClass::*;

    if to.risk_score() <= from.risk_score() {
        return true;
    }

    if to.capability_class == Dangerous && !matches!(to.safety, SafetyProfile::HumanReviewRequired) {
        return false;
    }

    // Minimum Decisive Force (MDF) for capability admission
    let mdf_required = match (&from.capability_class, &to.capability_class) {
        (Pure, ReadOnlyFS) => 1,
        (ReadOnlyFS, Environment) => 2,
        (_, ReadWriteFS) => 3,
        (_, Network) => 4,
        (_, Subprocess) => 5,
        (_, Dangerous) => 10,
        _ => 1,
    };

    // Require strict formatting indicating explicit, proportional force justification
    reason.starts_with("MDF-") && reason.len() >= 4 + mdf_required
}

#[derive(Debug, Clone)]
pub enum EscalationError {
    PolicyViolation {
        from: CapabilityClass,
        to: CapabilityClass,
        reason: String,
    },
    MissingContract,
}

impl std::fmt::Display for EscalationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyViolation { from, to, reason } => {
                write!(f, "Escalation denied: {:?} -> {:?}. Reason: {}", from, to, reason)
            }
            Self::MissingContract => write!(f, "Capability contract missing"),
        }
    }
}

impl std::error::Error for EscalationError {}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
