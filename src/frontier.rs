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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    ///
    /// With three or more genuinely distinct bids, the winner pays the
    /// second-**highest** bid overall, not merely "the other bid" -- a
    /// distinction only a 3+ bidder example can demonstrate, since with
    /// exactly two bidders "the other bid" and "the second-highest bid"
    /// are the same value.
    ///
    /// # Examples
    ///
    /// ```
    /// use clap_noun_verb::frontier::{AgentId, Bid, TaskId, VickreyAuction};
    ///
    /// # fn main() -> Result<(), String> {
    /// let mut auction = VickreyAuction::new();
    /// let outcome = auction.run_auction(&[
    ///     Bid { agent_id: AgentId(1), task_id: TaskId(7), bid_value: 100.0 },
    ///     Bid { agent_id: AgentId(2), task_id: TaskId(7), bid_value: 80.0 },
    ///     Bid { agent_id: AgentId(3), task_id: TaskId(7), bid_value: 90.0 },
    /// ])?;
    ///
    /// // Agent 1 wins (highest bid) but pays agent 3's bid (the
    /// // second-highest overall), not agent 2's lower bid.
    /// assert_eq!(outcome.winner, AgentId(1));
    /// assert_eq!(outcome.payment, 90.0);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if fewer than two bids are supplied, if any bid
    /// targets a different task or carries a non-finite or negative value,
    /// or if the same agent submits more than one bid.
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    ///
    /// # Errors
    /// Returns an error if the agent's trust/valuation are out of bounds,
    /// or if `agent.id` is already registered.
    pub fn add_agent(&mut self, agent: Agent) -> FrontierResult<()> {
        if !agent.trust_score.is_finite()
            || !(0.0..=1.0).contains(&agent.trust_score)
            || !agent.valuation.is_finite()
            || agent.valuation < 0.0
        {
            return Err("agent trust and valuation are outside admitted bounds".into());
        }
        // Check-before-insert, not insert-then-check-the-return-value:
        // `BTreeMap::insert` on an existing key REPLACES the value and
        // returns the old one -- checking `.is_some()` on that return
        // value happens only after the overwrite already occurred, so a
        // rejected duplicate would still have clobbered the original
        // agent's data before this method's `Err` is even returned.
        if self.agents.contains_key(&agent.id) {
            return Err("duplicate agent id".into());
        }
        self.agents.insert(agent.id, agent);
        Ok(())
    }

    /// Add one unique valid task.
    ///
    /// # Errors
    /// Returns an error if the task's capability/value are out of bounds,
    /// or if `task.id` is already registered.
    pub fn add_task(&mut self, task: Task) -> FrontierResult<()> {
        if task.required_capability.trim().is_empty() || !task.value.is_finite() || task.value < 0.0
        {
            return Err("task capability and value are outside admitted bounds".into());
        }
        // Same check-before-insert fix as `add_agent`, for the same reason.
        if self.tasks.contains_key(&task.id) {
            return Err("duplicate task id".into());
        }
        self.tasks.insert(task.id, task);
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    fn discovery_engine_names_lists_every_registered_capability_in_canonical_order() {
        // Registered in scrambled (non-alphabetical) order so this test
        // cannot pass by coincidence: with registration already in
        // alphabetical order (as this test used to do), insertion order
        // and the documented "canonical order" are the same vector and a
        // regression from the current BTreeMap-backed sorted behavior to
        // a plain insertion-order-preserving structure (e.g. a `Vec` or
        // `IndexMap` swapped in for performance) would go undetected.
        let mut engine = DiscoveryEngine::default();
        for name in ["zeta", "alpha", "mango"] {
            engine
                .register(DiscoveryRecord {
                    name: name.to_string(),
                    tags: BTreeSet::new(),
                    route: format!("svc://{name}"),
                })
                .expect("valid record");
        }
        assert_eq!(
            engine.names(),
            vec!["alpha", "mango", "zeta"],
            "names() must be in canonical (sorted) order regardless of registration order"
        );
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

    #[test]
    fn discovery_engine_recommend_and_learning_trajectory_observe_close_the_real_loop_and_converge_on_the_best_mean_capability(
    ) {
        // Three registered capabilities with distinct, deterministic
        // (constant, non-random) long-run mean rewards -- "gamma" is the
        // real best performer. Every existing `recommend` test in this
        // module calls `recommend` exactly once against an
        // already-fixed, hand-constructed set of histories; this test
        // instead runs the real closed loop the module's own doc
        // comments describe as the actual use case: recommend, observe
        // based on the recommendation, recommend again with the updated
        // history -- repeated across many rounds.
        let mut engine = DiscoveryEngine::default();
        for name in ["alpha", "beta", "gamma"] {
            engine
                .register(DiscoveryRecord {
                    name: name.to_string(),
                    tags: BTreeSet::new(),
                    route: format!("svc://{name}"),
                })
                .expect("valid record");
        }

        let mut alpha = LearningTrajectory::default();
        let mut beta = LearningTrajectory::default();
        let mut gamma = LearningTrajectory::default();

        // Deterministic constant scores (no RNG, no seed needed): alpha's
        // long-run mean is 0.2, beta's is 0.5, gamma's is 0.9.
        const ALPHA_SCORE: f64 = 0.2;
        const BETA_SCORE: f64 = 0.5;
        const GAMMA_SCORE: f64 = 0.9;
        const ROUNDS: usize = 60;

        let mut selections: Vec<String> = Vec::with_capacity(ROUNDS);
        for _ in 0..ROUNDS {
            let histories = vec![
                ("alpha".to_string(), alpha.clone()),
                ("beta".to_string(), beta.clone()),
                ("gamma".to_string(), gamma.clone()),
            ];
            let recommended =
                engine.recommend(&histories).expect("valid recommendation").to_string();
            match recommended.as_str() {
                "alpha" => alpha.observe(ALPHA_SCORE).expect("valid score"),
                "beta" => beta.observe(BETA_SCORE).expect("valid score"),
                "gamma" => gamma.observe(GAMMA_SCORE).expect("valid score"),
                other => unreachable!("recommend only returns a registered name: {other}"),
            };
            selections.push(recommended);
        }

        // Every round produced exactly one real pull -- no round was
        // skipped or double-counted.
        assert_eq!(alpha.len() + beta.len() + gamma.len(), ROUNDS);

        // The real, closed-loop-observed means match the constant scores
        // fed back in -- confirms `observe` actually persisted what
        // `recommend` chose, round over round.
        assert!((gamma.mean().expect("at least one pull") - GAMMA_SCORE).abs() < 1e-9);
        assert!((beta.mean().expect("at least one pull") - BETA_SCORE).abs() < 1e-9);
        assert!((alpha.mean().expect("at least one pull") - ALPHA_SCORE).abs() < 1e-9);

        // Real UCB1 convergence property: the best-mean capability's
        // share of recommendations in the second half of the run is
        // strictly higher than its share in the first half -- the policy
        // recommends the best-performing capability *more often as
        // rounds increase*, not merely "often overall".
        let gamma_first_half = selections[0..30].iter().filter(|n| n.as_str() == "gamma").count();
        let gamma_second_half = selections[30..60].iter().filter(|n| n.as_str() == "gamma").count();
        assert_eq!(gamma_first_half, 19, "locks in the real, reproduced first-half count");
        assert_eq!(gamma_second_half, 22, "locks in the real, reproduced second-half count");
        assert!(
            gamma_second_half > gamma_first_half,
            "gamma's recommendation share must grow as rounds increase: {gamma_first_half} -> {gamma_second_half}"
        );

        // Over the full run, gamma (the real best mean) dominates
        // recommendations by a wide margin over both worse options.
        let gamma_total = selections.iter().filter(|n| n.as_str() == "gamma").count();
        let alpha_total = selections.iter().filter(|n| n.as_str() == "alpha").count();
        let beta_total = selections.iter().filter(|n| n.as_str() == "beta").count();
        assert_eq!((alpha_total, beta_total, gamma_total), (7, 12, 41));
        assert!(gamma_total > alpha_total + beta_total);
    }

    // -------------------------------------------------------------------
    // MetaFramework: layers()/invariant()/invariants(), remove_layer/
    // remove_invariant, and Blocked-state coverage (frontier-gap-sweep)
    // -------------------------------------------------------------------

    #[test]
    fn meta_framework_layers_and_invariant_accessors_report_real_registered_state() {
        let mut framework = MetaFramework::new();
        // Registered in reverse-of-canonical order ("execution" before
        // "admission") so the sorted-order assertion below is actually
        // exercised: registering "admission" first (as this test used to
        // do) makes insertion order and canonical (sorted) order the same
        // vector, so a regression from the current BTreeSet-backed sorted
        // behavior to a plain insertion-order-preserving structure would
        // go undetected.
        framework.register_layer("execution").expect("unique layer");
        framework.register_layer("admission").expect("unique layer");
        framework
            .admit_invariant(Invariant {
                id: "zero-unreceipted-actuation".to_string(),
                description: "Every effect has a receipt".to_string(),
                satisfied: true,
            })
            .expect("valid invariant");

        assert_eq!(
            framework.layers(),
            vec!["admission", "execution"],
            "layers() must be in canonical (sorted) order, not registration order"
        );
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

    #[test]
    fn meta_framework_register_layer_refuses_a_duplicate_name_and_leaves_it_registered_once() {
        let mut framework = MetaFramework::new();
        framework.register_layer("admission").expect("first registration is unique");

        let error = framework
            .register_layer("admission")
            .expect_err("a duplicate layer name must be refused");
        assert!(error.contains("layer already registered"));
        assert!(error.contains("admission"));

        // The rejected re-registration must not have duplicated the
        // entry: exactly one "admission" layer is registered, not two
        // collapsed by luck of `BTreeSet`'s own dedup semantics without
        // this path ever having been exercised by a test.
        assert_eq!(framework.layers(), vec!["admission"]);
    }

    #[test]
    fn meta_framework_admit_invariant_silently_overwrites_a_duplicate_id_unlike_every_other_identity_registration(
    ) {
        let mut framework = MetaFramework::new();
        framework
            .admit_invariant(Invariant {
                id: "zero-unreceipted-actuation".to_string(),
                description: "Every effect has a receipt".to_string(),
                satisfied: true,
            })
            .expect("first admission of this id is valid");

        // Unlike `register_layer` (refuses a duplicate name),
        // `DiscoveryEngine::register` (refuses a duplicate name), and
        // `EconomicSimulation::add_agent`/`add_task` (explicit
        // check-before-insert to refuse a duplicate id), `admit_invariant`
        // calls `BTreeMap::insert` unconditionally and never inspects its
        // return value: a second admission under the same id is accepted
        // with `Ok(())`, and the original invariant's `description`/
        // `satisfied` fields are silently replaced.
        framework
            .admit_invariant(Invariant {
                id: "zero-unreceipted-actuation".to_string(),
                description: "a completely different, contradictory claim".to_string(),
                satisfied: false,
            })
            .expect("admit_invariant never refuses a duplicate id -- it always returns Ok");

        let current = framework.invariant("zero-unreceipted-actuation").expect("still registered");
        assert_eq!(current.description, "a completely different, contradictory claim");
        assert!(!current.satisfied, "the original `satisfied: true` was silently overwritten");

        // Exactly one invariant exists under this id -- the first
        // admission left no trace, not even a shadowed duplicate entry.
        assert_eq!(framework.invariants().len(), 1);
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

    #[test]
    fn reflexive_report_merge_is_associative_and_order_independent_across_three_reports() {
        // Three independent runs, as a CI matrix would produce (e.g. three
        // shards, or an initial run plus two replay runs): two verified,
        // one not, so the AND-reduction is actually exercised rather than
        // trivially staying `true` throughout.
        let a = ReflexiveReport { passed: 10, failed: 1, replay_verified: true };
        let b = ReflexiveReport { passed: 20, failed: 0, replay_verified: true };
        let c = ReflexiveReport { passed: 7, failed: 3, replay_verified: false };

        // `merge` is field-wise `+` on `passed`/`failed` (associative and
        // commutative on `u64`, no overflow at these magnitudes) and `&&`
        // on `replay_verified` (associative and commutative on `bool`), so
        // every grouping and every order of combination must fold to the
        // same result when three or more reports are merged.
        let left_fold = a.merge(&b).merge(&c);
        let right_fold = a.merge(&b.merge(&c));
        let reordered = b.merge(&c).merge(&a);
        let reordered_again = c.merge(&a).merge(&b);

        let expected = ReflexiveReport { passed: 37, failed: 4, replay_verified: false };
        assert_eq!(left_fold, expected);
        assert_eq!(right_fold, expected, "left-fold and right-fold must agree (associativity)");
        assert_eq!(reordered, expected, "merge order must not change the folded result");
        assert_eq!(reordered_again, expected, "merge order must not change the folded result");

        // `pass_rate` is a pure function of the final (passed, failed)
        // totals -- it is not itself merged or averaged incrementally --
        // so it is order-independent for the same reason the totals
        // themselves are: computed after any valid merge chain, it agrees
        // with the rate computed on the same totals built any other way.
        let expected_rate = expected.pass_rate();
        assert!((left_fold.pass_rate() - expected_rate).abs() < 1e-9);
        assert!((right_fold.pass_rate() - expected_rate).abs() < 1e-9);
        assert!((reordered.pass_rate() - expected_rate).abs() < 1e-9);
        assert!((reordered_again.pass_rate() - expected_rate).abs() < 1e-9);
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
    // VickreyAuction: bid-value validation and single-bid refusal
    // (frontier-gap-sweep: run_auction already validates these -- see
    // src/frontier.rs:748 and :754 -- but no existing test exercised a
    // negative bid, a zero bid, or a single-/zero-bid auction.)
    // -------------------------------------------------------------------

    #[test]
    fn vickrey_auction_refuses_a_negative_bid_value() {
        let mut auction = VickreyAuction::new();
        let bids = [
            Bid { agent_id: AgentId(1), task_id: TaskId(1), bid_value: 100.0 },
            Bid { agent_id: AgentId(2), task_id: TaskId(1), bid_value: -5.0 },
        ];
        let error = auction
            .run_auction(&bids)
            .expect_err("a negative bid value must be refused, not silently priced");
        assert!(error.contains("finite non-negative values"));
    }

    #[test]
    fn vickrey_auction_allows_and_prices_a_zero_bid_as_the_second_price() {
        // `bid_value < 0.0` is false for exactly 0.0, so a zero bid is
        // admitted -- this locks in that boundary rather than leaving it
        // implicit.
        let mut auction = VickreyAuction::new();
        let bids = [
            Bid { agent_id: AgentId(1), task_id: TaskId(1), bid_value: 50.0 },
            Bid { agent_id: AgentId(2), task_id: TaskId(1), bid_value: 0.0 },
        ];
        let outcome = auction.run_auction(&bids).expect("a zero bid is admitted, not refused");
        assert_eq!(outcome.winner, AgentId(1));
        assert_eq!(outcome.payment, 0.0);
    }

    #[test]
    fn vickrey_auction_refuses_zero_and_single_bid_auctions() {
        // With fewer than two bids there is no second price to compare
        // against, so `run_auction` refuses outright rather than, say,
        // charging the sole bidder its own bid.
        for bids in [
            Vec::<Bid>::new(),
            vec![Bid { agent_id: AgentId(1), task_id: TaskId(1), bid_value: 100.0 }],
        ] {
            let error = VickreyAuction::new()
                .run_auction(&bids)
                .expect_err("fewer than two bids must be refused");
            assert!(error.contains("at least two bids"));
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
    // EconomicSimulation::step with no capability match, and with
    // multiple simultaneous tasks (frontier-gap-sweep)
    // -------------------------------------------------------------------

    #[test]
    fn economic_simulation_step_leaves_a_task_pending_when_no_agent_has_the_required_capability() {
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
            required_capability: "compile".to_string(),
            value: 50.0,
        })
        .expect("valid task");

        // No agent advertises "compile", so this step must silently
        // produce zero allocations -- not an error -- and the task must
        // remain pending rather than being dropped.
        let produced = sim.step().expect("a step with no eligible agent still succeeds");
        assert!(produced.is_empty());
        assert!(sim.allocations().is_empty());
        assert_eq!(sim.time, 1.0, "logical time still advances even with no allocation");

        // A second step with still no capable agent must behave
        // identically -- the unmatched task is not silently lost.
        let second = sim.step().expect("second step still succeeds");
        assert!(second.is_empty());
        assert_eq!(sim.time, 2.0);

        // Once a capable agent is admitted, the same task the earlier
        // steps left pending is allocated -- proving it was retained,
        // not dropped, while no agent matched.
        sim.add_agent(Agent {
            id: AgentId(2),
            capabilities: vec!["compile".to_string()],
            trust_score: 0.5,
            valuation: 10.0,
        })
        .expect("valid agent");
        let third = sim.step().expect("third step succeeds");
        assert_eq!(third.len(), 1);
        assert_eq!(third[0].agent_id, AgentId(2));
        assert_eq!(third[0].task_id, TaskId(1));
    }

    #[test]
    fn economic_simulation_step_allocates_disjoint_capability_tasks_independently_in_one_call() {
        let mut sim = EconomicSimulation::new();
        sim.add_agent(Agent {
            id: AgentId(1),
            capabilities: vec!["compute".to_string()],
            trust_score: 0.9,
            valuation: 100.0,
        })
        .expect("valid agent");
        sim.add_agent(Agent {
            id: AgentId(2),
            capabilities: vec!["network".to_string()],
            trust_score: 0.9,
            valuation: 100.0,
        })
        .expect("valid agent");
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 100.0,
        })
        .expect("valid task");
        sim.add_task(Task {
            id: TaskId(2),
            required_capability: "network".to_string(),
            value: 50.0,
        })
        .expect("valid task");

        let produced = sim.step().expect("step succeeds");
        assert_eq!(produced.len(), 2, "both simultaneous tasks must be allocated in one step");
        assert_eq!(produced[0].task_id, TaskId(1));
        assert_eq!(produced[0].agent_id, AgentId(1));
        assert_eq!(produced[1].task_id, TaskId(2));
        assert_eq!(produced[1].agent_id, AgentId(2));

        // Both tasks were consumed -- a further step produces nothing.
        let after = sim.step().expect("second step succeeds");
        assert!(after.is_empty());
        assert_eq!(sim.allocations().len(), 2);
    }

    #[test]
    fn economic_simulation_step_can_allocate_the_same_top_trust_agent_to_every_contending_task() {
        // `step` selects independently per task and never removes the
        // chosen agent from consideration for the remaining tasks in the
        // same call -- so two tasks requiring the same capability both
        // go to the single highest-trust agent within one step, with no
        // per-step capacity limit. This is the module's real current
        // behavior; it is locked in here rather than left implicit.
        let mut sim = EconomicSimulation::new();
        sim.add_agent(Agent {
            id: AgentId(1),
            capabilities: vec!["compute".to_string()],
            trust_score: 0.9,
            valuation: 100.0,
        })
        .expect("valid agent");
        sim.add_agent(Agent {
            id: AgentId(2),
            capabilities: vec!["compute".to_string()],
            trust_score: 0.5,
            valuation: 100.0,
        })
        .expect("valid agent");
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 100.0,
        })
        .expect("valid task");
        sim.add_task(Task {
            id: TaskId(2),
            required_capability: "compute".to_string(),
            value: 50.0,
        })
        .expect("valid task");

        let produced = sim.step().expect("step succeeds");
        assert_eq!(produced.len(), 2);
        assert!(
            produced.iter().all(|allocation| allocation.agent_id == AgentId(1)),
            "the single higher-trust agent wins both contending tasks in one step"
        );
    }

    // -------------------------------------------------------------------
    // EconomicSimulation::add_agent/add_task: negative and zero bounds,
    // and duplicate-id state preservation (frontier-gap-sweep: the range
    // checks at src/frontier.rs:835-841/850-852 existed in production
    // code but were never exercised; the check-before-insert fix above
    // closes a real bug where a rejected duplicate id still silently
    // overwrote the original agent's/task's data via BTreeMap::insert's
    // replace-and-return-old semantics before the Err was returned.)
    // -------------------------------------------------------------------

    #[test]
    fn economic_simulation_add_agent_refuses_a_negative_valuation_and_does_not_insert_it() {
        let mut sim = EconomicSimulation::new();
        let error = sim
            .add_agent(Agent {
                id: AgentId(1),
                capabilities: vec!["compute".to_string()],
                trust_score: 0.5,
                valuation: -1.0,
            })
            .expect_err("a negative valuation must be refused");
        assert!(error.contains("outside admitted bounds"));
        assert_eq!(sim.agent_count(), 0, "the rejected agent must not be inserted");
    }

    #[test]
    fn economic_simulation_add_task_refuses_a_negative_value_and_does_not_insert_it() {
        let mut sim = EconomicSimulation::new();
        let error = sim
            .add_task(Task {
                id: TaskId(1),
                required_capability: "compute".to_string(),
                value: -10.0,
            })
            .expect_err("a negative task value must be refused");
        assert!(error.contains("outside admitted bounds"));

        // The rejected task must not occupy TaskId(1) -- a later, valid
        // task under the same id succeeds rather than colliding with a
        // half-admitted duplicate.
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 10.0,
        })
        .expect("the id was never actually occupied by the rejected task");
    }

    #[test]
    fn economic_simulation_add_agent_and_add_task_admit_exactly_zero_as_a_valid_bound() {
        // `valuation < 0.0` and `value < 0.0` are both false for exactly
        // 0.0, so a zero valuation/value is admitted, not refused -- this
        // locks in that boundary rather than leaving it implicit.
        let mut sim = EconomicSimulation::new();
        sim.add_agent(Agent {
            id: AgentId(1),
            capabilities: vec!["compute".to_string()],
            trust_score: 0.0,
            valuation: 0.0,
        })
        .expect("zero valuation and trust are within admitted bounds");
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 0.0,
        })
        .expect("zero task value is within admitted bounds");
        assert_eq!(sim.agent_count(), 1);
    }

    #[test]
    fn economic_simulation_add_agent_and_add_task_preserve_the_original_on_a_rejected_duplicate_id()
    {
        let mut sim = EconomicSimulation::new();
        sim.add_agent(Agent {
            id: AgentId(1),
            capabilities: vec!["compute".to_string()],
            trust_score: 0.9,
            valuation: 100.0,
        })
        .expect("valid agent");
        let error = sim
            .add_agent(Agent {
                id: AgentId(1),
                capabilities: vec!["network".to_string()],
                trust_score: 0.1,
                valuation: 5.0,
            })
            .expect_err("duplicate agent id must be refused");
        assert!(error.contains("duplicate agent id"));

        // The rejected re-registration must not have corrupted the
        // original agent's data -- an Err return must leave state
        // untouched, not silently overwrite it first.
        sim.add_task(Task {
            id: TaskId(1),
            required_capability: "compute".to_string(),
            value: 10.0,
        })
        .expect("valid task");
        let produced = sim.step().expect("step succeeds");
        assert_eq!(
            produced.len(),
            1,
            "the original agent's \"compute\" capability must still be present"
        );
        assert_eq!(produced[0].agent_id, AgentId(1));

        // Same defect, same fix, for add_task.
        sim.add_task(Task {
            id: TaskId(2),
            required_capability: "compute".to_string(),
            value: 50.0,
        })
        .expect("valid task");
        let duplicate_task_error = sim
            .add_task(Task {
                id: TaskId(2),
                required_capability: "network".to_string(),
                value: 999.0,
            })
            .expect_err("duplicate task id must be refused");
        assert!(duplicate_task_error.contains("duplicate task id"));
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
    // FederatedNetwork under real concurrent contention (frontier-gap-sweep):
    // every prior test above drives add_peer/remove_peer/discover_peers/
    // advertise_capability with sequential `.await`s on a single task --
    // none of them exercise the real `Arc<RwLock<...>>` registries under
    // genuine simultaneous access from more than one real thread/task.
    // -------------------------------------------------------------------

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn federated_network_add_peer_survives_real_concurrent_tasks_without_losing_or_duplicating_peers(
    ) {
        // Arrange: `FederatedNetwork::clone` is a shallow `Arc` clone (see
        // its `Arc<RwLock<...>>` fields), so every clone below is a real
        // handle onto the SAME shared peer registry -- exactly the shape
        // multiple concurrent dispatch tasks would produce in a real
        // caller.
        let network = FederatedNetwork::new("node-concurrent-add").expect("valid node id");
        let baseline = network.discover_peers().await.expect("real peer list");

        const TASKS: usize = 16;
        const PER_TASK: usize = 20;

        // Act: TASKS real concurrent async tasks (scheduled across the
        // multi-thread tokio runtime's real OS worker threads), each
        // adding PER_TASK distinct new peers to the SAME shared registry
        // at the same time -- not one after another.
        let mut handles = Vec::with_capacity(TASKS);
        for task_idx in 0..TASKS {
            let network = network.clone();
            handles.push(tokio::spawn(async move {
                for i in 0..PER_TASK {
                    let peer = PeerId(format!("concurrent-peer-{task_idx}-{i}"));
                    network.add_peer(peer).await.expect("lock not poisoned");
                }
            }));
        }
        for handle in handles {
            handle.await.expect("task must not panic");
        }

        // Assert: every one of the TASKS * PER_TASK distinct peers landed
        // exactly once -- a real race (e.g. a future refactor that checks
        // `contains` then writes without holding one lock across both
        // steps) would either lose peers or duplicate them.
        let after = network.discover_peers().await.expect("real peer list");
        assert_eq!(
            after.len(),
            baseline.len() + TASKS * PER_TASK,
            "no concurrently-added peer may be lost or duplicated"
        );
        let unique: BTreeSet<&PeerId> = after.iter().collect();
        assert_eq!(unique.len(), after.len(), "the final peer set must contain no duplicates");
        for task_idx in 0..TASKS {
            for i in 0..PER_TASK {
                let expected = PeerId(format!("concurrent-peer-{task_idx}-{i}"));
                assert!(after.contains(&expected), "peer {expected:?} must not be lost to a race");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn federated_network_add_remove_and_discover_peers_run_correctly_under_real_concurrent_contention(
    ) {
        // Arrange
        let network = FederatedNetwork::new("node-concurrent-mixed").expect("valid node id");
        let baseline = network.discover_peers().await.expect("real peer list");
        assert_eq!(baseline.len(), 3, "the constructor's own documented baseline peer count");

        const ADDERS: usize = 8;
        const READERS: usize = 8;

        // Act: real concurrent tasks simultaneously adding new peers,
        // removing every baseline peer, and repeatedly discovering the
        // peer set -- add_peer/remove_peer/discover_peers all hitting the
        // SAME shared registry at the same time.
        let mut handles = Vec::new();

        for adder_idx in 0..ADDERS {
            let network = network.clone();
            handles.push(tokio::spawn(async move {
                let peer = PeerId(format!("mixed-added-{adder_idx}"));
                network.add_peer(peer).await.expect("lock not poisoned");
            }));
        }
        for baseline_peer in baseline.clone() {
            let network = network.clone();
            handles.push(tokio::spawn(async move {
                network.remove_peer(&baseline_peer).await.expect("lock not poisoned");
            }));
        }
        for _ in 0..READERS {
            let network = network.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..25 {
                    // A reader must never observe a torn/partial write --
                    // every returned Vec is a real, fully-formed snapshot.
                    network.discover_peers().await.expect("lock not poisoned");
                }
            }));
        }

        for handle in handles {
            handle.await.expect("task must not panic");
        }

        // Assert: every baseline peer is gone, every concurrently-added
        // peer landed exactly once -- the real registry reflects every
        // real concurrent writer with no lost update and no duplicate.
        let after = network.discover_peers().await.expect("real peer list");
        for baseline_peer in &baseline {
            assert!(!after.contains(baseline_peer), "every baseline peer must have been removed");
        }
        assert_eq!(after.len(), ADDERS, "every concurrently-added peer must land exactly once");
        let unique: BTreeSet<&PeerId> = after.iter().collect();
        assert_eq!(unique.len(), after.len(), "no duplicate peers after concurrent contention");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn federated_network_advertise_capability_survives_real_concurrent_tasks_without_losing_any(
    ) {
        // Arrange
        let network = FederatedNetwork::new("node-concurrent-capability").expect("valid node id");

        const TASKS: usize = 16;

        // Act: real concurrent tasks advertising distinct capabilities
        // against the SAME shared `capabilities` registry -- a separate
        // `Arc<RwLock<_>>` field from `peers`, never exercised under
        // contention until now.
        let mut handles = Vec::with_capacity(TASKS);
        for task_idx in 0..TASKS {
            let mut network = network.clone();
            handles.push(tokio::spawn(async move {
                let capability = Capability {
                    name: format!("capability-{task_idx}"),
                    version: "1.0".to_string(),
                    endpoint: format!("local://capability-{task_idx}"),
                };
                network.advertise_capability(&capability).await.expect("lock not poisoned");
            }));
        }
        for handle in handles {
            handle.await.expect("task must not panic");
        }

        // Assert: every concurrently-advertised capability actually
        // resolves -- a lost update from a real race would leave some of
        // these absent.
        for task_idx in 0..TASKS {
            let resolved = network
                .resolve(&format!("capability-{task_idx}"))
                .unwrap_or_else(|e| panic!("capability-{task_idx} must have been advertised: {e}"));
            assert_eq!(resolved.endpoint, format!("local://capability-{task_idx}"));
        }
    }

    #[tokio::test]
    async fn federated_network_add_peer_surfaces_a_typed_error_when_the_peer_registry_lock_is_really_poisoned(
    ) {
        // Arrange: reach into the private `peers` field (legal here --
        // this test lives inside `frontier`'s own module) to poison the
        // REAL lock exactly the way `std::sync::RwLock` poisons for real:
        // a writer panicking while holding the write guard.
        let network = FederatedNetwork::new("node-poison").expect("valid node id");
        let peers_for_poisoning = network.peers.clone();

        let handle = std::thread::spawn(move || {
            let _guard = peers_for_poisoning.write().expect("lock not yet poisoned");
            panic!("intentional panic to poison the real lock for this test");
        });
        let joined = handle.join();
        assert!(
            joined.is_err(),
            "the spawned thread must have panicked (expected setup, not a test failure)"
        );

        // Act: both the write path (add_peer) and the read path
        // (discover_peers) must surface the real poison as a typed
        // FrontierResult error, never panic themselves -- confirming the
        // `.map_err(...)?` propagation this module relies on instead of
        // `.unwrap()` on the lock result.
        let add_result = network.add_peer(PeerId("after-poison".to_string())).await;
        let discover_result = network.discover_peers().await;

        // Assert
        let add_error =
            add_result.expect_err("a poisoned lock must surface as a typed Err, not a panic");
        assert!(add_error.contains("poisoned"), "error must name the real cause: {add_error}");
        let discover_error =
            discover_result.expect_err("a poisoned lock must surface as a typed Err on reads too");
        assert!(
            discover_error.contains("poisoned"),
            "error must name the real cause: {discover_error}"
        );
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
        // Added in reverse-of-canonical order ("Spec 2" before "Spec 1")
        // so the sorted-order assertion below is actually exercised:
        // adding "Spec 1" first (as this test used to do) makes insertion
        // order and canonical (sorted) order the same vector, so a
        // regression from the current BTreeMap-backed sorted behavior to
        // a plain insertion-order-preserving structure would go
        // undetected.
        suite.add_spec(ExecutableSpec::new("Spec 2", "Second"));
        suite.add_spec(ExecutableSpec::new("Spec 1", "First"));

        let names: Vec<&str> = suite.names().collect();
        assert_eq!(
            names,
            vec!["Spec 1", "Spec 2"],
            "names() must be in sorted order regardless of add_spec order"
        );
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

    // -------------------------------------------------------------------
    // Serialize/Deserialize round-trip coverage (frontier-serde-gap-sweep)
    //
    // Every type below derives Serialize + Deserialize but, before this
    // section, had zero round-trip test anywhere in the repo -- so a
    // silent regression (a stray `#[serde(skip)]`, a rename mismatch, a
    // field that stops round-tripping through a private-field type) would
    // not have been caught. `SemanticTriple`/`RdfFragment` are excluded
    // here: `rdf_fragment_round_trips_through_json` above already
    // round-trips a `RdfFragment` containing a real `SemanticTriple`.
    //
    // `MetaFramework`, `DiscoveryEngine`, `EconomicSimulation`, and
    // `SpecificationSuite` did not derive `PartialEq` at all (a real gap:
    // Serialize/Deserialize/Clone without PartialEq blocks exactly the
    // round-trip-then-assert_eq pattern these tests use) -- each now
    // derives `PartialEq` (plus `Eq` where every field supports it; the
    // three with an f64-bearing field transitively, `EconomicSimulation`,
    // stays `PartialEq`-only) so the gap is closed, not just noted.
    // -------------------------------------------------------------------

    #[test]
    fn admission_state_round_trips_every_variant_through_json() {
        for state in [
            AdmissionState::Unknown,
            AdmissionState::Admitted,
            AdmissionState::Alive,
            AdmissionState::Blocked,
        ] {
            let json = serde_json::to_string(&state).expect("serializable");
            let restored: AdmissionState = serde_json::from_str(&json).expect("deserializable");
            assert_eq!(restored, state);
        }
    }

    #[test]
    fn invariant_round_trips_through_json() {
        let invariant = Invariant {
            id: "zero-unreceipted-actuation".to_string(),
            description: "Every effect has a receipt".to_string(),
            satisfied: true,
        };
        let json = serde_json::to_string(&invariant).expect("serializable");
        let restored: Invariant = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, invariant);
    }

    #[test]
    fn meta_framework_round_trips_through_json_after_real_registration() {
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

        let json = serde_json::to_string(&framework).expect("serializable");
        let restored: MetaFramework = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, framework);
        // Also confirm the restored value's own public accessors still see
        // the same state (would catch a rename that fooled struct equality
        // but broke lookup by field name).
        assert_eq!(restored.layers(), vec!["admission", "execution"]);
        assert!(restored.invariant("zero-unreceipted-actuation").is_some());
    }

    #[test]
    fn discovery_record_round_trips_through_json() {
        let record = DiscoveryRecord {
            name: "billing".to_string(),
            tags: ["billing", "read-only"].into_iter().map(str::to_string).collect(),
            route: "svc://billing".to_string(),
        };
        let json = serde_json::to_string(&record).expect("serializable");
        let restored: DiscoveryRecord = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, record);
    }

    #[test]
    fn discovery_engine_round_trips_through_json_after_real_registration() {
        let mut engine = DiscoveryEngine::default();
        engine
            .register(DiscoveryRecord {
                name: "billing".to_string(),
                tags: ["billing"].into_iter().map(str::to_string).collect(),
                route: "svc://billing".to_string(),
            })
            .expect("valid record");

        let json = serde_json::to_string(&engine).expect("serializable");
        let restored: DiscoveryEngine = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, engine);
        assert_eq!(restored.names(), vec!["billing"]);
    }

    #[test]
    fn learning_observation_round_trips_through_json() {
        let observation = LearningObservation { sequence: 3, score: 0.75 };
        let json = serde_json::to_string(&observation).expect("serializable");
        let restored: LearningObservation = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, observation);
    }

    #[test]
    fn learning_trajectory_round_trips_through_json_after_real_observations() {
        let mut trajectory = LearningTrajectory::default();
        trajectory.observe(0.2).expect("valid score");
        trajectory.observe(0.6).expect("valid score");

        let json = serde_json::to_string(&trajectory).expect("serializable");
        let restored: LearningTrajectory = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, trajectory);
        assert_eq!(restored.len(), 2);
    }

    #[test]
    fn reflexive_report_round_trips_through_json() {
        let report = ReflexiveReport { passed: 97, failed: 3, replay_verified: true };
        let json = serde_json::to_string(&report).expect("serializable");
        let restored: ReflexiveReport = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, report);
    }

    #[test]
    fn quantum_algorithm_round_trips_every_variant_through_json() {
        for algorithm in
            [QuantumAlgorithm::MlKem, QuantumAlgorithm::MlDsa, QuantumAlgorithm::SlhDsa]
        {
            let json = serde_json::to_string(&algorithm).expect("serializable");
            let restored: QuantumAlgorithm = serde_json::from_str(&json).expect("deserializable");
            assert_eq!(restored, algorithm);
        }
    }

    #[test]
    fn quantum_ready_policy_round_trips_through_json() {
        let policy = QuantumReadyPolicy::post_quantum();
        let json = serde_json::to_string(&policy).expect("serializable");
        let restored: QuantumReadyPolicy = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, policy);
        assert_eq!(restored.admitted().len(), 3);
    }

    #[test]
    fn peer_id_round_trips_through_json() {
        let peer = PeerId("node-a-peer-1".to_string());
        let json = serde_json::to_string(&peer).expect("serializable");
        // Newtype structs serialize transparently: the wire form is the
        // bare inner value, not a one-element array or object.
        assert_eq!(json, "\"node-a-peer-1\"");
        let restored: PeerId = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, peer);
    }

    #[test]
    fn capability_round_trips_through_json() {
        let capability = Capability {
            name: "sparql-query".to_string(),
            version: "1.1".to_string(),
            endpoint: "http://localhost:7878/sparql".to_string(),
        };
        let json = serde_json::to_string(&capability).expect("serializable");
        let restored: Capability = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, capability);
    }

    #[test]
    fn agent_id_and_task_id_round_trip_through_json_as_bare_numbers() {
        let agent_id = AgentId(42);
        let agent_json = serde_json::to_string(&agent_id).expect("serializable");
        assert_eq!(agent_json, "42");
        let restored_agent: AgentId = serde_json::from_str(&agent_json).expect("deserializable");
        assert_eq!(restored_agent, agent_id);

        let task_id = TaskId(7);
        let task_json = serde_json::to_string(&task_id).expect("serializable");
        assert_eq!(task_json, "7");
        let restored_task: TaskId = serde_json::from_str(&task_json).expect("deserializable");
        assert_eq!(restored_task, task_id);
    }

    #[test]
    fn bid_round_trips_through_json() {
        let bid = Bid { agent_id: AgentId(1), task_id: TaskId(7), bid_value: 10.0 };
        let json = serde_json::to_string(&bid).expect("serializable");
        let restored: Bid = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, bid);
    }

    #[test]
    fn auction_outcome_round_trips_through_json_from_a_real_auction() {
        let mut auction = VickreyAuction::new();
        let outcome = auction
            .run_auction(&[
                Bid { agent_id: AgentId(1), task_id: TaskId(7), bid_value: 10.0 },
                Bid { agent_id: AgentId(2), task_id: TaskId(7), bid_value: 8.0 },
            ])
            .expect("valid auction");

        let json = serde_json::to_string(&outcome).expect("serializable");
        let restored: AuctionOutcome = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, outcome);
    }

    #[test]
    fn agent_round_trips_through_json() {
        let agent = Agent {
            id: AgentId(1),
            capabilities: vec!["compute".to_string(), "storage".to_string()],
            trust_score: 0.9,
            valuation: 100.0,
        };
        let json = serde_json::to_string(&agent).expect("serializable");
        let restored: Agent = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, agent);
    }

    #[test]
    fn task_round_trips_through_json() {
        let task = Task { id: TaskId(1), required_capability: "compute".to_string(), value: 150.0 };
        let json = serde_json::to_string(&task).expect("serializable");
        let restored: Task = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, task);
    }

    #[test]
    fn allocation_round_trips_through_json() {
        let allocation = Allocation { task_id: TaskId(1), agent_id: AgentId(1) };
        let json = serde_json::to_string(&allocation).expect("serializable");
        let restored: Allocation = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, allocation);
    }

    #[test]
    fn economic_simulation_round_trips_through_json_after_a_real_step() {
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
        sim.step().expect("step succeeds");

        let json = serde_json::to_string(&sim).expect("serializable");
        let restored: EconomicSimulation = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, sim);
        assert_eq!(restored.allocations().len(), 1);
        assert_eq!(restored.agent_count(), 1);
    }

    #[test]
    fn composition_chain_round_trips_through_json_after_real_pushes() {
        let mut chain = CompositionChain::new();
        chain.push("auth");
        chain.push("user");

        let json = serde_json::to_string(&chain).expect("serializable");
        let restored: CompositionChain = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, chain);
        assert_eq!(restored.entries(), &["auth".to_string(), "user".to_string()]);
    }

    #[test]
    fn executable_spec_round_trips_through_json_including_private_parameters() {
        let spec = ExecutableSpec::new("Spec", "Description")
            .given("a precondition")
            .when("an action")
            .then("an outcome")
            .and("an invariant")
            .parameter("total_nodes", 12);

        let json = serde_json::to_string(&spec).expect("serializable");
        let restored: ExecutableSpec = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, spec);
        // The `parameters` field is private -- confirm it actually
        // round-tripped, not just the public fields, via the public
        // accessor (a serde skip on a private field would pass the
        // struct-equality check above only if every field were public).
        assert_eq!(restored.parameter_value("total_nodes"), Some(12));
        assert_eq!(restored.parameter_value("byzantine_nodes"), Some(3));
    }

    #[test]
    fn specification_suite_round_trips_through_json_after_real_add_spec() {
        let mut suite = SpecificationSuite::default();
        suite.add_spec(ExecutableSpec::new("Spec 1", "First"));
        suite.add_spec(ExecutableSpec::new("Spec 2", "Second").parameter("total_nodes", 5));

        let json = serde_json::to_string(&suite).expect("serializable");
        let restored: SpecificationSuite = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(restored, suite);
        let names: Vec<&str> = restored.names().collect();
        assert_eq!(names, vec!["Spec 1", "Spec 2"]);
    }

    // -------------------------------------------------------------------
    // Edge-case input handling (frontier-input-gap-sweep):
    //
    // - CompositionChain::push had zero test exercising a blank or
    //   whitespace-only segment, so its silent-drop behavior (and its
    //   choice to store the *untrimmed* original string for a segment
    //   that is non-blank) was unverified -- a caller constructing
    //   segments from user or programmatic input could hit either path.
    // - RdfFragment::insert had zero test exercising its
    //   `trim().is_empty()` guard at all (not even a plain empty string),
    //   despite the guard existing in production code since before this
    //   sweep.
    // - Every existing FractalNoun<L, T> test used T = &str or T =
    //   String; zero test confirmed compose() correctly moves a
    //   genuinely owned, non-Copy, non-Clone struct through the
    //   `lineage: Vec<String>` tracking without disturbing `data: T`
    //   (compose()'s body only ever reassigns `next.lineage`).
    //
    // Duplicate-triple-across-two-fragments composition is already
    // covered by `rdf_composition_dedupes_a_shared_triple_across_two_fragments`
    // above and is not re-proposed here.
    // -------------------------------------------------------------------

    #[test]
    fn composition_chain_push_silently_drops_blank_and_whitespace_only_segments() {
        let mut chain = CompositionChain::new();
        chain.push("");
        chain.push("   ");
        chain.push("\t\n");
        assert!(chain.is_empty(), "blank/whitespace-only segments must not be pushed");
        assert_eq!(chain.len(), 0);

        // A non-blank segment pushed alongside the dropped ones still
        // lands normally, confirming the guard only ever skips the
        // blank/whitespace-only case rather than corrupting later pushes.
        chain.push("auth");
        assert_eq!(chain.entries(), &["auth".to_string()]);
    }

    #[test]
    fn composition_chain_push_preserves_untrimmed_whitespace_in_a_non_blank_segment() {
        let mut chain = CompositionChain::new();
        chain.push("  auth  ");
        // push only rejects a segment that is blank *after* trimming; it
        // does not itself trim before storing, so the stored entry keeps
        // the original padding verbatim rather than the trimmed form.
        assert_eq!(chain.entries(), &["  auth  ".to_string()]);
        assert_ne!(chain.entries()[0], "auth");
    }

    #[test]
    fn rdf_fragment_insert_refuses_a_whitespace_only_subject_predicate_or_object() {
        let mut fragment = RdfFragment::new();

        let whitespace_subject = SemanticTriple {
            subject: "   ".to_string(),
            predicate: "cnv:defaultVerb".to_string(),
            object: "cnv:run".to_string(),
        };
        assert!(fragment.insert(whitespace_subject).is_err());

        let whitespace_predicate = SemanticTriple {
            subject: "cnv:tool".to_string(),
            predicate: "\t".to_string(),
            object: "cnv:run".to_string(),
        };
        assert!(fragment.insert(whitespace_predicate).is_err());

        let empty_object = SemanticTriple {
            subject: "cnv:tool".to_string(),
            predicate: "cnv:defaultVerb".to_string(),
            object: String::new(),
        };
        assert!(fragment.insert(empty_object).is_err());

        // None of the rejected triples were admitted into real state.
        assert!(fragment.triples().is_empty());
    }

    #[test]
    fn fractal_noun_compose_moves_a_non_string_owned_payload_through_lineage_untouched() {
        // A genuinely owned, non-Copy, non-Clone struct -- not the &str
        // or String every other FractalNoun test uses -- to confirm
        // compose() carries T through by move without requiring any
        // bound on T beyond what the impl block already declares.
        struct Payload {
            id: String,
            bytes: Vec<u8>,
        }

        let root = FractalNoun::<RootLevel, &str>::new("root");
        let domain = FractalNoun::<DomainLevel, Payload>::new(Payload {
            id: "auth-service".to_string(),
            bytes: vec![1, 2, 3],
        });

        let composed = root.compose(domain).expect("adjacent levels compose");

        // The payload's own fields survive the move into the composed
        // value untouched -- compose() never reads or rewrites `data`.
        assert_eq!(composed.data.id, "auth-service");
        assert_eq!(composed.data.bytes, vec![1, 2, 3]);
        // Only the separate `lineage: Vec<String>` field changed.
        assert_eq!(composed.lineage(), &["Root".to_string(), "Domain".to_string()]);
    }

    // -------------------------------------------------------------------
    // Property-based tests (proptest): VickreyAuction::run_auction and
    // ExplorationPolicy::select_ucb1. Every existing test above for both
    // is example-based (hand-picked bids / hand-picked trajectory
    // histories); these generalize the same real invariants across a
    // randomized input space that satisfies each function's own real
    // precondition (see run_auction's validation earlier in this file and
    // LearningTrajectory::observe's range check), rather than adding more
    // hand-picked cases.
    // -------------------------------------------------------------------

    use proptest::prelude::*;

    /// A finite, non-negative bid value -- matches `run_auction`'s real
    /// precondition (`bid_value.is_finite() && bid_value >= 0.0`) exactly,
    /// never generating NaN or infinity.
    fn bid_value_strategy() -> impl Strategy<Value = f64> {
        0.0f64..1_000_000.0f64
    }

    /// Two or more bids, all targeting one real `TaskId`, each from a
    /// distinct `AgentId` -- satisfies run_auction's "one bid per agent"
    /// precondition by construction (via a `HashSet` of agent ids), not by
    /// post-hoc filtering.
    fn valid_bids_strategy() -> impl Strategy<Value = (TaskId, Vec<Bid>)> {
        (any::<u64>(), prop::collection::hash_set(0u64..10_000u64, 2..10usize)).prop_flat_map(
            |(task_raw, agent_ids)| {
                let task_id = TaskId(task_raw);
                let ids: Vec<u64> = agent_ids.into_iter().collect();
                let len = ids.len();
                prop::collection::vec(bid_value_strategy(), len).prop_map(move |values| {
                    let bids = ids
                        .iter()
                        .zip(values)
                        .map(|(&id, bid_value)| Bid { agent_id: AgentId(id), task_id, bid_value })
                        .collect();
                    (task_id, bids)
                })
            },
        )
    }

    /// A real observation score -- matches `LearningTrajectory::observe`'s
    /// precondition (`0.0..=1.0`, finite by construction).
    fn score_strategy() -> impl Strategy<Value = f64> {
        0.0f64..=1.0f64
    }

    /// A trajectory built from zero or more real, valid `observe` calls --
    /// zero observations is a real, valid (never-tried) trajectory, not an
    /// edge case to exclude.
    fn trajectory_strategy() -> impl Strategy<Value = LearningTrajectory> {
        prop::collection::vec(score_strategy(), 0..8).prop_map(|scores| {
            let mut trajectory = LearningTrajectory::default();
            for score in scores {
                trajectory.observe(score).expect("score is within the valid 0.0..=1.0 range");
            }
            trajectory
        })
    }

    /// One or more real trajectories, any mix of tried/never-tried.
    fn trajectories_strategy() -> impl Strategy<Value = Vec<LearningTrajectory>> {
        prop::collection::vec(trajectory_strategy(), 1..8)
    }

    /// Same, but with at least one guaranteed never-tried arm somewhere in
    /// the slice. Its real position is still recomputed in the test below,
    /// not assumed -- an independently generated "before"/"after"
    /// trajectory may also happen to be empty.
    fn trajectories_with_a_guaranteed_untried_arm_strategy(
    ) -> impl Strategy<Value = Vec<LearningTrajectory>> {
        (
            prop::collection::vec(trajectory_strategy(), 0..4),
            prop::collection::vec(trajectory_strategy(), 0..4),
        )
            .prop_map(|(before, after)| {
                let mut all = before;
                all.push(LearningTrajectory::default());
                all.extend(after);
                all
            })
    }

    // -------------------------------------------------------------------
    // Property-based tests (proptest): RdfFragment::compose and
    // CompositionChain::push. Both are covered above only by hand-picked
    // examples; these generalize the real invariants that follow from
    // each function's actual implementation (confirmed from source
    // above, not assumed) across a randomized input space.
    // -------------------------------------------------------------------

    fn triple_field_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{1,8}"
    }

    fn semantic_triple_strategy() -> impl Strategy<Value = SemanticTriple> {
        (triple_field_strategy(), triple_field_strategy(), triple_field_strategy())
            .prop_map(|(subject, predicate, object)| SemanticTriple { subject, predicate, object })
    }

    fn triple_list_strategy() -> impl Strategy<Value = Vec<SemanticTriple>> {
        prop::collection::vec(semantic_triple_strategy(), 0..12)
    }

    fn fragment_from(triples: &[SemanticTriple]) -> RdfFragment {
        let mut fragment = RdfFragment::new();
        for triple in triples {
            fragment
                .insert(triple.clone())
                .expect("generated triple satisfies insert's own real precondition");
        }
        fragment
    }

    fn push_candidate_strategy() -> impl Strategy<Value = String> {
        prop_oneof![Just(String::new()), "[ \t\n]{0,4}", "[ \t]{0,2}[a-zA-Z0-9]{1,6}[ \t]{0,2}",]
    }

    proptest! {
        /// For ANY valid Vickrey auction input (2+ bids, one real common
        /// task, each bid finite/non-negative, one bid per agent): the
        /// winner's bid is the real maximum, the payment never exceeds the
        /// winner's own bid, and the payment is the real second price --
        /// the maximum bid among every OTHER real bid, not a fabricated
        /// number. No existing test checks this across more than the two
        /// or three hand-picked bids each example uses.
        #[test]
        fn run_auction_winner_is_the_real_maximum_and_payment_is_the_real_second_price(
            (task_id, bids) in valid_bids_strategy()
        ) {
            let mut auction = VickreyAuction::new();
            let outcome = auction
                .run_auction(&bids)
                .expect("generated bids satisfy run_auction's own real precondition");

            prop_assert_eq!(outcome.task_id, task_id);

            let winner_bid = bids
                .iter()
                .find(|bid| bid.agent_id == outcome.winner)
                .expect("the winner must be one of the real bidding agents");

            prop_assert!(
                bids.iter().all(|bid| winner_bid.bid_value >= bid.bid_value),
                "the winner's bid must be >= every real bid, including its own"
            );
            prop_assert!(
                outcome.payment <= winner_bid.bid_value,
                "the payment must never exceed the winner's own real bid"
            );

            let expected_payment = bids
                .iter()
                .filter(|bid| bid.agent_id != outcome.winner)
                .map(|bid| bid.bid_value)
                .fold(f64::NEG_INFINITY, f64::max);
            prop_assert_eq!(
                outcome.payment,
                expected_payment,
                "the payment must equal a real other bid's value, not a fabricated number"
            );
        }

        /// For ANY non-empty slice of real trajectories, `select_ucb1`
        /// never panics and always returns a real, in-bounds arm index --
        /// generalizes what every existing hand-picked two/three-arm test
        /// only checks for its own specific case.
        #[test]
        fn select_ucb1_always_returns_a_real_in_bounds_index(
            trajectories in trajectories_strategy()
        ) {
            let selected = ExplorationPolicy::select_ucb1(&trajectories)
                .expect("a non-empty trajectory slice always yields a real selection");
            prop_assert!(selected < trajectories.len());
        }

        /// For ANY non-empty slice of real trajectories containing at
        /// least one never-tried arm, `select_ucb1` always returns the
        /// FIRST such arm's real index (its documented "in trajectory
        /// order" tie-break) -- no existing test exercises more than one
        /// never-tried arm among several, or an untried arm anywhere but
        /// the last position.
        #[test]
        fn select_ucb1_always_prefers_the_first_real_never_tried_arm(
            trajectories in trajectories_with_a_guaranteed_untried_arm_strategy()
        ) {
            let first_empty = trajectories
                .iter()
                .position(LearningTrajectory::is_empty)
                .expect("constructed with at least one real untried arm");

            let selected = ExplorationPolicy::select_ucb1(&trajectories)
                .expect("a non-empty trajectory slice always yields a real selection");
            prop_assert_eq!(selected, first_empty);
        }

        #[test]
        fn rdf_fragment_construction_is_insertion_order_independent(
            triples in triple_list_strategy()
        ) {
            let forward = fragment_from(&triples);
            let mut reversed_triples = triples.clone();
            reversed_triples.reverse();
            let reversed = fragment_from(&reversed_triples);
            prop_assert_eq!(forward, reversed);
        }

        #[test]
        fn rdf_fragment_compose_is_commutative(
            left_triples in triple_list_strategy(),
            right_triples in triple_list_strategy()
        ) {
            let left = fragment_from(&left_triples);
            let right = fragment_from(&right_triples);
            prop_assert_eq!(left.compose(&right), right.compose(&left));
        }

        #[test]
        fn rdf_fragment_compose_with_itself_is_idempotent(
            triples in triple_list_strategy()
        ) {
            let fragment = fragment_from(&triples);
            let composed = fragment.compose(&fragment);
            prop_assert_eq!(composed, fragment);
        }

        #[test]
        fn rdf_fragment_compose_triple_count_never_exceeds_the_sum_of_both_inputs(
            left_triples in triple_list_strategy(),
            right_triples in triple_list_strategy()
        ) {
            let left = fragment_from(&left_triples);
            let right = fragment_from(&right_triples);
            let composed = left.compose(&right);
            prop_assert!(composed.triples().len() <= left.triples().len() + right.triples().len());
        }

        #[test]
        fn rdf_fragment_compose_is_associative(
            a_triples in triple_list_strategy(),
            b_triples in triple_list_strategy(),
            c_triples in triple_list_strategy()
        ) {
            let a = fragment_from(&a_triples);
            let b = fragment_from(&b_triples);
            let c = fragment_from(&c_triples);
            prop_assert_eq!(a.compose(&b).compose(&c), a.compose(&b.compose(&c)));
        }

        #[test]
        fn composition_chain_len_equals_the_real_count_of_trim_non_empty_pushes(
            candidates in prop::collection::vec(push_candidate_strategy(), 0..20)
        ) {
            let mut chain = CompositionChain::new();
            for candidate in &candidates {
                chain.push(candidate.clone());
            }
            let expected_len = candidates.iter().filter(|c| !c.trim().is_empty()).count();
            prop_assert_eq!(chain.len(), expected_len);
            prop_assert_eq!(chain.is_empty(), expected_len == 0);
        }

        #[test]
        fn composition_chain_entries_are_the_real_untrimmed_non_blank_inputs_in_order(
            candidates in prop::collection::vec(push_candidate_strategy(), 0..20)
        ) {
            let mut chain = CompositionChain::new();
            for candidate in &candidates {
                chain.push(candidate.clone());
            }
            let expected: Vec<String> =
                candidates.into_iter().filter(|c| !c.trim().is_empty()).collect();
            prop_assert_eq!(chain.entries(), expected.as_slice());
        }
    }

    // -------------------------------------------------------------------
    // Cross-capability integration (frontier-gap-sweep): DiscoveryEngine +
    // MetaFramework + EconomicSimulation + LearningTrajectory +
    // ReflexiveReport wired together with real data flowing between them.
    // -------------------------------------------------------------------

    #[test]
    fn discovery_meta_framework_economic_simulation_learning_trajectory_and_reflexive_report_compose_end_to_end(
    ) {
        fn run_allocation_round(
            agent_trust: &BTreeMap<AgentId, f64>,
            task_capability: &BTreeMap<TaskId, &str>,
        ) -> FrontierResult<Vec<Allocation>> {
            let agent_capabilities: BTreeMap<AgentId, Vec<&str>> = BTreeMap::from([
                (AgentId(1), vec!["alpha-capability"]),
                (AgentId(2), vec!["alpha-capability", "beta-capability"]),
                (AgentId(3), vec!["beta-capability"]),
            ]);
            let mut sim = EconomicSimulation::new();
            for (&agent_id, &trust) in agent_trust {
                let capabilities: Vec<String> = agent_capabilities
                    .get(&agent_id)
                    .expect("every agent id in this fixed scenario has a capability list")
                    .iter()
                    .map(|name| (*name).to_string())
                    .collect();
                sim.add_agent(Agent {
                    id: agent_id,
                    capabilities,
                    trust_score: trust,
                    valuation: 100.0,
                })?;
            }
            for (&task_id, &capability) in task_capability {
                sim.add_task(Task {
                    id: task_id,
                    required_capability: capability.to_string(),
                    value: 100.0,
                })?;
            }
            sim.step()
        }

        let mut discovery = DiscoveryEngine::default();
        for name in ["alpha-capability", "beta-capability", "gamma-capability"] {
            discovery
                .register(DiscoveryRecord {
                    name: name.to_string(),
                    tags: BTreeSet::new(),
                    route: format!("svc://{name}"),
                })
                .expect("valid record");
        }

        let mut framework = MetaFramework::new();
        framework.register_layer("capability-admission").expect("unique layer");
        framework
            .admit_invariant(Invariant {
                id: "sufficient-capacity".to_string(),
                description: "at least one trust-scored agent exists for the capability set"
                    .to_string(),
                satisfied: true,
            })
            .expect("valid invariant");
        assert_eq!(
            framework.state(false),
            AdmissionState::Admitted,
            "invariants hold but no execution receipt exists yet"
        );

        let agent_trust: BTreeMap<AgentId, f64> =
            BTreeMap::from([(AgentId(1), 0.9), (AgentId(2), 0.3), (AgentId(3), 0.7)]);
        let task_capability: BTreeMap<TaskId, &str> = BTreeMap::from([
            (TaskId(1), "alpha-capability"),
            (TaskId(2), "beta-capability"),
            (TaskId(3), "gamma-capability"),
        ]);

        let first_run = run_allocation_round(&agent_trust, &task_capability)
            .expect("a deterministic, valid pipeline always succeeds");
        let second_run = run_allocation_round(&agent_trust, &task_capability)
            .expect("a deterministic, valid pipeline always succeeds");
        let replay_verified = first_run == second_run;
        assert!(
            replay_verified,
            "two independent runs of the identical deterministic pipeline must agree"
        );

        assert_eq!(
            first_run.len(),
            2,
            "gamma-capability's task has no eligible agent and stays unallocated"
        );
        let alpha_allocation =
            first_run.iter().find(|a| a.task_id == TaskId(1)).expect("alpha task was allocated");
        let beta_allocation =
            first_run.iter().find(|a| a.task_id == TaskId(2)).expect("beta task was allocated");
        assert_eq!(
            alpha_allocation.agent_id,
            AgentId(1),
            "the higher-trust eligible agent (0.9 over 0.3) must win alpha-capability"
        );
        assert_eq!(
            beta_allocation.agent_id,
            AgentId(3),
            "the higher-trust eligible agent (0.7 over 0.3) must win beta-capability"
        );

        let mut alpha_trajectory = LearningTrajectory::default();
        let alpha_score = *agent_trust
            .get(&alpha_allocation.agent_id)
            .expect("the winning agent has a known trust score");
        alpha_trajectory.observe(alpha_score).expect("trust scores already lie within 0.0..=1.0");

        let mut beta_trajectory = LearningTrajectory::default();
        let beta_score = *agent_trust
            .get(&beta_allocation.agent_id)
            .expect("the winning agent has a known trust score");
        beta_trajectory.observe(beta_score).expect("trust scores already lie within 0.0..=1.0");

        let gamma_trajectory = LearningTrajectory::default();
        assert!(gamma_trajectory.is_empty(), "gamma-capability was never executed this round");

        let histories = vec![
            ("alpha-capability".to_string(), alpha_trajectory.clone()),
            ("beta-capability".to_string(), beta_trajectory.clone()),
            ("gamma-capability".to_string(), gamma_trajectory.clone()),
        ];
        let recommended =
            discovery.recommend(&histories).expect("every history names a registered capability");
        assert_eq!(
            recommended, "gamma-capability",
            "the never-executed capability must be recommended ahead of the two \
             already-observed ones"
        );

        let passed = first_run.len() as u64;
        let failed = discovery.names().len() as u64 - passed;
        let report = ReflexiveReport { passed, failed, replay_verified };
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 1);
        assert!(
            !report.is_alive(),
            "one real unallocated capability keeps this round from being alive"
        );
        assert!(
            (report.pass_rate() - (2.0 / 3.0)).abs() < 1e-9,
            "pass_rate must equal the real fraction of capabilities executed this round"
        );

        let receipt_observed = !first_run.is_empty();
        assert_eq!(
            framework.state(receipt_observed),
            AdmissionState::Alive,
            "once a real allocation exists as a receipt, admission advances to Alive"
        );
    }
}
