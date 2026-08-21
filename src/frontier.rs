// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bounded implementations for the optional frontier capability set.
//!
//! The module deliberately keeps construction and evaluation deterministic.
//! It does not perform direct network, filesystem, or process actuation. Callers
//! can inspect the resulting plans and route admitted intents through their own
//! integration boundary.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

/// Result type used by frontier capabilities.
pub type FrontierResult<T> = std::result::Result<T, String>;

// -----------------------------------------------------------------------------
// Meta-framework
// -----------------------------------------------------------------------------

/// Admission state for a bounded capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionState {
    /// The observation has not been admitted.
    Unknown,
    /// The observation is admitted but no execution receipt exists.
    Admitted,
    /// Execution evidence exists for the bounded capability.
    Alive,
    /// A typed blocker prevents execution.
    Blocked,
}

/// One named invariant evaluated by the meta-framework.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invariant {
    /// Stable invariant identifier.
    pub id: String,
    /// Human-readable invariant description.
    pub description: String,
    /// Whether the invariant currently holds.
    pub satisfied: bool,
}

/// Deterministic registry of layers and invariants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaFramework {
    layers: BTreeSet<String>,
    invariants: BTreeMap<String, Invariant>,
}

impl MetaFramework {
    /// Create an empty meta-framework.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one unique layer.
    pub fn register_layer(&mut self, name: impl Into<String>) -> FrontierResult<()> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err("layer name cannot be empty".to_string());
        }
        if !self.layers.insert(name.clone()) {
            return Err(format!("layer already registered: {name}"));
        }
        Ok(())
    }

    /// Register or replace an invariant by stable identifier.
    pub fn admit_invariant(&mut self, invariant: Invariant) -> FrontierResult<()> {
        if invariant.id.trim().is_empty() {
            return Err("invariant id cannot be empty".to_string());
        }
        self.invariants.insert(invariant.id.clone(), invariant);
        Ok(())
    }

    /// Evaluate all admitted invariants.
    #[must_use]
    pub fn validate_invariants(&self) -> bool {
        !self.invariants.is_empty() && self.invariants.values().all(|item| item.satisfied)
    }

    /// Derive standing without claiming execution that was not observed.
    #[must_use]
    pub fn state(&self, receipt_observed: bool) -> AdmissionState {
        if !self.validate_invariants() {
            AdmissionState::Blocked
        } else if receipt_observed {
            AdmissionState::Alive
        } else {
            AdmissionState::Admitted
        }
    }

    /// All currently registered layer names, in canonical order.
    #[must_use]
    pub fn layers(&self) -> Vec<&str> {
        self.layers.iter().map(String::as_str).collect()
    }

    /// Resolve one previously admitted invariant by its stable identifier.
    #[must_use]
    pub fn invariant(&self, id: &str) -> Option<&Invariant> {
        self.invariants.get(id)
    }

    /// All admitted invariants, in canonical (id-sorted) order.
    #[must_use]
    pub fn invariants(&self) -> Vec<&Invariant> {
        self.invariants.values().collect()
    }

    /// Remove a previously registered layer.
    ///
    /// # Errors
    /// Returns an error if `name` is not currently registered.
    pub fn remove_layer(&mut self, name: &str) -> FrontierResult<()> {
        if self.layers.remove(name) {
            Ok(())
        } else {
            Err(format!("layer not registered: {name}"))
        }
    }

    /// Remove a previously admitted invariant, returning the removed value.
    ///
    /// # Errors
    /// Returns an error if no invariant is registered under `id`.
    pub fn remove_invariant(&mut self, id: &str) -> FrontierResult<Invariant> {
        self.invariants.remove(id).ok_or_else(|| format!("invariant not registered: {id}"))
    }
}

// -----------------------------------------------------------------------------
// RDF composition
// -----------------------------------------------------------------------------

/// A deterministic RDF-like triple used for bounded graph composition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemanticTriple {
    /// Subject identifier.
    pub subject: String,
    /// Predicate identifier.
    pub predicate: String,
    /// Object identifier or lexical value.
    pub object: String,
}

/// Ordered, duplicate-free semantic fragment.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdfFragment {
    triples: BTreeSet<SemanticTriple>,
}

impl RdfFragment {
    /// Create an empty fragment.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit a non-empty triple.
    pub fn insert(&mut self, triple: SemanticTriple) -> FrontierResult<bool> {
        if triple.subject.trim().is_empty()
            || triple.predicate.trim().is_empty()
            || triple.object.trim().is_empty()
        {
            return Err("semantic triples require non-empty subject, predicate, and object".into());
        }
        Ok(self.triples.insert(triple))
    }

    /// Retract a triple; returns `true` if it was present.
    pub fn remove(&mut self, triple: &SemanticTriple) -> bool {
        self.triples.remove(triple)
    }

    /// Compose two fragments with deterministic set-union semantics.
    #[must_use]
    pub fn compose(&self, other: &Self) -> Self {
        let mut triples = self.triples.clone();
        triples.extend(other.triples.iter().cloned());
        Self { triples }
    }

    /// Return triples in canonical order.
    #[must_use]
    pub fn triples(&self) -> Vec<&SemanticTriple> {
        self.triples.iter().collect()
    }

    /// Match triples against an optional (subject, predicate, object)
    /// pattern. `None` in any position matches every value in that
    /// position.
    #[must_use]
    pub fn matching(
        &self,
        subject: Option<&str>,
        predicate: Option<&str>,
        object: Option<&str>,
    ) -> Vec<&SemanticTriple> {
        self.triples
            .iter()
            .filter(|t| subject.map_or(true, |s| t.subject == s))
            .filter(|t| predicate.map_or(true, |p| t.predicate == p))
            .filter(|t| object.map_or(true, |o| t.object == o))
            .collect()
    }
}

// -----------------------------------------------------------------------------
// Discovery and learning
// -----------------------------------------------------------------------------

/// One discoverable capability descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryRecord {
    /// Stable capability name.
    pub name: String,
    /// Searchable tags.
    pub tags: BTreeSet<String>,
    /// Endpoint or local handler identifier.
    pub route: String,
}

/// Deterministic capability discovery index.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryEngine {
    records: BTreeMap<String, DiscoveryRecord>,
}

impl DiscoveryEngine {
    /// Add one unique record.
    pub fn register(&mut self, record: DiscoveryRecord) -> FrontierResult<()> {
        if record.name.trim().is_empty() || record.route.trim().is_empty() {
            return Err("discovery record requires a name and route".into());
        }
        if self.records.contains_key(&record.name) {
            return Err(format!("capability already registered: {}", record.name));
        }
        self.records.insert(record.name.clone(), record);
        Ok(())
    }

    /// Search by exact name or tag, returning canonical name order.
    #[must_use]
    pub fn search(&self, term: &str) -> Vec<&DiscoveryRecord> {
        self.records
            .values()
            .filter(|record| record.name == term || record.tags.contains(term))
            .collect()
    }

    /// Records whose tag set is a superset of every tag in `terms` (AND
    /// semantics). An empty `terms` slice matches every registered record.
    #[must_use]
    pub fn search_all_tags(&self, terms: &[&str]) -> Vec<&DiscoveryRecord> {
        self.records
            .values()
            .filter(|record| terms.iter().all(|term| record.tags.contains(*term)))
            .collect()
    }

    /// Records whose route matches exactly (route is not required to be
    /// unique across records, unlike name).
    #[must_use]
    pub fn search_by_route(&self, route: &str) -> Vec<&DiscoveryRecord> {
        self.records.values().filter(|record| record.route == route).collect()
    }

    /// Remove a previously registered capability by name, returning the
    /// removed record. A real gap `register`/`search` alone left open:
    /// without this, a capability that goes away (a decommissioned
    /// route, a withdrawn service) could never be un-discovered.
    ///
    /// # Errors
    /// Returns an error if no capability is registered under `name`.
    pub fn deregister(&mut self, name: &str) -> FrontierResult<DiscoveryRecord> {
        self.records
            .remove(name)
            .ok_or_else(|| format!("no capability registered under name: {name}"))
    }

    /// All currently registered capability names, in canonical order.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.records.keys().map(String::as_str).collect()
    }

    /// Recommend which registered capability to try next, given each
    /// candidate's own [`LearningTrajectory`] -- the real composition
    /// point between static discovery and [`ExplorationPolicy`]: instead
    /// of a caller hand-picking a capability by name, `recommend` lets
    /// UCB1 decide which discovered capability most deserves the next
    /// real invocation. `histories` must name every candidate exactly
    /// once and only candidates actually registered here.
    ///
    /// # Errors
    /// Returns an error if `histories` is empty, if any name in
    /// `histories` is not currently registered (a caller recommending
    /// against a stale or foreign capability set is a real bug, not
    /// something to silently ignore), or if a name appears more than
    /// once. A duplicate would otherwise be cloned into the UCB1 arm
    /// set twice, silently inflating the shared `total_pulls` term
    /// that every arm's exploration bonus depends on and biasing the
    /// recommendation -- not just for the duplicated candidate.
    pub fn recommend<'a>(
        &self,
        histories: &'a [(String, LearningTrajectory)],
    ) -> FrontierResult<&'a str> {
        if histories.is_empty() {
            return Err("recommend requires at least one candidate history".to_string());
        }
        let mut seen_names = BTreeSet::new();
        for (name, _) in histories {
            if !self.records.contains_key(name) {
                return Err(format!("recommend candidate is not a registered capability: {name}"));
            }
            if !seen_names.insert(name) {
                return Err(format!("recommend candidate named more than once: {name}"));
            }
        }
        let trajectories: Vec<LearningTrajectory> =
            histories.iter().map(|(_, trajectory)| trajectory.clone()).collect();
        let index = ExplorationPolicy::select_ucb1(&trajectories)?;
        Ok(histories[index].0.as_str())
    }
}

/// One measured learning observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningObservation {
    /// Monotonic sequence number.
    pub sequence: u64,
    /// Score in the inclusive range 0.0..=1.0.
    pub score: f64,
}

/// A monotonic, replayable learning trajectory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LearningTrajectory {
    observations: Vec<LearningObservation>,
}

impl LearningTrajectory {
    /// Append the next bounded observation.
    pub fn observe(&mut self, score: f64) -> FrontierResult<u64> {
        if !score.is_finite() || !(0.0..=1.0).contains(&score) {
            return Err("learning score must be finite and within 0.0..=1.0".into());
        }
        let sequence = self.observations.len() as u64;
        self.observations.push(LearningObservation { sequence, score });
        Ok(sequence)
    }

    /// Return the latest score.
    #[must_use]
    pub fn latest(&self) -> Option<f64> {
        self.observations.last().map(|item| item.score)
    }

    /// Verify that scores never regress.
    #[must_use]
    pub fn is_monotonic(&self) -> bool {
        self.observations.windows(2).all(|pair| pair[0].score <= pair[1].score)
    }

    /// Number of observations recorded so far (the arm's "pull count" in
    /// multi-armed-bandit terms).
    #[must_use]
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// Whether no observation has been recorded yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// The arithmetic mean of every recorded score, or `None` if empty.
    #[must_use]
    pub fn mean(&self) -> Option<f64> {
        if self.observations.is_empty() {
            return None;
        }
        let sum: f64 = self.observations.iter().map(|observation| observation.score).sum();
        Some(sum / self.observations.len() as f64)
    }

    /// Discard every recorded observation, returning the trajectory to its
    /// initial (never-tried) state -- e.g. after a capability's underlying
    /// route is redeployed and its prior learning history should no longer
    /// bias [`ExplorationPolicy::select_ucb1`]/[`DiscoveryEngine::recommend`].
    pub fn reset(&mut self) {
        self.observations.clear();
    }
}

/// Deterministic explore/exploit arm selection over a set of
/// [`LearningTrajectory`]s -- the innovative-exploration counterpart to
/// [`DiscoveryEngine`]'s static capability search: given several
/// candidate options each with their own history of measured rewards,
/// [`ExplorationPolicy::select_ucb1`] decides which one to try next.
///
/// Implements UCB1 (Auer, Cesa-Bianchi & Fischer, 2002): an arm that has
/// never been tried always wins first (an infinite upper confidence
/// bound), guaranteeing every option gets a first real observation before
/// exploitation begins; afterward each arm's score is
/// `mean_reward + sqrt(2 * ln(total_pulls) / pulls)`, so an option with
/// fewer observations keeps a wider confidence bonus even if its current
/// mean is lower -- the formal, well-known trade-off between trying an
/// under-sampled option (explore) and picking the best-known one
/// (exploit). Entirely deterministic given real recorded trajectories --
/// no randomness, so it is exactly reproducible and testable.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExplorationPolicy;

impl ExplorationPolicy {
    /// Select the index into `trajectories` to try next via UCB1.
    ///
    /// # Errors
    /// Returns an error if `trajectories` is empty -- there is no arm to
    /// select among.
    pub fn select_ucb1(trajectories: &[LearningTrajectory]) -> FrontierResult<usize> {
        if trajectories.is_empty() {
            return Err("UCB1 selection requires at least one trajectory".to_string());
        }

        // Any never-tried arm always wins (an infinite confidence bound),
        // in trajectory order -- guarantees every arm gets a first real
        // observation before exploitation kicks in.
        if let Some(index) = trajectories.iter().position(LearningTrajectory::is_empty) {
            return Ok(index);
        }

        let total_pulls: f64 = trajectories.iter().map(|trajectory| trajectory.len() as f64).sum();

        let mut best_index = 0;
        let mut best_score = f64::NEG_INFINITY;
        for (index, trajectory) in trajectories.iter().enumerate() {
            // Safe: the empty-arm check above already returned for any
            // trajectory with zero observations, so `mean()`/`len()` are
            // always `Some`/nonzero here.
            let mean = trajectory.mean().unwrap_or(0.0);
            let pulls = trajectory.len() as f64;
            let exploration_bonus = (2.0 * total_pulls.ln() / pulls).sqrt();
            let score = mean + exploration_bonus;
            if score > best_score {
                best_score = score;
                best_index = index;
            }
        }
        Ok(best_index)
    }
}

// -----------------------------------------------------------------------------
// Reflexive testing and quantum-ready policy
// -----------------------------------------------------------------------------

/// Machine-readable result of a reflexive verifier run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflexiveReport {
    /// Number of checks that passed.
    pub passed: u64,
    /// Number of checks that failed.
    pub failed: u64,
    /// Whether replay produced the same result.
    pub replay_verified: bool,
}

impl ReflexiveReport {
    /// Return true only for a fully observed, replayable verifier run.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.passed > 0 && self.failed == 0 && self.replay_verified
    }

    /// Combine two reports from separate verifier runs (e.g. sharded
    /// execution, or an initial run plus a replay run). The merged run is
    /// only replay-verified if both constituent runs were.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            passed: self.passed + other.passed,
            failed: self.failed + other.failed,
            replay_verified: self.replay_verified && other.replay_verified,
        }
    }

    /// Fraction of checks that passed, in `[0.0, 1.0]`. Returns `0.0` when
    /// no checks were recorded (avoids a divide-by-zero at call sites).
    #[must_use]
    pub fn pass_rate(&self) -> f64 {
        let total = self.passed + self.failed;
        if total == 0 {
            0.0
        } else {
            self.passed as f64 / total as f64
        }
    }
}

/// Post-quantum algorithm family admitted by policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumAlgorithm {
    /// ML-KEM key encapsulation.
    MlKem,
    /// ML-DSA signatures.
    MlDsa,
    /// Stateless hash-based SLH-DSA signatures.
    SlhDsa,
}

/// Bounded crypto-agility policy. This type selects algorithms; it does not
/// implement cryptographic operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumReadyPolicy {
    allowed: BTreeSet<String>,
}

impl QuantumReadyPolicy {
    /// Construct the default PQC allow-list.
    #[must_use]
    pub fn post_quantum() -> Self {
        Self { allowed: ["ML-KEM", "ML-DSA", "SLH-DSA"].into_iter().map(str::to_string).collect() }
    }

    /// Check whether an algorithm is admitted.
    #[must_use]
    pub fn admits(&self, algorithm: QuantumAlgorithm) -> bool {
        self.allowed.contains(Self::algorithm_name(algorithm))
    }

    /// Construct a policy admitting exactly the given algorithms (may be
    /// empty, or a subset of the full PQC set `post_quantum` admits).
    #[must_use]
    pub fn restricted(algorithms: impl IntoIterator<Item = QuantumAlgorithm>) -> Self {
        Self {
            allowed: algorithms.into_iter().map(Self::algorithm_name).map(str::to_string).collect(),
        }
    }

    /// List every algorithm this policy currently admits.
    #[must_use]
    pub fn admitted(&self) -> Vec<QuantumAlgorithm> {
        [QuantumAlgorithm::MlKem, QuantumAlgorithm::MlDsa, QuantumAlgorithm::SlhDsa]
            .into_iter()
            .filter(|algorithm| self.admits(*algorithm))
            .collect()
    }

    const fn algorithm_name(algorithm: QuantumAlgorithm) -> &'static str {
        match algorithm {
            QuantumAlgorithm::MlKem => "ML-KEM",
            QuantumAlgorithm::MlDsa => "ML-DSA",
            QuantumAlgorithm::SlhDsa => "SLH-DSA",
        }
    }
}

// -----------------------------------------------------------------------------
// Federated network
// -----------------------------------------------------------------------------

/// Stable peer identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

/// Capability advertised by a federated peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name.
    pub name: String,
    /// Semantic version or protocol version.
    pub version: String,
    /// Invocation endpoint.
    pub endpoint: String,
}

/// Deterministic in-memory federation model used by the optional feature.
#[derive(Debug, Clone)]
pub struct FederatedNetwork {
    /// Local node identity.
    pub local_node: String,
    peers: Arc<RwLock<Vec<PeerId>>>,
    capabilities: Arc<RwLock<BTreeMap<String, Capability>>>,
}

impl FederatedNetwork {
    /// Create a bounded federation node.
    pub fn new(node_id: impl Into<String>) -> FrontierResult<Self> {
        let local_node = node_id.into();
        if local_node.trim().is_empty() {
            return Err("federation node id cannot be empty".into());
        }
        let peers = vec![
            PeerId(format!("{local_node}-peer-1")),
            PeerId(format!("{local_node}-peer-2")),
            PeerId(format!("{local_node}-peer-3")),
        ];
        Ok(Self {
            local_node,
            peers: Arc::new(RwLock::new(peers)),
            capabilities: Arc::new(RwLock::new(BTreeMap::new())),
        })
    }

    /// Discover the currently admitted peer set.
    pub async fn discover_peers(&self) -> FrontierResult<Vec<PeerId>> {
        self.peers
            .read()
            .map(|peers| peers.clone())
            .map_err(|_| "federation peer registry lock poisoned".into())
    }

    /// Evaluate a Byzantine quorum. Insufficient votes are a typed refusal.
    pub async fn consensus_vote<F>(&self, peers: &[PeerId], vote: F) -> FrontierResult<bool>
    where
        F: Fn(&PeerId) -> bool,
    {
        if peers.is_empty() {
            return Err("consensus requires at least one peer".into());
        }
        let tolerated_faults = peers.len().saturating_sub(1) / 3;
        let threshold = tolerated_faults.saturating_mul(2).saturating_add(1);
        let approvals = peers.iter().filter(|peer| vote(peer)).count();
        if approvals >= threshold {
            Ok(true)
        } else {
            Err(format!(
                "consensus refused: approvals={approvals}, threshold={threshold}, peers={}",
                peers.len()
            ))
        }
    }

    /// Admit a new peer into the local registry. Returns `false` (not an
    /// error) if the peer was already present -- adding an existing peer
    /// is idempotent, not a failure.
    ///
    /// # Errors
    /// Returns an error if the peer registry lock is poisoned.
    pub async fn add_peer(&self, peer: PeerId) -> FrontierResult<bool> {
        let mut peers =
            self.peers.write().map_err(|_| "federation peer registry lock poisoned".to_string())?;
        if peers.contains(&peer) {
            return Ok(false);
        }
        peers.push(peer);
        Ok(true)
    }

    /// Remove a peer from the local registry (e.g. offline or
    /// Byzantine-excluded). Returns `false` (not an error) if the peer was
    /// not present.
    ///
    /// # Errors
    /// Returns an error if the peer registry lock is poisoned.
    pub async fn remove_peer(&self, peer: &PeerId) -> FrontierResult<bool> {
        let mut peers =
            self.peers.write().map_err(|_| "federation peer registry lock poisoned".to_string())?;
        let before = peers.len();
        peers.retain(|p| p != peer);
        Ok(peers.len() != before)
    }

    /// Advertise one validated capability without performing remote I/O.
    pub async fn advertise_capability(&mut self, capability: &Capability) -> FrontierResult<()> {
        if capability.name.trim().is_empty()
            || capability.version.trim().is_empty()
            || capability.endpoint.trim().is_empty()
        {
            return Err("capability name, version, and endpoint are required".into());
        }
        self.capabilities
            .write()
            .map_err(|_| "federation capability registry lock poisoned".to_string())?
            .insert(capability.name.clone(), capability.clone());
        Ok(())
    }

    /// Resolve an advertised capability locally.
    pub fn resolve(&self, name: &str) -> FrontierResult<Capability> {
        self.capabilities
            .read()
            .map_err(|_| "federation capability registry lock poisoned".to_string())?
            .get(name)
            .cloned()
            .ok_or_else(|| format!("capability not advertised: {name}"))
    }
}

// -----------------------------------------------------------------------------
// Economic simulation
// -----------------------------------------------------------------------------

/// Stable simulated agent identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

/// Stable simulated task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

/// One auction bid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    /// Bidder identity.
    pub agent_id: AgentId,
    /// Task being bid on.
    pub task_id: TaskId,
    /// Bid value.
    pub bid_value: f64,
}

/// Result of a second-price auction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionOutcome {
    /// Winning bidder.
    pub winner: AgentId,
    /// Task allocated by the auction.
    pub task_id: TaskId,
    /// Second-highest price paid by the winner.
    pub payment: f64,
}

/// Deterministic Vickrey auction.
#[derive(Debug, Clone, Default)]
pub struct VickreyAuction;

impl VickreyAuction {
    /// Create an auction.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Allocate to the highest bidder at the second-highest price.
    pub fn run_auction(&mut self, bids: &[Bid]) -> FrontierResult<AuctionOutcome> {
        if bids.len() < 2 {
            return Err("a Vickrey auction requires at least two bids".into());
        }
        let task_id = bids[0].task_id;
        if bids
            .iter()
            .any(|bid| bid.task_id != task_id || !bid.bid_value.is_finite() || bid.bid_value < 0.0)
        {
            return Err("all bids must target one task with finite non-negative values".into());
        }
        // Each agent may submit at most one bid: without this, a single
        // agent's own second-highest bid could become the "second price"
        // it pays, instead of a genuine competitor's valuation -- silently
        // corrupting the Vickrey mechanism's truthfulness guarantee.
        let mut seen_agents = BTreeSet::new();
        if !bids.iter().all(|bid| seen_agents.insert(bid.agent_id)) {
            return Err("each agent may submit at most one bid per auction".into());
        }
        let mut ordered = bids.to_vec();
        ordered.sort_by(|left, right| {
            right
                .bid_value
                .total_cmp(&left.bid_value)
                .then_with(|| left.agent_id.cmp(&right.agent_id))
        });
        Ok(AuctionOutcome { winner: ordered[0].agent_id, task_id, payment: ordered[1].bid_value })
    }

    /// Verify non-negative truthful utility for the winner.
    #[must_use]
    pub fn verify_truthfulness(&self, valuation: f64, outcome: &AuctionOutcome) -> bool {
        valuation.is_finite() && valuation >= outcome.payment
    }
}

/// Agent participating in the bounded simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    /// Agent identity.
    pub id: AgentId,
    /// Capability names the agent can execute.
    pub capabilities: Vec<String>,
    /// Trust score in 0.0..=1.0.
    pub trust_score: f64,
    /// Economic valuation.
    pub valuation: f64,
}

/// Task available for allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Task identity.
    pub id: TaskId,
    /// Capability required to execute the task.
    pub required_capability: String,
    /// Task value.
    pub value: f64,
}

/// One deterministic allocation receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Allocation {
    /// Allocated task.
    pub task_id: TaskId,
    /// Selected agent.
    pub agent_id: AgentId,
}

/// Bounded economic simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EconomicSimulation {
    agents: BTreeMap<AgentId, Agent>,
    tasks: BTreeMap<TaskId, Task>,
    allocations: Vec<Allocation>,
    /// Logical simulation time.
    pub time: f64,
}

impl EconomicSimulation {
    /// Create an empty simulation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one unique valid agent.
    pub fn add_agent(&mut self, agent: Agent) -> FrontierResult<()> {
        if !agent.trust_score.is_finite()
            || !(0.0..=1.0).contains(&agent.trust_score)
            || !agent.valuation.is_finite()
            || agent.valuation < 0.0
        {
            return Err("agent trust and valuation are outside admitted bounds".into());
        }
        if self.agents.insert(agent.id, agent).is_some() {
            return Err("duplicate agent id".into());
        }
        Ok(())
    }

    /// Add one unique valid task.
    pub fn add_task(&mut self, task: Task) -> FrontierResult<()> {
        if task.required_capability.trim().is_empty() || !task.value.is_finite() || task.value < 0.0
        {
            return Err("task capability and value are outside admitted bounds".into());
        }
        if self.tasks.insert(task.id, task).is_some() {
            return Err("duplicate task id".into());
        }
        Ok(())
    }

    /// Execute one logical allocation step.
    pub fn step(&mut self) -> FrontierResult<Vec<Allocation>> {
        let mut produced = Vec::new();
        for task in self.tasks.values() {
            let selected = self
                .agents
                .values()
                .filter(|agent| {
                    agent.capabilities.iter().any(|item| item == &task.required_capability)
                })
                .max_by(|left, right| {
                    left.trust_score
                        .total_cmp(&right.trust_score)
                        .then_with(|| right.id.cmp(&left.id))
                });
            if let Some(agent) = selected {
                produced.push(Allocation { task_id: task.id, agent_id: agent.id });
            }
        }
        // Consume every task allocated this step -- without this, a task
        // stays in `self.tasks` forever and a second `step()` call would
        // silently re-allocate (and double-count) the same task.
        let allocated: BTreeSet<TaskId> = produced.iter().map(|a| a.task_id).collect();
        self.tasks.retain(|id, _| !allocated.contains(id));
        self.allocations.extend(produced.iter().cloned());
        self.time += 1.0;
        Ok(produced)
    }

    /// Number of admitted agents.
    #[must_use]
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// All allocations in logical-time order.
    #[must_use]
    pub fn allocations(&self) -> &[Allocation] {
        &self.allocations
    }
}

// -----------------------------------------------------------------------------
// Fractal composition
// -----------------------------------------------------------------------------

/// Marker trait for one level in a fractal noun hierarchy.
pub trait FractalLevel: Clone + Copy + Send + Sync + 'static {
    /// Zero-based hierarchy depth.
    fn depth() -> usize;
    /// Human-readable level name.
    fn name() -> &'static str;
}

macro_rules! level {
    ($name:ident, $depth:expr, $label:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name;
        impl FractalLevel for $name {
            fn depth() -> usize {
                $depth
            }
            fn name() -> &'static str {
                $label
            }
        }
    };
}

level!(RootLevel, 0, "Root");
level!(DomainLevel, 1, "Domain");
level!(NounLevel, 2, "Noun");
level!(VerbLevel, 3, "Verb");

/// A value placed at a typed hierarchy level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FractalNoun<L: FractalLevel, T> {
    /// Value at this level.
    pub data: T,
    lineage: Vec<String>,
    marker: PhantomData<L>,
}

impl<L: FractalLevel, T> FractalNoun<L, T> {
    /// Create a value at a typed level.
    #[must_use]
    pub fn new(data: T) -> Self {
        Self { data, lineage: vec![L::name().to_string()], marker: PhantomData }
    }

    /// Current hierarchy depth.
    #[must_use]
    pub fn depth(&self) -> usize {
        L::depth()
    }

    /// Current hierarchy level name.
    #[must_use]
    pub fn level_name(&self) -> &'static str {
        L::name()
    }

    /// Compose only with the immediately adjacent child level.
    pub fn compose<N: FractalLevel, U>(
        self,
        mut next: FractalNoun<N, U>,
    ) -> FrontierResult<FractalNoun<N, U>> {
        if N::depth() != L::depth().saturating_add(1) {
            return Err(format!(
                "non-adjacent fractal composition refused: {}({}) -> {}({})",
                L::name(),
                L::depth(),
                N::name(),
                N::depth()
            ));
        }
        let mut lineage = self.lineage;
        lineage.push(N::name().to_string());
        next.lineage = lineage;
        Ok(next)
    }

    /// Typed lineage from the root of this composition.
    #[must_use]
    pub fn lineage(&self) -> &[String] {
        &self.lineage
    }
}

/// Untyped composition trace for diagnostics and serialization boundaries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionChain {
    entries: Vec<String>,
}

impl CompositionChain {
    /// Create an empty chain.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one non-empty segment.
    pub fn push(&mut self, value: impl Into<String>) {
        let value = value.into();
        if !value.trim().is_empty() {
            self.entries.push(value);
        }
    }

    /// Number of segments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the chain is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Ordered view of the chain's segments.
    #[must_use]
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

impl<L: FractalLevel, T> From<&FractalNoun<L, T>> for CompositionChain {
    /// Cross the serialization boundary: `FractalNoun` is typed and does
    /// not derive `Serialize`/`Deserialize`, while `CompositionChain` is
    /// this module's untyped, serializable trace of a composition. Without
    /// this conversion a caller holding a composed `FractalNoun` had no
    /// path from `lineage()` to the one serializable trace type in the
    /// module except hand-looping `push` themselves.
    fn from(noun: &FractalNoun<L, T>) -> Self {
        let mut chain = CompositionChain::new();
        for segment in noun.lineage() {
            chain.push(segment.clone());
        }
        chain
    }
}

// -----------------------------------------------------------------------------
// Executable specifications
// -----------------------------------------------------------------------------

/// Deterministic Given/When/Then specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableSpec {
    /// Specification name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Given clauses.
    pub preconditions: Vec<String>,
    /// When clauses.
    pub actions: Vec<String>,
    /// Then clauses.
    pub outcomes: Vec<String>,
    /// Additional invariants.
    pub invariants: Vec<String>,
    parameters: BTreeMap<String, u64>,
}

impl ExecutableSpec {
    /// Create a specification with conventional Byzantine-proof parameters.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            preconditions: Vec::new(),
            actions: Vec::new(),
            outcomes: Vec::new(),
            invariants: Vec::new(),
            parameters: BTreeMap::from([
                ("total_nodes".to_string(), 10),
                ("byzantine_nodes".to_string(), 3),
            ]),
        }
    }

    /// Add a Given clause.
    #[must_use]
    pub fn given(mut self, clause: impl Into<String>) -> Self {
        self.preconditions.push(clause.into());
        self
    }

    /// Add a When clause.
    #[must_use]
    pub fn when(mut self, clause: impl Into<String>) -> Self {
        self.actions.push(clause.into());
        self
    }

    /// Add a Then clause.
    #[must_use]
    pub fn then(mut self, clause: impl Into<String>) -> Self {
        self.outcomes.push(clause.into());
        self
    }

    /// Add an invariant clause.
    #[must_use]
    pub fn and(mut self, clause: impl Into<String>) -> Self {
        self.invariants.push(clause.into());
        self
    }

    /// Override one bounded integer parameter.
    #[must_use]
    pub fn parameter(mut self, name: impl Into<String>, value: u64) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }

    /// Read one parameter value, if set. A safe alternative to indexing
    /// the raw parameter map a `validate` predicate receives -- indexing
    /// a missing key panics, this returns `None` instead.
    #[must_use]
    pub fn parameter_value(&self, name: &str) -> Option<u64> {
        self.parameters.get(name).copied()
    }

    /// Evaluate the specification predicate.
    pub fn validate<F>(&self, predicate: F) -> FrontierResult<bool>
    where
        F: FnOnce(&BTreeMap<String, u64>) -> bool,
    {
        if predicate(&self.parameters) {
            Ok(true)
        } else {
            Err(format!("executable specification failed: {}", self.name))
        }
    }

    /// Render a deterministic Gherkin projection.
    #[must_use]
    pub fn to_gherkin(&self) -> String {
        let mut lines = vec![format!("Feature: {}", self.name), format!("  {}", self.description)];
        lines.extend(self.preconditions.iter().map(|item| format!("  Given {item}")));
        lines.extend(self.actions.iter().map(|item| format!("  When {item}")));
        lines.extend(self.outcomes.iter().map(|item| format!("  Then {item}")));
        lines.extend(self.invariants.iter().map(|item| format!("  And {item}")));
        lines.join("\n") + "\n"
    }

    /// Stable specification name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Ordered collection of executable specifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecificationSuite {
    specs: BTreeMap<String, ExecutableSpec>,
}

impl SpecificationSuite {
    /// Create an empty suite.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace a specification by name.
    pub fn add_spec(&mut self, spec: ExecutableSpec) {
        self.specs.insert(spec.name.clone(), spec);
    }

    /// Resolve a specification by exact name.
    pub fn get_spec(&self, name: &str) -> FrontierResult<&ExecutableSpec> {
        self.specs.get(name).ok_or_else(|| format!("specification not found: {name}"))
    }

    /// Names of all specifications currently in the suite, in sorted order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.specs.keys().map(String::as_str)
    }

    /// Remove a specification by name, returning it if present.
    pub fn remove_spec(&mut self, name: &str) -> Option<ExecutableSpec> {
        self.specs.remove(name)
    }

    /// Number of specifications in the suite.
    #[must_use]
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    /// Whether the suite has no specifications.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_price_is_deterministic() {
        let mut auction = VickreyAuction::new();
        let outcome = auction
            .run_auction(&[
                Bid { agent_id: AgentId(1), task_id: TaskId(7), bid_value: 10.0 },
                Bid { agent_id: AgentId(2), task_id: TaskId(7), bid_value: 8.0 },
            ])
            .expect("valid auction");
        assert_eq!(outcome.winner, AgentId(1));
        assert_eq!(outcome.payment, 8.0);
    }

    #[test]
    fn composition_refuses_level_skip() {
        let root = FractalNoun::<RootLevel, _>::new("root");
        let noun = FractalNoun::<NounLevel, _>::new("noun");
        assert!(root.compose(noun).is_err());
    }

    #[test]
    fn alive_requires_replay() {
        assert!(!ReflexiveReport { passed: 10, failed: 0, replay_verified: false }.is_alive());
        assert!(ReflexiveReport { passed: 10, failed: 0, replay_verified: true }.is_alive());
    }

    #[test]
    fn ucb1_refuses_an_empty_arm_set() {
        let error = ExplorationPolicy::select_ucb1(&[]).expect_err("no arms to select among");
        assert!(error.contains("at least one trajectory"));
    }

    #[test]
    fn ucb1_always_explores_a_never_tried_arm_first() {
        let mut tried = LearningTrajectory::default();
        tried.observe(0.9).expect("valid score");
        let never_tried = LearningTrajectory::default();

        // Even though `tried`'s mean (0.9) is high, the untried arm must
        // win -- every option gets a first real observation before
        // exploitation kicks in.
        let selected =
            ExplorationPolicy::select_ucb1(&[tried, never_tried]).expect("valid selection");
        assert_eq!(selected, 1);
    }

    #[test]
    fn ucb1_prefers_the_under_sampled_arm_when_means_are_close_via_the_confidence_bonus() {
        // Arm 0: tried many times with a slightly higher mean.
        let mut well_sampled = LearningTrajectory::default();
        for _ in 0..20 {
            well_sampled.observe(0.6).expect("valid score");
        }
        // Arm 1: tried only twice, slightly lower mean -- but its wider
        // confidence bonus (fewer pulls) should let it win over arm 0
        // once both have at least one observation.
        let mut under_sampled = LearningTrajectory::default();
        under_sampled.observe(0.55).expect("valid score");
        under_sampled.observe(0.55).expect("valid score");

        let selected = ExplorationPolicy::select_ucb1(&[well_sampled, under_sampled])
            .expect("valid selection");
        assert_eq!(selected, 1, "the under-sampled arm's exploration bonus must win here");
    }

    #[test]
    fn ucb1_exploits_the_clear_winner_once_every_arm_has_comparable_history() {
        let mut winner = LearningTrajectory::default();
        let mut loser = LearningTrajectory::default();
        for _ in 0..50 {
            winner.observe(0.95).expect("valid score");
            loser.observe(0.10).expect("valid score");
        }

        let selected = ExplorationPolicy::select_ucb1(&[loser, winner]).expect("valid selection");
        assert_eq!(selected, 1, "with equal sampling, the arm with the far higher mean must win");
    }

    #[test]
    fn learning_trajectory_mean_and_len_report_real_aggregate_state() {
        let mut trajectory = LearningTrajectory::default();
        assert!(trajectory.is_empty());
        assert_eq!(trajectory.mean(), None);

        trajectory.observe(0.2).expect("valid score");
        trajectory.observe(0.4).expect("valid score");
        trajectory.observe(0.6).expect("valid score");

        assert_eq!(trajectory.len(), 3);
        assert!(!trajectory.is_empty());
        assert!((trajectory.mean().expect("real mean") - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn learning_trajectory_observe_refuses_non_finite_scores() {
        // `is_finite()` is false for NaN, +inf, and -inf alike -- confirm all
        // three are rejected before the 0.0..=1.0 range check runs, and that
        // a rejected observation never partially lands in the trajectory.
        let mut traj = LearningTrajectory::default();
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(traj.observe(bad).is_err());
        }
        assert!(traj.is_empty());
    }

    #[test]
    fn discovery_engine_deregister_removes_a_real_record_and_refuses_an_unknown_one() {
        let mut engine = DiscoveryEngine::default();
        engine
            .register(DiscoveryRecord {
                name: "billing".to_string(),
                tags: BTreeSet::new(),
                route: "svc://billing".to_string(),
            })
            .expect("valid record");

        let removed = engine.deregister("billing").expect("real registered capability");
        assert_eq!(removed.route, "svc://billing");
        assert!(engine.search("billing").is_empty());

        let error = engine.deregister("billing").expect_err("already removed");
        assert!(error.contains("no capability registered"));
    }

    #[test]
    fn discovery_engine_names_lists_every_registered_capability() {
        let mut engine = DiscoveryEngine::default();
        for name in ["alpha", "beta"] {
            engine
                .register(DiscoveryRecord {
                    name: name.to_string(),
                    tags: BTreeSet::new(),
                    route: format!("svc://{name}"),
                })
                .expect("valid record");
        }
        assert_eq!(engine.names(), vec!["alpha", "beta"]);
    }

    #[test]
    fn discovery_engine_recommend_defers_to_ucb1_over_real_candidate_histories() {
        let mut engine = DiscoveryEngine::default();
        for name in ["proven", "fresh"] {
            engine
                .register(DiscoveryRecord {
                    name: name.to_string(),
                    tags: BTreeSet::new(),
                    route: format!("svc://{name}"),
                })
                .expect("valid record");
        }

        let mut proven_history = LearningTrajectory::default();
        proven_history.observe(0.9).expect("valid score");
        let fresh_history = LearningTrajectory::default();

        let histories =
            vec![("proven".to_string(), proven_history), ("fresh".to_string(), fresh_history)];

        // The never-tried "fresh" capability must win first, exactly like
        // ExplorationPolicy::select_ucb1 on its own.
        let recommended = engine.recommend(&histories).expect("valid recommendation");
        assert_eq!(recommended, "fresh");
    }

    #[test]
    fn discovery_engine_recommend_refuses_a_candidate_that_was_never_registered() {
        let mut engine = DiscoveryEngine::default();
        engine
            .register(DiscoveryRecord {
                name: "known".to_string(),
                tags: BTreeSet::new(),
                route: "svc://known".to_string(),
            })
            .expect("valid record");

        let histories = vec![("unknown-capability".to_string(), LearningTrajectory::default())];
        let error = engine.recommend(&histories).expect_err("unregistered candidate refused");
        assert!(error.contains("unknown-capability"));
    }

    #[test]
    fn discovery_engine_recommend_refuses_an_empty_candidate_set() {
        let engine = DiscoveryEngine::default();
        let error = engine.recommend(&[]).expect_err("no candidates to recommend among");
        assert!(error.contains("at least one candidate"));
    }

    #[test]
    fn discovery_engine_recommend_refuses_a_duplicate_candidate_name() {
        let mut engine = DiscoveryEngine::default();
        engine
            .register(DiscoveryRecord {
                name: "solo".to_string(),
                tags: BTreeSet::new(),
                route: "svc://solo".to_string(),
            })
            .expect("valid record");

        // Two distinct trajectories under the same name -- if `recommend`
        // silently accepted this, both would be folded into UCB1's arm
        // set, inflating the shared `total_pulls` term and biasing every
        // arm's score, not just the duplicated one.
        let mut first = LearningTrajectory::default();
        first.observe(0.2).expect("valid score");
        let mut second = LearningTrajectory::default();
        second.observe(0.8).expect("valid score");

        let histories = vec![("solo".to_string(), first), ("solo".to_string(), second)];
        let error = engine.recommend(&histories).expect_err("duplicate candidate name refused");
        assert!(error.contains("solo"));
    }

    // -------------------------------------------------------------------
    // MetaFramework: layers()/invariant()/invariants(), remove_layer/
    // remove_invariant, and Blocked-state coverage (frontier-gap-sweep)
    // -------------------------------------------------------------------

    #[test]
    fn meta_framework_layers_and_invariant_accessors_report_real_registered_state() {
        let mut framework = MetaFramework::new();
        framework.register_layer("admission").expect("unique layer");
        framework.register_layer("execution").expect("unique layer");
        framework
            .admit_invariant(Invariant {
                id: "zero-unreceipted-actuation".to_string(),
                description: "Every effect has a receipt".to_string(),
                satisfied: true,
            })
            .expect("valid invariant");

        assert_eq!(framework.layers(), vec!["admission", "execution"]);
        assert_eq!(framework.invariants().len(), 1);
        let invariant =
            framework.invariant("zero-unreceipted-actuation").expect("registered invariant");
        assert!(invariant.satisfied);
        assert!(framework.invariant("unknown-id").is_none());
    }

    #[test]
    fn meta_framework_remove_layer_and_invariant_actually_retract_registered_state() {
        let mut framework = MetaFramework::new();
        framework.register_layer("admission").expect("unique layer");
        framework
            .admit_invariant(Invariant {
                id: "zero-unreceipted-actuation".to_string(),
                description: "Every effect has a receipt".to_string(),
                satisfied: true,
            })
            .expect("valid invariant");

        framework.remove_layer("admission").expect("registered layer");
        assert!(framework.layers().is_empty());
        let error = framework.remove_layer("admission").expect_err("already removed");
        assert!(error.contains("not registered"));

        let removed =
            framework.remove_invariant("zero-unreceipted-actuation").expect("registered invariant");
        assert!(removed.satisfied);
        assert!(framework.invariant("zero-unreceipted-actuation").is_none());
        // Removing the only invariant leaves the set empty, so validation
        // must fail again -- retraction actually changes admission standing.
        assert!(!framework.validate_invariants());
    }

    #[test]
    fn meta_framework_blocks_on_an_unsatisfied_invariant_even_with_a_receipt() {
        let mut framework = MetaFramework::new();
        framework.register_layer("admission").expect("unique layer");
        framework
            .admit_invariant(Invariant {
                id: "zero-unreceipted-actuation".to_string(),
                description: "Every effect has a receipt".to_string(),
                satisfied: false,
            })
            .expect("valid invariant");

        assert!(!framework.validate_invariants());
        assert_eq!(framework.state(false), AdmissionState::Blocked);
        assert_eq!(
            framework.state(true),
            AdmissionState::Blocked,
            "a receipt must not override a blocked invariant"
        );
    }

    #[test]
    fn meta_framework_blocks_when_no_invariants_are_admitted_at_all() {
        let framework = MetaFramework::new();
        assert!(!framework.validate_invariants());
        assert_eq!(framework.state(false), AdmissionState::Blocked);
        assert_eq!(framework.state(true), AdmissionState::Blocked);
    }

    // -------------------------------------------------------------------
    // RdfFragment: remove()/matching() + round-trip + compose dedup
    // -------------------------------------------------------------------

    #[test]
    fn rdf_fragment_remove_retracts_a_present_triple_and_refuses_twice() {
        let triple = SemanticTriple {
            subject: "cnv:tool".to_string(),
            predicate: "cnv:defaultVerb".to_string(),
            object: "cnv:run".to_string(),
        };
        let mut fragment = RdfFragment::new();
        fragment.insert(triple.clone()).expect("valid triple");

        assert!(fragment.remove(&triple));
        assert!(fragment.triples().is_empty());
        assert!(!fragment.remove(&triple));
    }

    #[test]
    fn rdf_fragment_matching_filters_by_subject_pattern() {
        let mut fragment = RdfFragment::new();
        fragment
            .insert(SemanticTriple {
                subject: "cnv:tool".to_string(),
                predicate: "cnv:defaultVerb".to_string(),
                object: "cnv:run".to_string(),
            })
            .expect("valid triple");
        fragment
            .insert(SemanticTriple {
                subject: "cnv:tool".to_string(),
                predicate: "rdf:type".to_string(),
                object: "cnv:Noun".to_string(),
            })
            .expect("valid triple");
        fragment
            .insert(SemanticTriple {
                subject: "cnv:noun".to_string(),
                predicate: "rdf:type".to_string(),
                object: "cnv:Concept".to_string(),
            })
            .expect("valid triple");

        let matches = fragment.matching(Some("cnv:tool"), None, None);
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|t| t.subject == "cnv:tool"));
    }

    #[test]
    fn rdf_fragment_round_trips_through_json() {
        let mut fragment = RdfFragment::new();
        fragment
            .insert(SemanticTriple {
                subject: "cnv:tool".to_string(),
                predicate: "cnv:defaultVerb".to_string(),
                object: "cnv:run".to_string(),
            })
            .expect("valid triple");

        let json = serde_json::to_string(&fragment).expect("serializable");
        let restored: RdfFragment = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, fragment);
    }

    #[test]
    fn rdf_composition_dedupes_a_shared_triple_across_two_fragments() {
        let shared = SemanticTriple {
            subject: "cnv:tool".to_string(),
            predicate: "cnv:defaultVerb".to_string(),
            object: "cnv:run".to_string(),
        };
        let mut left = RdfFragment::new();
        left.insert(shared.clone()).expect("valid triple");

        let mut right = RdfFragment::new();
        right.insert(shared).expect("valid triple");
        right
            .insert(SemanticTriple {
                subject: "cnv:tool".to_string(),
                predicate: "rdf:type".to_string(),
                object: "cnv:Noun".to_string(),
            })
            .expect("valid triple");

        let composed = left.compose(&right);
        assert_eq!(composed.triples().len(), 2, "the shared triple must not be duplicated");
    }

    // -------------------------------------------------------------------
    // DiscoveryEngine: search_all_tags/search_by_route
    // -------------------------------------------------------------------

    #[test]
    fn discovery_engine_search_all_tags_uses_and_semantics_across_tags() {
        let mut engine = DiscoveryEngine::default();
        engine
            .register(DiscoveryRecord {
                name: "billing".to_string(),
                tags: ["billing", "read-only"].into_iter().map(str::to_string).collect(),
                route: "svc://billing".to_string(),
            })
            .expect("valid record");
        engine
            .register(DiscoveryRecord {
                name: "refunds".to_string(),
                tags: ["billing"].into_iter().map(str::to_string).collect(),
                route: "svc://refunds".to_string(),
            })
            .expect("valid record");

        assert_eq!(engine.search_all_tags(&["billing", "read-only"]).len(), 1);
        assert_eq!(engine.search_all_tags(&["billing", "read-only"])[0].name, "billing");
        assert_eq!(engine.search_all_tags(&["billing"]).len(), 2);
        assert_eq!(engine.search_all_tags(&[]).len(), 2, "vacuous AND over no terms matches all");
    }

    #[test]
    fn discovery_engine_search_by_route_matches_a_shared_route_and_refuses_unknown() {
        let mut engine = DiscoveryEngine::default();
        engine
            .register(DiscoveryRecord {
                name: "primary".to_string(),
                tags: BTreeSet::new(),
                route: "svc://shared".to_string(),
            })
            .expect("valid record");
        engine
            .register(DiscoveryRecord {
                name: "replica".to_string(),
                tags: BTreeSet::new(),
                route: "svc://shared".to_string(),
            })
            .expect("valid record");

        assert_eq!(engine.search_by_route("svc://shared").len(), 2);
        assert!(engine.search_by_route("svc://missing").is_empty());
    }

    // -------------------------------------------------------------------
    // LearningTrajectory::reset
    // -------------------------------------------------------------------

    #[test]
    fn learning_trajectory_reset_clears_history_and_a_fresh_observation_starts_at_zero() {
        let mut trajectory = LearningTrajectory::default();
        trajectory.observe(0.2).expect("valid score");
        trajectory.observe(0.4).expect("valid score");
        trajectory.observe(0.6).expect("valid score");

        trajectory.reset();
        assert!(trajectory.is_empty());
        assert_eq!(trajectory.len(), 0);
        assert_eq!(trajectory.mean(), None);
        assert_eq!(trajectory.latest(), None);

        let sequence = trajectory.observe(0.9).expect("valid score");
        assert_eq!(sequence, 0, "the underlying history must actually be cleared, not just hidden");
    }

    // -------------------------------------------------------------------
    // ReflexiveReport::merge/pass_rate
    // -------------------------------------------------------------------

    #[test]
    fn reflexive_report_merge_ands_replay_verified_and_sums_counts() {
        let a = ReflexiveReport { passed: 10, failed: 0, replay_verified: true };
        let b = ReflexiveReport { passed: 5, failed: 2, replay_verified: false };
        let merged = a.merge(&b);
        assert_eq!(merged.passed, 15);
        assert_eq!(merged.failed, 2);
        assert!(!merged.replay_verified, "AND semantics, not OR");
        assert!(!merged.is_alive());
    }

    #[test]
    fn reflexive_report_pass_rate_computes_fraction_and_handles_empty_run() {
        let report = ReflexiveReport { passed: 97, failed: 3, replay_verified: true };
        assert!((report.pass_rate() - 0.97).abs() < 1e-9);

        let empty = ReflexiveReport { passed: 0, failed: 0, replay_verified: false };
        assert_eq!(empty.pass_rate(), 0.0);
    }

    // -------------------------------------------------------------------
    // QuantumReadyPolicy::restricted/admitted
    // -------------------------------------------------------------------

    #[test]
    fn quantum_ready_policy_restricted_admits_only_given_algorithms() {
        let policy = QuantumReadyPolicy::restricted([QuantumAlgorithm::MlKem]);
        assert!(policy.admits(QuantumAlgorithm::MlKem));
        assert!(!policy.admits(QuantumAlgorithm::MlDsa));
        assert!(!policy.admits(QuantumAlgorithm::SlhDsa));
    }

    #[test]
    fn quantum_ready_policy_restricted_empty_admits_nothing() {
        let policy = QuantumReadyPolicy::restricted(std::iter::empty());
        assert!(policy.admitted().is_empty());
    }

    #[test]
    fn quantum_ready_policy_admitted_enumerates_post_quantum_set() {
        let admitted = QuantumReadyPolicy::post_quantum().admitted();
        assert_eq!(admitted.len(), 3);
        assert!(admitted.contains(&QuantumAlgorithm::MlKem));
        assert!(admitted.contains(&QuantumAlgorithm::MlDsa));
        assert!(admitted.contains(&QuantumAlgorithm::SlhDsa));
    }

    // -------------------------------------------------------------------
    // VickreyAuction: reject duplicate-agent bids (real truthfulness bug)
    // -------------------------------------------------------------------

    #[test]
    fn vickrey_auction_refuses_a_duplicate_agent_bid_instead_of_using_its_own_second_bid() {
        let mut auction = VickreyAuction::new();
        let bids = [
            Bid { agent_id: AgentId(1), task_id: TaskId(1), bid_value: 100.0 },
            Bid { agent_id: AgentId(1), task_id: TaskId(1), bid_value: 90.0 },
            Bid { agent_id: AgentId(2), task_id: TaskId(1), bid_value: 10.0 },
        ];
        let error = auction
            .run_auction(&bids)
            .expect_err("an agent bidding twice must be refused, not silently priced");
        assert!(error.contains("one bid per agent") || error.contains("one bid per auction"));
    }

    #[test]
    fn vickrey_auction_breaks_an_exact_bid_tie_by_lowest_agent_id_regardless_of_order() {
        // Locks in the tie-break direction (`then_with(|| left.agent_id.cmp(&right.agent_id))`
        // in `run_auction`'s sort comparator) against a future refactor -- no
        // existing test used an exact tie, so this direction was untested.
        for bids in [
            vec![
                Bid { agent_id: AgentId(5), task_id: TaskId(1), bid_value: 7.0 },
                Bid { agent_id: AgentId(2), task_id: TaskId(1), bid_value: 7.0 },
            ],
            vec![
                Bid { agent_id: AgentId(2), task_id: TaskId(1), bid_value: 7.0 },
                Bid { agent_id: AgentId(5), task_id: TaskId(1), bid_value: 7.0 },
            ],
        ] {
            let outcome = VickreyAuction::new().run_auction(&bids).expect("valid auction");
            assert_eq!(outcome.winner, AgentId(2));
            assert_eq!(outcome.payment, 7.0);
        }
    }

    // -------------------------------------------------------------------
    // EconomicSimulation::step consumes allocated tasks
    // -------------------------------------------------------------------

    #[test]
    fn economic_simulation_step_does_not_double_allocate_the_same_task_on_a_second_call() {
        let mut sim = EconomicSimulation::new();
        sim.add_agent(Agent {
            id: AgentId(1),
            capabilities: vec!["compute".to_string()],
            trust_score: 0.9,
            valuation: 100.0,
        })
        .expect("valid agent");
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 150.0,
        })
        .expect("valid task");

        let first = sim.step().expect("first step succeeds");
        assert_eq!(first.len(), 1);
        let second = sim.step().expect("second step succeeds");
        assert!(second.is_empty(), "the task was already allocated and consumed");
        assert_eq!(sim.allocations().len(), 1);
    }

    #[test]
    fn economic_simulation_step_breaks_a_trust_tie_by_lowest_agent_id() {
        // `self.agents` is a BTreeMap keyed by AgentId, so `.values()` always
        // iterates in ascending id order regardless of insertion order --
        // `step`'s `max_by` comparator reverses the id comparison
        // (`then_with(|| right.id.cmp(&left.id))`) specifically so an exact
        // trust-score tie still resolves to the lowest id. Insertion order
        // here (3, 1, 2) guards against that reversal being "corrected" into
        // a silent highest-id-wins flip in a future refactor.
        let mut sim = EconomicSimulation::new();
        for id in [3u64, 1, 2] {
            sim.add_agent(Agent {
                id: AgentId(id),
                capabilities: vec!["compute".to_string()],
                trust_score: 0.9,
                valuation: 10.0,
            })
            .expect("valid agent");
        }
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 1.0,
        })
        .expect("valid task");

        let produced = sim.step().expect("step succeeds");
        assert_eq!(produced[0].agent_id, AgentId(1));
    }

    // -------------------------------------------------------------------
    // FederatedNetwork::add_peer/remove_peer
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn federated_network_add_and_remove_peer_actually_mutate_the_registry() {
        let network = FederatedNetwork::new("node-a").expect("valid node id");
        let before = network.discover_peers().await.expect("real peer list");
        assert_eq!(before.len(), 3);

        let target = before[0].clone();
        let removed = network.remove_peer(&target).await.expect("lock not poisoned");
        assert!(removed);
        let after_remove = network.discover_peers().await.expect("real peer list");
        assert_eq!(after_remove.len(), 2);
        assert!(!after_remove.contains(&target));

        // Removing again is a real no-op, not an error.
        let removed_again = network.remove_peer(&target).await.expect("lock not poisoned");
        assert!(!removed_again);

        let fresh_peer = PeerId("node-a-peer-fresh".to_string());
        let added = network.add_peer(fresh_peer.clone()).await.expect("lock not poisoned");
        assert!(added);
        let after_add = network.discover_peers().await.expect("real peer list");
        assert_eq!(after_add.len(), 3);
        assert!(after_add.contains(&fresh_peer));

        // Adding the same peer again is idempotent, not an error.
        let added_again = network.add_peer(fresh_peer).await.expect("lock not poisoned");
        assert!(!added_again);
    }

    // -------------------------------------------------------------------
    // CompositionChain::entries
    // -------------------------------------------------------------------

    #[test]
    fn composition_chain_entries_returns_the_real_ordered_segments() {
        let mut chain = CompositionChain::new();
        chain.push("auth");
        chain.push("user");
        assert_eq!(chain.entries(), &["auth".to_string(), "user".to_string()]);
    }

    // -------------------------------------------------------------------
    // SpecificationSuite: names/remove_spec/len/is_empty
    // -------------------------------------------------------------------

    #[test]
    fn specification_suite_enumeration_and_removal_reflect_real_state() {
        let mut suite = SpecificationSuite::default();
        suite.add_spec(ExecutableSpec::new("Spec 1", "First"));
        suite.add_spec(ExecutableSpec::new("Spec 2", "Second"));

        let names: Vec<&str> = suite.names().collect();
        assert_eq!(names, vec!["Spec 1", "Spec 2"]);
        assert_eq!(suite.len(), 2);
        assert!(!suite.is_empty());

        let removed = suite.remove_spec("Spec 1").expect("registered spec");
        assert_eq!(removed.name(), "Spec 1");
        assert!(suite.get_spec("Spec 1").is_err());
        assert_eq!(suite.len(), 1);
    }

    // -------------------------------------------------------------------
    // ExecutableSpec::parameter_value (safe accessor vs. the raw-index
    // panic boundary the validate() predicate is otherwise exposed to)
    // -------------------------------------------------------------------

    #[test]
    fn executable_spec_parameter_value_avoids_the_raw_index_panic_for_a_missing_key() {
        let spec = ExecutableSpec::new("Partial", "Only default params");
        assert_eq!(spec.parameter_value("total_nodes"), Some(10));
        assert_eq!(spec.parameter_value("nonexistent_key"), None);

        let result = spec.validate(|_| spec.parameter_value("nonexistent_key").is_none());
        assert!(result.is_ok());
        assert!(result.expect("checked Ok above"));
    }

    #[test]
    #[should_panic]
    fn executable_spec_validate_predicate_indexing_a_missing_key_panics() {
        let spec = ExecutableSpec::new("Partial", "Only default params");
        // Documents the existing contract: indexing a BTreeMap with an
        // absent key panics rather than surfacing as the Err the
        // FrontierResult signature implies -- exactly why parameter_value
        // exists as the safe alternative.
        let _ = spec.validate(|params| params["nonexistent_key"] == 0);
    }
}
