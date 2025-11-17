//! Effect and risk metadata for CLI commands

use serde::{Deserialize, Serialize};

/// Type of effect a command has on the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Read-only operation, no mutations
    ReadOnly,
    /// Mutates application or runtime state
    MutateState,
    /// Mutates configuration
    MutateConfig,
    /// Mutates ontology/schema
    MutateOntology,
    /// Mutates security settings or credentials
    MutateSecurity,
    /// Performs network access (external communication)
    NetworkAccess,
    /// Writes to persistent storage
    StorageWrite,
    /// Performs privileged operations (requires elevated permissions)
    Privileged,
}

impl EffectType {
    /// Check if this effect type is mutating
    pub fn is_mutating(&self) -> bool {
        !matches!(self, EffectType::ReadOnly | EffectType::NetworkAccess)
    }

    /// Check if this effect type is critical (security or ontology mutations)
    pub fn is_critical(&self) -> bool {
        matches!(self, EffectType::MutateSecurity | EffectType::MutateOntology | EffectType::Privileged)
    }

    /// Check if this effect type requires network access
    pub fn requires_network(&self) -> bool {
        matches!(self, EffectType::NetworkAccess)
    }

    /// Check if this effect type writes to storage
    pub fn writes_storage(&self) -> bool {
        matches!(self, EffectType::StorageWrite | EffectType::MutateState | EffectType::MutateConfig)
    }
}

impl Default for EffectType {
    fn default() -> Self {
        EffectType::ReadOnly
    }
}

/// Sensitivity level of a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sensitivity {
    /// Low sensitivity, minimal impact
    Low,
    /// Medium sensitivity, moderate impact
    Medium,
    /// High sensitivity, significant impact
    High,
    /// Critical sensitivity, severe impact potential
    Critical,
}

impl Default for Sensitivity {
    fn default() -> Self {
        Sensitivity::Low
    }
}

/// Data sensitivity tags for policy routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSensitivityTag {
    /// Handles personally identifiable information
    Pii,
    /// Handles financial records or transactions
    Financial,
    /// Handles health-related information
    HealthData,
    /// Handles cryptographic keys or secrets
    Secrets,
    /// Handles authentication credentials
    Credentials,
}

/// Isolation requirement for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationRequirement {
    /// Can run in shared process (default)
    Shared,
    /// Should run in isolated process
    Isolated,
    /// Must run in sandboxed environment
    Sandboxed,
    /// Requires containerized execution
    Containerized,
}

impl Default for IsolationRequirement {
    fn default() -> Self {
        IsolationRequirement::Shared
    }
}

/// Effect metadata for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectMetadata {
    /// Primary effect type
    pub effect_type: EffectType,
    /// Sensitivity level
    pub sensitivity: Sensitivity,
    /// Whether the command is idempotent
    pub idempotent: bool,
    /// Required role or permission level (free-form for external auth)
    pub required_role: Option<String>,
    /// Data sensitivity tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub data_sensitivity: Vec<DataSensitivityTag>,
    /// Isolation requirement
    #[serde(default)]
    pub isolation: IsolationRequirement,
    /// Whether this command supports dry-run/plan mode
    #[serde(default)]
    pub supports_dry_run: bool,
}

impl Default for EffectMetadata {
    fn default() -> Self {
        Self {
            effect_type: EffectType::ReadOnly,
            sensitivity: Sensitivity::Low,
            idempotent: true,
            required_role: None,
            data_sensitivity: Vec::new(),
            isolation: IsolationRequirement::Shared,
            supports_dry_run: false,
        }
    }
}

impl EffectMetadata {
    /// Create a new effect metadata with specified effect type
    pub fn new(effect_type: EffectType) -> Self {
        Self { effect_type, ..Default::default() }
    }

    /// Set sensitivity level
    pub fn with_sensitivity(mut self, sensitivity: Sensitivity) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    /// Set idempotence flag
    pub fn with_idempotent(mut self, idempotent: bool) -> Self {
        self.idempotent = idempotent;
        self
    }

    /// Set required role
    pub fn with_required_role(mut self, role: impl Into<String>) -> Self {
        self.required_role = Some(role.into());
        self
    }

    /// Add a data sensitivity tag
    pub fn with_data_sensitivity(mut self, tag: DataSensitivityTag) -> Self {
        self.data_sensitivity.push(tag);
        self
    }

    /// Set isolation requirement
    pub fn with_isolation(mut self, isolation: IsolationRequirement) -> Self {
        self.isolation = isolation;
        self
    }

    /// Enable dry-run support
    pub fn supports_dry_run(mut self) -> Self {
        self.supports_dry_run = true;
        self
    }

    /// Check if this command should be considered high-risk
    pub fn is_high_risk(&self) -> bool {
        self.sensitivity >= Sensitivity::High
            || self.effect_type.is_critical()
            || !self.data_sensitivity.is_empty()
    }

    /// Check if this command handles sensitive data
    pub fn handles_sensitive_data(&self) -> bool {
        !self.data_sensitivity.is_empty()
    }

    /// Check if this command requires isolation
    pub fn requires_isolation(&self) -> bool {
        !matches!(self.isolation, IsolationRequirement::Shared)
    }
}
