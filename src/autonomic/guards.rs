//! Guard and budget declarations for CLI commands

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Guard configuration for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardConfig {
    /// Maximum latency budget in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Maximum latency budget in nanoseconds (for hot-path operations)
    pub max_latency_ns: Option<u64>,
    /// Maximum memory budget in kilobytes
    pub max_memory_kb: Option<u64>,
    /// Maximum CPU time in milliseconds
    pub max_cpu_ms: Option<u64>,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self { max_latency_ms: None, max_latency_ns: None, max_memory_kb: None, max_cpu_ms: None }
    }
}

impl GuardConfig {
    /// Create a new guard configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum latency in milliseconds
    pub fn with_max_latency_ms(mut self, ms: u64) -> Self {
        self.max_latency_ms = Some(ms);
        self
    }

    /// Set maximum latency in nanoseconds
    pub fn with_max_latency_ns(mut self, ns: u64) -> Self {
        self.max_latency_ns = Some(ns);
        self
    }

    /// Set maximum memory in kilobytes
    pub fn with_max_memory_kb(mut self, kb: u64) -> Self {
        self.max_memory_kb = Some(kb);
        self
    }

    /// Set maximum CPU time in milliseconds
    pub fn with_max_cpu_ms(mut self, ms: u64) -> Self {
        self.max_cpu_ms = Some(ms);
        self
    }

    /// Get maximum latency as Duration
    pub fn max_latency(&self) -> Option<Duration> {
        if let Some(ns) = self.max_latency_ns {
            Some(Duration::from_nanos(ns))
        } else if let Some(ms) = self.max_latency_ms {
            Some(Duration::from_millis(ms))
        } else {
            None
        }
    }

    /// Check if any guards are configured
    pub fn has_guards(&self) -> bool {
        self.max_latency_ms.is_some()
            || self.max_latency_ns.is_some()
            || self.max_memory_kb.is_some()
            || self.max_cpu_ms.is_some()
    }
}

/// Status of guard evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardStatus {
    /// Within budget
    WithinBudget,
    /// Exceeded budget
    ExceededBudget,
    /// Not enforced
    NotEnforced,
    /// No guards configured
    NoGuards,
}

/// Result of guard evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardResult {
    /// Whether guards were enforced
    pub enforced: bool,
    /// Actual latency in milliseconds
    pub latency_ms: Option<u64>,
    /// Maximum allowed latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Guard evaluation status
    pub status: GuardStatus,
    /// Additional details about guard violations
    pub details: Option<String>,
}

impl Default for GuardResult {
    fn default() -> Self {
        Self {
            enforced: false,
            latency_ms: None,
            max_latency_ms: None,
            status: GuardStatus::NoGuards,
            details: None,
        }
    }
}

impl GuardResult {
    /// Create a new guard result
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a result indicating guards were not configured
    pub fn no_guards() -> Self {
        Self { status: GuardStatus::NoGuards, ..Default::default() }
    }

    /// Create a result indicating guards were not enforced
    pub fn not_enforced() -> Self {
        Self { status: GuardStatus::NotEnforced, enforced: false, ..Default::default() }
    }

    /// Create a result for successful guard evaluation
    pub fn within_budget(latency_ms: u64, max_latency_ms: u64) -> Self {
        Self {
            enforced: true,
            latency_ms: Some(latency_ms),
            max_latency_ms: Some(max_latency_ms),
            status: GuardStatus::WithinBudget,
            details: None,
        }
    }

    /// Create a result for failed guard evaluation
    pub fn exceeded_budget(latency_ms: u64, max_latency_ms: u64) -> Self {
        Self {
            enforced: true,
            latency_ms: Some(latency_ms),
            max_latency_ms: Some(max_latency_ms),
            status: GuardStatus::ExceededBudget,
            details: Some(format!("Latency {}ms exceeded budget {}ms", latency_ms, max_latency_ms)),
        }
    }

    /// Check if guards were violated
    pub fn is_violated(&self) -> bool {
        self.status == GuardStatus::ExceededBudget
    }
}
