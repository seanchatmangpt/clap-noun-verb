//! Phase 4: Deterministic Session Logs and Replay Framework
//!
//! Transforms every CNV invocation into a replayable, verifiable μ-step.
//! Enables: O (observations/logs) → μ (action) mapping for autonomic systems.
//!
//! ## Key Concepts
//!
//! - **SessionLogFrame**: Content-addressed observation of a complete invocation
//! - **DeterministicReplay**: Reconstructs execution with deterministic substitutes
//! - **FrameDelta**: Difference between two frames for the same capability
//! - **SessionCompression**: Compact representation of repeated invocations
//!
//! ## Architecture
//!
//! ```text
//! CNV Invocation
//!     ↓
//! [Session Log Frame Generator]
//!     ↓ (produces canonical O)
//! [Frame] (hashable, orderable, reconstructible)
//!     ↓
//! [Replay Engine] ← produces Γ (proof objects)
//!     ├→ Verify mode (compare A vs A')
//!     └→ Simulate mode (trace logic without side effects)
//!     ↓
//! [FrameDelta & Compression]
//!     ↓ (swarm-scale analysis)
//! [AHI Integration] (ΔO_CNV → ΔΣ)
//! ```

use crate::kernel::capability::{CapabilityContract, CapabilityContext};
use crate::kernel::output::{OutputEnvelope, StructuredError, StructuredResult};
use crate::kernel::telemetry::TelemetryProfile;
use crate::autonomic::tenancy::InvocationContext;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;
use uuid::Uuid;

/// Deterministic clock for replay - uses logical ticks + wall-clock envelope
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct LogicalClock {
    /// Logical timestamp (lamport-like counter)
    pub logical_tick: u64,
    /// Nanoseconds since Unix epoch (captured, not used for ordering)
    pub wall_clock_ns: u64,
}

impl LogicalClock {
    /// Create a new logical clock
    pub fn new(logical_tick: u64, wall_clock_ns: u64) -> Self {
        Self {
            logical_tick,
            wall_clock_ns,
        }
    }

    /// Increment logical tick
    pub fn tick(&self) -> Self {
        Self {
            logical_tick: self.logical_tick.saturating_add(1),
            wall_clock_ns: self.wall_clock_ns,
        }
    }

    /// Merge with another clock (take max)
    pub fn merge(&self, other: &Self) -> Self {
        let max_tick = self.logical_tick.max(other.logical_tick);
        Self {
            logical_tick: max_tick.saturating_add(1),
            wall_clock_ns: self.wall_clock_ns.max(other.wall_clock_ns),
        }
    }
}

/// Quota usage profile captured during execution
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct QuotaFootprint {
    /// Runtime in milliseconds
    pub runtime_ms: u64,
    /// Peak memory in bytes
    pub peak_memory_bytes: u64,
    /// IO operations count
    pub io_operations: u64,
    /// Network bytes transferred
    pub network_bytes: u64,
    /// CPU cycles (if available)
    pub cpu_cycles: Option<u64>,
}

impl QuotaFootprint {
    /// Create a zero footprint
    pub fn zero() -> Self {
        Self {
            runtime_ms: 0,
            peak_memory_bytes: 0,
            io_operations: 0,
            network_bytes: 0,
            cpu_cycles: None,
        }
    }
}

/// Default invocation context for deserialization
fn default_invocation_context() -> Arc<InvocationContext> {
    use crate::autonomic::{AgentIdentity, TenantIdentity, QoSHints};
    Arc::new(InvocationContext {
        agent: AgentIdentity::anonymous(),
        tenant: TenantIdentity::default_tenant(),
        policy: None,
        qos: QoSHints::default(),
        correlation_id: String::from("default"),
        parent_invocation_id: None,
    })
}

/// Complete deterministic session log frame
///
/// This is the canonical O (observation) for μ_CNV.
/// Must be:
/// - Orderable (total order per agent/session)
/// - Hashable (content-addressed)
/// - Reconstructible (enough data to replay deterministically)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLogFrame {
    /// Stable noun/verb IDs for capability identification
    pub noun_id: String,
    pub verb_id: String,

    /// Capability contract and version
    pub capability_id: String,
    pub capability_version: u32,

    /// Invocation context (tenant, agent, policy)
    #[serde(skip, default = "default_invocation_context")]
    pub invocation_context: Arc<InvocationContext>,

    /// Attestation chain hash (for verification)
    pub attestation_chain_hash: Option<String>,

    /// Quota tier used (compile-time) + actual runtime footprint
    pub quota_tier: String,
    pub quota_footprint: QuotaFootprint,

    /// Input arguments (serialized for determinism)
    pub input_args: serde_json::Value,

    /// Environment variables captured (deterministic subset)
    pub env_vars: BTreeMap<String, String>,

    /// Logical clock for ordering
    pub logical_clock: LogicalClock,

    /// Output result (success or error)
    pub output_result: ResultFrame,

    /// Exit code class (Success, UserError, SystemError, etc.)
    pub exit_code_class: ExitCodeClass,

    /// Telemetry profile used (verbosity, color, context)
    pub telemetry_profile: TelemetryProfile,

    /// Content hash (SHA256 of serialized frame)
    pub content_hash: String,

    /// Frame metadata
    pub metadata: FrameMetadata,
}

/// Result frame - captures both success and error outcomes deterministically
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResultFrame {
    Success(serde_json::Value),
    Error(ErrorFrame),
}

/// Error frame - structured error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFrame {
    pub error_code: String,
    pub error_message: String,
    pub error_details: Option<serde_json::Value>,
}

/// Exit code classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitCodeClass {
    Success = 0,
    UserError = 1,
    SystemError = 2,
    ValidationError = 3,
    NotFound = 4,
    PermissionDenied = 5,
    InternalError = 6,
}

impl ExitCodeClass {
    /// Get the numeric exit code
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

/// Frame metadata for ordering and tracing
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct FrameMetadata {
    /// Unique frame ID (can be regenerated from content hash)
    pub frame_id: String,
    /// Session ID for ordering within a session
    pub session_id: String,
    /// Agent ID that executed this
    pub agent_id: String,
    /// Sequence number within session
    pub sequence_number: u64,
    /// Parent frame hash (for causal ordering)
    pub parent_frame_hash: Option<String>,
    /// Custom tags for filtering
    pub tags: Vec<String>,
}

impl SessionLogFrame {
    /// Create a new frame with computed hash
    pub fn new(
        noun_id: String,
        verb_id: String,
        capability_id: String,
        capability_version: u32,
        invocation_context: Arc<InvocationContext>,
        attestation_chain_hash: Option<String>,
        quota_tier: String,
        quota_footprint: QuotaFootprint,
        input_args: serde_json::Value,
        env_vars: BTreeMap<String, String>,
        logical_clock: LogicalClock,
        output_result: ResultFrame,
        exit_code_class: ExitCodeClass,
        telemetry_profile: TelemetryProfile,
        metadata: FrameMetadata,
    ) -> Result<Self, serde_json::Error> {
        // Build frame without hash first
        let mut frame = Self {
            noun_id,
            verb_id,
            capability_id,
            capability_version,
            invocation_context,
            attestation_chain_hash,
            quota_tier,
            quota_footprint,
            input_args,
            env_vars,
            logical_clock,
            output_result,
            exit_code_class,
            telemetry_profile,
            content_hash: String::new(),
            metadata,
        };

        // Compute content hash
        frame.content_hash = frame.compute_content_hash()?;
        Ok(frame)
    }

    /// Compute SHA256 hash of frame content
    pub fn compute_content_hash(&self) -> Result<String, serde_json::Error> {
        use sha2::{Sha256, Digest};

        // Serialize frame without hash
        let json = serde_json::to_string(&self)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Get the total order key for ordering frames
    pub fn order_key(&self) -> FrameOrderKey {
        FrameOrderKey {
            session_id: self.metadata.session_id.clone(),
            logical_tick: self.logical_clock.logical_tick,
            sequence_number: self.metadata.sequence_number,
        }
    }

    /// Verify frame integrity
    pub fn verify_integrity(&self) -> Result<bool, serde_json::Error> {
        let computed_hash = self.compute_content_hash()?;
        Ok(computed_hash == self.content_hash)
    }
}

/// Order key for total ordering of frames
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameOrderKey {
    pub session_id: String,
    pub logical_tick: u64,
    pub sequence_number: u64,
}

impl PartialOrd for FrameOrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FrameOrderKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.session_id
            .cmp(&other.session_id)
            .then_with(|| self.logical_tick.cmp(&other.logical_tick))
            .then_with(|| self.sequence_number.cmp(&other.sequence_number))
    }
}

/// Replay mode for deterministic execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayMode {
    /// Verify: re-run and compare results (A vs A')
    Verify,
    /// Simulate: run with quotas relaxed but preserve logical sequence
    Simulate,
    /// Audit: collect all side effects for analysis
    Audit,
}

/// Replay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    pub mode: ReplayMode,
    /// Replace non-deterministic sources (time, randomness)
    pub use_deterministic_substitutes: bool,
    /// Allow network calls to be stubbed
    pub allow_network_stubs: bool,
    /// Allow filesystem to be stubbed
    pub allow_fs_stubs: bool,
    /// Timeout for replay in milliseconds
    pub timeout_ms: Option<u64>,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            mode: ReplayMode::Verify,
            use_deterministic_substitutes: true,
            allow_network_stubs: true,
            allow_fs_stubs: false,
            timeout_ms: Some(30000),
        }
    }
}

/// Result of replay verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub frame: SessionLogFrame,
    pub mode: ReplayMode,
    pub success: bool,
    pub outcome_match: bool,
    pub timing_envelope_drift: Option<TimingDrift>,
    pub quota_check: QuotaCheckResult,
    pub error_details: Option<String>,
}

/// Timing envelope drift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingDrift {
    pub original_ms: u64,
    pub replayed_ms: u64,
    pub drift_percent: f64,
}

/// Result of quota check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub under_quota: bool,
    pub original_usage: QuotaFootprint,
    pub replayed_usage: QuotaFootprint,
    pub exceeded_resources: Vec<String>,
}

/// Frame delta - difference between two frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameDelta {
    pub frame_a_hash: String,
    pub frame_b_hash: String,

    /// Changed fields
    pub arg_changes: Option<serde_json::Value>,
    pub context_changes: Option<serde_json::Value>,
    pub policy_changes: Option<serde_json::Value>,

    /// Changed metrics
    pub timing_delta_ms: i64,
    pub memory_delta_bytes: i64,
    pub io_delta: i64,

    /// Changed outcomes
    pub outcome_changed: bool,
}

impl FrameDelta {
    /// Compute delta between two frames
    pub fn compute(frame_a: &SessionLogFrame, frame_b: &SessionLogFrame) -> Self {
        let arg_changes = if frame_a.input_args != frame_b.input_args {
            Some(serde_json::json!({
                "original": frame_a.input_args,
                "new": frame_b.input_args,
            }))
        } else {
            None
        };

        let outcome_changed = !matches!(
            (&frame_a.output_result, &frame_b.output_result),
            (ResultFrame::Success(a), ResultFrame::Success(b)) if a == b
        ) && !matches!(
            (&frame_a.output_result, &frame_b.output_result),
            (ResultFrame::Error(_), ResultFrame::Error(_))
        );

        let timing_delta_ms = frame_b.quota_footprint.runtime_ms as i64
            - frame_a.quota_footprint.runtime_ms as i64;
        let memory_delta_bytes = frame_b.quota_footprint.peak_memory_bytes as i64
            - frame_a.quota_footprint.peak_memory_bytes as i64;
        let io_delta = frame_b.quota_footprint.io_operations as i64
            - frame_a.quota_footprint.io_operations as i64;

        Self {
            frame_a_hash: frame_a.content_hash.clone(),
            frame_b_hash: frame_b.content_hash.clone(),
            arg_changes,
            context_changes: None,
            policy_changes: None,
            timing_delta_ms,
            memory_delta_bytes,
            io_delta,
            outcome_changed,
        }
    }
}

/// Session compression - compact representation of repeated invocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCompression {
    pub original_frame_count: usize,
    pub compressed_frame_count: usize,
    pub compression_ratio: f64,

    /// Invocation histogram (capability ID → count)
    pub invocation_histogram: BTreeMap<String, u64>,

    /// Timing statistics (percentiles)
    pub timing_percentiles: TimingPercentiles,

    /// Resource usage statistics
    pub resource_stats: ResourceStats,
}

/// Timing percentiles for compressed sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingPercentiles {
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub p999_ms: u64,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub total_memory_gb: f64,
    pub total_io_ops: u64,
    pub total_network_bytes: u64,
    pub peak_concurrent_agents: u64,
}

/// Session log store trait for pluggable backends
pub trait SessionLogStore: Send + Sync {
    /// Append a frame to the log
    fn append(&self, frame: SessionLogFrame) -> Result<(), String>;

    /// Retrieve a frame by hash
    fn get_by_hash(&self, hash: &str) -> Result<Option<SessionLogFrame>, String>;

    /// Query frames by predicate
    fn query(&self, predicate: &dyn Fn(&SessionLogFrame) -> bool) -> Result<Vec<SessionLogFrame>, String>;

    /// Get frames in order for a session
    fn get_session_frames(&self, session_id: &str) -> Result<Vec<SessionLogFrame>, String>;

    /// Compute compression for a range of frames
    fn compute_compression(&self, start_seq: u64, end_seq: u64) -> Result<SessionCompression, String>;
}

/// In-memory session log store (for testing and small deployments)
pub struct InMemorySessionLogStore {
    frames: parking_lot::RwLock<BTreeMap<String, SessionLogFrame>>,
}

impl InMemorySessionLogStore {
    pub fn new() -> Self {
        Self {
            frames: parking_lot::RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for InMemorySessionLogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLogStore for InMemorySessionLogStore {
    fn append(&self, frame: SessionLogFrame) -> Result<(), String> {
        let mut frames = self.frames.write();
        frames.insert(frame.content_hash.clone(), frame);
        Ok(())
    }

    fn get_by_hash(&self, hash: &str) -> Result<Option<SessionLogFrame>, String> {
        let frames = self.frames.read();
        Ok(frames.get(hash).cloned())
    }

    fn query(&self, predicate: &dyn Fn(&SessionLogFrame) -> bool) -> Result<Vec<SessionLogFrame>, String> {
        let frames = self.frames.read();
        Ok(frames.values().filter(|f| predicate(f)).cloned().collect())
    }

    fn get_session_frames(&self, session_id: &str) -> Result<Vec<SessionLogFrame>, String> {
        let frames = self.frames.read();
        let mut result: Vec<_> = frames
            .values()
            .filter(|f| f.metadata.session_id == session_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| a.order_key().cmp(&b.order_key()));
        Ok(result)
    }

    fn compute_compression(&self, start_seq: u64, end_seq: u64) -> Result<SessionCompression, String> {
        let frames = self.frames.read();
        let selected: Vec<_> = frames
            .values()
            .filter(|f| f.metadata.sequence_number >= start_seq && f.metadata.sequence_number <= end_seq)
            .collect();

        if selected.is_empty() {
            return Ok(SessionCompression {
                original_frame_count: 0,
                compressed_frame_count: 0,
                compression_ratio: 1.0,
                invocation_histogram: BTreeMap::new(),
                timing_percentiles: TimingPercentiles {
                    p50_ms: 0,
                    p95_ms: 0,
                    p99_ms: 0,
                    p999_ms: 0,
                },
                resource_stats: ResourceStats {
                    total_memory_gb: 0.0,
                    total_io_ops: 0,
                    total_network_bytes: 0,
                    peak_concurrent_agents: 0,
                },
            });
        }

        let mut histogram: BTreeMap<String, u64> = BTreeMap::new();
        let mut timings: Vec<u64> = Vec::new();
        let mut total_memory: u64 = 0;
        let mut total_io: u64 = 0;
        let mut total_network: u64 = 0;

        for frame in &selected {
            *histogram
                .entry(frame.capability_id.clone())
                .or_insert(0) += 1;
            timings.push(frame.quota_footprint.runtime_ms);
            total_memory += frame.quota_footprint.peak_memory_bytes;
            total_io += frame.quota_footprint.io_operations;
            total_network += frame.quota_footprint.network_bytes;
        }

        timings.sort_unstable();
        let len = timings.len();

        Ok(SessionCompression {
            original_frame_count: selected.len(),
            compressed_frame_count: histogram.len(),
            compression_ratio: histogram.len() as f64 / selected.len() as f64,
            invocation_histogram: histogram,
            timing_percentiles: TimingPercentiles {
                p50_ms: timings[len / 2],
                p95_ms: timings[(len * 95) / 100],
                p99_ms: timings[(len * 99) / 100],
                p999_ms: timings[(len * 999) / 1000],
            },
            resource_stats: ResourceStats {
                total_memory_gb: total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
                total_io_ops: total_io,
                total_network_bytes: total_network,
                peak_concurrent_agents: 0, // TODO: track from metadata
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_clock_ordering() {
        let clock1 = LogicalClock::new(1, 100);
        let clock2 = LogicalClock::new(2, 50);
        assert!(clock1 < clock2);
    }

    #[test]
    fn test_frame_order_key() {
        let key1 = FrameOrderKey {
            session_id: "s1".to_string(),
            logical_tick: 1,
            sequence_number: 1,
        };
        let key2 = FrameOrderKey {
            session_id: "s1".to_string(),
            logical_tick: 1,
            sequence_number: 2,
        };
        assert!(key1 < key2);
    }

    #[test]
    fn test_quota_footprint() {
        let footprint = QuotaFootprint::zero();
        assert_eq!(footprint.runtime_ms, 0);
        assert_eq!(footprint.peak_memory_bytes, 0);
    }
}
