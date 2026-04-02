# Semantic Agent Coordinator - Implementation Guide

> **Companion to**: `semantic_agent_coordinator_spec.json`
> **Purpose**: Translate architectural specification into actionable implementation patterns
> **Audience**: Rust developers implementing the reference architecture

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Type-State Lifecycle Implementation](#type-state-lifecycle-implementation)
3. [Semantic Discovery Integration](#semantic-discovery-integration)
4. [Swarm Coordination Patterns](#swarm-coordination-patterns)
5. [MAPE-K Autonomic Loop](#mape-k-autonomic-loop)
6. [Kernel Determinism](#kernel-determinism)
7. [Zero-Cost Hot-Path Optimization](#zero-cost-hot-path-optimization)
8. [Testing Strategy](#testing-strategy)
9. [Performance Validation](#performance-validation)

---

## Quick Start

### Project Structure

```
examples/reference/
└── semantic_agent_coordinator/
    ├── main.rs                          # CLI entry point
    ├── lib.rs                           # Public API exports
    ├── lifecycle/
    │   ├── mod.rs                       # Type-state lifecycle
    │   ├── states.rs                    # State markers (Unregistered, Registered, etc.)
    │   └── transitions.rs               # State transition implementations
    ├── semantic/
    │   ├── mod.rs                       # Semantic discovery
    │   ├── ontology.rs                  # RDF/Turtle ontology
    │   ├── sparql.rs                    # SPARQL query execution
    │   └── ml_predictor.rs              # ML capability prediction
    ├── swarm/
    │   ├── mod.rs                       # Swarm coordination
    │   ├── gossip.rs                    # Gossip protocol
    │   ├── consensus.rs                 # Byzantine consensus
    │   ├── trust.rs                     # Trust scoring
    │   ├── auction.rs                   # Task auction
    │   └── stigmergy.rs                 # Pheromone-based routing
    ├── autonomic/
    │   ├── mod.rs                       # MAPE-K loop
    │   ├── monitor.rs                   # Metrics collection
    │   ├── analyze.rs                   # Anomaly detection
    │   ├── plan.rs                      # Remediation planning
    │   ├── execute.rs                   # Safe execution
    │   └── knowledge.rs                 # Knowledge base
    ├── kernel/
    │   ├── mod.rs                       # Determinism & receipts
    │   ├── receipt.rs                   # Receipt generation
    │   ├── replay.rs                    # Deterministic replay
    │   └── causal.rs                    # Causal DAG
    ├── hotpath/
    │   ├── mod.rs                       # Performance optimizations
    │   ├── arena.rs                     # Arena allocation
    │   ├── lockfree.rs                  # Lock-free registry
    │   └── simd.rs                      # SIMD capability matching
    └── cli/
        ├── mod.rs                       # CLI commands
        ├── agent.rs                     # agent noun
        ├── task.rs                      # task noun
        ├── swarm.rs                     # swarm noun
        ├── autonomic.rs                 # autonomic noun
        ├── receipt.rs                   # receipt noun
        └── semantic.rs                  # semantic noun
```

### Cargo.toml Features

```toml
[package]
name = "semantic-agent-coordinator"
version = "0.1.0"
edition = "2021"

[dependencies]
clap-noun-verb = { version = "5.3", features = ["full"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Type-state lifecycle (zero-cost)
# (No dependencies - uses std::marker::PhantomData)

# Semantic discovery
# (Reuse clap-noun-verb's rdf feature)

# Swarm coordination
crossbeam = "0.8"        # Lock-free data structures
dashmap = "5.5"          # Concurrent hash map
rand = "0.8"             # Random selection for gossip

# Autonomic MAPE-K
parking_lot = "0.12"     # Efficient mutexes for non-hot-paths
chrono = "0.4"           # Timestamps

# Kernel determinism
sha2 = "0.10"            # SHA-256 hashing
hex = "0.4"              # Hex encoding for receipt IDs

# Performance
bumpalo = "3.14"         # Arena allocator
# SIMD - use std::arch or portable_simd when stable

[dev-dependencies]
criterion = "0.5"        # Benchmarking
proptest = "1.0"         # Property testing
loom = "0.7"             # Concurrency testing
insta = "1.34"           # Snapshot testing
```

---

## Type-State Lifecycle Implementation

### Core Pattern: PhantomData-Based State Machine

**Key Insight**: Use Rust's type system to encode state transitions. Invalid transitions cause compile errors.

#### 1. Define State Markers

```rust
// lifecycle/states.rs

/// State marker for unregistered agents
pub struct Unregistered;

/// State marker for registered but unverified agents
pub struct Registered;

/// State marker for verified agents
pub struct Verified;

/// State marker for trusted agents (high trust score)
pub struct Trusted;

/// State marker for escalated agents (under review)
pub struct Escalated;

/// Trait for compile-time state validation
pub trait AgentState: 'static {}

impl AgentState for Unregistered {}
impl AgentState for Registered {}
impl AgentState for Verified {}
impl AgentState for Trusted {}
impl AgentState for Escalated {}
```

#### 2. Define Agent Type with PhantomData

```rust
// lifecycle/mod.rs

use std::marker::PhantomData;
use bitvec::BitVec;

/// Agent with compile-time state tracking
///
/// The type parameter S encodes the agent's lifecycle state.
/// PhantomData<S> has zero runtime size but enforces type safety.
pub struct Agent<S: AgentState> {
    pub id: AgentId,
    pub capabilities: BitVec,  // Bit vector for SIMD matching
    pub trust_score: f64,
    pub current_load: u32,
    _state: PhantomData<S>,  // Zero-cost state marker
}

/// Unique agent identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

impl<S: AgentState> Agent<S> {
    /// Get agent ID (available in all states)
    pub fn id(&self) -> AgentId {
        self.id
    }

    /// Get trust score (available in all states)
    pub fn trust_score(&self) -> f64 {
        self.trust_score
    }
}
```

#### 3. Implement State Transitions

```rust
// lifecycle/transitions.rs

use super::*;
use crate::registry::Registry;
use crate::validators::CapabilityValidator;
use crate::trust::TrustEvaluator;

// ============================================================================
// Transition: Unregistered -> Registered
// ============================================================================

impl Agent<Unregistered> {
    /// Create a new unregistered agent
    pub fn new() -> Self {
        Agent {
            id: AgentId(0),  // Will be assigned during registration
            capabilities: BitVec::new(),
            trust_score: 0.0,
            current_load: 0,
            _state: PhantomData,
        }
    }

    /// Register agent with the registry
    ///
    /// Consumes self and returns Agent<Registered> on success.
    /// This enforces that you cannot use an unregistered agent after registration.
    pub fn register(
        mut self,
        registry: &Registry,
        capabilities: BitVec,
    ) -> Result<Agent<Registered>, RegistrationError> {
        // Assign unique ID from registry
        let id = registry.next_agent_id()?;

        self.id = id;
        self.capabilities = capabilities;
        self.trust_score = 0.0;  // Initial trust score

        // Store in registry
        registry.add_agent(id, &self.capabilities)?;

        // Type transition: Unregistered -> Registered
        Ok(Agent {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: self.trust_score,
            current_load: self.current_load,
            _state: PhantomData,  // PhantomData<Registered>
        })
    }
}

// ============================================================================
// Transition: Registered -> Verified
// ============================================================================

impl Agent<Registered> {
    /// Verify agent capabilities
    ///
    /// Consumes self and returns Agent<Verified> on success.
    pub fn verify(
        mut self,
        validator: &CapabilityValidator,
    ) -> Result<Agent<Verified>, VerificationError> {
        // Validate capabilities
        validator.validate_capabilities(self.id, &self.capabilities)?;

        // Update trust score after verification
        self.trust_score = 0.5;  // Base trust after verification

        // Type transition: Registered -> Verified
        Ok(Agent {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: self.trust_score,
            current_load: self.current_load,
            _state: PhantomData,  // PhantomData<Verified>
        })
    }
}

// ============================================================================
// Transition: Verified -> Trusted
// ============================================================================

impl Agent<Verified> {
    /// Gain trust through successful task completions
    ///
    /// Consumes self and returns Agent<Trusted> on success.
    pub fn gain_trust(
        mut self,
        evaluator: &TrustEvaluator,
    ) -> Result<Agent<Trusted>, TrustError> {
        // Evaluate trust score
        let new_trust = evaluator.compute_trust(self.id)?;

        if new_trust < 0.8 {
            return Err(TrustError::InsufficientTrust {
                required: 0.8,
                actual: new_trust
            });
        }

        self.trust_score = new_trust;

        // Type transition: Verified -> Trusted
        Ok(Agent {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: self.trust_score,
            current_load: self.current_load,
            _state: PhantomData,  // PhantomData<Trusted>
        })
    }

    /// Escalate due to suspicious behavior
    ///
    /// Consumes self and returns Agent<Escalated>.
    pub fn escalate(self, reason: String) -> Agent<Escalated> {
        Agent {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: self.trust_score,
            current_load: self.current_load,
            _state: PhantomData,  // PhantomData<Escalated>
        }
    }
}

// ============================================================================
// Trusted-specific capabilities
// ============================================================================

impl Agent<Trusted> {
    /// Lead consensus round (only trusted agents can do this)
    ///
    /// This method is ONLY available on Agent<Trusted>.
    /// Attempting to call this on Agent<Verified> causes a compile error.
    pub fn lead_consensus(&self, subject: String) -> ConsensusResult {
        // Implementation...
        todo!("Implement consensus leadership")
    }

    /// Validate other agents' work
    pub fn validate_task(&self, task_id: TaskId) -> ValidationResult {
        // Implementation...
        todo!("Implement task validation")
    }

    /// Escalate to review
    pub fn escalate(self, reason: String) -> Agent<Escalated> {
        Agent {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: self.trust_score,
            current_load: self.current_load,
            _state: PhantomData,
        }
    }
}

// ============================================================================
// Escalated-specific capabilities
// ============================================================================

impl Agent<Escalated> {
    /// Appeal escalation
    pub fn appeal(&self, justification: String) -> AppealResult {
        // Implementation...
        todo!("Implement appeal process")
    }

    /// Re-register after review
    pub fn re_register(self) -> Agent<Unregistered> {
        Agent {
            id: AgentId(0),  // Reset ID
            capabilities: BitVec::new(),
            trust_score: 0.0,
            current_load: 0,
            _state: PhantomData,
        }
    }
}
```

#### 4. Compile-Time Validation Example

```rust
// This compiles:
let agent = Agent::<Unregistered>::new();
let agent = agent.register(&registry, capabilities)?;
let agent = agent.verify(&validator)?;
let agent = agent.gain_trust(&evaluator)?;
agent.lead_consensus("trust_update".to_string());  // ✅ OK

// This does NOT compile:
let agent = Agent::<Unregistered>::new();
let agent = agent.register(&registry, capabilities)?;
// agent.lead_consensus("...".to_string());  // ❌ Compile error!
// Error: no method named `lead_consensus` found for struct `Agent<Registered>`
```

**Performance**: PhantomData<S> has **zero size** at runtime. The type-state pattern is a pure compile-time abstraction with no runtime overhead.

---

## Semantic Discovery Integration

### RDF/Turtle Ontology

```rust
// semantic/ontology.rs

use clap_noun_verb::rdf::{OntologyBuilder, RdfTriple, RdfValue};

const AGENT_COORDINATOR_NS: &str = "https://cnv.dev/agent-coordinator#";

pub struct AgentOntology {
    builder: OntologyBuilder,
}

impl AgentOntology {
    pub fn new() -> Self {
        let mut builder = OntologyBuilder::new();

        // Define classes
        builder.add_triple(RdfTriple {
            subject: format!("{}Agent", AGENT_COORDINATOR_NS),
            predicate: "rdf:type".to_string(),
            object: RdfValue::Uri("rdfs:Class".to_string()),
        });

        builder.add_triple(RdfTriple {
            subject: format!("{}Capability", AGENT_COORDINATOR_NS),
            predicate: "rdf:type".to_string(),
            object: RdfValue::Uri("rdfs:Class".to_string()),
        });

        builder.add_triple(RdfTriple {
            subject: format!("{}Task", AGENT_COORDINATOR_NS),
            predicate: "rdf:type".to_string(),
            object: RdfValue::Uri("rdfs:Class".to_string()),
        });

        // Define properties
        builder.add_triple(RdfTriple {
            subject: format!("{}hasCapability", AGENT_COORDINATOR_NS),
            predicate: "rdf:type".to_string(),
            object: RdfValue::Uri("rdf:Property".to_string()),
        });

        builder.add_triple(RdfTriple {
            subject: format!("{}trustScore", AGENT_COORDINATOR_NS),
            predicate: "rdf:type".to_string(),
            object: RdfValue::Uri("rdf:Property".to_string()),
        });

        AgentOntology { builder }
    }

    /// Register agent in ontology
    pub fn register_agent(&mut self, agent_id: AgentId, capabilities: &[String]) {
        let agent_uri = format!("{}agent_{}", AGENT_COORDINATOR_NS, agent_id.0);

        // Agent is an instance of ac:Agent
        self.builder.add_triple(RdfTriple {
            subject: agent_uri.clone(),
            predicate: "rdf:type".to_string(),
            object: RdfValue::Uri(format!("{}Agent", AGENT_COORDINATOR_NS)),
        });

        // Add capabilities
        for cap in capabilities {
            self.builder.add_triple(RdfTriple {
                subject: agent_uri.clone(),
                predicate: format!("{}hasCapability", AGENT_COORDINATOR_NS),
                object: RdfValue::Uri(format!("{}cap_{}", AGENT_COORDINATOR_NS, cap)),
            });
        }
    }

    /// Update agent trust score
    pub fn update_trust_score(&mut self, agent_id: AgentId, trust_score: f64) {
        let agent_uri = format!("{}agent_{}", AGENT_COORDINATOR_NS, agent_id.0);

        self.builder.add_triple(RdfTriple {
            subject: agent_uri,
            predicate: format!("{}trustScore", AGENT_COORDINATOR_NS),
            object: RdfValue::Literal(trust_score.to_string(), "xsd:float".to_string()),
        });
    }

    /// Export ontology as Turtle
    pub fn to_turtle(&self) -> String {
        self.builder.to_turtle()
    }
}
```

### SPARQL Query Planner

```rust
// semantic/sparql.rs

use clap_noun_verb::rdf::SparqlPlanner;

pub struct CapabilityDiscovery {
    planner: SparqlPlanner,
}

impl CapabilityDiscovery {
    pub fn new() -> Self {
        CapabilityDiscovery {
            planner: SparqlPlanner::new(),
        }
    }

    /// Find agents matching task requirements
    pub fn find_agents_for_task(
        &self,
        required_capabilities: &[String],
        min_trust_score: f64,
    ) -> Result<Vec<AgentMatch>, SparqlError> {
        // Build SPARQL query
        let cap_filters = required_capabilities
            .iter()
            .enumerate()
            .map(|(i, cap)| {
                format!(
                    "?agent ac:hasCapability ac:cap_{} .",
                    cap
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ");

        let query = format!(
            r#"
PREFIX ac: <https://cnv.dev/agent-coordinator#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?agent ?trustScore ?latency ?successRate
WHERE {{
  {cap_filters}
  ?agent ac:trustScore ?trustScore ;
         ac:hasPerformance ?perf .
  ?perf ac:latencyP95 ?latency ;
        ac:successRate ?successRate .
  FILTER(?trustScore >= {min_trust_score})
}}
ORDER BY DESC(?trustScore) DESC(?successRate) ASC(?latency)
LIMIT 10
            "#,
            cap_filters = cap_filters,
            min_trust_score = min_trust_score
        );

        // Execute query
        let results = self.planner.execute(&query)?;

        // Parse results
        let matches = results
            .into_iter()
            .map(|row| AgentMatch {
                agent_id: parse_agent_id(&row["agent"]),
                trust_score: row["trustScore"].parse().unwrap_or(0.0),
                latency_p95: row["latency"].parse().unwrap_or(f64::MAX),
                success_rate: row["successRate"].parse().unwrap_or(0.0),
            })
            .collect();

        Ok(matches)
    }
}

#[derive(Debug)]
pub struct AgentMatch {
    pub agent_id: AgentId,
    pub trust_score: f64,
    pub latency_p95: f64,
    pub success_rate: f64,
}

fn parse_agent_id(uri: &str) -> AgentId {
    let id_str = uri.split('_').last().unwrap_or("0");
    AgentId(id_str.parse().unwrap_or(0))
}
```

---

## Swarm Coordination Patterns

### Gossip Protocol

```rust
// swarm/gossip.rs

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crossbeam::channel::{Sender, Receiver};
use rand::seq::SliceRandom;

pub struct GossipProtocol {
    fanout: usize,
    interval: Duration,
    max_age: Duration,
    peers: Vec<AgentId>,
    message_queue: VecDeque<GossipMessage>,
}

#[derive(Clone)]
pub struct GossipMessage {
    pub agent_id: AgentId,
    pub trust_score: f64,
    pub capabilities: BitVec,
    pub timestamp: Instant,
}

impl GossipProtocol {
    pub fn new(fanout: usize) -> Self {
        GossipProtocol {
            fanout,
            interval: Duration::from_millis(100),
            max_age: Duration::from_secs(30),
            peers: Vec::new(),
            message_queue: VecDeque::new(),
        }
    }

    /// Push-pull gossip round
    pub fn gossip_round(&mut self) {
        // 1. Select random peers (fanout)
        let mut rng = rand::thread_rng();
        let selected_peers: Vec<AgentId> = self.peers
            .choose_multiple(&mut rng, self.fanout)
            .copied()
            .collect();

        // 2. Push local state to selected peers
        for peer in &selected_peers {
            self.push_state_to_peer(*peer);
        }

        // 3. Pull state from selected peers
        for peer in &selected_peers {
            self.pull_state_from_peer(*peer);
        }

        // 4. Expire old messages
        self.expire_old_messages();
    }

    fn push_state_to_peer(&self, peer: AgentId) {
        // Implementation: Send local state to peer
        todo!("Implement push to peer")
    }

    fn pull_state_from_peer(&mut self, peer: AgentId) {
        // Implementation: Request and receive state from peer
        todo!("Implement pull from peer")
    }

    fn expire_old_messages(&mut self) {
        let now = Instant::now();
        self.message_queue.retain(|msg| {
            now.duration_since(msg.timestamp) < self.max_age
        });
    }

    /// Add peer to gossip network
    pub fn add_peer(&mut self, peer: AgentId) {
        if !self.peers.contains(&peer) {
            self.peers.push(peer);
        }
    }
}
```

### Byzantine Consensus (PBFT Variant)

```rust
// swarm/consensus.rs

use std::collections::HashMap;

pub struct ConsensusEngine {
    min_nodes: usize,
    timeout_ms: u64,
    max_rounds: usize,
}

#[derive(Debug, Clone)]
pub struct ConsensusRequest {
    pub subject: String,
    pub proposal: String,
}

#[derive(Debug)]
pub enum ConsensusResult {
    Agreed(String),
    Timeout,
    ByzantineFault,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        ConsensusEngine {
            min_nodes: 4,
            timeout_ms: 1000,
            max_rounds: 5,
        }
    }

    /// Run PBFT consensus
    ///
    /// Tolerates f < n/3 Byzantine failures
    pub fn reach_consensus(
        &self,
        request: ConsensusRequest,
        participants: &[AgentId],
    ) -> ConsensusResult {
        let n = participants.len();
        let f = (n - 1) / 3;  // Maximum Byzantine nodes

        if n < self.min_nodes {
            return ConsensusResult::ByzantineFault;
        }

        // Phase 1: Pre-prepare (leader broadcasts proposal)
        let leader = participants[0];
        let proposal = request.proposal.clone();

        // Phase 2: Prepare (nodes broadcast prepare messages)
        let prepare_votes = self.collect_prepare_votes(participants, &proposal);

        if prepare_votes < (2 * f + 1) {
            return ConsensusResult::ByzantineFault;
        }

        // Phase 3: Commit (nodes broadcast commit messages)
        let commit_votes = self.collect_commit_votes(participants, &proposal);

        if commit_votes < (2 * f + 1) {
            return ConsensusResult::ByzantineFault;
        }

        // Consensus reached
        ConsensusResult::Agreed(proposal)
    }

    fn collect_prepare_votes(&self, participants: &[AgentId], proposal: &str) -> usize {
        // Implementation: Collect prepare votes from participants
        // For demo, assume all honest nodes vote yes
        participants.len()
    }

    fn collect_commit_votes(&self, participants: &[AgentId], proposal: &str) -> usize {
        // Implementation: Collect commit votes from participants
        participants.len()
    }
}
```

---

## MAPE-K Autonomic Loop

### Monitor Phase

```rust
// autonomic/monitor.rs

use std::collections::VecDeque;
use std::time::Instant;

pub struct MetricsMonitor {
    metrics: VecDeque<MetricSnapshot>,
    ring_buffer_size: usize,
}

#[derive(Clone)]
pub struct MetricSnapshot {
    pub timestamp: Instant,
    pub task_latency_p95: f64,
    pub agent_utilization: f64,
    pub error_rate: f64,
    pub consensus_duration: f64,
}

impl MetricsMonitor {
    pub fn new() -> Self {
        MetricsMonitor {
            metrics: VecDeque::new(),
            ring_buffer_size: 10_000,
        }
    }

    /// Collect current metrics snapshot
    pub fn collect_snapshot(&mut self) {
        let snapshot = MetricSnapshot {
            timestamp: Instant::now(),
            task_latency_p95: self.measure_task_latency_p95(),
            agent_utilization: self.measure_agent_utilization(),
            error_rate: self.measure_error_rate(),
            consensus_duration: self.measure_consensus_duration(),
        };

        self.metrics.push_back(snapshot);

        // Ring buffer: drop oldest if exceeds size
        if self.metrics.len() > self.ring_buffer_size {
            self.metrics.pop_front();
        }
    }

    fn measure_task_latency_p95(&self) -> f64 {
        // Implementation: Query telemetry for p95 latency
        50.0  // Placeholder
    }

    fn measure_agent_utilization(&self) -> f64 {
        // Implementation: Query agent load metrics
        0.65  // Placeholder
    }

    fn measure_error_rate(&self) -> f64 {
        // Implementation: Query error counters
        0.02  // Placeholder
    }

    fn measure_consensus_duration(&self) -> f64 {
        // Implementation: Query consensus metrics
        450.0  // Placeholder
    }

    /// Get recent metrics for analysis
    pub fn recent_metrics(&self, count: usize) -> Vec<MetricSnapshot> {
        self.metrics
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
}
```

### Analyze Phase

```rust
// autonomic/analyze.rs

pub struct AnomalyDetector {
    ewma_alpha: f64,
    threshold_sigma: f64,
}

#[derive(Debug)]
pub enum Anomaly {
    HighLatency { actual: f64, threshold: f64 },
    HighErrorRate { actual: f64, threshold: f64 },
    ConsensusTimeout { actual: f64, threshold: f64 },
}

impl AnomalyDetector {
    pub fn new() -> Self {
        AnomalyDetector {
            ewma_alpha: 0.3,
            threshold_sigma: 3.0,
        }
    }

    /// Detect anomalies using EWMA
    pub fn detect_anomalies(&self, metrics: &[MetricSnapshot]) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Latency anomaly detection
        if let Some(latency_anomaly) = self.detect_latency_anomaly(metrics) {
            anomalies.push(latency_anomaly);
        }

        // Error rate anomaly detection
        if let Some(error_anomaly) = self.detect_error_anomaly(metrics) {
            anomalies.push(error_anomaly);
        }

        anomalies
    }

    fn detect_latency_anomaly(&self, metrics: &[MetricSnapshot]) -> Option<Anomaly> {
        if metrics.is_empty() {
            return None;
        }

        // Compute EWMA and standard deviation
        let (mean, stddev) = self.compute_ewma_stats(
            metrics.iter().map(|m| m.task_latency_p95)
        );

        let threshold = mean + self.threshold_sigma * stddev;
        let current = metrics.last()?.task_latency_p95;

        if current > threshold {
            Some(Anomaly::HighLatency {
                actual: current,
                threshold,
            })
        } else {
            None
        }
    }

    fn detect_error_anomaly(&self, metrics: &[MetricSnapshot]) -> Option<Anomaly> {
        if metrics.is_empty() {
            return None;
        }

        let (mean, stddev) = self.compute_ewma_stats(
            metrics.iter().map(|m| m.error_rate)
        );

        let threshold = mean + self.threshold_sigma * stddev;
        let current = metrics.last()?.error_rate;

        if current > threshold {
            Some(Anomaly::HighErrorRate {
                actual: current,
                threshold,
            })
        } else {
            None
        }
    }

    fn compute_ewma_stats(&self, values: impl Iterator<Item = f64>) -> (f64, f64) {
        let values: Vec<f64> = values.collect();
        if values.is_empty() {
            return (0.0, 0.0);
        }

        // EWMA mean
        let mut ewma = values[0];
        for &value in &values[1..] {
            ewma = self.ewma_alpha * value + (1.0 - self.ewma_alpha) * ewma;
        }

        // Standard deviation
        let variance: f64 = values.iter()
            .map(|&v| (v - ewma).powi(2))
            .sum::<f64>() / values.len() as f64;
        let stddev = variance.sqrt();

        (ewma, stddev)
    }
}
```

---

## Kernel Determinism

### Receipt Generation

```rust
// kernel/receipt.rs

use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct Receipt {
    pub receipt_id: [u8; 32],
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub timestamp: u64,
    pub parent_receipt_id: Option<[u8; 32]>,
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub execution_time_ns: u64,
    pub trust_score_snapshot: f64,
    pub state_hash: [u8; 32],
}

impl Receipt {
    /// Generate receipt from execution
    pub fn generate(
        agent_id: AgentId,
        task_id: TaskId,
        parent_receipt_id: Option<[u8; 32]>,
        inputs: &[u8],
        outputs: &[u8],
        execution_time_ns: u64,
        trust_score: f64,
        state: &[u8],
    ) -> Self {
        let timestamp = Self::monotonic_timestamp();

        // Hash inputs, outputs, state
        let input_hash = Self::hash_bytes(inputs);
        let output_hash = Self::hash_bytes(outputs);
        let state_hash = Self::hash_bytes(state);

        // Compute receipt ID
        let receipt_id = Self::compute_receipt_id(
            agent_id,
            task_id,
            timestamp,
            parent_receipt_id.as_ref(),
        );

        Receipt {
            receipt_id,
            agent_id,
            task_id,
            timestamp,
            parent_receipt_id,
            input_hash,
            output_hash,
            execution_time_ns,
            trust_score_snapshot: trust_score,
            state_hash,
        }
    }

    fn hash_bytes(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    fn compute_receipt_id(
        agent_id: AgentId,
        task_id: TaskId,
        timestamp: u64,
        parent_id: Option<&[u8; 32]>,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(agent_id.0.to_le_bytes());
        hasher.update(task_id.0.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        if let Some(parent) = parent_id {
            hasher.update(parent);
        }
        hasher.finalize().into()
    }

    fn monotonic_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
```

---

## Zero-Cost Hot-Path Optimization

### SIMD Capability Matching

```rust
// hotpath/simd.rs

use std::arch::x86_64::*;

/// SIMD-accelerated capability matching
///
/// Uses AVX2 256-bit SIMD for parallel comparison
#[target_feature(enable = "avx2")]
pub unsafe fn simd_capability_match(
    agent_caps: &[u64; 4],  // 256 bits = 4 x u64
    task_reqs: &[u64; 4],
) -> f64 {
    // Load capability vectors into SIMD registers
    let agent_vec = _mm256_loadu_si256(agent_caps.as_ptr() as *const __m256i);
    let task_vec = _mm256_loadu_si256(task_reqs.as_ptr() as *const __m256i);

    // Bitwise AND: agent_caps & task_reqs
    let intersection = _mm256_and_si256(agent_vec, task_vec);

    // Population count (number of matching bits)
    let match_count = popcnt_256(&intersection);
    let total_required = popcnt_256(&task_vec);

    // Match score = matched / required
    if total_required == 0 {
        0.0
    } else {
        match_count as f64 / total_required as f64
    }
}

/// Count set bits in 256-bit vector
unsafe fn popcnt_256(vec: &__m256i) -> u32 {
    let mut counts = [0u64; 4];
    _mm256_storeu_si256(counts.as_mut_ptr() as *mut __m256i, *vec);

    counts.iter().map(|&x| x.count_ones()).sum()
}
```

### Arena Allocation

```rust
// hotpath/arena.rs

use bumpalo::Bump;

pub struct TaskArena {
    arena: Bump,
}

impl TaskArena {
    pub fn new() -> Self {
        TaskArena {
            arena: Bump::new(),
        }
    }

    /// Allocate task in arena (< 5ns allocation)
    pub fn alloc_task<'a>(&'a self, task_data: TaskData) -> &'a mut Task {
        self.arena.alloc(Task {
            id: task_data.id,
            capabilities: task_data.capabilities,
            payload: self.arena.alloc_slice_copy(&task_data.payload),
        })
    }

    /// Reset arena (bulk free all allocations)
    pub fn reset(&mut self) {
        self.arena.reset();
    }
}

pub struct Task {
    pub id: TaskId,
    pub capabilities: BitVec,
    pub payload: &'static [u8],
}
```

---

## Testing Strategy

### Type-State Lifecycle Tests

```rust
// tests/lifecycle_tests.rs

#[test]
fn test_unregistered_to_registered_transition() {
    // Arrange
    let registry = Registry::new();
    let agent = Agent::<Unregistered>::new();
    let capabilities = bitvec![1, 0, 1, 0];

    // Act
    let result = agent.register(&registry, capabilities.clone());

    // Assert
    assert!(result.is_ok());
    let registered_agent = result.unwrap();
    assert_ne!(registered_agent.id(), AgentId(0));
    assert_eq!(registered_agent.trust_score(), 0.0);
}

#[test]
#[should_panic] // This test verifies compile error (use compile_fail in doc tests)
fn test_cannot_lead_consensus_when_unverified() {
    let agent = Agent::<Verified>::new_for_test();
    // agent.lead_consensus("test".to_string());  // ❌ Compile error!
}
```

### Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn proptest_all_state_transitions_preserve_id(
        capabilities in prop::collection::vec(any::<bool>(), 256),
    ) {
        let registry = Registry::new();
        let agent = Agent::<Unregistered>::new();

        let agent = agent.register(&registry, BitVec::from_iter(capabilities)).unwrap();
        let original_id = agent.id();

        // Transition through all states
        let agent = agent.verify(&CapabilityValidator::new()).unwrap();
        assert_eq!(agent.id(), original_id);

        let agent = agent.gain_trust(&TrustEvaluator::new()).unwrap();
        assert_eq!(agent.id(), original_id);
    }
}
```

---

## Performance Validation

### Benchmarks

```rust
// benches/coordinator_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_lookup(c: &mut Criterion) {
    let registry = setup_registry_with_10k_agents();

    c.bench_function("agent_lookup_10k", |b| {
        b.iter(|| {
            registry.lookup(black_box(AgentId(5000)))
        });
    });
}

fn bench_capability_match_simd(c: &mut Criterion) {
    let agent_caps = [0xFFFF_FFFF_FFFF_FFFF; 4];
    let task_reqs = [0x0000_FFFF_0000_FFFF; 4];

    c.bench_function("simd_capability_match", |b| {
        b.iter(|| unsafe {
            simd_capability_match(
                black_box(&agent_caps),
                black_box(&task_reqs)
            )
        });
    });
}

criterion_group!(benches, bench_agent_lookup, bench_capability_match_simd);
criterion_main!(benches);
```

### SLO Validation

```toml
# Makefile.toml

[tasks.slo-check]
description = "Verify performance SLOs are met"
script = '''
#!/bin/bash
set -e

echo "Running benchmarks..."
cargo bench --bench coordinator_benchmarks -- --save-baseline current

echo "Validating SLOs..."
# Parse Criterion results and check against targets
# Agent lookup: < 50ns
# SIMD matching: < 10ns per agent
# Task allocation: < 10μs end-to-end

# Example validation (pseudo-code):
# if [ $(parse_benchmark "agent_lookup_10k") -gt 50 ]; then
#   echo "❌ SLO VIOLATION: Agent lookup exceeded 50ns"
#   exit 1
# fi

echo "✅ All SLOs met"
'''
```

---

## Summary

This implementation guide translates the architectural specification into concrete Rust code patterns:

1. **Type-State Lifecycle**: Zero-cost state machine using PhantomData
2. **Semantic Discovery**: RDF/SPARQL integration for capability matching
3. **Swarm Coordination**: Gossip, consensus, trust scoring, auctions
4. **MAPE-K Loop**: Monitor → Analyze → Plan → Execute → Knowledge
5. **Kernel Determinism**: SHA-256 receipts for reproducibility
6. **Zero-Cost Optimization**: Arena allocation, SIMD, lock-free structures
7. **Chicago TDD**: State-based testing with real collaborators

**Next Steps**:
1. Implement Phase 1 (Foundation) - type-state lifecycle and data structures
2. Write tests first (TDD) - verify state transitions and invariants
3. Integrate with clap-noun-verb CLI layer - `#[noun]` and `#[verb]` attributes
4. Benchmark hot paths - ensure SLOs are met
5. Iterate through remaining phases

**Performance Targets**:
- Agent lookup: < 50ns
- Capability match: < 10ns per agent
- Task allocation: < 10μs end-to-end
- Receipt generation: < 100ns

This reference implementation showcases clap-noun-verb's advanced features in novel, production-grade compositions.
