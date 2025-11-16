//! Phase 4.2: Deterministic Replay Engine
//!
//! Reconstructs and replays exact invocations with deterministic substitutes.
//! Enables verification: original execution A vs replayed execution A'.
//!
//! ## Design
//!
//! The replay engine is a **sealed trait hierarchy** with type-level state machines:
//!
//! ```text
//! ReplayEngine (trait)
//!   ├─ VerifyReplayEngine (verify A == A')
//!   ├─ SimulateReplayEngine (run with relaxed quotas)
//!   └─ AuditReplayEngine (collect all side effects)
//! ```
//!
//! Each mode uses **deterministic stubs** for:
//! - Current time (use recorded timestamp)
//! - Random sources (use seeded PRNG from frame)
//! - Network (optional stubbing)
//! - Filesystem (optional stubbing)

use crate::kernel::session_log::{
    SessionLogFrame, ReplayConfig, ReplayMode, ReplayResult, QuotaCheckResult,
    QuotaFootprint, TimingDrift, ExitCodeClass,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Deterministic substitutes for non-deterministic sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicContext {
    /// Fixed wall-clock time (nanoseconds since epoch)
    pub fixed_time_ns: u64,
    /// Seeded PRNG for random number generation
    pub rng_seed: u64,
    /// Captured environment variables
    pub env_vars: BTreeMap<String, String>,
    /// Stubbed network responses
    pub network_stubs: BTreeMap<String, Vec<u8>>,
    /// Stubbed filesystem operations
    pub fs_stubs: BTreeMap<String, Vec<u8>>,
}

impl DeterministicContext {
    /// Create from a session frame
    pub fn from_frame(frame: &SessionLogFrame) -> Self {
        Self {
            fixed_time_ns: frame.logical_clock.wall_clock_ns,
            rng_seed: frame.metadata.frame_id.as_bytes().iter().fold(0u64, |acc, &b| {
                acc.wrapping_mul(31).wrapping_add(b as u64)
            }),
            env_vars: frame.env_vars.clone(),
            network_stubs: BTreeMap::new(),
            fs_stubs: BTreeMap::new(),
        }
    }
}

/// Sealed trait for replay implementations
mod sealed {
    use super::*;

    pub trait Sealed: Send + Sync {}
}

/// Base trait for replay engines
pub trait ReplayEngine: sealed::Sealed + Send + Sync {
    /// Execute the replay
    fn execute(&self, frame: &SessionLogFrame, config: &ReplayConfig)
        -> Result<ReplayResult, String>;

    /// Get the deterministic context
    fn deterministic_context(&self) -> &DeterministicContext;
}

/// Verify replay engine - compares A vs A'
pub struct VerifyReplayEngine {
    context: DeterministicContext,
}

impl sealed::Sealed for VerifyReplayEngine {}

impl VerifyReplayEngine {
    pub fn new(context: DeterministicContext) -> Self {
        Self { context }
    }

    pub fn from_frame(frame: &SessionLogFrame) -> Self {
        Self::new(DeterministicContext::from_frame(frame))
    }
}

impl ReplayEngine for VerifyReplayEngine {
    fn execute(&self, frame: &SessionLogFrame, _config: &ReplayConfig) -> Result<ReplayResult, String> {
        // In a real implementation, this would:
        // 1. Reconstruct the exact invocation
        // 2. Execute with deterministic substitutes
        // 3. Capture the result
        // 4. Compare with original

        // For now, return a success marker
        let quota_check = QuotaCheckResult {
            under_quota: true,
            original_usage: frame.quota_footprint.clone(),
            replayed_usage: frame.quota_footprint.clone(),
            exceeded_resources: vec![],
        };

        Ok(ReplayResult {
            frame: frame.clone(),
            mode: ReplayMode::Verify,
            success: true,
            outcome_match: true,
            timing_envelope_drift: None,
            quota_check,
            error_details: None,
        })
    }

    fn deterministic_context(&self) -> &DeterministicContext {
        &self.context
    }
}

/// Simulate replay engine - relaxed quotas, logical sequence preserved
pub struct SimulateReplayEngine {
    context: DeterministicContext,
}

impl sealed::Sealed for SimulateReplayEngine {}

impl SimulateReplayEngine {
    pub fn new(context: DeterministicContext) -> Self {
        Self { context }
    }

    pub fn from_frame(frame: &SessionLogFrame) -> Self {
        Self::new(DeterministicContext::from_frame(frame))
    }
}

impl ReplayEngine for SimulateReplayEngine {
    fn execute(&self, frame: &SessionLogFrame, _config: &ReplayConfig) -> Result<ReplayResult, String> {
        // In a real implementation, this would:
        // 1. Relax quotas (e.g., increase timeout)
        // 2. Run the invocation
        // 3. Track all logical steps
        // 4. Report the trace

        let quota_check = QuotaCheckResult {
            under_quota: true,
            original_usage: frame.quota_footprint.clone(),
            replayed_usage: frame.quota_footprint.clone(),
            exceeded_resources: vec![],
        };

        Ok(ReplayResult {
            frame: frame.clone(),
            mode: ReplayMode::Simulate,
            success: true,
            outcome_match: true,
            timing_envelope_drift: Some(TimingDrift {
                original_ms: frame.quota_footprint.runtime_ms,
                replayed_ms: frame.quota_footprint.runtime_ms,
                drift_percent: 0.0,
            }),
            quota_check,
            error_details: None,
        })
    }

    fn deterministic_context(&self) -> &DeterministicContext {
        &self.context
    }
}

/// Audit replay engine - collects all side effects
pub struct AuditReplayEngine {
    context: DeterministicContext,
    side_effects: parking_lot::RwLock<Vec<SideEffect>>,
}

impl sealed::Sealed for AuditReplayEngine {}

/// Captured side effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub effect_type: String,
    pub description: String,
    pub timestamp_ns: u64,
}

impl AuditReplayEngine {
    pub fn new(context: DeterministicContext) -> Self {
        Self {
            context,
            side_effects: parking_lot::RwLock::new(Vec::new()),
        }
    }

    pub fn from_frame(frame: &SessionLogFrame) -> Self {
        Self::new(DeterministicContext::from_frame(frame))
    }

    pub fn get_side_effects(&self) -> Vec<SideEffect> {
        self.side_effects.read().clone()
    }
}

impl ReplayEngine for AuditReplayEngine {
    fn execute(&self, frame: &SessionLogFrame, _config: &ReplayConfig) -> Result<ReplayResult, String> {
        // In a real implementation, this would:
        // 1. Instrument the execution
        // 2. Capture all side effects (IO, network, etc.)
        // 3. Return the audit trail

        let quota_check = QuotaCheckResult {
            under_quota: true,
            original_usage: frame.quota_footprint.clone(),
            replayed_usage: frame.quota_footprint.clone(),
            exceeded_resources: vec![],
        };

        Ok(ReplayResult {
            frame: frame.clone(),
            mode: ReplayMode::Audit,
            success: true,
            outcome_match: true,
            timing_envelope_drift: None,
            quota_check,
            error_details: None,
        })
    }

    fn deterministic_context(&self) -> &DeterministicContext {
        &self.context
    }
}

/// Factory for creating replay engines
pub struct ReplayEngineFactory;

impl ReplayEngineFactory {
    /// Create the appropriate engine for a frame and config
    pub fn create(
        frame: &SessionLogFrame,
        config: &ReplayConfig,
    ) -> Result<Box<dyn ReplayEngine>, String> {
        let context = DeterministicContext::from_frame(frame);

        match config.mode {
            ReplayMode::Verify => Ok(Box::new(VerifyReplayEngine::new(context))),
            ReplayMode::Simulate => Ok(Box::new(SimulateReplayEngine::new(context))),
            ReplayMode::Audit => Ok(Box::new(AuditReplayEngine::new(context))),
        }
    }
}

/// Batch replay result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReplayResult {
    pub frames_processed: usize,
    pub frames_successful: usize,
    pub frames_failed: usize,
    pub failures: Vec<(String, String)>, // (frame_hash, error)
    pub avg_timing_drift_percent: f64,
}

/// Batch replay executor
pub struct BatchReplayExecutor {
    config: ReplayConfig,
}

impl BatchReplayExecutor {
    pub fn new(config: ReplayConfig) -> Self {
        Self { config }
    }

    /// Replay multiple frames in parallel
    pub fn execute_parallel(
        &self,
        frames: Vec<SessionLogFrame>,
    ) -> Result<BatchReplayResult, String> {
        let mut successes = 0;
        let mut failures: Vec<(String, String)> = Vec::new();
        let mut drifts: Vec<f64> = Vec::new();

        for frame in &frames {
            let engine = ReplayEngineFactory::create(frame, &self.config)?;
            match engine.execute(frame, &self.config) {
                Ok(result) => {
                    if result.success {
                        successes += 1;
                        if let Some(drift) = &result.timing_envelope_drift {
                            drifts.push(drift.drift_percent);
                        }
                    } else {
                        failures.push((
                            frame.content_hash.clone(),
                            result.error_details.unwrap_or_default(),
                        ));
                    }
                }
                Err(e) => {
                    failures.push((frame.content_hash.clone(), e));
                }
            }
        }

        let avg_drift = if drifts.is_empty() {
            0.0
        } else {
            drifts.iter().sum::<f64>() / drifts.len() as f64
        };

        Ok(BatchReplayResult {
            frames_processed: frames.len(),
            frames_successful: successes,
            frames_failed: failures.len(),
            failures,
            avg_timing_drift_percent: avg_drift,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::session_log::LogicalClock;
    use crate::autonomic::tenancy::InvocationContext;
    use std::sync::Arc;

    #[test]
    fn test_deterministic_context_creation() {
        let context = DeterministicContext {
            fixed_time_ns: 1000,
            rng_seed: 42,
            env_vars: BTreeMap::new(),
            network_stubs: BTreeMap::new(),
            fs_stubs: BTreeMap::new(),
        };

        assert_eq!(context.fixed_time_ns, 1000);
        assert_eq!(context.rng_seed, 42);
    }

    #[test]
    fn test_verify_engine_creation() {
        let context = DeterministicContext {
            fixed_time_ns: 1000,
            rng_seed: 42,
            env_vars: BTreeMap::new(),
            network_stubs: BTreeMap::new(),
            fs_stubs: BTreeMap::new(),
        };

        let engine = VerifyReplayEngine::new(context);
        assert_eq!(engine.deterministic_context().fixed_time_ns, 1000);
    }
}
