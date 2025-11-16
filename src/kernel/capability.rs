//! CNV Capability Contracts
//!
//! Machine-verifiable command guarantees for agent-grade CLI applications.
//! Every verb declares its capabilities at compile time, enabling:
//!
//! - **Side-effect classification**: Pure, ReadOnly, ReadWrite, Network, Dangerous
//! - **Resource profiles**: Expected runtime, memory, CPU usage bands
//! - **Stability guarantees**: Stable, Experimental, Deprecated, NonDeterministic
//! - **Safety profiles**: AgentSafe, HumanReviewRequired
//!
//! # Design
//!
//! Capability contracts are:
//! - Type-enforced at compile time via trait bounds
//! - Introspectable at runtime via grammar metadata
//! - Serializable for agent reasoning and routing
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::{CapabilityClass, VerbCapability};
//!
//! // Define a pure verb (no side effects)
//! #[capability(Pure)]
//! fn calculate(x: i32, y: i32) -> i32 {
//!     x + y
//! }
//!
//! // Define a verb that reads files
//! #[capability(ReadOnlyFS)]
//! fn read_config(path: &str) -> Result<Config> {
//!     // Can read files but not write
//!     std::fs::read_to_string(path)
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Capability class for command side effects
///
/// Defines what external resources a command can access:
/// - **Pure**: No side effects, deterministic, cacheable
/// - **ReadOnlyFS**: Can read filesystem, no writes
/// - **ReadWriteFS**: Can read and write filesystem
/// - **Network**: Can make network requests
/// - **Subprocess**: Can spawn subprocesses
/// - **Environment**: Can read/write environment variables
/// - **Dangerous**: Can perform system-level operations (requires human review)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityClass {
    /// Pure computation - no side effects
    ///
    /// - No filesystem access
    /// - No network access
    /// - No subprocess spawning
    /// - Deterministic (same inputs → same outputs)
    /// - Safe for aggressive caching
    Pure,

    /// Read-only filesystem access
    ///
    /// - Can read files and directories
    /// - Cannot write, create, or delete files
    /// - Cannot modify permissions
    /// - Safe for data inspection
    ReadOnlyFS,

    /// Read-write filesystem access
    ///
    /// - Can read and write files
    /// - Can create and delete files/directories
    /// - Can modify permissions
    /// - Requires caution in automated pipelines
    ReadWriteFS,

    /// Network access
    ///
    /// - Can make HTTP/HTTPS requests
    /// - Can open sockets
    /// - Can perform DNS lookups
    /// - Requires network connectivity
    Network,

    /// Subprocess spawning
    ///
    /// - Can execute external programs
    /// - Inherits subprocess security risks
    /// - Requires PATH access
    Subprocess,

    /// Environment variable access
    ///
    /// - Can read environment variables
    /// - Can modify environment (subprocess only)
    /// - May leak sensitive data
    Environment,

    /// Dangerous system operations
    ///
    /// - Can perform privileged operations
    /// - Can modify system state
    /// - Requires explicit human review
    /// - Never safe for blind automation
    Dangerous,
}

impl CapabilityClass {
    /// Check if this capability is a superset of another
    ///
    /// Example: ReadWriteFS includes ReadOnlyFS
    pub fn includes(&self, other: &CapabilityClass) -> bool {
        match (self, other) {
            (Self::Dangerous, _) => true, // Dangerous includes everything
            (Self::ReadWriteFS, Self::ReadOnlyFS) => true,
            (Self::ReadWriteFS, Self::Pure) => true,
            (Self::ReadOnlyFS, Self::Pure) => true,
            (a, b) => a == b,
        }
    }

    /// Get risk level (0-10)
    pub fn risk_level(&self) -> u8 {
        match self {
            Self::Pure => 0,
            Self::ReadOnlyFS => 2,
            Self::Environment => 3,
            Self::Network => 5,
            Self::ReadWriteFS => 6,
            Self::Subprocess => 8,
            Self::Dangerous => 10,
        }
    }

    /// Check if safe for blind automation
    pub fn is_agent_safe(&self) -> bool {
        matches!(self, Self::Pure | Self::ReadOnlyFS | Self::Environment)
    }
}

impl fmt::Display for CapabilityClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pure => write!(f, "pure"),
            Self::ReadOnlyFS => write!(f, "read_only_fs"),
            Self::ReadWriteFS => write!(f, "read_write_fs"),
            Self::Network => write!(f, "network"),
            Self::Subprocess => write!(f, "subprocess"),
            Self::Environment => write!(f, "environment"),
            Self::Dangerous => write!(f, "dangerous"),
        }
    }
}

/// Resource band for expected usage
///
/// Provides coarse-grained resource expectations:
/// - **Instant**: < 100ms, < 10MB
/// - **Fast**: < 1s, < 100MB
/// - **Medium**: < 10s, < 1GB
/// - **Slow**: < 60s, < 5GB
/// - **Cold**: > 60s or > 5GB (long-running operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceBand {
    /// Instant: < 100ms, < 10MB
    Instant,
    /// Fast: < 1s, < 100MB
    Fast,
    /// Medium: < 10s, < 1GB
    Medium,
    /// Slow: < 60s, < 5GB
    Slow,
    /// Cold: > 60s or > 5GB (long-running)
    Cold,
}

impl ResourceBand {
    /// Get expected runtime in milliseconds (upper bound)
    pub fn max_runtime_ms(&self) -> u64 {
        match self {
            Self::Instant => 100,
            Self::Fast => 1_000,
            Self::Medium => 10_000,
            Self::Slow => 60_000,
            Self::Cold => u64::MAX,
        }
    }

    /// Get expected memory in bytes (upper bound)
    pub fn max_memory_bytes(&self) -> u64 {
        match self {
            Self::Instant => 10 * 1024 * 1024,        // 10MB
            Self::Fast => 100 * 1024 * 1024,          // 100MB
            Self::Medium => 1024 * 1024 * 1024,       // 1GB
            Self::Slow => 5 * 1024 * 1024 * 1024,     // 5GB
            Self::Cold => u64::MAX,
        }
    }
}

impl fmt::Display for ResourceBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instant => write!(f, "instant"),
            Self::Fast => write!(f, "fast"),
            Self::Medium => write!(f, "medium"),
            Self::Slow => write!(f, "slow"),
            Self::Cold => write!(f, "cold"),
        }
    }
}

/// Stability profile for behavioral compatibility
///
/// Indicates how stable the command behavior is:
/// - **Stable**: Behavior locked, safe for production
/// - **Preview**: API stable, implementation may change
/// - **Experimental**: Breaking changes expected
/// - **Deprecated**: Will be removed in future versions
/// - **NonDeterministic**: Output may vary with same inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityProfile {
    /// Stable - behavior locked, safe for production
    Stable,
    /// Preview - API stable, implementation may change
    Preview,
    /// Experimental - breaking changes expected
    Experimental,
    /// Deprecated - will be removed
    Deprecated,
    /// Non-deterministic - output varies with same inputs
    NonDeterministic,
}

impl fmt::Display for StabilityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Preview => write!(f, "preview"),
            Self::Experimental => write!(f, "experimental"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::NonDeterministic => write!(f, "non_deterministic"),
        }
    }
}

/// Safety profile for automation
///
/// Determines whether a command is safe for blind automation:
/// - **AgentSafe**: Safe for fully automated execution
/// - **HumanReviewRequired**: Requires human oversight
/// - **InteractiveOnly**: Requires human interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyProfile {
    /// Safe for fully automated execution by agents
    AgentSafe,
    /// Requires human review before execution
    HumanReviewRequired,
    /// Requires human interaction (prompts, confirmations)
    InteractiveOnly,
}

impl SafetyProfile {
    /// Check if safe for agent execution
    pub fn is_agent_safe(&self) -> bool {
        matches!(self, Self::AgentSafe)
    }
}

impl fmt::Display for SafetyProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentSafe => write!(f, "agent_safe"),
            Self::HumanReviewRequired => write!(f, "human_review_required"),
            Self::InteractiveOnly => write!(f, "interactive_only"),
        }
    }
}

/// Complete capability contract for a verb
///
/// Bundles all capability dimensions:
/// - Side effects (capability class)
/// - Resource expectations
/// - Behavioral stability
/// - Safety for automation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityContract {
    /// Side-effect classification
    pub capability_class: CapabilityClass,

    /// Expected resource usage band
    pub resource_band: ResourceBand,

    /// Stability guarantee
    pub stability: StabilityProfile,

    /// Safety for automation
    pub safety: SafetyProfile,

    /// Additional capability metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CapabilityContract {
    /// Create a new capability contract
    pub fn new(
        capability_class: CapabilityClass,
        resource_band: ResourceBand,
        stability: StabilityProfile,
        safety: SafetyProfile,
    ) -> Self {
        Self {
            capability_class,
            resource_band,
            stability,
            safety,
            metadata: HashMap::new(),
        }
    }

    /// Create a pure capability (most restrictive)
    pub fn pure() -> Self {
        Self::new(
            CapabilityClass::Pure,
            ResourceBand::Instant,
            StabilityProfile::Stable,
            SafetyProfile::AgentSafe,
        )
    }

    /// Create a read-only capability
    pub fn read_only() -> Self {
        Self::new(
            CapabilityClass::ReadOnlyFS,
            ResourceBand::Fast,
            StabilityProfile::Stable,
            SafetyProfile::AgentSafe,
        )
    }

    /// Create a read-write capability
    pub fn read_write() -> Self {
        Self::new(
            CapabilityClass::ReadWriteFS,
            ResourceBand::Fast,
            StabilityProfile::Stable,
            SafetyProfile::HumanReviewRequired,
        )
    }

    /// Create a network capability
    pub fn network() -> Self {
        Self::new(
            CapabilityClass::Network,
            ResourceBand::Medium,
            StabilityProfile::Stable,
            SafetyProfile::HumanReviewRequired,
        )
    }

    /// Create a dangerous capability
    pub fn dangerous() -> Self {
        Self::new(
            CapabilityClass::Dangerous,
            ResourceBand::Slow,
            StabilityProfile::Stable,
            SafetyProfile::HumanReviewRequired,
        )
    }

    /// Add custom metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }

    /// Check if this contract is compatible with another
    ///
    /// A contract is compatible if:
    /// - Capability class is compatible (subset)
    /// - Resource band is compatible (subset)
    /// - Stability is acceptable
    /// - Safety requirements are met
    pub fn is_compatible_with(&self, required: &CapabilityContract) -> bool {
        required.capability_class.includes(&self.capability_class)
            && self.resource_band as u8 <= required.resource_band as u8
            && self.is_stable_enough_for(&required.stability)
            && self.is_safe_enough_for(&required.safety)
    }

    /// Check if stability is sufficient
    fn is_stable_enough_for(&self, required: &StabilityProfile) -> bool {
        use StabilityProfile::*;
        match (required, &self.stability) {
            (Stable, Stable) => true,
            (Preview, Stable | Preview) => true,
            (Experimental, Stable | Preview | Experimental) => true,
            (NonDeterministic, _) => true,
            (Deprecated, Deprecated) => true, // Deprecated matches itself
            (Deprecated, _) => false, // But only deprecated matches deprecated requirement
            _ => false,
        }
    }

    /// Check if safety is sufficient
    fn is_safe_enough_for(&self, required: &SafetyProfile) -> bool {
        use SafetyProfile::*;
        match (required, &self.safety) {
            (AgentSafe, AgentSafe) => true,
            (HumanReviewRequired, AgentSafe | HumanReviewRequired) => true,
            (InteractiveOnly, _) => true,
            _ => false,
        }
    }

    /// Get overall risk score (0-100)
    pub fn risk_score(&self) -> u8 {
        let mut score = self.capability_class.risk_level() * 10;

        // Adjust for resource band
        score += match self.resource_band {
            ResourceBand::Instant => 0,
            ResourceBand::Fast => 5,
            ResourceBand::Medium => 10,
            ResourceBand::Slow => 15,
            ResourceBand::Cold => 20,
        };

        // Adjust for stability
        score += match self.stability {
            StabilityProfile::Stable => 0,
            StabilityProfile::Preview => 5,
            StabilityProfile::Experimental => 10,
            StabilityProfile::Deprecated => 20,
            StabilityProfile::NonDeterministic => 15,
        };

        score.min(100)
    }

    /// Check if suitable for agent automation
    pub fn is_agent_safe(&self) -> bool {
        self.safety.is_agent_safe()
            && self.capability_class.is_agent_safe()
            && matches!(self.stability, StabilityProfile::Stable | StabilityProfile::Preview)
    }
}

impl Default for CapabilityContract {
    fn default() -> Self {
        Self::pure()
    }
}

impl fmt::Display for CapabilityContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}|{}|{}|{}]",
            self.capability_class, self.resource_band, self.stability, self.safety
        )
    }
}

/// Trait for types that have capability contracts
pub trait HasCapability {
    /// Get the capability contract for this type
    fn capability(&self) -> &CapabilityContract;
}

/// Capability-aware execution context
///
/// Provides capability-scoped APIs for verb handlers:
/// - Pure verbs: No IO APIs available
/// - ReadOnlyFS: Can read but not write
/// - Network: Can make requests
/// - etc.
pub struct CapabilityContext {
    /// The capability contract for this context
    contract: CapabilityContract,
}

impl CapabilityContext {
    /// Create a new capability context
    pub fn new(contract: CapabilityContract) -> Self {
        Self { contract }
    }

    /// Get the capability contract
    pub fn contract(&self) -> &CapabilityContract {
        &self.contract
    }

    /// Check if filesystem reads are allowed
    pub fn can_read_fs(&self) -> bool {
        matches!(
            self.contract.capability_class,
            CapabilityClass::ReadOnlyFS
                | CapabilityClass::ReadWriteFS
                | CapabilityClass::Dangerous
        )
    }

    /// Check if filesystem writes are allowed
    pub fn can_write_fs(&self) -> bool {
        matches!(
            self.contract.capability_class,
            CapabilityClass::ReadWriteFS | CapabilityClass::Dangerous
        )
    }

    /// Check if network access is allowed
    pub fn can_access_network(&self) -> bool {
        matches!(
            self.contract.capability_class,
            CapabilityClass::Network | CapabilityClass::Dangerous
        )
    }

    /// Check if subprocess spawning is allowed
    pub fn can_spawn_subprocess(&self) -> bool {
        matches!(
            self.contract.capability_class,
            CapabilityClass::Subprocess | CapabilityClass::Dangerous
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_class_includes() {
        assert!(CapabilityClass::ReadWriteFS.includes(&CapabilityClass::ReadOnlyFS));
        assert!(CapabilityClass::ReadWriteFS.includes(&CapabilityClass::Pure));
        assert!(CapabilityClass::Dangerous.includes(&CapabilityClass::Network));
        assert!(!CapabilityClass::ReadOnlyFS.includes(&CapabilityClass::ReadWriteFS));
    }

    #[test]
    fn test_capability_class_risk_level() {
        assert_eq!(CapabilityClass::Pure.risk_level(), 0);
        assert_eq!(CapabilityClass::ReadOnlyFS.risk_level(), 2);
        assert_eq!(CapabilityClass::Dangerous.risk_level(), 10);
    }

    #[test]
    fn test_capability_class_agent_safe() {
        assert!(CapabilityClass::Pure.is_agent_safe());
        assert!(CapabilityClass::ReadOnlyFS.is_agent_safe());
        assert!(!CapabilityClass::ReadWriteFS.is_agent_safe());
        assert!(!CapabilityClass::Dangerous.is_agent_safe());
    }

    #[test]
    fn test_resource_band_bounds() {
        assert_eq!(ResourceBand::Instant.max_runtime_ms(), 100);
        assert_eq!(ResourceBand::Fast.max_runtime_ms(), 1_000);
        assert_eq!(ResourceBand::Instant.max_memory_bytes(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_capability_contract_pure() {
        let contract = CapabilityContract::pure();
        assert_eq!(contract.capability_class, CapabilityClass::Pure);
        assert_eq!(contract.resource_band, ResourceBand::Instant);
        assert_eq!(contract.stability, StabilityProfile::Stable);
        assert_eq!(contract.safety, SafetyProfile::AgentSafe);
        assert!(contract.is_agent_safe());
    }

    #[test]
    fn test_capability_contract_read_write() {
        let contract = CapabilityContract::read_write();
        assert_eq!(contract.capability_class, CapabilityClass::ReadWriteFS);
        assert_eq!(contract.safety, SafetyProfile::HumanReviewRequired);
        assert!(!contract.is_agent_safe());
    }

    #[test]
    fn test_capability_contract_compatibility() {
        let pure = CapabilityContract::pure();
        let read_only = CapabilityContract::read_only();
        let read_write = CapabilityContract::read_write();

        // Pure is compatible with read-only requirements
        assert!(pure.is_compatible_with(&read_only));
        // Read-only is NOT compatible with pure requirements
        assert!(!read_only.is_compatible_with(&pure));
        // Read-write is NOT compatible with read-only requirements
        assert!(!read_write.is_compatible_with(&read_only));
    }

    #[test]
    fn test_capability_contract_risk_score() {
        let pure = CapabilityContract::pure();
        let dangerous = CapabilityContract::dangerous();

        assert!(pure.risk_score() < dangerous.risk_score());
        assert_eq!(pure.risk_score(), 0); // Pure + Instant + Stable + AgentSafe
    }

    #[test]
    fn test_capability_context_permissions() {
        let pure_ctx = CapabilityContext::new(CapabilityContract::pure());
        assert!(!pure_ctx.can_read_fs());
        assert!(!pure_ctx.can_write_fs());
        assert!(!pure_ctx.can_access_network());

        let ro_ctx = CapabilityContext::new(CapabilityContract::read_only());
        assert!(ro_ctx.can_read_fs());
        assert!(!ro_ctx.can_write_fs());

        let rw_ctx = CapabilityContext::new(CapabilityContract::read_write());
        assert!(rw_ctx.can_read_fs());
        assert!(rw_ctx.can_write_fs());
    }
}
