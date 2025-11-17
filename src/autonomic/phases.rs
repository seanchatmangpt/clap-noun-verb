//! Swarm Lifecycle Phases for Trillion-Agent Coordination
//!
//! This module implements a comprehensive phase system for managing the lifecycle
//! of autonomous agent swarms at trillion-invocation scale.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐
//! │  Bootstrap  │──────────────┐
//! └──────┬──────┘              │
//!        │                     │ (emergency)
//!        ▼                     │
//! ┌─────────────┐              │
//! │ Negotiation │              │
//! └──────┬──────┘              │
//!        │                     │
//!        ▼                     ▼
//! ┌─────────────┐      ┌──────────────┐
//! │ Activation  │────▶ │  Emergency   │
//! └──────┬──────┘      └──────────────┘
//!        │                     ▲
//!        ▼                     │
//! ┌─────────────┐              │
//! │ Operational │──────────────┤
//! └──────┬──────┘              │
//!        │                     │
//!        ├──────────────────────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │  Degraded   │
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │  Recovery   │
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │  Shutdown   │
//! └─────────────┘
//! ```
//!
//! ## Type-State Pattern
//!
//! Phase transitions are enforced at compile time using phantom types,
//! preventing invalid state transitions and runtime errors.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::telemetry;

// ============================================================================
// Phase States (Zero-Sized Types for Compile-Time Guarantees)
// ============================================================================

/// Bootstrap phase - Initial setup and discovery
pub struct Bootstrap;

/// Negotiation phase - Protocol and capability negotiation
pub struct Negotiation;

/// Activation phase - Agents joining the swarm
pub struct Activation;

/// Operational phase - Normal execution
pub struct Operational;

/// Degraded phase - Partial failures, reduced capacity
pub struct Degraded;

/// Recovery phase - Healing and restoration
pub struct Recovery;

/// Shutdown phase - Graceful termination
pub struct Shutdown;

/// Emergency phase - Circuit breaker activated
pub struct Emergency;

// ============================================================================
// Phase Context with Type-State Pattern
// ============================================================================

/// Phase context with compile-time state enforcement
///
/// This uses the type-state pattern to ensure phase transitions are valid
/// at compile time, preventing runtime errors from invalid transitions.
pub struct PhaseContext<State = Bootstrap> {
    /// Phase metadata
    metadata: Arc<PhaseMetadata>,

    /// Phantom type for compile-time phase tracking
    _state: PhantomData<State>,
}

impl<S> Clone for PhaseContext<S> {
    fn clone(&self) -> Self {
        Self {
            metadata: Arc::clone(&self.metadata),
            _state: PhantomData,
        }
    }
}

/// Phase metadata shared across all states
pub struct PhaseMetadata {
    /// Current phase ID (for runtime tracking)
    current_phase: AtomicU8,

    /// Phase start time
    started_at: Instant,

    /// Transition count
    transitions: AtomicU64,

    /// Active agent count
    active_agents: AtomicU64,

    /// Failed agent count
    failed_agents: AtomicU64,

    /// Phase-specific metrics
    metrics: Arc<std::sync::Mutex<PhaseMetrics>>,
}

/// Phase-specific metrics
#[derive(Debug, Clone)]
pub struct PhaseMetrics {
    /// Phase durations
    pub phase_durations: Vec<(PhaseId, Duration)>,

    /// Transition history
    pub transitions: Vec<PhaseTransition>,

    /// Error count per phase
    pub errors_by_phase: std::collections::HashMap<PhaseId, u64>,
}

impl Default for PhaseMetrics {
    fn default() -> Self {
        Self {
            phase_durations: Vec::new(),
            transitions: Vec::new(),
            errors_by_phase: std::collections::HashMap::new(),
        }
    }
}

/// Phase transition record
#[derive(Debug, Clone)]
pub struct PhaseTransition {
    /// Source phase
    pub from: PhaseId,

    /// Target phase
    pub to: PhaseId,

    /// Transition timestamp
    pub timestamp: Instant,

    /// Transition reason
    pub reason: String,
}

/// Phase identifier for runtime tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PhaseId {
    Bootstrap = 0,
    Negotiation = 1,
    Activation = 2,
    Operational = 3,
    Degraded = 4,
    Recovery = 5,
    Shutdown = 6,
    Emergency = 7,
}

impl PhaseId {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PhaseId::Bootstrap),
            1 => Some(PhaseId::Negotiation),
            2 => Some(PhaseId::Activation),
            3 => Some(PhaseId::Operational),
            4 => Some(PhaseId::Degraded),
            5 => Some(PhaseId::Recovery),
            6 => Some(PhaseId::Shutdown),
            7 => Some(PhaseId::Emergency),
            _ => None,
        }
    }

    /// Get phase name
    pub fn name(&self) -> &'static str {
        match self {
            PhaseId::Bootstrap => "Bootstrap",
            PhaseId::Negotiation => "Negotiation",
            PhaseId::Activation => "Activation",
            PhaseId::Operational => "Operational",
            PhaseId::Degraded => "Degraded",
            PhaseId::Recovery => "Recovery",
            PhaseId::Shutdown => "Shutdown",
            PhaseId::Emergency => "Emergency",
        }
    }
}

// ============================================================================
// Phase Context Implementations
// ============================================================================

impl PhaseContext<Bootstrap> {
    /// Create a new phase context in Bootstrap state
    pub fn new() -> Self {
        Self {
            metadata: Arc::new(PhaseMetadata {
                current_phase: AtomicU8::new(PhaseId::Bootstrap as u8),
                started_at: Instant::now(),
                transitions: AtomicU64::new(0),
                active_agents: AtomicU64::new(0),
                failed_agents: AtomicU64::new(0),
                metrics: Arc::new(std::sync::Mutex::new(PhaseMetrics::default())),
            }),
            _state: PhantomData,
        }
    }

    /// Transition to Negotiation phase
    pub fn begin_negotiation(
        self,
        reason: impl Into<String>,
    ) -> PhaseContext<Negotiation> {
        self.transition(PhaseId::Negotiation, reason)
    }

    /// Emergency transition (available from any phase)
    pub fn enter_emergency(self, reason: impl Into<String>) -> PhaseContext<Emergency> {
        self.transition(PhaseId::Emergency, reason)
    }
}

impl PhaseContext<Negotiation> {
    /// Transition to Activation phase
    pub fn begin_activation(self, reason: impl Into<String>) -> PhaseContext<Activation> {
        self.transition(PhaseId::Activation, reason)
    }

    /// Emergency transition
    pub fn enter_emergency(self, reason: impl Into<String>) -> PhaseContext<Emergency> {
        self.transition(PhaseId::Emergency, reason)
    }
}

impl PhaseContext<Activation> {
    /// Transition to Operational phase
    pub fn become_operational(
        self,
        reason: impl Into<String>,
    ) -> PhaseContext<Operational> {
        self.transition(PhaseId::Operational, reason)
    }

    /// Register an agent joining the swarm
    pub fn register_agent(&self) {
        self.metadata.active_agents.fetch_add(1, Ordering::Relaxed);
        telemetry::telemetry().counter_inc("swarm_agents_joined", 1);
    }

    /// Emergency transition
    pub fn enter_emergency(self, reason: impl Into<String>) -> PhaseContext<Emergency> {
        self.transition(PhaseId::Emergency, reason)
    }
}

impl PhaseContext<Operational> {
    /// Transition to Degraded phase due to failures
    pub fn enter_degraded(self, reason: impl Into<String>) -> PhaseContext<Degraded> {
        self.transition(PhaseId::Degraded, reason)
    }

    /// Transition to Shutdown phase
    pub fn begin_shutdown(self, reason: impl Into<String>) -> PhaseContext<Shutdown> {
        self.transition(PhaseId::Shutdown, reason)
    }

    /// Record successful invocation
    pub fn record_success(&self) {
        telemetry::telemetry().counter_inc("swarm_invocations_success", 1);
    }

    /// Record failed invocation
    pub fn record_failure(&self) {
        telemetry::telemetry().counter_inc("swarm_invocations_failed", 1);
        self.metadata.failed_agents.fetch_add(1, Ordering::Relaxed);
    }

    /// Get health ratio (active / total)
    pub fn health_ratio(&self) -> f64 {
        let active = self.metadata.active_agents.load(Ordering::Relaxed) as f64;
        let failed = self.metadata.failed_agents.load(Ordering::Relaxed) as f64;
        let total = active + failed;

        if total == 0.0 {
            1.0
        } else {
            active / total
        }
    }

    /// Emergency transition
    pub fn enter_emergency(self, reason: impl Into<String>) -> PhaseContext<Emergency> {
        self.transition(PhaseId::Emergency, reason)
    }
}

impl PhaseContext<Degraded> {
    /// Transition to Recovery phase
    pub fn begin_recovery(self, reason: impl Into<String>) -> PhaseContext<Recovery> {
        self.transition(PhaseId::Recovery, reason)
    }

    /// Transition to Shutdown if unrecoverable
    pub fn begin_shutdown(self, reason: impl Into<String>) -> PhaseContext<Shutdown> {
        self.transition(PhaseId::Shutdown, reason)
    }

    /// Emergency transition
    pub fn enter_emergency(self, reason: impl Into<String>) -> PhaseContext<Emergency> {
        self.transition(PhaseId::Emergency, reason)
    }
}

impl PhaseContext<Recovery> {
    /// Transition back to Operational after recovery
    pub fn restore_operational(
        self,
        reason: impl Into<String>,
    ) -> PhaseContext<Operational> {
        self.transition(PhaseId::Operational, reason)
    }

    /// Transition to Shutdown if recovery fails
    pub fn begin_shutdown(self, reason: impl Into<String>) -> PhaseContext<Shutdown> {
        self.transition(PhaseId::Shutdown, reason)
    }

    /// Reset failed agent counter
    pub fn reset_failures(&self) {
        self.metadata.failed_agents.store(0, Ordering::Relaxed);
    }

    /// Emergency transition
    pub fn enter_emergency(self, reason: impl Into<String>) -> PhaseContext<Emergency> {
        self.transition(PhaseId::Emergency, reason)
    }
}

impl PhaseContext<Emergency> {
    /// Attempt recovery to Operational
    pub fn attempt_recovery(self, reason: impl Into<String>) -> PhaseContext<Recovery> {
        self.transition(PhaseId::Recovery, reason)
    }

    /// Force shutdown
    pub fn force_shutdown(self, reason: impl Into<String>) -> PhaseContext<Shutdown> {
        self.transition(PhaseId::Shutdown, reason)
    }
}

impl PhaseContext<Shutdown> {
    /// Finalize shutdown and return statistics
    pub fn finalize(self) -> ShutdownReport {
        let elapsed = self.metadata.started_at.elapsed();
        let transitions = self.metadata.transitions.load(Ordering::Relaxed);
        let active_agents = self.metadata.active_agents.load(Ordering::Relaxed);
        let failed_agents = self.metadata.failed_agents.load(Ordering::Relaxed);

        let metrics = self.metadata.metrics.lock().unwrap().clone();

        ShutdownReport {
            total_duration: elapsed,
            total_transitions: transitions,
            final_active_agents: active_agents,
            final_failed_agents: failed_agents,
            metrics,
        }
    }
}

/// Shutdown report with final statistics
#[derive(Debug, Clone)]
pub struct ShutdownReport {
    /// Total swarm lifetime
    pub total_duration: Duration,

    /// Total phase transitions
    pub total_transitions: u64,

    /// Final active agent count
    pub final_active_agents: u64,

    /// Final failed agent count
    pub final_failed_agents: u64,

    /// Detailed metrics
    pub metrics: PhaseMetrics,
}

// ============================================================================
// Generic Phase Transition Implementation
// ============================================================================

impl<S> PhaseContext<S> {
    /// Internal transition helper
    fn transition<T>(self, to_phase: PhaseId, reason: impl Into<String>) -> PhaseContext<T> {
        let from_phase = PhaseId::from_u8(
            self.metadata.current_phase.load(Ordering::Relaxed)
        ).unwrap_or(PhaseId::Bootstrap);

        // Update phase
        self.metadata.current_phase.store(to_phase as u8, Ordering::Release);

        // Increment transition counter
        self.metadata.transitions.fetch_add(1, Ordering::Relaxed);

        // Record transition
        let mut metrics = self.metadata.metrics.lock().unwrap();
        metrics.transitions.push(PhaseTransition {
            from: from_phase,
            to: to_phase,
            timestamp: Instant::now(),
            reason: reason.into(),
        });

        // Record telemetry
        telemetry::telemetry().counter_inc(
            &format!("swarm_phase_transition_{}_{}", from_phase.name(), to_phase.name()),
            1,
        );

        drop(metrics);

        PhaseContext {
            metadata: self.metadata,
            _state: PhantomData,
        }
    }

    /// Get current phase ID (runtime check)
    pub fn current_phase(&self) -> PhaseId {
        PhaseId::from_u8(self.metadata.current_phase.load(Ordering::Acquire))
            .unwrap_or(PhaseId::Bootstrap)
    }

    /// Get uptime since phase context creation
    pub fn uptime(&self) -> Duration {
        self.metadata.started_at.elapsed()
    }

    /// Get transition count
    pub fn transition_count(&self) -> u64 {
        self.metadata.transitions.load(Ordering::Relaxed)
    }

    /// Get active agent count
    pub fn active_agents(&self) -> u64 {
        self.metadata.active_agents.load(Ordering::Relaxed)
    }

    /// Get failed agent count
    pub fn failed_agents(&self) -> u64 {
        self.metadata.failed_agents.load(Ordering::Relaxed)
    }

    /// Get phase metrics snapshot
    pub fn metrics_snapshot(&self) -> PhaseMetrics {
        self.metadata.metrics.lock().unwrap().clone()
    }
}

impl Default for PhaseContext<Bootstrap> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Phase Coordinator for Lock-Free Multi-Swarm Management
// ============================================================================

/// Lock-free coordinator for managing multiple swarm phases
pub struct PhaseCoordinator {
    /// Swarm phase contexts (indexed by swarm ID)
    swarms: Arc<std::sync::RwLock<std::collections::HashMap<String, Arc<PhaseMetadata>>>>,
}

impl PhaseCoordinator {
    /// Create a new phase coordinator
    pub fn new() -> Self {
        Self {
            swarms: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a new swarm
    pub fn register_swarm(&self, swarm_id: String, context: &PhaseContext<impl Sized>) {
        let mut swarms = self.swarms.write().unwrap();
        swarms.insert(swarm_id, Arc::clone(&context.metadata));
    }

    /// Get current phase for a swarm
    pub fn get_phase(&self, swarm_id: &str) -> Option<PhaseId> {
        let swarms = self.swarms.read().unwrap();
        swarms.get(swarm_id).map(|metadata| {
            PhaseId::from_u8(metadata.current_phase.load(Ordering::Acquire))
                .unwrap_or(PhaseId::Bootstrap)
        })
    }

    /// Get aggregate statistics across all swarms
    pub fn aggregate_stats(&self) -> AggregateStats {
        let swarms = self.swarms.read().unwrap();

        let mut stats = AggregateStats::default();

        for metadata in swarms.values() {
            stats.total_swarms += 1;
            stats.total_active_agents += metadata.active_agents.load(Ordering::Relaxed);
            stats.total_failed_agents += metadata.failed_agents.load(Ordering::Relaxed);
            stats.total_transitions += metadata.transitions.load(Ordering::Relaxed);

            let phase = PhaseId::from_u8(metadata.current_phase.load(Ordering::Acquire))
                .unwrap_or(PhaseId::Bootstrap);

            *stats.swarms_by_phase.entry(phase).or_insert(0) += 1;
        }

        stats
    }
}

impl Default for PhaseCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate statistics across all swarms
#[derive(Debug, Default)]
pub struct AggregateStats {
    /// Total number of swarms
    pub total_swarms: usize,

    /// Total active agents across all swarms
    pub total_active_agents: u64,

    /// Total failed agents across all swarms
    pub total_failed_agents: u64,

    /// Total phase transitions across all swarms
    pub total_transitions: u64,

    /// Swarms grouped by current phase
    pub swarms_by_phase: std::collections::HashMap<PhaseId, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transitions() {
        // Bootstrap → Negotiation → Activation → Operational
        let ctx = PhaseContext::new();
        assert_eq!(ctx.current_phase(), PhaseId::Bootstrap);

        let ctx = ctx.begin_negotiation("Starting negotiation");
        assert_eq!(ctx.current_phase(), PhaseId::Negotiation);

        let ctx = ctx.begin_activation("Activating swarm");
        assert_eq!(ctx.current_phase(), PhaseId::Activation);

        ctx.register_agent();
        ctx.register_agent();
        assert_eq!(ctx.active_agents(), 2);

        let ctx = ctx.become_operational("Swarm ready");
        assert_eq!(ctx.current_phase(), PhaseId::Operational);

        assert_eq!(ctx.transition_count(), 3);
    }

    #[test]
    fn test_degraded_recovery_flow() {
        let ctx = PhaseContext::new()
            .begin_negotiation("test")
            .begin_activation("test")
            .become_operational("test");

        // Simulate failures
        ctx.record_failure();
        ctx.record_failure();

        let health = ctx.health_ratio();
        assert!(health < 1.0);

        // Enter degraded mode
        let ctx = ctx.enter_degraded("Too many failures");
        assert_eq!(ctx.current_phase(), PhaseId::Degraded);

        // Begin recovery
        let ctx = ctx.begin_recovery("Attempting recovery");
        assert_eq!(ctx.current_phase(), PhaseId::Recovery);

        ctx.reset_failures();

        // Restore to operational
        let ctx = ctx.restore_operational("Recovery successful");
        assert_eq!(ctx.current_phase(), PhaseId::Operational);
    }

    #[test]
    fn test_emergency_transition() {
        let ctx = PhaseContext::new()
            .begin_negotiation("test")
            .enter_emergency("Critical failure");

        assert_eq!(ctx.current_phase(), PhaseId::Emergency);

        let ctx = ctx.attempt_recovery("Trying to recover");
        assert_eq!(ctx.current_phase(), PhaseId::Recovery);
    }

    #[test]
    fn test_shutdown_report() {
        let ctx = PhaseContext::new()
            .begin_negotiation("test")
            .begin_activation("test");

        ctx.register_agent();
        ctx.register_agent();
        ctx.register_agent();

        let ctx = ctx.become_operational("test")
            .begin_shutdown("Graceful shutdown");

        let report = ctx.finalize();

        assert_eq!(report.final_active_agents, 3);
        assert!(report.total_duration > Duration::from_secs(0));
        assert!(report.total_transitions >= 4);
    }

    #[test]
    fn test_phase_coordinator() {
        let coordinator = PhaseCoordinator::new();

        let swarm1 = PhaseContext::new();
        coordinator.register_swarm("swarm-1".to_string(), &swarm1);

        let swarm2 = PhaseContext::new().begin_negotiation("test");
        coordinator.register_swarm("swarm-2".to_string(), &swarm2);

        assert_eq!(coordinator.get_phase("swarm-1"), Some(PhaseId::Bootstrap));
        assert_eq!(coordinator.get_phase("swarm-2"), Some(PhaseId::Negotiation));

        let stats = coordinator.aggregate_stats();
        assert_eq!(stats.total_swarms, 2);
        assert_eq!(*stats.swarms_by_phase.get(&PhaseId::Bootstrap).unwrap_or(&0), 1);
        assert_eq!(*stats.swarms_by_phase.get(&PhaseId::Negotiation).unwrap_or(&0), 1);
    }
}
