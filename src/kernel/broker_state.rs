//! BrokerKernel State Machine & DoS Hardening (Task 2)
//!
//! Implements explicit broker state machine with timeout and queue controls

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Broker lifecycle state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerState {
    /// Initial state, not started
    Cold,
    /// Starting up, performing initialization
    Starting,
    /// Fully operational
    Running,
    /// Operating with reduced capacity
    Degraded,
    /// Draining connections, no new admissions
    Draining,
    /// Stopped
    Stopped,
}

impl BrokerState {
    /// Check if transitions from this state to another are legal
    pub fn can_transition_to(&self, target: BrokerState) -> bool {
        match (self, target) {
            // From Cold: can only go to Starting
            (Self::Cold, Self::Starting) => true,
            // From Starting: can go to Running
            (Self::Starting, Self::Running) => true,
            // From Running: can go to Degraded or Draining
            (Self::Running, Self::Degraded) | (Self::Running, Self::Draining) => true,
            // From Degraded: can go to Running or Draining
            (Self::Degraded, Self::Running) | (Self::Degraded, Self::Draining) => true,
            // From Draining: can go to Stopped
            (Self::Draining, Self::Stopped) => true,
            // From Stopped: cannot transition (terminal state)
            (Self::Stopped, _) => false,
            // Any other transitions are illegal
            _ => false,
        }
    }

    /// States that allow admitting new invocations
    pub fn allows_admission(&self) -> bool {
        matches!(self, Self::Running | Self::Degraded)
    }

    /// States that allow opening new sessions
    pub fn allows_session_open(&self) -> bool {
        matches!(self, Self::Running | Self::Degraded)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running | Self::Degraded)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped)
    }
}

/// Backpressure error - returned when broker reaches limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureError {
    pub reason: String,
    pub retry_after_ms: u64,
    pub current_load: f64,
    pub max_capacity: usize,
}

impl std::fmt::Display for BackpressureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Broker backpressure: {} (load: {:.2}%, retry after {}ms)",
            self.reason, self.current_load, self.retry_after_ms
        )
    }
}

/// Timeout configuration for broker operations
#[derive(Debug, Clone, Copy)]
pub struct BrokerTimeouts {
    /// Handshake timeout (default 5 seconds)
    pub handshake_timeout_ms: u64,
    /// Idle connection timeout (default 30 seconds)
    pub idle_timeout_ms: u64,
    /// State transition timeout (default 10 seconds)
    pub state_transition_timeout_ms: u64,
}

impl Default for BrokerTimeouts {
    fn default() -> Self {
        Self {
            handshake_timeout_ms: 5000,
            idle_timeout_ms: 30000,
            state_transition_timeout_ms: 10000,
        }
    }
}

impl BrokerTimeouts {
    pub fn handshake_timeout(&self) -> Duration {
        Duration::from_millis(self.handshake_timeout_ms)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_millis(self.idle_timeout_ms)
    }

    pub fn state_transition_timeout(&self) -> Duration {
        Duration::from_millis(self.state_transition_timeout_ms)
    }
}

/// Queue limits for backpressure control
#[derive(Debug, Clone, Copy)]
pub struct QueueLimits {
    /// Maximum frames per tenant
    pub per_tenant_queue_size: usize,
    /// Global queue size limit
    pub global_queue_size: usize,
    /// Total bytes in all queues
    pub max_total_queue_bytes: usize,
}

impl Default for QueueLimits {
    fn default() -> Self {
        Self {
            per_tenant_queue_size: 10_000,
            global_queue_size: 100_000,
            max_total_queue_bytes: 512 * 1024 * 1024,  // 512 MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions_cold_to_running() {
        let mut state = BrokerState::Cold;
        assert!(state.can_transition_to(BrokerState::Starting));
        state = BrokerState::Starting;
        assert!(state.can_transition_to(BrokerState::Running));
        state = BrokerState::Running;
        assert!(state.is_running());
    }

    #[test]
    fn test_state_transitions_invalid() {
        let state = BrokerState::Cold;
        assert!(!state.can_transition_to(BrokerState::Running));
        assert!(!state.can_transition_to(BrokerState::Stopped));
    }

    #[test]
    fn test_state_transitions_degraded() {
        let state = BrokerState::Running;
        assert!(state.can_transition_to(BrokerState::Degraded));

        let state = BrokerState::Degraded;
        assert!(state.allows_admission());
        assert!(state.can_transition_to(BrokerState::Running));
    }

    #[test]
    fn test_state_terminal() {
        let state = BrokerState::Stopped;
        assert!(state.is_terminal());
        assert!(!state.can_transition_to(BrokerState::Running));
    }

    #[test]
    fn test_admission_guards() {
        assert!(BrokerState::Running.allows_admission());
        assert!(BrokerState::Degraded.allows_admission());
        assert!(!BrokerState::Cold.allows_admission());
        assert!(!BrokerState::Draining.allows_admission());
        assert!(!BrokerState::Stopped.allows_admission());
    }

    #[test]
    fn test_backpressure_error_display() {
        let error = BackpressureError {
            reason: "Queue limit exceeded".to_string(),
            retry_after_ms: 1000,
            current_load: 95.5,
            max_capacity: 10000,
        };
        assert!(error.to_string().contains("95.5"));
        assert!(error.to_string().contains("1000"));
    }

    #[test]
    fn test_broker_timeouts_defaults() {
        let timeouts = BrokerTimeouts::default();
        assert_eq!(timeouts.handshake_timeout_ms, 5000);
        assert_eq!(timeouts.idle_timeout_ms, 30000);
    }

    #[test]
    fn test_queue_limits_defaults() {
        let limits = QueueLimits::default();
        assert!(limits.per_tenant_queue_size > 0);
        assert!(limits.global_queue_size > limits.per_tenant_queue_size);
    }
}
