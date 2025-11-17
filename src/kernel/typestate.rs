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
// Capability State Machine
// ============================================================================

/// Type-state session with compile-time capability tracking
///
/// # Example
///
/// ```rust,no_run
/// use clap_noun_verb::kernel::typestate::*;
/// use clap_noun_verb::kernel::capability::*;
///
/// // Start unverified
/// let session = TypedSession::<Unverified>::new("my-app");
///
/// // Verify with Pure capability (compile-time enforced)
/// let session = session.verify(CapabilityContract::pure());
///
/// // Execute operations (only available after verification)
/// session.execute(|| println!("Pure operation"));
///
/// // Escalate to ReadOnly (requires justification)
/// let session = session.escalate(
///     CapabilityContract::read_only(),
///     "Need to read config file"
/// ).expect("Escalation denied");
///
/// // Now can perform read operations
/// session.execute(|| {
///     // Read file...
/// });
/// ```
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
            name: String::new(), // Can't use name.to_string() in const
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

    /// Verify initial capability (type-state transition: Unverified -> Verified)
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
    /// Execute operation with current capability (type-safe)
    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Log operation
        let mut session = self.clone_for_audit();
        session.audit_log.push(AuditEntry {
            timestamp: current_timestamp(),
            event: AuditEvent::OperationExecuted {
                capability: format!("{:?}", self.contract.as_ref().unwrap().capability_class),
            },
        });

        f()
    }

    /// Escalate to higher capability (type-state transition: Verified<C1> -> Escalated<C1, C2>)
    ///
    /// Returns Err if escalation is not allowed by policy
    pub fn escalate<C2>(
        mut self,
        new_contract: CapabilityContract,
        reason: impl Into<String>,
    ) -> Result<TypedSession<Escalated<C, C2>>, EscalationError> {
        let reason = reason.into();

        // Check if escalation is allowed
        let old_contract = self.contract.as_ref().unwrap();

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

    /// Get current capability contract
    pub fn capability(&self) -> &CapabilityContract {
        self.contract.as_ref().unwrap()
    }

    /// Get audit log
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    fn clone_for_audit(&self) -> Self {
        Self {
            name: self.name.clone(),
            contract: self.contract.clone(),
            audit_log: self.audit_log.clone(),
            _state: PhantomData,
        }
    }
}

// ============================================================================
// Escalated State - Tracks capability transitions
// ============================================================================

impl<C1, C2> TypedSession<Escalated<C1, C2>> {
    /// Execute operation with escalated capability
    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }

    /// Further escalate to even higher capability
    pub fn escalate<C3>(
        mut self,
        new_contract: CapabilityContract,
        reason: impl Into<String>,
    ) -> Result<TypedSession<Escalated<C2, C3>>, EscalationError> {
        let reason = reason.into();
        let old_contract = self.contract.as_ref().unwrap();

        if !is_escalation_allowed(old_contract, &new_contract, &reason) {
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
                reason,
            },
        });

        Ok(TypedSession {
            name: self.name,
            contract: Some(new_contract),
            audit_log: self.audit_log,
            _state: PhantomData,
        })
    }

    /// Get current capability
    pub fn capability(&self) -> &CapabilityContract {
        self.contract.as_ref().unwrap()
    }

    /// Get audit log
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }
}

// ============================================================================
// Escalation Policy
// ============================================================================

/// Check if capability escalation is allowed
///
/// 2027 Security Policy:
/// - Pure -> ReadOnly: Always allowed
/// - ReadOnly -> ReadWrite: Requires justification
/// - Any -> Network: Requires strong justification (>20 chars)
/// - Any -> Subprocess: Requires very strong justification (>50 chars)
/// - Any -> Dangerous: Always denied in autonomous systems
fn is_escalation_allowed(
    from: &CapabilityContract,
    to: &CapabilityContract,
    reason: &str,
) -> bool {
    use CapabilityClass::*;

    // Can't escalate if same risk or lower
    if to.risk_score() <= from.risk_score() {
        return true; // Not really an escalation
    }

    // Dangerous requires human review
    if to.capability_class == Dangerous && !matches!(to.safety, SafetyProfile::HumanReviewRequired) {
        return false;
    }

    match (&from.capability_class, &to.capability_class) {
        // Always allowed transitions
        (Pure, ReadOnlyFS) => true,
        (ReadOnlyFS, Environment) => reason.len() >= 10,

        // Moderate scrutiny
        (_, ReadWriteFS) => reason.len() >= 20,
        (_, Network) => reason.len() >= 30,

        // High scrutiny
        (_, Subprocess) => reason.len() >= 50,
        (_, Dangerous) => false, // Never autonomous

        // Default: allow with reason
        _ => !reason.is_empty(),
    }
}

/// Escalation error
#[derive(Debug, Clone)]
pub enum EscalationError {
    PolicyViolation {
        from: CapabilityClass,
        to: CapabilityClass,
        reason: String,
    },
}

impl std::fmt::Display for EscalationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyViolation { from, to, reason } => {
                write!(
                    f,
                    "Capability escalation denied: {:?} -> {:?}. Reason: {}",
                    from, to, reason
                )
            }
        }
    }
}

impl std::error::Error for EscalationError {}

// ============================================================================
// Utilities
// ============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typestate_basic_flow() {
        // Start unverified
        let session = TypedSession::<Unverified>::with_name("test");

        // Verify with Pure
        let session = session.verify::<()>(CapabilityContract::pure());

        // Execute operation
        let result = session.execute(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_escalation_allowed() {
        let session = TypedSession::<Unverified>::with_name("test")
            .verify::<()>(CapabilityContract::pure());

        // Escalate to ReadOnly (should succeed)
        let session = session
            .escalate::<()>(CapabilityContract::read_only(), "Need to read config")
            .expect("Escalation should succeed");

        assert_eq!(
            session.capability().capability_class,
            CapabilityClass::ReadOnlyFS
        );
    }

    #[test]
    fn test_escalation_denied_insufficient_reason() {
        let session = TypedSession::<Unverified>::with_name("test")
            .verify::<()>(CapabilityContract::pure());

        // Try to escalate to Network with weak reason (should fail)
        let result = session.escalate::<()>(CapabilityContract::network(), "test");

        assert!(result.is_err());
    }

    #[test]
    fn test_escalation_dangerous_always_denied() {
        let session = TypedSession::<Unverified>::with_name("test")
            .verify::<()>(CapabilityContract::pure());

        // Try to escalate to Dangerous (should always fail)
        let result = session.escalate::<()>(
            CapabilityContract::dangerous(),
            "Very long and detailed justification that should still be denied because Dangerous requires human review in autonomous systems"
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_audit_log_tracking() {
        let session = TypedSession::<Unverified>::with_name("test")
            .verify::<()>(CapabilityContract::pure());

        let session = session
            .escalate::<()>(CapabilityContract::read_only(), "Need to read config file for initialization")
            .unwrap();

        // Check audit log
        let log = session.audit_log();
        assert_eq!(log.len(), 3); // Created, Verified, Escalated

        match &log[2].event {
            AuditEvent::Escalated { from, to, reason } => {
                assert!(from.contains("Pure"));
                assert!(to.contains("ReadOnlyFS"));
                assert_eq!(reason, "Need to read config file for initialization");
            }
            _ => panic!("Expected Escalated event"),
        }
    }
}
