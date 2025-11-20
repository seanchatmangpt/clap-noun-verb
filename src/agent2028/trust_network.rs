use chrono::{DateTime, Duration, Utc};
/// Agent Trust Network System
///
/// Decentralized trust and reputation management for agents without requiring
/// a central authority. Uses Bayesian reputation models and transitive trust.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Decentralized Identifier for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentIdentity {
    /// Unique identifier (DID format: did:agent:agent-id)
    pub did: String,
    /// Public key material for verification
    pub public_key: Vec<u8>,
    /// Key type (e.g., "ed25519", "quantum-safe")
    pub key_type: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Is identity currently active
    pub active: bool,
}

impl AgentIdentity {
    pub fn new(agent_id: String, key_type: String) -> Self {
        let did = format!("did:agent:{}", agent_id);
        let public_key = Self::generate_placeholder_key(&agent_id);

        Self { did, public_key, key_type, created_at: Utc::now(), active: true }
    }

    fn generate_placeholder_key(agent_id: &str) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(agent_id);
        hasher.finalize().to_vec()
    }

    /// Deactivate this identity
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Observation of agent behavior for reputation calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub observer_id: String,
    pub subject_id: String,
    pub outcome: ExecutionOutcome,
    pub timestamp: DateTime<Utc>,
    pub weight: f64, // Recency and importance weight
}

/// Execution outcome affecting trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    Success { duration_ms: u64 },
    Timeout { expected_ms: u64 },
    PartialFailure { error_rate: f64 },
    CompleteFailure,
}

impl ExecutionOutcome {
    /// Convert outcome to reputation delta (-1.0 to 1.0)
    pub fn to_delta(&self) -> f64 {
        match self {
            ExecutionOutcome::Success { .. } => 0.8,
            ExecutionOutcome::Timeout { .. } => -0.3,
            ExecutionOutcome::PartialFailure { error_rate } => -0.5 * error_rate,
            ExecutionOutcome::CompleteFailure => -1.0,
        }
    }
}

/// Trust score for an agent (Bayesian model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub subject_id: String,
    /// Score from 0.0 (completely untrustworthy) to 1.0 (fully trusted)
    pub score: f64,
    /// Confidence in the score (increases with observations)
    pub confidence: f64,
    /// Number of observations used to calculate score
    pub sample_size: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl TrustScore {
    pub fn new(subject_id: String) -> Self {
        Self {
            subject_id,
            score: 0.5, // Start neutral
            confidence: 0.0,
            sample_size: 0,
            last_updated: Utc::now(),
        }
    }

    /// Get score with confidence interval (lower bound)
    pub fn conservative_score(&self) -> f64 {
        let z_score = 1.96; // 95% confidence interval
        let margin =
            z_score * ((self.score * (1.0 - self.score)) / (self.sample_size as f64 + 1.0).sqrt());
        (self.score - margin).max(0.0).min(1.0)
    }

    /// Update score with new observation
    pub fn update(&mut self, outcome: &ExecutionOutcome) {
        let delta = outcome.to_delta();
        let alpha = 0.15; // Learning rate

        // Bayesian update
        self.score = self.score * (1.0 - alpha) + (0.5 + delta / 2.0) * alpha;
        self.score = self.score.max(0.0).min(1.0);

        // Increase sample size and confidence
        self.sample_size += 1;
        self.confidence = ((self.sample_size as f64) / (self.sample_size as f64 + 10.0)).min(1.0);

        self.last_updated = Utc::now();
    }

    /// Decay score over time (forget old observations)
    pub fn decay_old(&mut self, max_age_days: i64) {
        let age = (Utc::now() - self.last_updated).num_days();
        if age > max_age_days {
            // Decay towards neutral (0.5)
            let decay_factor = 0.01 * (age - max_age_days) as f64;
            self.score = self.score * (1.0 - decay_factor) + 0.5 * decay_factor;
            self.confidence *= 0.95; // Reduce confidence in old score
        }
    }
}

/// Trust Score Calculator
pub struct TrustScoreCalculator {
    scores: Arc<RwLock<HashMap<String, TrustScore>>>,
    observations: Arc<RwLock<Vec<Observation>>>,
}

impl TrustScoreCalculator {
    pub fn new() -> Self {
        Self {
            scores: Arc::new(RwLock::new(HashMap::new())),
            observations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record an observation of behavior
    pub async fn observe(
        &self,
        observer_id: String,
        subject_id: String,
        outcome: ExecutionOutcome,
    ) {
        let observation = Observation {
            observer_id,
            subject_id: subject_id.clone(),
            outcome: outcome.clone(),
            timestamp: Utc::now(),
            weight: 1.0,
        };

        // Update trust score
        let mut scores = self.scores.write().await;
        let subject_id_clone = subject_id.clone();
        let score =
            scores.entry(subject_id).or_insert_with(|| TrustScore::new(subject_id_clone.clone()));
        score.update(&outcome);

        // Record observation
        let mut observations = self.observations.write().await;
        observations.push(observation);
    }

    /// Get trust score for an agent
    pub async fn score(&self, agent_id: &str) -> TrustScore {
        let scores = self.scores.read().await;
        scores.get(agent_id).cloned().unwrap_or_else(|| TrustScore::new(agent_id.to_string()))
    }

    /// Get conservative (lower bound) trust score
    pub async fn conservative_score(&self, agent_id: &str) -> f64 {
        self.score(agent_id).await.conservative_score()
    }

    /// Get recent observations for an agent
    pub async fn recent_observations(&self, agent_id: &str, hours: i64) -> Vec<Observation> {
        let observations = self.observations.read().await;
        let cutoff = Utc::now() - Duration::hours(hours);

        observations
            .iter()
            .filter(|o| o.subject_id == agent_id && o.timestamp > cutoff)
            .cloned()
            .collect()
    }
}

impl Default for TrustScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Transitive trust relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChainLink {
    pub from_agent: String,
    pub to_agent: String,
    pub trust_level: f64,
    pub transitive_depth: usize,
    pub timestamp: DateTime<Utc>,
}

/// Transitive Trust Graph
pub struct TrustChain {
    links: Arc<RwLock<Vec<TrustChainLink>>>,
}

impl TrustChain {
    pub fn new() -> Self {
        Self { links: Arc::new(RwLock::new(Vec::new())) }
    }

    /// Add a direct trust relationship
    pub async fn add_link(&self, from: String, to: String, trust_level: f64) {
        let link = TrustChainLink {
            from_agent: from,
            to_agent: to,
            trust_level,
            transitive_depth: 0,
            timestamp: Utc::now(),
        };

        let mut links = self.links.write().await;
        links.push(link);
    }

    /// Calculate transitive trust (how much does A trust Z through B, C, etc.)
    pub async fn transitive_trust(&self, from: &str, to: &str) -> f64 {
        let links = self.links.read().await;

        // Simple BFS for shortest path
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(from.to_string(), 1.0)];

        while let Some((current, accumulated_trust)) = queue.pop() {
            if current == to {
                return accumulated_trust;
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for link in links.iter() {
                if link.from_agent == current && !visited.contains(&link.to_agent) {
                    let new_trust = accumulated_trust * link.trust_level;
                    if new_trust > 0.1 {
                        // Prune low-trust paths
                        queue.push((link.to_agent.clone(), new_trust));
                    }
                }
            }
        }

        0.0 // No path found
    }

    /// Get all direct trusters of an agent
    pub async fn trustees(&self, agent_id: &str) -> Vec<TrustChainLink> {
        let links = self.links.read().await;
        links.iter().filter(|l| l.to_agent == agent_id).cloned().collect()
    }
}

impl Default for TrustChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability delegation proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDelegation {
    pub delegator: String,
    pub delegate: String,
    pub capability: String,
    pub delegated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub proof_hash: Vec<u8>,
}

impl CapabilityDelegation {
    pub fn new(
        delegator: String,
        delegate: String,
        capability: String,
        lifetime_days: i64,
    ) -> Self {
        let delegated_at = Utc::now();
        let expires_at = delegated_at + Duration::days(lifetime_days);

        let mut proof_bytes = delegator.as_bytes().to_vec();
        proof_bytes.extend_from_slice(delegate.as_bytes());
        proof_bytes.extend_from_slice(capability.as_bytes());

        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(&proof_bytes);

        Self {
            delegator,
            delegate,
            capability,
            delegated_at,
            expires_at,
            proof_hash: hasher.finalize().to_vec(),
        }
    }

    /// Check if delegation is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

/// Peer Validator for consensus-based trust updates
pub struct PeerValidator {
    delegations: Arc<RwLock<Vec<CapabilityDelegation>>>,
    trust_chain: Arc<TrustChain>,
}

impl PeerValidator {
    pub fn new(trust_chain: Arc<TrustChain>) -> Self {
        Self { delegations: Arc::new(RwLock::new(Vec::new())), trust_chain }
    }

    /// Validate that agent B's capability is backed by agent A's delegation
    pub async fn validate_capability(&self, agent_id: &str, capability: &str) -> bool {
        let delegations = self.delegations.read().await;

        let valid_delegation = delegations
            .iter()
            .any(|d| d.delegate == agent_id && d.capability == capability && d.is_valid());

        valid_delegation
    }

    /// Delegate a capability from one agent to another
    pub async fn delegate_capability(
        &self,
        delegator: String,
        delegate: String,
        capability: String,
        lifetime_days: i64,
    ) {
        let delegation = CapabilityDelegation::new(delegator, delegate, capability, lifetime_days);
        let mut delegations = self.delegations.write().await;
        delegations.push(delegation);
    }

    /// Get all valid delegations for an agent
    pub async fn agent_delegations(&self, agent_id: &str) -> Vec<CapabilityDelegation> {
        let delegations = self.delegations.read().await;
        delegations.iter().filter(|d| d.delegate == agent_id && d.is_valid()).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_identity() {
        let identity = AgentIdentity::new("agent-1".to_string(), "ed25519".to_string());

        assert_eq!(identity.did, "did:agent:agent-1");
        assert!(identity.active);
    }

    #[test]
    fn test_trust_score_update() {
        let mut score = TrustScore::new("agent-1".to_string());
        let outcome = ExecutionOutcome::Success { duration_ms: 100 };

        score.update(&outcome);
        assert!(score.score > 0.5);
        assert_eq!(score.sample_size, 1);
    }

    #[tokio::test]
    async fn test_trust_calculator() {
        let calculator = TrustScoreCalculator::new();
        let outcome = ExecutionOutcome::Success { duration_ms: 100 };

        calculator.observe("agent-1".to_string(), "agent-2".to_string(), outcome).await;

        let score = calculator.score("agent-2").await;
        assert!(score.score > 0.5);
    }

    #[tokio::test]
    async fn test_trust_chain() {
        let chain = TrustChain::new();
        chain.add_link("agent-a".to_string(), "agent-b".to_string(), 0.9).await;
        chain.add_link("agent-b".to_string(), "agent-c".to_string(), 0.8).await;

        let transitive = chain.transitive_trust("agent-a", "agent-c").await;
        assert!(transitive > 0.0);
    }

    #[tokio::test]
    async fn test_capability_delegation() {
        let delegation = CapabilityDelegation::new(
            "agent-a".to_string(),
            "agent-b".to_string(),
            "database.query".to_string(),
            30,
        );

        assert!(delegation.is_valid());
    }
}
