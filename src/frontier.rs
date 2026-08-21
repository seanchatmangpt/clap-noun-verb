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
    /// Returns an error if `histories` is empty, or if any name in
    /// `histories` is not currently registered (a caller recommending
    /// against a stale or foreign capability set is a real bug, not
    /// something to silently ignore).
    pub fn recommend<'a>(
        &self,
        histories: &'a [(String, LearningTrajectory)],
    ) -> FrontierResult<&'a str> {
        if histories.is_empty() {
            return Err("recommend requires at least one candidate history".to_string());
        }
        for (name, _) in histories {
            if !self.records.contains_key(name) {
                return Err(format!("recommend candidate is not a registered capability: {name}"));
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
        let name = match algorithm {
            QuantumAlgorithm::MlKem => "ML-KEM",
            QuantumAlgorithm::MlDsa => "ML-DSA",
            QuantumAlgorithm::SlhDsa => "SLH-DSA",
        };
        self.allowed.contains(name)
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
}
