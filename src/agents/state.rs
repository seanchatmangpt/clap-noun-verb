//! Type-state machine for agent lifecycle management
//!
//! This module implements a compile-time enforced state machine using PhantomData.
//! Invalid state transitions are caught at compile time, ensuring type safety.
//!
//! # State Transitions
//!
//! ```text
//! Unregistered -> Registered -> Verified -> Trusted
//!                                   |
//!                                   v
//!                              Escalated
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::agents::state::*;
//!
//! // Create new unregistered agent
//! let agent = AgentState::<Unregistered>::new("agent-001".to_string());
//!
//! // Register agent (compile-time state transition)
//! let agent = agent.register(vec!["capability1".to_string()]);
//!
//! // Verify agent (compile-time state transition)
//! let agent = agent.verify(b"proof-data");
//!
//! // Trust agent (compile-time state transition)
//! let agent = agent.trust(0.95);
//! ```

use std::marker::PhantomData;

#[cfg(feature = "crypto")]
use sha3::{Digest, Sha3_256};

use crate::error::{NounVerbError, Result};

// =============================================================================
// State Markers - Zero-cost PhantomData types
// =============================================================================

/// Unregistered state - agent not yet registered
#[derive(Debug, Clone, Copy)]
pub struct Unregistered;

/// Registered state - agent registered with capabilities
#[derive(Debug, Clone, Copy)]
pub struct Registered;

/// Verified state - agent identity verified
#[derive(Debug, Clone, Copy)]
pub struct Verified;

/// Trusted state - agent trust score established
#[derive(Debug, Clone, Copy)]
pub struct Trusted;

/// Escalated state - agent escalated for review
#[derive(Debug, Clone, Copy)]
pub struct Escalated;

// =============================================================================
// AgentState - Type-state machine with PhantomData
// =============================================================================

/// Type-state machine for agent lifecycle
///
/// Uses PhantomData to encode state at compile time. Invalid transitions
/// will fail to compile, providing zero-cost safety guarantees.
#[derive(Debug, Clone)]
pub struct AgentState<S> {
    /// Agent unique identifier
    pub id: String,

    /// Agent capabilities (semantic tags)
    pub capabilities: Vec<String>,

    /// Trust score (0.0 - 1.0)
    pub trust_score: f64,

    /// Verification proof hash
    pub proof_hash: Option<String>,

    /// State marker (zero-cost)
    _state: PhantomData<S>,
}

// =============================================================================
// State-specific implementations
// =============================================================================

impl AgentState<Unregistered> {
    /// Create new unregistered agent
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let agent = AgentState::<Unregistered>::new("agent-001".to_string());
    /// ```
    pub fn new(id: String) -> Self {
        Self {
            id,
            capabilities: Vec::new(),
            trust_score: 0.0,
            proof_hash: None,
            _state: PhantomData,
        }
    }

    /// Register agent with capabilities
    ///
    /// Transitions: Unregistered -> Registered
    ///
    /// # Arguments
    ///
    /// * `capabilities` - List of semantic capability tags
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let agent = agent.register(vec!["nlp".to_string(), "vision".to_string()]);
    /// ```
    pub fn register(self, capabilities: Vec<String>) -> AgentState<Registered> {
        AgentState {
            id: self.id,
            capabilities,
            trust_score: 0.0,
            proof_hash: None,
            _state: PhantomData,
        }
    }
}

impl AgentState<Registered> {
    /// Verify agent identity with cryptographic proof
    ///
    /// Transitions: Registered -> Verified
    ///
    /// # Arguments
    ///
    /// * `proof` - Cryptographic proof data
    ///
    /// # Returns
    ///
    /// Verified agent state
    #[cfg(feature = "crypto")]
    pub fn verify(self, proof: &[u8]) -> AgentState<Verified> {
        let mut hasher = Sha3_256::new();
        hasher.update(proof);
        hasher.update(self.id.as_bytes());
        let proof_hash = format!("{:x}", hasher.finalize());

        AgentState {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: 0.5, // Initial trust after verification
            proof_hash: Some(proof_hash),
            _state: PhantomData,
        }
    }

    /// Verify agent without crypto (for testing)
    #[cfg(not(feature = "crypto"))]
    pub fn verify(self, _proof: &[u8]) -> AgentState<Verified> {
        AgentState {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: 0.5,
            proof_hash: Some("no-crypto-proof".to_string()),
            _state: PhantomData,
        }
    }
}

impl AgentState<Verified> {
    /// Establish trust score for agent
    ///
    /// Transitions: Verified -> Trusted
    ///
    /// # Arguments
    ///
    /// * `trust_score` - Trust score (0.0 - 1.0)
    ///
    /// # Returns
    ///
    /// Result containing trusted agent state
    pub fn trust(self, trust_score: f64) -> Result<AgentState<Trusted>> {
        if !(0.0..=1.0).contains(&trust_score) {
            return Err(NounVerbError::validation_range_error(
                "trust_score",
                format!("{}", trust_score),
                Some("0.0"),
                Some("1.0"),
            ));
        }

        Ok(AgentState {
            id: self.id,
            capabilities: self.capabilities,
            trust_score,
            proof_hash: self.proof_hash,
            _state: PhantomData,
        })
    }

    /// Escalate agent for review
    ///
    /// Transitions: Verified -> Escalated
    ///
    /// # Arguments
    ///
    /// * `reason` - Escalation reason
    ///
    /// # Returns
    ///
    /// Escalated agent state
    pub fn escalate(self, _reason: String) -> AgentState<Escalated> {
        AgentState {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: 0.0, // Reset trust on escalation
            proof_hash: self.proof_hash,
            _state: PhantomData,
        }
    }
}

impl AgentState<Trusted> {
    /// Get trust score
    pub fn get_trust_score(&self) -> f64 {
        self.trust_score
    }

    /// Update trust score based on behavior
    ///
    /// # Arguments
    ///
    /// * `delta` - Trust score adjustment (-1.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Result containing updated agent state
    pub fn update_trust(mut self, delta: f64) -> Result<Self> {
        let new_score = (self.trust_score + delta).clamp(0.0, 1.0);

        if new_score < 0.5 {
            return Err(NounVerbError::ValidationFailed(
                "trust score fell below threshold of 0.5".to_string(),
            ));
        }

        self.trust_score = new_score;
        Ok(self)
    }

    /// Escalate trusted agent for review
    ///
    /// Transitions: Trusted -> Escalated
    ///
    /// # Arguments
    ///
    /// * `reason` - Escalation reason
    pub fn escalate(self, _reason: String) -> AgentState<Escalated> {
        AgentState {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: 0.0,
            proof_hash: self.proof_hash,
            _state: PhantomData,
        }
    }
}

impl AgentState<Escalated> {
    /// Get escalation details
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Resolve escalation and return to verified state
    ///
    /// Transitions: Escalated -> Verified
    pub fn resolve(self) -> AgentState<Verified> {
        AgentState {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: 0.3, // Lower initial trust after escalation
            proof_hash: self.proof_hash,
            _state: PhantomData,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions_happy_path() {
        // Arrange: Create unregistered agent
        let agent = AgentState::<Unregistered>::new("test-agent".to_string());

        // Act: Register agent
        let agent = agent.register(vec!["nlp".to_string()]);

        // Assert: Agent is registered
        assert_eq!(agent.capabilities.len(), 1);
        assert_eq!(agent.capabilities[0], "nlp");

        // Act: Verify agent
        let agent = agent.verify(b"proof-data");

        // Assert: Agent is verified with initial trust
        assert_eq!(agent.trust_score, 0.5);
        assert!(agent.proof_hash.is_some());

        // Act: Trust agent
        let agent = agent.trust(0.95);

        // Assert: Agent is trusted with correct score
        assert!(agent.is_ok());
        let agent = agent.unwrap();
        assert_eq!(agent.trust_score, 0.95);
    }

    #[test]
    fn test_trust_score_validation() {
        // Arrange: Create verified agent
        let agent = AgentState::<Unregistered>::new("test-agent".to_string())
            .register(vec![])
            .verify(b"proof");

        // Act: Try invalid trust score
        let result = agent.trust(1.5);

        // Assert: Error returned
        assert!(result.is_err());
    }

    #[test]
    fn test_escalation_path() {
        // Arrange: Create verified agent
        let agent = AgentState::<Unregistered>::new("test-agent".to_string())
            .register(vec![])
            .verify(b"proof");

        // Act: Escalate agent
        let agent = agent.escalate("security concern".to_string());

        // Assert: Trust score reset
        assert_eq!(agent.trust_score, 0.0);
        assert_eq!(agent.get_id(), "test-agent");

        // Act: Resolve escalation
        let agent = agent.resolve();

        // Assert: Returned to verified with lower trust
        assert_eq!(agent.trust_score, 0.3);
    }

    #[test]
    fn test_trust_update() {
        // Arrange: Create trusted agent
        let agent = AgentState::<Unregistered>::new("test-agent".to_string())
            .register(vec![])
            .verify(b"proof")
            .trust(0.8)
            .unwrap();

        // Act: Increase trust
        let agent = agent.update_trust(0.1);

        // Assert: Trust increased
        assert!(agent.is_ok());
        assert_eq!(agent.unwrap().trust_score, 0.9);
    }

    #[test]
    fn test_trust_update_below_threshold() {
        // Arrange: Create trusted agent
        let agent = AgentState::<Unregistered>::new("test-agent".to_string())
            .register(vec![])
            .verify(b"proof")
            .trust(0.6)
            .unwrap();

        // Act: Decrease trust below threshold
        let result = agent.update_trust(-0.2);

        // Assert: Error returned
        assert!(result.is_err());
    }
}
