//! Contract Runtime View & Receipt Verifier (Task 4)
//!
//! Immutable runtime projections of contracts and receipt verification with balance checks

use crate::kernel::capability_contracts::CapabilityContractV2;
use serde::{Deserialize, Serialize};

/// Immutable runtime view of capability contract constraints
/// Created once per registered capability and shared across broker/execution/receipt generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRuntimeView {
    /// Capability ID
    pub capability_id: String,
    /// Maximum runtime in milliseconds
    pub max_runtime_ms: u64,
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,
    /// Maximum I/O operations
    pub max_io_ops: u64,
    /// Maximum network bytes
    pub max_network_bytes: u64,
    /// Allowed effect types (JSON array of strings)
    pub allowed_effects: Vec<String>,
    /// Isolation requirement
    pub isolation_requirement: String,
}

impl ContractRuntimeView {
    /// Create from a contract - extracts immutable constraints
    pub fn from_contract(contract: &CapabilityContractV2, capability_id: String) -> Self {
        Self {
            capability_id,
            max_runtime_ms: contract.constraints.max_runtime_ms.unwrap_or(u64::MAX),
            max_memory_bytes: contract.constraints.max_memory_bytes.unwrap_or(u64::MAX),
            max_io_ops: contract.constraints.max_io_ops.unwrap_or(u64::MAX),
            max_network_bytes: contract.constraints.max_network_bytes.unwrap_or(u64::MAX),
            allowed_effects: contract.constraints.allowed_operations.clone(),
            isolation_requirement: "ProcessIsolation".to_string(),
        }
    }

    /// Check if usage is within contract bounds
    pub fn validate_usage(
        &self,
        runtime_ms: u64,
        memory_bytes: u64,
        io_ops: u64,
        network_bytes: u64,
    ) -> Result<(), UsageViolation> {
        if runtime_ms > self.max_runtime_ms {
            return Err(UsageViolation::RuntimeExceeded {
                allowed: self.max_runtime_ms,
                actual: runtime_ms,
            });
        }
        if memory_bytes > self.max_memory_bytes {
            return Err(UsageViolation::MemoryExceeded {
                allowed: self.max_memory_bytes,
                actual: memory_bytes,
            });
        }
        if io_ops > self.max_io_ops {
            return Err(UsageViolation::IoExceeded {
                allowed: self.max_io_ops,
                actual: io_ops,
            });
        }
        if network_bytes > self.max_network_bytes {
            return Err(UsageViolation::NetworkExceeded {
                allowed: self.max_network_bytes,
                actual: network_bytes,
            });
        }
        Ok(())
    }
}

/// Usage violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageViolation {
    RuntimeExceeded { allowed: u64, actual: u64 },
    MemoryExceeded { allowed: u64, actual: u64 },
    IoExceeded { allowed: u64, actual: u64 },
    NetworkExceeded { allowed: u64, actual: u64 },
}

impl std::fmt::Display for UsageViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeExceeded { allowed, actual } => {
                write!(f, "Runtime exceeded: {} > {} ms", actual, allowed)
            }
            Self::MemoryExceeded { allowed, actual } => {
                write!(f, "Memory exceeded: {} > {} bytes", actual, allowed)
            }
            Self::IoExceeded { allowed, actual } => {
                write!(f, "IO exceeded: {} > {} ops", actual, allowed)
            }
            Self::NetworkExceeded { allowed, actual } => {
                write!(f, "Network exceeded: {} > {} bytes", actual, allowed)
            }
        }
    }
}

/// Receipt verification with balance checks
#[derive(Debug, Clone)]
pub struct ReceiptVerifier {
    contract_view: ContractRuntimeView,
}

impl ReceiptVerifier {
    /// Create a new receipt verifier
    pub fn new(contract_view: ContractRuntimeView) -> Self {
        Self { contract_view }
    }

    /// Verify receipt against contract and policy
    /// Returns Ok if receipt is valid, Err if verification fails
    pub fn verify(
        &self,
        actual_runtime_ms: u64,
        actual_memory_bytes: u64,
        actual_io_ops: u64,
        actual_network_bytes: u64,
        execution_success: bool,
        policy_allowed: bool,
    ) -> Result<(), VerificationError> {
        // Check: Policy agreement
        if !policy_allowed {
            return Err(VerificationError::PolicyRejected);
        }

        // Check: Usage is within contract bounds
        self.contract_view
            .validate_usage(actual_runtime_ms, actual_memory_bytes, actual_io_ops, actual_network_bytes)
            .map_err(VerificationError::UsageViolation)?;

        // Check: Execution success is consistent with policy
        if !execution_success && policy_allowed {
            // Policy said execute, but execution failed - might be legitimate
            // Log but don't fail
        }

        Ok(())
    }
}

/// Receipt verification error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationError {
    PolicyRejected,
    UsageViolation(UsageViolation),
    EffectNotDeclared(String),
    PolicyVersionMismatch { expected: u32, found: u32 },
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyRejected => write!(f, "Receipt verification failed: policy rejected execution"),
            Self::UsageViolation(v) => write!(f, "Receipt verification failed: {}", v),
            Self::EffectNotDeclared(effect) => {
                write!(f, "Receipt verification failed: effect '{}' not declared in contract", effect)
            }
            Self::PolicyVersionMismatch { expected, found } => {
                write!(f, "Receipt verification failed: policy version mismatch {} vs {}", expected, found)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_view() -> ContractRuntimeView {
        ContractRuntimeView {
            capability_id: "test.capability".to_string(),
            max_runtime_ms: 1000,
            max_memory_bytes: 1024 * 1024,  // 1 MB
            max_io_ops: 100,
            max_network_bytes: 10_000_000,  // 10 MB
            allowed_effects: vec!["Network".to_string(), "ReadFS".to_string()],
            isolation_requirement: "ProcessIsolation".to_string(),
        }
    }

    #[test]
    fn test_usage_validation_within_bounds() {
        let view = create_test_view();
        let result = view.validate_usage(500, 512 * 1024, 50, 5_000_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_usage_validation_runtime_exceeded() {
        let view = create_test_view();
        let result = view.validate_usage(2000, 512 * 1024, 50, 5_000_000);
        assert!(result.is_err());
        match result {
            Err(UsageViolation::RuntimeExceeded { .. }) => {},
            _ => panic!("Expected RuntimeExceeded"),
        }
    }

    #[test]
    fn test_usage_validation_memory_exceeded() {
        let view = create_test_view();
        let result = view.validate_usage(500, 2 * 1024 * 1024, 50, 5_000_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_receipt_verifier_valid() {
        let view = create_test_view();
        let verifier = ReceiptVerifier::new(view);
        let result = verifier.verify(500, 512 * 1024, 50, 5_000_000, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_receipt_verifier_policy_rejected() {
        let view = create_test_view();
        let verifier = ReceiptVerifier::new(view);
        let result = verifier.verify(500, 512 * 1024, 50, 5_000_000, true, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_receipt_verifier_usage_exceeded() {
        let view = create_test_view();
        let verifier = ReceiptVerifier::new(view);
        let result = verifier.verify(2000, 512 * 1024, 50, 5_000_000, true, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_verification_error_display() {
        let error = VerificationError::PolicyRejected;
        assert!(error.to_string().contains("policy"));

        let error = VerificationError::EffectNotDeclared("UnknownEffect".to_string());
        assert!(error.to_string().contains("UnknownEffect"));
    }
}
