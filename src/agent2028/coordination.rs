use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
/// Distributed Agent Coordination System
///
/// Enables safe coordination of millions of agents across distributed systems
/// with service discovery, intelligent routing, and consensus mechanisms.
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a single agent in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub address: SocketAddr,
    pub capabilities: Vec<String>,
    pub health_score: f64, // 0.0 to 1.0
    pub latency_ms: f64,
    pub reliability: f64, // Success rate
    pub last_seen: DateTime<Utc>,
    pub max_concurrency: usize,
    pub current_load: usize,
}

impl Agent {
    /// Calculate fitness score for this agent for a given capability
    pub fn fitness_score(
        &self,
        capability: &str,
        latency_weight: f64,
        reliability_weight: f64,
    ) -> f64 {
        if !self.capabilities.contains(&capability.to_string()) {
            return 0.0;
        }

        let health_factor = self.health_score;
        let latency_factor = 1.0 / (1.0 + self.latency_ms / 100.0); // Normalize latency
        let reliability_factor = self.reliability;
        let capacity_factor = 1.0 - (self.current_load as f64 / self.max_concurrency as f64);

        health_factor * 0.3
            + latency_factor * latency_weight * 0.3
            + reliability_factor * reliability_weight * 0.2
            + capacity_factor * 0.2
    }
}

/// Agent Registry - Service discovery and health tracking
#[derive(Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self { agents: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Register an agent
    pub async fn register(&self, agent: Agent) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id.clone(), agent);
    }

    /// Unregister an agent
    pub async fn unregister(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
    }

    /// Find agents with a specific capability
    pub async fn find_by_capability(&self, capability: &str) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    /// Get all agents
    pub async fn all(&self) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Update agent health score
    pub async fn update_health(&self, agent_id: &str, health_score: f64) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.health_score = health_score.min(1.0).max(0.0);
            agent.last_seen = Utc::now();
        }
    }

    /// Update agent metrics
    pub async fn update_metrics(&self, agent_id: &str, latency_ms: f64, success: bool) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.latency_ms = latency_ms;
            // Update running average of reliability
            agent.reliability = (agent.reliability * 0.9) + (if success { 1.0 } else { 0.0 }) * 0.1;
            agent.last_seen = Utc::now();
        }
    }

    /// Update agent load
    pub async fn update_load(&self, agent_id: &str, current_load: usize) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.current_load = current_load;
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Routing strategy for command broker
#[derive(Debug, Clone, Copy)]
pub enum RoutingStrategy {
    /// Route to agent with lowest latency
    MinLatency,
    /// Route to agent with highest reliability
    MaxReliability,
    /// Route to agent with best overall fitness
    BestFit,
    /// Round-robin distribution
    RoundRobin,
    /// Route to least loaded agent
    LeastLoaded,
}

/// Command Broker - Intelligent routing of commands to agents
pub struct CommandBroker {
    registry: AgentRegistry,
    strategy: RoutingStrategy,
    routing_history: Arc<RwLock<Vec<(String, String, DateTime<Utc>)>>>, // (agent_id, capability, timestamp)
}

impl CommandBroker {
    pub fn new(registry: AgentRegistry, strategy: RoutingStrategy) -> Self {
        Self { registry, strategy, routing_history: Arc::new(RwLock::new(Vec::new())) }
    }

    /// Route a command to the best agent for the given capability
    pub async fn route(&self, capability: &str) -> Option<Agent> {
        let agents = self.registry.find_by_capability(capability).await;

        if agents.is_empty() {
            return None;
        }

        let selected = match self.strategy {
            RoutingStrategy::MinLatency => agents
                .iter()
                .min_by(|a, b| a.latency_ms.partial_cmp(&b.latency_ms).unwrap_or(Ordering::Equal))
                .cloned(),
            RoutingStrategy::MaxReliability => agents
                .iter()
                .max_by(|a, b| a.reliability.partial_cmp(&b.reliability).unwrap_or(Ordering::Equal))
                .cloned(),
            RoutingStrategy::BestFit => agents
                .iter()
                .max_by(|a, b| {
                    a.fitness_score(capability, 0.5, 0.5)
                        .partial_cmp(&b.fitness_score(capability, 0.5, 0.5))
                        .unwrap_or(Ordering::Equal)
                })
                .cloned(),
            RoutingStrategy::LeastLoaded => agents.iter().min_by_key(|a| a.current_load).cloned(),
            RoutingStrategy::RoundRobin => {
                // Simple round-robin: use history length to determine next
                let history = self.routing_history.read().await;
                let idx = history.len() % agents.len();
                agents.get(idx).cloned()
            }
        };

        if let Some(agent) = selected.as_ref() {
            let mut history = self.routing_history.write().await;
            history.push((agent.id.clone(), capability.to_string(), Utc::now()));
        }

        selected
    }

    /// Get routing statistics
    pub async fn stats(&self) -> HashMap<String, usize> {
        let history = self.routing_history.read().await;
        let mut stats = HashMap::new();
        for (agent_id, _, _) in history.iter() {
            *stats.entry(agent_id.clone()).or_insert(0) += 1;
        }
        stats
    }
}

/// Consensus mechanism for critical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProposal {
    pub id: String,
    pub operation: String,
    pub proposer_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Consensus types
#[derive(Debug, Clone)]
pub enum ConsensusType {
    /// Simple majority voting
    Majority,
    /// Byzantine Fault Tolerant (requires 2f+1 where f is max faulty nodes)
    Byzantine { min_votes: usize },
    /// Unanimous agreement required
    Unanimous,
}

/// Consensus Engine for multi-agent agreement
pub struct ConsensusEngine {
    votes: Arc<RwLock<HashMap<String, Vec<String>>>>, // proposal_id -> agent_ids
    consensus_type: ConsensusType,
}

impl ConsensusEngine {
    pub fn new(consensus_type: ConsensusType) -> Self {
        Self { votes: Arc::new(RwLock::new(HashMap::new())), consensus_type }
    }

    /// Submit a vote for a proposal
    pub async fn vote(&self, proposal_id: &str, agent_id: String) {
        let mut votes = self.votes.write().await;
        votes.entry(proposal_id.to_string()).or_insert_with(Vec::new).push(agent_id);
    }

    /// Check if consensus has been reached
    pub async fn has_consensus(&self, proposal_id: &str, total_agents: usize) -> bool {
        let votes = self.votes.read().await;
        let vote_count = votes.get(proposal_id).map(|v| v.len()).unwrap_or(0);

        match self.consensus_type {
            ConsensusType::Majority => vote_count > total_agents / 2,
            ConsensusType::Byzantine { min_votes } => vote_count >= min_votes,
            ConsensusType::Unanimous => vote_count == total_agents,
        }
    }

    /// Get vote count for a proposal
    pub async fn vote_count(&self, proposal_id: &str) -> usize {
        let votes = self.votes.read().await;
        votes.get(proposal_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Clear votes for a proposal
    pub async fn clear(&self, proposal_id: &str) {
        let mut votes = self.votes.write().await;
        votes.remove(proposal_id);
    }
}

/// Execution Receipt for distributed audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub command_id: String,
    pub agent_id: String,
    pub capability: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Distributed Session for multi-agent conversations
#[derive(Clone)]
pub struct DistributedSession {
    pub session_id: String,
    receipts: Arc<RwLock<Vec<ExecutionReceipt>>>,
}

impl DistributedSession {
    pub fn new() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            receipts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record execution receipt
    pub async fn record(&self, receipt: ExecutionReceipt) {
        let mut receipts = self.receipts.write().await;
        receipts.push(receipt);
    }

    /// Get all receipts for this session
    pub async fn history(&self) -> Vec<ExecutionReceipt> {
        let receipts = self.receipts.read().await;
        receipts.clone()
    }
}

impl Default for DistributedSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_registry() {
        let registry = AgentRegistry::new();
        let agent = Agent {
            id: "agent-1".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            capabilities: vec!["database.query".to_string()],
            health_score: 0.95,
            latency_ms: 10.0,
            reliability: 0.99,
            last_seen: Utc::now(),
            max_concurrency: 100,
            current_load: 5,
        };

        registry.register(agent).await;
        let found = registry.find_by_capability("database.query").await;
        assert_eq!(found.len(), 1);
    }

    #[tokio::test]
    async fn test_command_broker() {
        let registry = AgentRegistry::new();
        let broker = CommandBroker::new(registry.clone(), RoutingStrategy::BestFit);

        let agent = Agent {
            id: "agent-1".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            capabilities: vec!["compute".to_string()],
            health_score: 0.95,
            latency_ms: 5.0,
            reliability: 0.99,
            last_seen: Utc::now(),
            max_concurrency: 100,
            current_load: 10,
        };

        registry.register(agent).await;
        let routed = broker.route("compute").await;
        assert!(routed.is_some());
    }

    #[tokio::test]
    async fn test_consensus_engine() {
        let engine = ConsensusEngine::new(ConsensusType::Majority);
        let proposal_id = "prop-1";

        engine.vote(proposal_id, "agent-1".to_string()).await;
        engine.vote(proposal_id, "agent-2".to_string()).await;

        assert!(engine.has_consensus(proposal_id, 3).await);
    }
}
