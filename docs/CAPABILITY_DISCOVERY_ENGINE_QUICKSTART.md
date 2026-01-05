# Capability Discovery Engine - Quick Start Guide

**Version**: 1.0.0
**Date**: 2026-01-05
**For**: Developers and System Architects

---

## What Is This?

The **Capability Discovery Engine** autonomously finds optimal combinations of agent capabilities by:
- Exploring exponential search spaces efficiently (2^N combinations)
- Evaluating combinations with multi-objective fitness functions
- Validating safety using Byzantine consensus
- Leveraging RDF semantic queries for pattern matching

**Think**: Genetic algorithms meet Byzantine fault tolerance for capability composition.

---

## Quick Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│  Developer: "Find optimal capabilities for X"       │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  Discovery Engine                                    │
│  • PSO/GA/ACO algorithms explore search space       │
│  • Multi-objective fitness scoring                  │
│  • Byzantine consensus validates safety             │
└────────────────┬────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────┐
│  Validated Suggestions with Safety Proofs           │
│  • Combination: [cap_0, cap_3, cap_7]               │
│  • Fitness: 0.87 (novelty: 0.9, safety: 1.0)        │
│  • Proof: Validated by 5/7 Byzantine nodes          │
└─────────────────────────────────────────────────────┘
```

---

## Core Type Signatures

### Capability Combination (Zero-Cost Bitmap)

```rust
/// Represents a combination of up to N capabilities
pub struct Combination<const N: usize> {
    bits: [u64; (N + 63) / 64],  // Bitmap: O(N/64) space
    count: usize,                 // Cached for performance
}

// Example: N=64 capabilities -> 8 bytes (one u64)
let mut combo = Combination::<64>::empty();
combo.add(3);   // O(1) - set bit 3
combo.add(7);   // O(1) - set bit 7
assert!(combo.contains(3));  // O(1) check
```

**Why Bitmap?**
- O(1) add/remove/contains operations
- O(N/64) Hamming distance calculation
- Stack-allocated, no heap fragmentation
- Cache-friendly iteration

### Fitness Function Trait

```rust
pub trait FitnessFunction: Send + Sync {
    fn score(&self, combo: &Combination<impl CapabilityCount>) -> f64;
    fn name(&self) -> &str;
    fn weight(&self) -> f64 { 1.0 }
}

// Example implementation
pub struct NoveltyFitness;

impl FitnessFunction for NoveltyFitness {
    fn score(&self, combo: &Combination<impl CapabilityCount>) -> f64 {
        // Measure novelty as distance to seen combinations
        // Returns 0.0 (not novel) to 1.0 (maximally novel)
        0.85
    }

    fn name(&self) -> &str { "Novelty" }
}
```

**Built-in Fitness Dimensions**:
1. **Novelty**: How unique is this combination?
2. **Coverage**: How many effect types are covered?
3. **Utility**: How likely is this to be useful? (RDF pattern frequency)
4. **Safety**: Can this be proven safe? (Byzantine validation)
5. **Performance**: How efficient is this combination?

### Discovery Algorithm Trait

```rust
pub trait DiscoveryAlgorithm: Send + Sync {
    fn initialize<const N: usize>(
        &mut self,
        space: &CapabilitySpace<N>,
        fitness: &FitnessComposite,
    );

    fn step<const N: usize>(
        &mut self,
        space: &CapabilitySpace<N>,
        fitness: &FitnessComposite,
    ) -> Vec<Combination<N>>;

    fn converged(&self) -> bool;
    fn name(&self) -> &str;
}
```

**Implemented Algorithms**:
- **PSO (Particle Swarm Optimization)**: Best for continuous exploration
- **GA (Genetic Algorithm)**: Best for discrete optimization
- **ACO (Ant Colony Optimization)**: Best for path-based problems

---

## Algorithm Specifications

### Particle Swarm Optimization (PSO)

**Pseudocode**:
```
FOR each iteration:
    FOR each particle:
        velocity = inertia * velocity
                 + cognitive * random() * (personal_best - position)
                 + social * random() * (global_best - position)

        FOR each capability:
            IF random() < sigmoid(velocity[capability]):
                position[capability] = 1  # Include capability
            ELSE:
                position[capability] = 0  # Exclude capability

        fitness = EVALUATE(position)

        IF fitness > personal_best_fitness:
            personal_best = position

        IF fitness > global_best_fitness:
            global_best = position
```

**Complexity**:
- Time: O(P × N × I) where P=particles, N=capabilities, I=iterations
- Space: O(P × N)
- **Example**: 50 particles, 64 capabilities, 1000 iterations = 3.2M operations (~5 seconds)

**When to Use**:
- Continuous search spaces
- Need fast convergence
- Want exploration diversity

### Genetic Algorithm (GA)

**Pseudocode**:
```
population = RANDOM_POPULATION(size)

FOR each generation:
    scores = EVALUATE(population)
    elite = TOP_K(scores, elite_size)

    new_population = elite

    WHILE LENGTH(new_population) < population_size:
        parent1 = TOURNAMENT_SELECT(scores)
        parent2 = TOURNAMENT_SELECT(scores)

        child = CROSSOVER(parent1, parent2)
        child = MUTATE(child, mutation_rate)

        new_population.ADD(child)

    population = new_population
```

**Complexity**:
- Time: O(G × N log G) where G=population_size
- Space: O(G × N)
- **Example**: 100 population, 64 capabilities, 500 generations = 3.2M operations (~5 seconds)

**When to Use**:
- Discrete optimization
- Need elitism (preserve best solutions)
- Want crossover between solutions

---

## Validation System

### Byzantine Consensus Validation

```rust
pub struct SuggestionValidator {
    consensus: Arc<ConsensusEngine>,
    validators: Vec<ValidatorNode>,
    rdf_engine: Arc<SemanticQueryEngine>,
}

impl SuggestionValidator {
    pub async fn validate<const N: usize>(
        &self,
        combo: &Combination<N>,
    ) -> Result<SafetyProof, ValidationError> {
        let proposal_id = blake3::hash(combo.as_bytes()).to_hex();

        // Each validator votes on safety
        for validator in &self.validators {
            if validator.validate(combo).safe {
                self.consensus.vote(&proposal_id, validator.node_id).await;
            }
        }

        // Byzantine threshold: 2f+1 where f = max faulty nodes
        let min_votes = (self.validators.len() * 2 / 3) + 1;

        if self.consensus.has_consensus(&proposal_id, min_votes).await {
            Ok(SafetyProof {
                combination: combo.clone(),
                validators: /* ... */,
                proof_id: proposal_id,
            })
        } else {
            Err(ValidationError::ConsensusNotReached)
        }
    }
}
```

**Validation Rules**:
1. **Conflict Detection**: No capabilities with `cnv:conflictsWith` relationships
2. **Dependency Satisfaction**: All dependencies included in combination
3. **Resource Bounds**: Memory/CPU limits respected
4. **Semantic Coherence**: RDF queries validate logical consistency

**Safety Proof**:
```rust
pub struct SafetyProof {
    pub combination: Combination<N>,
    pub validators: Vec<String>,      // Node IDs that voted
    pub timestamp: DateTime<Utc>,
    pub proof_id: String,              // Blake3 hash
}
```

---

## RDF Integration

### SPARQL Query Examples

**Find Conflicting Capabilities**:
```sparql
PREFIX cnv: <http://clap-noun-verb.rs/ontology#>

ASK {
    ?cap1 cnv:conflictsWith ?cap2 .
    VALUES ?cap1 { cnv:cap_0 cnv:cap_3 cnv:cap_7 }
    VALUES ?cap2 { cnv:cap_0 cnv:cap_3 cnv:cap_7 }
}
```

**Measure Semantic Coherence**:
```sparql
PREFIX cnv: <http://clap-noun-verb.rs/ontology#>

SELECT (COUNT(*) AS ?relations) WHERE {
    ?cap1 cnv:relatedTo ?cap2 .
    VALUES ?cap1 { cnv:cap_0 cnv:cap_3 cnv:cap_7 }
    VALUES ?cap2 { cnv:cap_0 cnv:cap_3 cnv:cap_7 }
}
```

**Find Similar Patterns**:
```sparql
PREFIX cnv: <http://clap-noun-verb.rs/ontology#>

SELECT ?pattern (COUNT(?cap) AS ?overlap) WHERE {
    VALUES ?inputCap { cnv:cap_0 cnv:cap_3 cnv:cap_7 }

    ?pattern a cnv:CapabilityPattern ;
             cnv:includes ?cap .

    FILTER(?cap IN (?inputCap))
}
GROUP BY ?pattern
HAVING (COUNT(?cap) >= 2)
ORDER BY DESC(?overlap)
```

---

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Example (N=64) |
|-----------|-----------|----------------|
| Combination add/remove | O(1) | 1 nanosecond |
| Combination contains | O(1) | 1 nanosecond |
| Hamming distance | O(N/64) | 8 operations |
| Fitness evaluation | O(F × N) | F=5 functions × 64 = 320 ops |
| PSO iteration | O(P × N) | 50 particles × 64 = 3,200 ops |
| Byzantine validation | O(V^2) | 7 validators = 49 ops |
| SPARQL query | O(T log T) | T=1000 triples ≈ 10,000 ops |

### Space Complexity

| Structure | Complexity | Example (N=64) |
|-----------|-----------|----------------|
| Combination | O(N/64) | 8 bytes (one u64) |
| PSO Swarm (50 particles) | O(P × N) | 50 × 8 = 400 bytes |
| GA Population (100) | O(G × N) | 100 × 8 = 800 bytes |
| ACO Pheromone Matrix | O(N^2) | 64 × 64 × 8 = 32KB |

**Total Memory** (PSO with 50 particles, 64 capabilities):
- Swarm: 400 bytes
- Fitness cache: ~10KB (1000 cached results)
- RDF graph: ~50KB (1000 triples)
- **Total**: ~60KB

---

## Integration Examples

### Example 1: Discover Optimal Agent Configurations

```rust
use clap_noun_verb::discovery::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load capability space from RDF ontology
    let ontology = OntologyBuilder::from_file("capabilities.ttl")?.build()?;
    let space = CapabilitySpace::<64>::from_ontology(&ontology)?;

    // Create fitness composite
    let fitness = FitnessComposite::new()
        .add(NoveltyFitness::new())
        .add(UtilityFitness::new(rdf_engine))
        .add(SafetyFitness::new(validator));

    // Initialize PSO
    let mut pso = ParticleSwarmOptimizer::new(PSOConfig {
        num_particles: 50,
        inertia: 0.7,
        cognitive: 1.5,
        social: 1.5,
        max_iterations: 1000,
    });

    pso.initialize(&space, &fitness);

    // Run discovery
    while !pso.converged() {
        let discoveries = pso.step(&space, &fitness);

        for combo in discoveries {
            println!("Discovered: {:?}", combo.iter().collect::<Vec<_>>());
            println!("Fitness: {:.3}", fitness.score(&combo).total);
        }
    }

    Ok(())
}
```

### Example 2: Suggest Capabilities for Workflow

```rust
use clap_noun_verb::discovery::CapabilitySuggester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut suggester = CapabilitySuggester::new(/* ... */);

    let suggestions = suggester.suggest_for_workflow(
        "distributed data processing with caching",
        &Constraints {
            max_capabilities: 5,
            required_effects: vec![EffectType::Compute, EffectType::NetworkIO],
            max_sensitivity: Sensitivity::Medium,
        },
    )?;

    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("\n=== Suggestion #{} ===", i + 1);
        println!("Fitness: {:.3}", suggestion.fitness.total);
        println!("  - Novelty: {:.2}", suggestion.fitness.dimensions[0].score);
        println!("  - Utility: {:.2}", suggestion.fitness.dimensions[1].score);
        println!("  - Safety: {:.2}", suggestion.fitness.dimensions[2].score);

        println!("Capabilities:");
        for idx in suggestion.combination.iter() {
            let cap = space.get_capability(idx)?;
            println!("  - {}: {}", cap.name, cap.effect_type);
        }

        if let Some(proof) = &suggestion.safety_proof {
            println!("Safety: ✅ Validated by {} Byzantine nodes", proof.validators.len());
        }
    }

    Ok(())
}
```

### Example 3: Export Discoveries to RDF

```rust
// Export discovered patterns to RDF for future queries
let discovered_combos = vec![/* ... */];
let ttl = space.export_to_rdf(&discovered_combos);

std::fs::write("discovered_patterns.ttl", ttl)?;

// Load into ontology
ontology.load_from_str(&ttl, GraphFormat::Turtle)?;

// Now patterns can be queried via SPARQL
let query = r#"
    PREFIX cnv: <http://clap-noun-verb.rs/ontology#>
    SELECT ?pattern ?cap WHERE {
        ?pattern a cnv:CapabilityPattern ;
                 cnv:includes ?cap .
    }
"#;
```

---

## Key Design Decisions (ADRs)

### ADR-001: Bitmap Representation
**Decision**: Use const-generic bitmap `[u64; (N + 63) / 64]`
**Why**: O(1) operations, zero-cost, cache-friendly
**Trade-off**: Requires const generics (Rust 1.51+)

### ADR-002: Byzantine Consensus
**Decision**: 2f+1 Byzantine threshold for validation
**Why**: Tolerates up to f faulty validators, provable safety
**Trade-off**: Higher latency due to consensus overhead

### ADR-003: Multi-Objective Fitness
**Decision**: Weighted composite of independent fitness functions
**Why**: Extensible, transparent, tunable
**Trade-off**: Requires domain knowledge to set weights

### ADR-004: RDF/SPARQL
**Decision**: Use SPARQL for semantic queries
**Why**: Standard, expressive, integrates with existing semantic CLI
**Trade-off**: SPARQL learning curve

### ADR-005: Swarm Intelligence
**Decision**: PSO/GA/ACO instead of exhaustive search
**Why**: Scales to exponential spaces, converges efficiently
**Trade-off**: No guarantee of global optimum (stochastic)

---

## Metrics and Coverage

### Search Coverage

For N=64 capabilities:
- Total combinations: 2^64 - 1 ≈ 1.8 × 10^19
- PSO evaluations (1000 iterations, 50 particles): 50,000
- Coverage: 50,000 / 1.8 × 10^19 ≈ **0.000000003%**

**Yet PSO finds near-optimal solutions!** Why?
- Swarm intelligence exploits fitness gradients
- Particles share information (social learning)
- Converges to local optima efficiently

### Convergence Metrics

```rust
pub struct DiscoveryMetrics {
    pub iterations: usize,
    pub combinations_evaluated: usize,
    pub best_fitness: f64,
    pub convergence_rate: f64,        // % improvement per iteration
    pub search_coverage: f64,         // % of space explored
}

// Example after 500 iterations
DiscoveryMetrics {
    iterations: 500,
    combinations_evaluated: 25_000,
    best_fitness: 0.87,
    convergence_rate: 0.001,  // 0.1% improvement per iteration
    search_coverage: 0.0000000014,  // Negligible but effective
}
```

---

## Common Use Cases

### Use Case 1: Agent Capability Optimization
**Goal**: Find optimal capability sets for distributed agents
**Algorithm**: PSO (fast convergence, exploration diversity)
**Fitness**: Novelty (0.3) + Utility (0.4) + Safety (0.3)
**Validation**: Byzantine consensus (7 validators, 5/7 threshold)

### Use Case 2: Workflow Automation
**Goal**: Suggest capabilities for user-described workflows
**Algorithm**: GA (discrete optimization, elitism)
**Fitness**: Coverage (0.4) + Utility (0.4) + Performance (0.2)
**Validation**: Quick safety check (no consensus)

### Use Case 3: Exploratory Research
**Goal**: Discover novel capability patterns
**Algorithm**: ACO (path-based exploration)
**Fitness**: Novelty (0.6) + Coverage (0.3) + Safety (0.1)
**Validation**: Full Byzantine consensus + RDF pattern matching

---

## Performance Tuning

### PSO Configuration

```rust
// Fast exploration (fewer particles, more iterations)
PSOConfig {
    num_particles: 30,
    max_iterations: 2000,
    inertia: 0.5,      // Lower inertia = faster convergence
    cognitive: 2.0,    // Higher cognitive = more personal exploration
    social: 1.0,       // Lower social = less swarm influence
}

// Thorough exploration (more particles, balanced)
PSOConfig {
    num_particles: 100,
    max_iterations: 1000,
    inertia: 0.7,      // Balanced momentum
    cognitive: 1.5,    // Balanced personal exploration
    social: 1.5,       // Balanced swarm influence
}
```

### GA Configuration

```rust
// Fast convergence (smaller population, high mutation)
GAConfig {
    population_size: 50,
    elite_size: 10,
    mutation_rate: 0.2,    // High mutation = more exploration
    crossover_rate: 0.6,   // Lower crossover = faster convergence
    max_generations: 500,
}

// Diverse exploration (larger population, balanced)
GAConfig {
    population_size: 200,
    elite_size: 20,
    mutation_rate: 0.1,    // Low mutation = preserve good solutions
    crossover_rate: 0.8,   // High crossover = combine solutions
    max_generations: 1000,
}
```

---

## Next Steps

1. **Read Full Architecture**: See [CAPABILITY_DISCOVERY_ENGINE_ARCHITECTURE.md](CAPABILITY_DISCOVERY_ENGINE_ARCHITECTURE.md)
2. **Review Type System**: Section 3 - Core types and traits
3. **Understand Algorithms**: Section 4 - PSO/GA/ACO implementations
4. **Study Validation**: Section 6 - Byzantine consensus validation
5. **Explore Integration**: Section 9 - Integration with agent coordination

---

## References

- **Full Architecture**: [CAPABILITY_DISCOVERY_ENGINE_ARCHITECTURE.md](CAPABILITY_DISCOVERY_ENGINE_ARCHITECTURE.md)
- **Agent Coordination**: [/home/user/clap-noun-verb/src/agent2028/coordination.rs](../src/agent2028/coordination.rs)
- **Semantic CLI**: [SEMANTIC_CLI_ARCHITECTURE.md](SEMANTIC_CLI_ARCHITECTURE.md)
- **Capability Catalog**: [CAPABILITY_CATALOG.md](CAPABILITY_CATALOG.md)

**Papers**:
- Kennedy & Eberhart (1995). "Particle Swarm Optimization"
- Dorigo & Stützle (2004). "Ant Colony Optimization"
- Castro & Liskov (1999). "Practical Byzantine Fault Tolerance"

---

**Document Status**: ✅ QUICKSTART COMPLETE
**Last Updated**: 2026-01-05
**Next**: Implement core types (Section 3 of full architecture)
