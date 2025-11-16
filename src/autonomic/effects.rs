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
}

impl EffectType {
    /// Check if this effect type is mutating
    pub fn is_mutating(&self) -> bool {
        !matches!(self, EffectType::ReadOnly)
    }

    /// Check if this effect type is critical (security or ontology mutations)
    pub fn is_critical(&self) -> bool {
        matches!(self, EffectType::MutateSecurity | EffectType::MutateOntology)
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
}

impl Default for EffectMetadata {
    fn default() -> Self {
        Self {
            effect_type: EffectType::ReadOnly,
            sensitivity: Sensitivity::Low,
            idempotent: true,
            required_role: None,
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

    /// Check if this command should be considered high-risk
    pub fn is_high_risk(&self) -> bool {
        self.sensitivity >= Sensitivity::High || self.effect_type.is_critical()
    }
}
