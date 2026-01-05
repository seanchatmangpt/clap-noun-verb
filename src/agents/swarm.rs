//! Swarm coordination with Byzantine fault tolerance
//!
//! This module implements distributed swarm coordination with:
//! - Agent registry with fitness scoring
//! - Task auction mechanism
//! - Gossip protocol for information dissemination
//! - Byzantine fault detection
//! - Bayesian trust scoring
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::agents::swarm::*;
//!
//! let mut coordinator = SwarmCoordinator::new();
//!
//! // Register agents
//! coordinator.register_agent(AgentInfo::new("agent-001", vec!["nlp"]));
//! coordinator.register_agent(AgentInfo::new("agent-002", vec!["vision"]));
//!
//! // Create task auction
//! let auction = TaskAuction::new("task-001", vec!["nlp"], 1.0);
//! let winner = coordinator.run_auction(auction)?;
//! ```

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::error::{NounVerbError, Result};

// =============================================================================
// Agent Info - Agent metadata for swarm
// =============================================================================

/// Agent information for swarm coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent unique identifier
    pub id: String,

    /// Agent capabilities
    pub capabilities: Vec<String>,

    /// Trust score (0.0 - 1.0)
    pub trust_score: f64,

    /// Number of successful tasks
    pub success_count: u64,

    /// Number of failed tasks
    pub failure_count: u64,

    /// Last seen timestamp
    pub last_seen: Option<SystemTime>,

    /// Byzantine fault score (higher = more suspicious)
    pub byzantine_score: f64,
}

impl AgentInfo {
    /// Create new agent info
    pub fn new(id: impl Into<String>, capabilities: Vec<impl Into<String>>) -> Self {
        Self {
            id: id.into(),
            capabilities: capabilities.into_iter().map(|c| c.into()).collect(),
            trust_score: 0.5, // Neutral initial trust
            success_count: 0,
            failure_count: 0,
            last_seen: Some(SystemTime::now()),
            byzantine_score: 0.0,
        }
    }

    /// Calculate fitness score for task
    ///
    /// Combines capability match, trust score, and success rate
    ///
    /// # Arguments
    ///
    /// * `required_capabilities` - Required capabilities
    pub fn fitness_score(&self, required_capabilities: &[String]) -> f64 {
        // Capability match score
        let matches =
            required_capabilities.iter().filter(|c| self.capabilities.contains(c)).count();

        let capability_score = if required_capabilities.is_empty() {
            1.0
        } else {
            matches as f64 / required_capabilities.len() as f64
        };

        // Success rate
        let total = self.success_count + self.failure_count;
        let success_rate = if total == 0 { 0.5 } else { self.success_count as f64 / total as f64 };

        // Combined fitness: 40% capability + 30% trust + 30% success rate
        capability_score * 0.4 + self.trust_score * 0.3 + success_rate * 0.3
    }

    /// Update success metrics
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.last_seen = Some(SystemTime::now());

        // Bayesian trust update (success)
        self.update_trust_bayesian(true);
    }

    /// Update failure metrics
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_seen = Some(SystemTime::now());

        // Bayesian trust update (failure)
        self.update_trust_bayesian(false);
    }

    /// Bayesian trust score update
    ///
    /// Uses Beta distribution for Bayesian inference
    ///
    /// # Arguments
    ///
    /// * `_success` - Whether task succeeded (used for future Bayesian update)
    fn update_trust_bayesian(&mut self, _success: bool) {
        // Beta distribution parameters (alpha, beta)
        let alpha = self.success_count as f64 + 1.0;
        let beta = self.failure_count as f64 + 1.0;

        // Expected value of Beta distribution
        self.trust_score = alpha / (alpha + beta);

        // Adjust for Byzantine behavior
        if self.byzantine_score > 0.5 {
            self.trust_score *= 1.0 - self.byzantine_score;
        }
    }
}

// =============================================================================
// Task Auction - Auction-based task allocation
// =============================================================================

/// Task auction for distributed task allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAuction {
    /// Task identifier
    pub task_id: String,

    /// Required capabilities
    pub required_capabilities: Vec<String>,

    /// Task complexity (0.0 - 1.0)
    pub complexity: f64,

    /// Minimum bid score required
    pub min_bid: f64,
}

impl TaskAuction {
    /// Create new task auction
    pub fn new(
        task_id: impl Into<String>,
        required_capabilities: Vec<impl Into<String>>,
        complexity: f64,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            required_capabilities: required_capabilities.into_iter().map(|c| c.into()).collect(),
            complexity,
            min_bid: 0.5, // Default minimum bid
        }
    }

    /// Set minimum bid threshold
    pub fn with_min_bid(mut self, min_bid: f64) -> Self {
        self.min_bid = min_bid;
        self
    }
}

// =============================================================================
// Gossip Protocol - Information dissemination
// =============================================================================

/// Gossip message for swarm communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    /// Message ID
    pub id: String,

    /// Source agent ID
    pub source: String,

    /// Message payload
    pub payload: String,

    /// Timestamp
    pub timestamp: SystemTime,

    /// TTL (time-to-live) for gossip propagation
    pub ttl: u32,
}

/// Gossip protocol for information dissemination
#[derive(Debug, Clone)]
pub struct GossipProtocol {
    /// Received messages (dedupliation)
    seen_messages: HashMap<String, SystemTime>,

    /// Maximum age for message retention
    max_age: Duration,
}

impl GossipProtocol {
    /// Create new gossip protocol
    pub fn new() -> Self {
        Self {
            seen_messages: HashMap::new(),
            max_age: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Check if message has been seen
    ///
    /// # Arguments
    ///
    /// * `message_id` - Message identifier
    pub fn has_seen(&self, message_id: &str) -> bool {
        self.seen_messages.contains_key(message_id)
    }

    /// Record message as seen
    ///
    /// # Arguments
    ///
    /// * `message` - Gossip message
    pub fn record_seen(&mut self, message: &GossipMessage) {
        self.seen_messages.insert(message.id.clone(), message.timestamp);
    }

    /// Should propagate message
    ///
    /// # Arguments
    ///
    /// * `message` - Gossip message
    pub fn should_propagate(&self, message: &GossipMessage) -> bool {
        message.ttl > 0 && !self.has_seen(&message.id)
    }

    /// Cleanup old messages
    pub fn cleanup(&mut self) {
        let now = SystemTime::now();
        self.seen_messages.retain(|_, timestamp| {
            now.duration_since(*timestamp).map(|d| d < self.max_age).unwrap_or(false)
        });
    }
}

impl Default for GossipProtocol {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Byzantine Detector - Detect malicious agents
// =============================================================================

/// Byzantine fault detector
#[derive(Debug, Clone)]
pub struct ByzantineDetector {
    /// Voting history for agents
    voting_history: HashMap<String, Vec<bool>>,

    /// Threshold for Byzantine detection (0.0 - 1.0)
    threshold: f64,
}

impl ByzantineDetector {
    /// Create new Byzantine detector
    pub fn new() -> Self {
        Self {
            voting_history: HashMap::new(),
            threshold: 0.7, // 70% disagreement threshold
        }
    }

    /// Record vote from agent
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `vote` - Vote value (true/false)
    pub fn record_vote(&mut self, agent_id: &str, vote: bool) {
        self.voting_history.entry(agent_id.to_string()).or_insert_with(Vec::new).push(vote);
    }

    /// Detect if agent is Byzantine (malicious)
    ///
    /// Uses statistical analysis of voting patterns
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `consensus` - Consensus vote
    pub fn is_byzantine(&self, agent_id: &str, consensus: bool) -> bool {
        if let Some(votes) = self.voting_history.get(agent_id) {
            if votes.len() < 10 {
                return false; // Not enough data
            }

            // Calculate disagreement rate
            let disagreements = votes.iter().filter(|&&v| v != consensus).count();
            let disagreement_rate = disagreements as f64 / votes.len() as f64;

            disagreement_rate > self.threshold
        } else {
            false
        }
    }

    /// Calculate Byzantine score for agent
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `consensus` - Consensus vote
    pub fn byzantine_score(&self, agent_id: &str, consensus: bool) -> f64 {
        if let Some(votes) = self.voting_history.get(agent_id) {
            if votes.is_empty() {
                return 0.0;
            }

            let disagreements = votes.iter().filter(|&&v| v != consensus).count();
            disagreements as f64 / votes.len() as f64
        } else {
            0.0
        }
    }
}

impl Default for ByzantineDetector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Trust Score - Bayesian trust scoring
// =============================================================================

/// Trust score calculator using Bayesian inference
#[derive(Debug, Clone)]
pub struct TrustScore {
    /// Prior alpha (successes)
    alpha: f64,

    /// Prior beta (failures)
    beta: f64,
}

impl TrustScore {
    /// Create new trust score calculator
    ///
    /// # Arguments
    ///
    /// * `prior_alpha` - Prior successes (default 1.0)
    /// * `prior_beta` - Prior failures (default 1.0)
    pub fn new(prior_alpha: f64, prior_beta: f64) -> Self {
        Self { alpha: prior_alpha, beta: prior_beta }
    }

    /// Calculate trust score
    ///
    /// # Arguments
    ///
    /// * `successes` - Number of successes
    /// * `failures` - Number of failures
    pub fn calculate(&self, successes: u64, failures: u64) -> f64 {
        let alpha = self.alpha + successes as f64;
        let beta = self.beta + failures as f64;

        alpha / (alpha + beta)
    }

    /// Calculate confidence interval width
    ///
    /// Smaller width = more confidence
    ///
    /// # Arguments
    ///
    /// * `successes` - Number of successes
    /// * `failures` - Number of failures
    pub fn confidence_width(&self, successes: u64, failures: u64) -> f64 {
        let alpha = self.alpha + successes as f64;
        let beta = self.beta + failures as f64;
        let n = alpha + beta;

        // Wilson score interval width (approximate)
        1.96 * ((alpha * beta) / (n * n * n)).sqrt()
    }
}

impl Default for TrustScore {
    fn default() -> Self {
        Self::new(1.0, 1.0)
    }
}

// =============================================================================
// Agent Registry - Centralized agent tracking
// =============================================================================

/// Agent registry for swarm coordination
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    /// Registered agents
    agents: HashMap<String, AgentInfo>,

    /// Trust score calculator
    trust_calculator: TrustScore,
}

impl AgentRegistry {
    /// Create new agent registry
    pub fn new() -> Self {
        Self { agents: HashMap::new(), trust_calculator: TrustScore::default() }
    }

    /// Register agent
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent info
    pub fn register(&mut self, agent: AgentInfo) {
        self.agents.insert(agent.id.clone(), agent);
    }

    /// Get agent info
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentInfo> {
        self.agents.get(agent_id)
    }

    /// Get mutable agent info
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    pub fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut AgentInfo> {
        self.agents.get_mut(agent_id)
    }

    /// Get all agents matching capabilities
    ///
    /// # Arguments
    ///
    /// * `required_capabilities` - Required capabilities
    pub fn find_agents(&self, required_capabilities: &[String]) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|agent| {
                required_capabilities.iter().all(|cap| agent.capabilities.contains(cap))
            })
            .collect()
    }

    /// Get agent count
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Swarm Coordinator - Main coordination engine
// =============================================================================

/// Swarm coordinator for distributed task allocation
#[derive(Debug, Clone)]
pub struct SwarmCoordinator {
    /// Agent registry
    registry: AgentRegistry,

    /// Gossip protocol
    gossip: GossipProtocol,

    /// Byzantine detector
    byzantine_detector: ByzantineDetector,
}

impl SwarmCoordinator {
    /// Create new swarm coordinator
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
            gossip: GossipProtocol::new(),
            byzantine_detector: ByzantineDetector::new(),
        }
    }

    /// Register agent with swarm
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent info
    pub fn register_agent(&mut self, agent: AgentInfo) {
        self.registry.register(agent);
    }

    /// Run task auction
    ///
    /// # Arguments
    ///
    /// * `auction` - Task auction
    ///
    /// # Returns
    ///
    /// Winner agent ID
    pub fn run_auction(&self, auction: TaskAuction) -> Result<String> {
        // Find eligible agents
        let candidates = self.registry.find_agents(&auction.required_capabilities);

        if candidates.is_empty() {
            return Err(NounVerbError::ExecutionError {
                message: "no agents match required capabilities".to_string(),
            });
        }

        // Calculate bids (fitness scores)
        let mut bids: Vec<(&AgentInfo, f64)> = candidates
            .into_iter()
            .map(|agent| {
                let fitness = agent.fitness_score(&auction.required_capabilities);
                (agent, fitness)
            })
            .collect();

        // Sort by fitness (descending)
        bids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select winner (highest fitness above minimum bid)
        if let Some((winner, score)) = bids.first() {
            if *score >= auction.min_bid {
                Ok(winner.id.clone())
            } else {
                Err(NounVerbError::ExecutionError {
                    message: "no agents meet minimum bid threshold".to_string(),
                })
            }
        } else {
            Err(NounVerbError::ExecutionError {
                message: "auction failed - no valid bids".to_string(),
            })
        }
    }

    /// Record task result
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `success` - Whether task succeeded
    pub fn record_task_result(&mut self, agent_id: &str, success: bool) -> Result<()> {
        if let Some(agent) = self.registry.get_agent_mut(agent_id) {
            if success {
                agent.record_success();
            } else {
                agent.record_failure();
            }
            Ok(())
        } else {
            Err(NounVerbError::ExecutionError { message: format!("agent not found: {}", agent_id) })
        }
    }

    /// Propagate gossip message
    ///
    /// # Arguments
    ///
    /// * `message` - Gossip message
    pub fn propagate_gossip(&mut self, message: GossipMessage) -> bool {
        if self.gossip.should_propagate(&message) {
            self.gossip.record_seen(&message);
            true
        } else {
            false
        }
    }

    /// Check Byzantine status
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `consensus` - Consensus value
    pub fn check_byzantine(&mut self, agent_id: &str, consensus: bool) -> bool {
        let is_byzantine = self.byzantine_detector.is_byzantine(agent_id, consensus);

        if is_byzantine {
            if let Some(agent) = self.registry.get_agent_mut(agent_id) {
                agent.byzantine_score =
                    self.byzantine_detector.byzantine_score(agent_id, consensus);
            }
        }

        is_byzantine
    }

    /// Get swarm health metrics
    pub fn health_metrics(&self) -> SwarmHealthMetrics {
        let total_agents = self.registry.agent_count();
        let trusted_agents = self.registry.agents.values().filter(|a| a.trust_score >= 0.7).count();

        let avg_trust = if total_agents > 0 {
            self.registry.agents.values().map(|a| a.trust_score).sum::<f64>() / total_agents as f64
        } else {
            0.0
        };

        SwarmHealthMetrics { total_agents, trusted_agents, average_trust: avg_trust }
    }
}

impl Default for SwarmCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Swarm health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmHealthMetrics {
    /// Total number of agents
    pub total_agents: usize,

    /// Number of trusted agents (trust >= 0.7)
    pub trusted_agents: usize,

    /// Average trust score
    pub average_trust: f64,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_fitness() {
        // Arrange
        let mut agent = AgentInfo::new("agent-001", vec!["nlp", "vision"]);
        agent.success_count = 8;
        agent.failure_count = 2;
        agent.trust_score = 0.8;

        // Act
        let fitness = agent.fitness_score(&["nlp".to_string()]);

        // Assert
        // capability: 1.0 (matches), trust: 0.8, success_rate: 0.8
        // fitness = 1.0 * 0.4 + 0.8 * 0.3 + 0.8 * 0.3 = 0.88
        assert!((fitness - 0.88).abs() < 0.01);
    }

    #[test]
    fn test_bayesian_trust_update() {
        // Arrange
        let mut agent = AgentInfo::new("agent-001", vec!["nlp"]);

        // Act: Record successes
        agent.record_success();
        agent.record_success();

        // Assert: Trust should increase
        assert!(agent.trust_score > 0.5);
        assert_eq!(agent.success_count, 2);
    }

    #[test]
    fn test_task_auction() {
        // Arrange
        let auction = TaskAuction::new("task-001", vec!["nlp"], 1.0).with_min_bid(0.6);

        // Assert
        assert_eq!(auction.task_id, "task-001");
        assert_eq!(auction.min_bid, 0.6);
    }

    #[test]
    fn test_gossip_deduplication() {
        // Arrange
        let mut gossip = GossipProtocol::new();
        let message = GossipMessage {
            id: "msg-001".to_string(),
            source: "agent-001".to_string(),
            payload: "test".to_string(),
            timestamp: SystemTime::now(),
            ttl: 5,
        };

        // Act: First propagation
        assert!(gossip.should_propagate(&message));
        gossip.record_seen(&message);

        // Assert: Second propagation should be blocked
        assert!(!gossip.should_propagate(&message));
    }

    #[test]
    fn test_byzantine_detection() {
        // Arrange
        let mut detector = ByzantineDetector::new();

        // Act: Record consistent disagreement
        for _ in 0..15 {
            detector.record_vote("agent-001", false);
        }

        // Assert: Agent should be detected as Byzantine (consensus is true)
        assert!(detector.is_byzantine("agent-001", true));
    }

    #[test]
    fn test_trust_score_calculation() {
        // Arrange
        let calculator = TrustScore::default();

        // Act: Calculate trust with 8 successes, 2 failures
        let trust = calculator.calculate(8, 2);

        // Assert: (1 + 8) / (1 + 8 + 1 + 2) = 9/12 = 0.75
        assert_eq!(trust, 0.75);
    }

    #[test]
    fn test_agent_registry() {
        // Arrange
        let mut registry = AgentRegistry::new();
        let agent1 = AgentInfo::new("agent-001", vec!["nlp"]);
        let agent2 = AgentInfo::new("agent-002", vec!["vision"]);

        // Act
        registry.register(agent1);
        registry.register(agent2);

        // Assert
        assert_eq!(registry.agent_count(), 2);
        let found = registry.find_agents(&["nlp".to_string()]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "agent-001");
    }

    #[test]
    fn test_swarm_coordinator_auction() {
        // Arrange
        let mut coordinator = SwarmCoordinator::new();
        let mut agent1 = AgentInfo::new("agent-001", vec!["nlp"]);
        agent1.trust_score = 0.9;
        agent1.success_count = 10;

        let mut agent2 = AgentInfo::new("agent-002", vec!["nlp"]);
        agent2.trust_score = 0.6;
        agent2.success_count = 5;

        coordinator.register_agent(agent1);
        coordinator.register_agent(agent2);

        let auction = TaskAuction::new("task-001", vec!["nlp"], 1.0);

        // Act
        let winner = coordinator.run_auction(auction);

        // Assert: agent-001 should win (higher trust and success)
        assert!(winner.is_ok());
        assert_eq!(winner.unwrap(), "agent-001");
    }

    #[test]
    fn test_swarm_health_metrics() {
        // Arrange
        let mut coordinator = SwarmCoordinator::new();
        let mut agent1 = AgentInfo::new("agent-001", vec!["nlp"]);
        agent1.trust_score = 0.8;
        let mut agent2 = AgentInfo::new("agent-002", vec!["vision"]);
        agent2.trust_score = 0.6;

        coordinator.register_agent(agent1);
        coordinator.register_agent(agent2);

        // Act
        let health = coordinator.health_metrics();

        // Assert
        assert_eq!(health.total_agents, 2);
        assert_eq!(health.trusted_agents, 1); // Only agent-001 >= 0.7
        assert_eq!(health.average_trust, 0.7); // (0.8 + 0.6) / 2
    }
}
