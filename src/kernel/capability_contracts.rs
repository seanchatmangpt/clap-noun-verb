//! Phase 6.1: Proof-Carrying CLI Contracts
//!
//! Every noun/verb becomes a **proof-carrying capability** with:
//! - Σ_CNV: Schema of input/output, effects, resources, invariants
//! - Q_CNV: Constraints (quotas, allowed operations)
//! - Γ_CNV: Proof objects (attestation, test coverage, static analysis)
//!
//! This enables AHI to reason about capabilities automatically.

use crate::autonomic::capability_id::CapabilityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Capability contract - machine-verifiable guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityContractV2 {
    /// Capability identification
    pub capability_id: CapabilityId,
    pub version: u32,

    /// Input/output schema (Σ_CNV)
    pub schema: CapabilitySchema,

    /// Constraints and quotas (Q_CNV)
    pub constraints: CapabilityConstraints,

    /// Effects declaration
    pub effects: EffectsDeclaration,

    /// Invariants that must hold
    pub invariants: Vec<Invariant>,

    /// Stability and safety guarantees
    pub guarantees: Guarantees,

    /// Proof references (Γ_CNV)
    pub proofs: ProofReferences,
}

/// Capability schema - input/output types and structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySchema {
    /// Input schema (JSON schema fragment)
    pub input_schema: serde_json::Value,

    /// Output schema (JSON schema fragment)
    pub output_schema: serde_json::Value,

    /// Error schema
    pub error_schema: Option<serde_json::Value>,

    /// Environment variables used
    pub env_vars: Vec<String>,

    /// File paths accessed
    pub file_paths: Vec<PathAccessPattern>,
}

/// File path access pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathAccessPattern {
    pub pattern: String,
    pub access_type: AccessType,
    pub required: bool,
}

/// Type of filesystem access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessType {
    Read,
    Write,
    Execute,
    Delete,
}

/// Capability constraints - resource limits and allowed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConstraints {
    /// Max runtime in milliseconds
    pub max_runtime_ms: Option<u64>,

    /// Max memory in bytes
    pub max_memory_bytes: Option<u64>,

    /// Max IO operations
    pub max_io_ops: Option<u64>,

    /// Max network bytes
    pub max_network_bytes: Option<u64>,

    /// Allowed operations
    pub allowed_operations: Vec<String>,

    /// Forbidden operations
    pub forbidden_operations: Vec<String>,

    /// Network access allowed
    pub network_access: bool,

    /// Filesystem write allowed
    pub fs_write_allowed: bool,

    /// Process spawning allowed
    pub process_spawn_allowed: bool,
}

impl Default for CapabilityConstraints {
    fn default() -> Self {
        Self {
            max_runtime_ms: Some(30000),
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_io_ops: Some(10000),
            max_network_bytes: Some(100 * 1024 * 1024), // 100MB
            allowed_operations: vec![],
            forbidden_operations: vec![],
            network_access: false,
            fs_write_allowed: true,
            process_spawn_allowed: false,
        }
    }
}

/// Effects declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsDeclaration {
    /// Side effects (ReadOnly, WriteFS, Network, etc.)
    pub side_effects: Vec<SideEffectType>,

    /// Isolation requirements
    pub isolation_requirements: Vec<IsolationRequirement>,

    /// Data sensitivity
    pub data_sensitivity: DataSensitivity,
}

/// Side effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffectType {
    /// Pure function - no side effects
    Pure,
    /// Reads filesystem
    ReadFS,
    /// Writes filesystem
    WriteFS,
    /// Makes network calls
    Network,
    /// Spawns processes
    SpawnProcess,
    /// Modifies environment
    ModifyEnv,
    /// Accesses system time
    AccessTime,
    /// Accesses random sources
    AccessRandom,
}

/// Isolation requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationRequirement {
    /// Must not run concurrently with others
    Exclusive,
    /// Must not run in same process
    ProcessIsolation,
    /// Should run in container
    ContainerIsolation,
    /// Can share resources
    Shared,
}

/// Data sensitivity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSensitivity {
    /// Public data
    Public,
    /// Internal data
    Internal,
    /// Sensitive (PII, credentials, etc.)
    Sensitive,
    /// Highly sensitive (encryption keys, secrets)
    HighlySensitive,
}

/// Invariants that must hold for the capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub name: String,
    pub description: String,
    /// Predicate as string (can be evaluated by policy engine)
    pub predicate: String,
    /// Severity if violated
    pub severity: InvariantSeverity,
}

/// Severity of invariant violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantSeverity {
    Warning,
    Error,
    Critical,
}

/// Guarantees provided by the capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guarantees {
    /// Stability level
    pub stability: StabilityLevel,

    /// Safety classification
    pub safety: SafetyLevel,

    /// Determinism guarantee
    pub determinism: DeterminismGuarantee,

    /// Idempotency guarantee
    pub idempotency: IdempotencyGuarantee,
}

/// Stability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityLevel {
    /// Stable API guaranteed
    Stable,
    /// Beta - may change
    Beta,
    /// Experimental - likely to change
    Experimental,
    /// Deprecated - will be removed
    Deprecated,
}

/// Safety level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Safe for any agent
    AgentSafe,
    /// Requires human review
    RequiresHumanReview,
    /// High risk
    HighRisk,
}

/// Determinism guarantee
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeterminismGuarantee {
    /// Fully deterministic
    Full,
    /// Deterministic for same input + state
    ConditionalOnState,
    /// Non-deterministic
    NonDeterministic,
}

/// Idempotency guarantee
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdempotencyGuarantee {
    /// Safe to call multiple times
    Idempotent,
    /// First call has side effects
    OnceOnly,
    /// Always has side effects
    NonIdempotent,
}

/// Proof references - links to test coverage, attestation, analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofReferences {
    /// Test coverage info
    pub test_coverage: Option<TestCoverage>,

    /// Cleanroom proof references
    pub cleanroom_receipts: Vec<String>,

    /// Static analysis findings
    pub static_analysis: Option<StaticAnalysis>,

    /// Attestation chain hash
    pub attestation_hash: Option<String>,

    /// Security audit references
    pub audit_references: Vec<AuditReference>,
}

/// Test coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverage {
    /// Line coverage percentage
    pub line_coverage: f64,

    /// Branch coverage percentage
    pub branch_coverage: f64,

    /// Test count
    pub test_count: u64,

    /// Test framework used
    pub test_framework: String,

    /// Coverage report URL
    pub coverage_url: Option<String>,
}

/// Static analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysis {
    /// Tool name
    pub tool: String,

    /// Result
    pub result: String,

    /// Issues found
    pub issues: Vec<AnalysisIssue>,

    /// Analysis timestamp
    pub timestamp: u64,
}

/// Analysis issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisIssue {
    pub severity: String,
    pub message: String,
    pub location: Option<String>,
}

/// Audit reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReference {
    pub audit_id: String,
    pub date: String,
    pub auditor: String,
    pub status: String,
}

/// Proof object - bundles all evidence that a capability was built correctly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofObject {
    /// CNV version hash
    pub cnv_version_hash: String,

    /// Capability contract
    pub contract: CapabilityContractV2,

    /// Build metadata
    pub build_metadata: BuildMetadata,

    /// Proof content
    pub proof_content: serde_json::Value,

    /// Signature (if signed)
    pub signature: Option<String>,

    /// Creation timestamp
    pub created_at: u64,
}

/// Build metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub builder: String,
    pub build_timestamp: u64,
    pub git_commit: Option<String>,
    pub build_environment: BTreeMap<String, String>,
}

impl CapabilityContractV2 {
    /// Check if capability is agent-safe
    pub fn is_agent_safe(&self) -> bool {
        self.guarantees.safety == SafetyLevel::AgentSafe
    }

    /// Check if capability is deterministic
    pub fn is_deterministic(&self) -> bool {
        self.guarantees.determinism == DeterminismGuarantee::Full
    }

    /// Check if capability requires network access
    pub fn requires_network(&self) -> bool {
        self.effects.side_effects.contains(&SideEffectType::Network)
    }

    /// Check if capability requires filesystem write
    pub fn requires_fs_write(&self) -> bool {
        self.effects.side_effects.contains(&SideEffectType::WriteFS)
    }

    /// Validate against constraints
    pub fn validate_execution(
        &self,
        runtime_ms: u64,
        memory_bytes: u64,
        io_ops: u64,
        network_bytes: u64,
    ) -> Result<(), String> {
        if let Some(max) = self.constraints.max_runtime_ms {
            if runtime_ms > max {
                return Err(format!("Runtime {} ms exceeds limit {} ms", runtime_ms, max));
            }
        }

        if let Some(max) = self.constraints.max_memory_bytes {
            if memory_bytes > max {
                return Err(format!(
                    "Memory {} bytes exceeds limit {} bytes",
                    memory_bytes, max
                ));
            }
        }

        if let Some(max) = self.constraints.max_io_ops {
            if io_ops > max {
                return Err(format!("IO ops {} exceeds limit {}", io_ops, max));
            }
        }

        if let Some(max) = self.constraints.max_network_bytes {
            if network_bytes > max {
                return Err(format!(
                    "Network bytes {} exceeds limit {}",
                    network_bytes, max
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_validation() {
        let mut contract = CapabilityContractV2 {
            capability_id: CapabilityId::new("test"),
            version: 1,
            schema: CapabilitySchema {
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                error_schema: None,
                env_vars: vec![],
                file_paths: vec![],
            },
            constraints: CapabilityConstraints::default(),
            effects: EffectsDeclaration {
                side_effects: vec![SideEffectType::Pure],
                isolation_requirements: vec![],
                data_sensitivity: DataSensitivity::Public,
            },
            invariants: vec![],
            guarantees: Guarantees {
                stability: StabilityLevel::Stable,
                safety: SafetyLevel::AgentSafe,
                determinism: DeterminismGuarantee::Full,
                idempotency: IdempotencyGuarantee::Idempotent,
            },
            proofs: ProofReferences {
                test_coverage: None,
                cleanroom_receipts: vec![],
                static_analysis: None,
                attestation_hash: None,
                audit_references: vec![],
            },
        };

        // Should pass with zero usage
        assert!(contract.validate_execution(0, 0, 0, 0).is_ok());

        // Should fail with excessive runtime
        contract.constraints.max_runtime_ms = Some(100);
        assert!(contract.validate_execution(200, 0, 0, 0).is_err());
    }

    #[test]
    fn test_capability_properties() {
        let contract = CapabilityContractV2 {
            capability_id: CapabilityId::new("test"),
            version: 1,
            schema: CapabilitySchema {
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                error_schema: None,
                env_vars: vec![],
                file_paths: vec![],
            },
            constraints: CapabilityConstraints::default(),
            effects: EffectsDeclaration {
                side_effects: vec![SideEffectType::Pure],
                isolation_requirements: vec![],
                data_sensitivity: DataSensitivity::Public,
            },
            invariants: vec![],
            guarantees: Guarantees {
                stability: StabilityLevel::Stable,
                safety: SafetyLevel::AgentSafe,
                determinism: DeterminismGuarantee::Full,
                idempotency: IdempotencyGuarantee::Idempotent,
            },
            proofs: ProofReferences {
                test_coverage: None,
                cleanroom_receipts: vec![],
                static_analysis: None,
                attestation_hash: None,
                audit_references: vec![],
            },
        };

        assert!(contract.is_agent_safe());
        assert!(contract.is_deterministic());
        assert!(!contract.requires_network());
        assert!(!contract.requires_fs_write());
    }
}
