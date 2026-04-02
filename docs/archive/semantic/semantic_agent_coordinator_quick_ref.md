# Semantic Agent Coordinator - Quick Reference Card

> **Cheat sheet for developers implementing the reference architecture**

---

## 🎯 Key Design Decisions

| Decision | Choice | Rationale | Trade-off |
|----------|--------|-----------|-----------|
| **State Machine** | Type-state with PhantomData | Compile-time safety, zero cost | More complex type signatures |
| **Capability Discovery** | SPARQL + ML hybrid | Semantic correctness + predictions | Requires RDF store |
| **Consensus** | PBFT variant | Byzantine fault tolerance | Higher latency than simple consensus |
| **Trust Scoring** | EigenTrust | Decentralized, Sybil-resistant | Requires graph convergence |
| **Task Allocation** | Vickrey auction | Incentive-compatible, truthful bidding | Requires bid bonds |
| **Coordination** | Stigmergy (pheromones) | Emergent, self-organizing | Non-deterministic routing |
| **Anomaly Detection** | EWMA + SPC + CUSUM | Multiple techniques for robustness | More complex analysis |
| **Receipts** | SHA-256 causal chains | Full auditability, replay | Storage overhead |
| **Hot-Path Optimization** | SIMD + Lock-free + Arena | Maximum throughput | Platform-specific (AVX2) |

---

## 📊 Performance Budget

| Operation | Target | Technique | Validation |
|-----------|--------|-----------|------------|
| Agent lookup | < 50ns | Lock-free DashMap | Criterion bench |
| Capability match | < 10ns/agent | AVX2 SIMD | Criterion bench |
| Task allocation | < 10μs | End-to-end pipeline | Integration test |
| Receipt generation | < 100ns | Arena + SHA-256 | Criterion bench |
| Gossip convergence | < 5s (1000 agents) | Push-pull protocol | Integration test |
| Consensus latency | < 500ms (10 nodes) | PBFT optimization | Integration test |
| MAPE-K cycle | < 1s | Lightweight monitoring | Integration test |
| Anomaly detection | < 3s (3 cycles) | EWMA thresholds | Integration test |
| Self-healing | < 10s | Automatic remediation | Chaos test |

---

## 🔧 Implementation Checklist

### Phase 1: Foundation (3-4 days)
- [ ] Define state markers (Unregistered, Registered, Verified, Trusted, Escalated)
- [ ] Implement `Agent<S>` with PhantomData
- [ ] Write state transition methods (register, verify, gain_trust, escalate)
- [ ] Create lock-free agent registry (DashMap)
- [ ] Basic CLI scaffolding (agent register/list/inspect)
- [ ] Test: Verify compile-time state guarantees
- [ ] Test: Verify zero runtime overhead (cargo asm)

### Phase 2: Semantic Layer (3-4 days)
- [ ] Define RDF/Turtle ontology (ac:Agent, ac:Capability, ac:Task)
- [ ] Implement SPARQL query planner
- [ ] Create capability discovery algorithm
- [ ] Stub ML predictor (mock predictions for now)
- [ ] Test: SPARQL query correctness
- [ ] Test: Performance < 50ms for 10k agents

### Phase 3: Swarm Coordination (4-5 days)
- [ ] Implement gossip protocol (fanout=3, interval=100ms)
- [ ] Implement PBFT consensus (min_nodes=4, f < n/3)
- [ ] Implement EigenTrust scoring (alpha=0.15)
- [ ] Implement Vickrey auction (2nd-price sealed-bid)
- [ ] Implement stigmergy pheromone field
- [ ] Test: Gossip convergence
- [ ] Test: Byzantine consensus with faults
- [ ] Test: Pheromone routing effectiveness

### Phase 4: Autonomic MAPE-K (3-4 days)
- [ ] Monitor: Metrics collection + ring buffer
- [ ] Analyze: EWMA anomaly detection
- [ ] Plan: Remediation plan generation
- [ ] Execute: Safe execution with rollback
- [ ] Knowledge: Historical database updates
- [ ] Test: Anomaly detection accuracy
- [ ] Test: Self-healing from agent crash

### Phase 5: Kernel Determinism (2-3 days)
- [ ] Receipt generation with SHA-256
- [ ] Deterministic replay engine
- [ ] Causal DAG construction
- [ ] Audit trail export (JSON/CSV/Parquet)
- [ ] Test: Receipt determinism
- [ ] Test: Replay correctness

### Phase 6: Performance Optimization (2-3 days)
- [ ] Arena allocation for hot paths
- [ ] SIMD capability matching (AVX2)
- [ ] Lock-free registry optimizations
- [ ] Benchmark suite with Criterion
- [ ] Test: Meet all SLOs
- [ ] Test: Regression < 5%

### Phase 7: Testing & Docs (2-3 days)
- [ ] Chicago TDD test suite (80%+ coverage)
- [ ] Property tests with proptest
- [ ] Chaos tests with loom
- [ ] API documentation
- [ ] User guide with examples
- [ ] Performance validation report

---

## 🧪 Testing Patterns

### Type-State Tests
```rust
#[test]
fn test_state_transition() {
    // Arrange
    let agent = Agent::<Unregistered>::new();

    // Act
    let agent = agent.register(&registry, caps)?;

    // Assert
    assert_ne!(agent.id(), AgentId(0));
    assert_eq!(agent.trust_score(), 0.0);
}

#[compile_fail]
fn test_invalid_transition() {
    let agent = Agent::<Registered>::new_for_test();
    agent.lead_consensus("..."); // Should not compile
}
```

### Property Tests
```rust
proptest! {
    #[test]
    fn proptest_id_preservation(caps in any::<BitVec>()) {
        let agent = Agent::<Unregistered>::new();
        let agent = agent.register(&registry, caps)?;
        let original_id = agent.id();

        let agent = agent.verify(&validator)?;
        assert_eq!(agent.id(), original_id);
    }
}
```

### Chaos Tests
```rust
#[test]
fn chaos_network_partition() {
    loom::model(|| {
        // Simulate network partition
        // Verify system continues operating
    });
}
```

---

## 📐 Architecture Formulas

### Weighted Agent Scoring
```
score = 0.4 × trust_score
      + 0.3 × success_rate
      + 0.2 × specialization_score
      + 0.1 × (1 - normalized_latency)
```

### EigenTrust
```
trust(i) = (1 - α) × Σ(trust(j) × rating(j→i)) + α × initial_trust
where α = 0.15 (damping factor)
```

### EWMA Anomaly Threshold
```
threshold = ewma_mean + 3 × ewma_stddev
ewma(t) = α × value(t) + (1 - α) × ewma(t-1)
where α = 0.3
```

### PBFT Consensus
```
f = (n - 1) / 3  (maximum Byzantine nodes)
quorum = 2f + 1  (minimum votes for consensus)
```

### Pheromone Routing Probability
```
P(edge) = (pheromone^α × heuristic^β) / Σ(pheromone^α × heuristic^β)
where α = 1.0, β = 2.0
```

---

## 🔑 Critical Code Patterns

### PhantomData State Transition
```rust
impl Agent<Registered> {
    pub fn verify(mut self, validator: &Validator)
        -> Result<Agent<Verified>>
    {
        validator.validate_capabilities(self.id, &self.capabilities)?;
        self.trust_score = 0.5;

        Ok(Agent {
            id: self.id,
            capabilities: self.capabilities,
            trust_score: self.trust_score,
            _state: PhantomData, // PhantomData<Verified>
        })
    }
}
```

### SIMD Capability Matching
```rust
#[target_feature(enable = "avx2")]
unsafe fn simd_match(agent_caps: &[u64; 4], task_reqs: &[u64; 4]) -> f64 {
    let agent = _mm256_loadu_si256(agent_caps.as_ptr() as *const __m256i);
    let task = _mm256_loadu_si256(task_reqs.as_ptr() as *const __m256i);
    let intersection = _mm256_and_si256(agent, task);

    let match_count = popcnt_256(&intersection);
    let total_required = popcnt_256(&task);

    match_count as f64 / total_required as f64
}
```

### Arena Allocation
```rust
pub struct TaskArena {
    arena: Bump,
}

impl TaskArena {
    pub fn alloc_task<'a>(&'a self, data: TaskData) -> &'a mut Task {
        self.arena.alloc(Task {
            id: data.id,
            payload: self.arena.alloc_slice_copy(&data.payload),
        })
    }
}
```

### Receipt Generation
```rust
pub fn generate_receipt(
    agent_id: AgentId,
    task_id: TaskId,
    inputs: &[u8],
    outputs: &[u8],
) -> Receipt {
    let input_hash = sha256(inputs);
    let output_hash = sha256(outputs);
    let receipt_id = sha256(&[
        &agent_id.0.to_le_bytes(),
        &task_id.0.to_le_bytes(),
        &timestamp().to_le_bytes(),
    ].concat());

    Receipt { receipt_id, input_hash, output_hash, ... }
}
```

---

## 🚨 Common Pitfalls

| Pitfall | Solution |
|---------|----------|
| **Calling methods on wrong state** | Use type-state pattern - compiler enforces |
| **Mutex contention on hot paths** | Use lock-free data structures (DashMap) |
| **Slow capability matching** | Use SIMD vectorization (AVX2) |
| **Memory allocation overhead** | Use arena allocators (bumpalo) |
| **Non-deterministic execution** | Generate receipts with SHA-256 hashing |
| **Byzantine agents affecting consensus** | Use PBFT with f < n/3 tolerance |
| **Pheromone stagnation** | Implement evaporation (decay over time) |
| **Anomaly detection false positives** | Use multiple techniques (EWMA + SPC + CUSUM) |
| **Remediation causing cascading failures** | Use canary rollout with automatic rollback |

---

## 📦 Required Dependencies

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["full"] }
crossbeam = "0.8"       # Lock-free
dashmap = "5.5"         # Concurrent hash map
sha2 = "0.10"           # SHA-256
bumpalo = "3.14"        # Arena allocator
rand = "0.8"            # Gossip randomization

[dev-dependencies]
criterion = "0.5"       # Benchmarking
proptest = "1.0"        # Property testing
loom = "0.7"            # Concurrency testing
insta = "1.34"          # Snapshot testing
```

---

## 🎓 Learning Resources

1. **Type-State Pattern**: [Session Types in Rust](https://willcrichton.net/rust-api-type-patterns/typestate.html)
2. **PBFT**: [Practical Byzantine Fault Tolerance](http://pmg.csail.mit.edu/papers/osdi99.pdf)
3. **EigenTrust**: [EigenTrust Algorithm](https://nlp.stanford.edu/pubs/eigentrust.pdf)
4. **MAPE-K**: [IBM Autonomic Computing](https://www.cs.cmu.edu/~garlan/15-821/papers/AC_Blueprint_White_Paper_V7.pdf)
5. **Stigmergy**: [Stigmergy in Agent Systems](https://www.cs.unb.ca/~bremner/teaching/cs4725/readings/parunak06.pdf)

---

## ✅ Definition of Done

Before marking implementation complete:

- [ ] All SLOs met (< 50ns lookup, < 10ns SIMD, < 10μs allocation, etc.)
- [ ] 80%+ line coverage, 75%+ branch coverage
- [ ] 100% critical path coverage (state transitions, consensus, receipts)
- [ ] Zero compiler warnings (`cargo make lint`)
- [ ] All tests pass (`cargo make test`)
- [ ] Benchmarks pass regression threshold (< 5% degradation)
- [ ] Documentation complete (API docs, user guide, examples)
- [ ] Chaos tests validate fault tolerance

---

**Last Updated**: 2026-01-05
**Version**: 1.0.0
**Status**: Design Complete - Ready for Implementation
