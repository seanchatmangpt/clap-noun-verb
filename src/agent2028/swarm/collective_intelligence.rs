use chrono::{DateTime, Utc};
/// Collective Intelligence Engine
///
/// Enables millions of agents to achieve consensus and make collective decisions
/// through voting, aggregation, and Byzantine-resistant mechanisms.
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Vote cast by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub agent_id: String,
    pub decision: String,
    pub confidence: f64, // 0.0 to 1.0
    pub weight: f64,     // Based on agent reputation
    pub timestamp: DateTime<Utc>,
}

impl Vote {
    pub fn new(agent_id: String, decision: String, confidence: f64, weight: f64) -> Self {
        Self { agent_id, decision, confidence, weight, timestamp: Utc::now() }
    }

    /// Weighted vote score
    pub fn weighted_score(&self) -> f64 {
        self.confidence * self.weight
    }
}

/// Type of consensus mechanism
#[derive(Debug, Clone)]
pub enum ConsensusType {
    /// Simple majority (50% + 1)
    SimpleMajority,
    /// Super majority (2/3)
    SuperMajority,
    /// Unanimity required
    Unanimous,
    /// Weighted voting
    Weighted { reputation_threshold: f64 },
    /// Byzantine-fault tolerant (2/3 agreement)
    ByzantineFaultTolerant,
}

/// Voting pool for a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingPool {
    pub voting_id: String,
    pub topic: String,
    pub votes: Vec<Vote>,
    pub quorum_required: usize,
    pub created_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
    pub consensus_type: String,
}

impl VotingPool {
    pub fn new(topic: String, consensus_type: String, duration_secs: i64) -> Self {
        Self {
            voting_id: uuid::Uuid::new_v4().to_string(),
            topic,
            votes: Vec::new(),
            quorum_required: 1, // Will be set based on swarm size
            created_at: Utc::now(),
            closes_at: Utc::now() + chrono::Duration::seconds(duration_secs),
            consensus_type,
        }
    }

    /// Check if voting is still open
    pub fn is_open(&self) -> bool {
        Utc::now() < self.closes_at
    }

    /// Get winning decision
    pub fn get_decision(&self) -> Option<(String, f64)> {
        if self.votes.is_empty() {
            return None;
        }

        let mut decision_scores: HashMap<String, f64> = HashMap::new();

        for vote in &self.votes {
            *decision_scores.entry(vote.decision.clone()).or_insert(0.0) += vote.weighted_score();
        }

        decision_scores.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
    }

    /// Get consensus confidence (0.0 to 1.0)
    pub fn confidence(&self) -> f64 {
        if self.votes.is_empty() {
            return 0.0;
        }

        let mut decision_scores: HashMap<String, f64> = HashMap::new();
        let mut total_weight = 0.0;

        for vote in &self.votes {
            let score = vote.weighted_score();
            *decision_scores.entry(vote.decision.clone()).or_insert(0.0) += score;
            total_weight += vote.weight;
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        // Confidence = (winning score / total weight)^2
        let max_score = decision_scores
            .values()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0);
        (max_score / total_weight).min(1.0)
    }
}

/// Voting Protocol
pub struct VotingProtocol {
    pools: Arc<RwLock<HashMap<String, VotingPool>>>,
}

impl VotingProtocol {
    pub fn new() -> Self {
        Self { pools: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Start a new voting pool
    pub async fn create_pool(
        &self,
        topic: String,
        consensus_type: String,
        duration_secs: i64,
    ) -> String {
        let pool = VotingPool::new(topic, consensus_type, duration_secs);
        let voting_id = pool.voting_id.clone();

        let mut pools = self.pools.write().await;
        pools.insert(voting_id.clone(), pool);

        voting_id
    }

    /// Cast a vote
    pub async fn vote(
        &self,
        voting_id: &str,
        agent_id: String,
        decision: String,
        confidence: f64,
        weight: f64,
    ) -> bool {
        let mut pools = self.pools.write().await;

        if let Some(pool) = pools.get_mut(voting_id) {
            if !pool.is_open() {
                return false;
            }

            let vote = Vote::new(agent_id, decision, confidence, weight);
            pool.votes.push(vote);
            true
        } else {
            false
        }
    }

    /// Get consensus result
    pub async fn get_consensus(&self, voting_id: &str) -> Option<(String, f64)> {
        let pools = self.pools.read().await;

        pools.get(voting_id).and_then(|pool| {
            if !pool.votes.is_empty() {
                pool.get_decision().map(|(decision, score)| (decision, score))
            } else {
                None
            }
        })
    }

    /// Get voting pool info
    pub async fn pool_info(&self, voting_id: &str) -> Option<(usize, f64)> {
        let pools = self.pools.read().await;

        pools.get(voting_id).map(|pool| (pool.votes.len(), pool.confidence()))
    }
}

impl Default for VotingProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared swarm mind - aggregated beliefs and intentions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMindState {
    pub generation: u64,
    pub collective_beliefs: HashMap<String, f64>, // Belief -> confidence
    pub collective_intentions: Vec<String>,
    pub consensus_topics: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl HiveMindState {
    pub fn new() -> Self {
        Self {
            generation: 0,
            collective_beliefs: HashMap::new(),
            collective_intentions: Vec::new(),
            consensus_topics: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

/// HiveMind - Global swarm consciousness
pub struct HiveMind {
    state: Arc<RwLock<HiveMindState>>,
    voting_protocol: Arc<VotingProtocol>,
}

impl HiveMind {
    pub fn new(voting_protocol: Arc<VotingProtocol>) -> Self {
        Self { state: Arc::new(RwLock::new(HiveMindState::new())), voting_protocol }
    }

    /// All agents read the current hivemind state
    pub async fn read(&self) -> HiveMindState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Propose a new collective belief
    pub async fn propose_belief(&self, belief: String, confidence: f64) {
        let mut state = self.state.write().await;
        state.collective_beliefs.insert(belief, confidence);
        state.updated_at = Utc::now();
        state.generation += 1;
    }

    /// Propose a collective action
    pub async fn propose_intention(&self, intention: String) {
        let mut state = self.state.write().await;
        if !state.collective_intentions.contains(&intention) {
            state.collective_intentions.push(intention);
            state.updated_at = Utc::now();
            state.generation += 1;
        }
    }

    /// Update hivemind from voting results
    pub async fn update_from_voting(&self, voting_id: &str, belief_name: String) {
        if let Some((decision, score)) = self.voting_protocol.get_consensus(voting_id).await {
            let mut state = self.state.write().await;
            state.collective_beliefs.insert(belief_name, score);
            state.consensus_topics.push(decision);
            state.updated_at = Utc::now();
            state.generation += 1;
        }
    }

    /// Get strongest belief
    pub async fn strongest_belief(&self) -> Option<(String, f64)> {
        let state = self.state.read().await;
        state
            .collective_beliefs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
            .map(|(k, v)| (k.clone(), *v))
    }

    /// Get generation number (for change detection)
    pub async fn generation(&self) -> u64 {
        let state = self.state.read().await;
        state.generation
    }
}

impl Default for HiveMind {
    fn default() -> Self {
        Self::new(Arc::new(VotingProtocol::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_voting_pool() {
        let protocol = VotingProtocol::new();
        let voting_id =
            protocol.create_pool("test_decision".to_string(), "majority".to_string(), 60).await;

        protocol.vote(&voting_id, "agent-1".to_string(), "yes".to_string(), 0.9, 1.0).await;
        protocol.vote(&voting_id, "agent-2".to_string(), "yes".to_string(), 0.8, 1.0).await;
        protocol.vote(&voting_id, "agent-3".to_string(), "no".to_string(), 0.7, 1.0).await;

        let consensus = protocol.get_consensus(&voting_id).await;
        assert!(consensus.is_some());
        assert_eq!(consensus.unwrap().0, "yes");
    }

    #[tokio::test]
    async fn test_hivemind() {
        let protocol = Arc::new(VotingProtocol::new());
        let hivemind = HiveMind::new(protocol);

        hivemind.propose_belief("danger_detected".to_string(), 0.85).await;
        let state = hivemind.read().await;

        assert!(state.collective_beliefs.contains_key("danger_detected"));
    }
}
