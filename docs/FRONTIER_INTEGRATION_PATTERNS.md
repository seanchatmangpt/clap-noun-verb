# Frontier Integration Patterns

**Version:** 1.0.0
**Date:** 2026-01-05
**Companion to:** FRONTIER_MASTER_ARCHITECTURE.md
**Purpose:** Design patterns and best practices for frontier package integration

---

## Table of Contents

1. [Integration Patterns Overview](#1-integration-patterns-overview)
2. [Zero-Cost Adapter Pattern](#2-zero-cost-adapter-pattern)
3. [Feature-Gate Patterns](#3-feature-gate-patterns)
4. [Type-State Pattern](#4-type-state-pattern)
5. [Builder Pattern for Complex Features](#5-builder-pattern-for-complex-features)
6. [Error Propagation Patterns](#6-error-propagation-patterns)
7. [Testing Patterns](#7-testing-patterns)
8. [Performance Optimization Patterns](#8-performance-optimization-patterns)

---

## 1. Integration Patterns Overview

### Pattern Catalog

| Pattern | Use Case | Benefits | Trade-offs |
|---------|----------|----------|------------|
| Zero-Cost Adapter | External package integration | No runtime overhead | More abstraction layers |
| Feature-Gate | Optional functionality | Compile-time selection | Testing complexity |
| Type-State | State machine enforcement | Compile-time safety | Verbose types |
| Builder | Complex object construction | Ergonomic API | More boilerplate |
| Newtype | Type safety | Strong typing | Unwrapping overhead |
| Trait Object | Runtime polymorphism | Flexibility | Dynamic dispatch cost |
| Monomorphization | Generic specialization | Zero-cost abstraction | Code bloat |

---

## 2. Zero-Cost Adapter Pattern

### Pattern Description

Wrap external package types in internal traits to enable:
1. API stability (swap backends without user code changes)
2. Zero runtime overhead (inlined, monomorphized)
3. Type safety (catch errors at compile time)

### Implementation Template

```rust
// Step 1: Define stable trait interface
pub trait BackendTrait {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

// Step 2: Implement adapter for external package
#[cfg(feature = "backend-oxigraph")]
mod oxigraph_adapter {
    use super::*;
    use oxigraph::store::Store;

    pub struct OxigraphBackend(Store);

    impl BackendTrait for OxigraphBackend {
        type Input = String;
        type Output = Vec<String>;
        type Error = oxigraph::store::StoreError;

        #[inline(always)] // Zero-cost: force inlining
        fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            // Delegate to external package
            self.0.query(&input)?
                .map(|result| result.to_string())
                .collect()
        }
    }
}

// Step 3: Type alias for current backend (can be swapped)
#[cfg(feature = "backend-oxigraph")]
pub type DefaultBackend = oxigraph_adapter::OxigraphBackend;

// Step 4: Alternative backend (future-proofing)
#[cfg(feature = "backend-sophia")]
pub type DefaultBackend = sophia_adapter::SophiaBackend;

// Step 5: Public API uses trait, not concrete type
pub fn create_processor() -> impl BackendTrait {
    DefaultBackend::new()
}
```

### Verification of Zero-Cost

```rust
// Benchmark to verify zero overhead
#[bench]
fn bench_direct_usage(b: &mut Bencher) {
    let store = oxigraph::store::Store::new().unwrap();
    b.iter(|| {
        store.query("SELECT * WHERE { ?s ?p ?o }").unwrap();
    });
}

#[bench]
fn bench_adapter_usage(b: &mut Bencher) {
    let backend = DefaultBackend::new();
    b.iter(|| {
        backend.process("SELECT * WHERE { ?s ?p ?o }".into()).unwrap();
    });
}

// Verify: Both benchmarks should show identical performance
```

### When to Use

✅ **Use when:**
- Integrating external packages with unstable APIs
- Need to support multiple backends
- Performance-critical code paths

❌ **Avoid when:**
- Simple wrapper with no abstraction needed
- API is guaranteed stable
- Not performance-sensitive

---

## 3. Feature-Gate Patterns

### Pattern 1: Optional Module

```rust
// lib.rs
#[cfg(feature = "meta-framework")]
pub mod meta_framework;

// Conditional re-export
#[cfg(feature = "meta-framework")]
pub use meta_framework::{MetaIntrospectable, CapabilityDescriptor};
```

### Pattern 2: Conditional Trait Implementations

```rust
// Always-available type
pub struct MyStruct {
    data: Vec<u8>,
}

// Conditional trait impl
#[cfg(feature = "rdf-composition")]
impl SemanticComposable for MyStruct {
    fn rdf_representation(&self) -> RdfGraph {
        // Implementation only when feature enabled
    }
}
```

### Pattern 3: Feature-Dependent Types

```rust
// Different types based on features
#[cfg(feature = "discovery-pso")]
pub type DefaultOptimizer = PsoOptimizer;

#[cfg(all(feature = "discovery-ga", not(feature = "discovery-pso")))]
pub type DefaultOptimizer = GeneticOptimizer;

#[cfg(not(any(feature = "discovery-pso", feature = "discovery-ga")))]
pub type DefaultOptimizer = FallbackOptimizer;
```

### Pattern 4: Runtime Feature Detection

```rust
pub fn capabilities() -> Vec<&'static str> {
    let mut caps = vec![];

    #[cfg(feature = "meta-framework")]
    caps.push("meta-framework");

    #[cfg(feature = "rdf-composition")]
    caps.push("rdf-composition");

    caps
}

// Macro for feature checking
macro_rules! require_feature {
    ($feature:literal) => {
        #[cfg(not(feature = $feature))]
        compile_error!(concat!("This code requires feature: ", $feature));
    };
}
```

---

## 4. Type-State Pattern

### Pattern Description

Use zero-sized types to encode state machines at compile time.
Invalid state transitions become compile errors.

### Implementation Example

```rust
// State markers (zero-sized)
pub struct Uninitialized;
pub struct Initialized;
pub struct Running;
pub struct Stopped;

// Parameterized type with state
pub struct Agent<S> {
    state: PhantomData<S>,
    data: AgentData,
}

// State-specific methods
impl Agent<Uninitialized> {
    pub fn new() -> Self {
        Agent {
            state: PhantomData,
            data: AgentData::default(),
        }
    }

    // Transition: Uninitialized → Initialized
    pub fn initialize(self, config: Config) -> Agent<Initialized> {
        Agent {
            state: PhantomData,
            data: self.data.with_config(config),
        }
    }
}

impl Agent<Initialized> {
    // Transition: Initialized → Running
    pub fn start(self) -> Agent<Running> {
        Agent {
            state: PhantomData,
            data: self.data.start_services(),
        }
    }
}

impl Agent<Running> {
    pub fn process(&self, task: Task) -> Result<Output> {
        // Only running agents can process
        self.data.execute(task)
    }

    // Transition: Running → Stopped
    pub fn stop(self) -> Agent<Stopped> {
        Agent {
            state: PhantomData,
            data: self.data.shutdown(),
        }
    }
}

// Usage enforces state machine
fn example() {
    let agent = Agent::new();          // Uninitialized
    let agent = agent.initialize(cfg); // Initialized
    let agent = agent.start();         // Running

    agent.process(task); // ✅ OK

    // agent.start(); // ❌ Compile error: already running
}
```

### Benefits

- Invalid states unrepresentable (compile-time guarantee)
- Zero runtime overhead (PhantomData is zero-sized)
- Self-documenting API (states explicit in types)
- Compiler enforces correct usage

---

## 5. Builder Pattern for Complex Features

### Pattern Description

Use builder pattern for types with many optional configurations.

### Implementation

```rust
pub struct FederatedNetworkBuilder {
    discovery: Option<DiscoveryConfig>,
    consensus: Option<ConsensusConfig>,
    transport: Option<TransportConfig>,
    trust: Option<TrustConfig>,
}

impl FederatedNetworkBuilder {
    pub fn new() -> Self {
        Self {
            discovery: None,
            consensus: None,
            transport: None,
            trust: None,
        }
    }

    pub fn with_discovery(mut self, config: DiscoveryConfig) -> Self {
        self.discovery = Some(config);
        self
    }

    pub fn with_consensus(mut self, config: ConsensusConfig) -> Self {
        self.consensus = Some(config);
        self
    }

    // Type-state enforcement: only build when all required configs present
    pub fn build(self) -> Result<FederatedNetwork, BuildError> {
        Ok(FederatedNetwork {
            discovery: self.discovery.ok_or(BuildError::MissingDiscovery)?,
            consensus: self.consensus.ok_or(BuildError::MissingConsensus)?,
            transport: self.transport.unwrap_or_default(),
            trust: self.trust.unwrap_or_default(),
        })
    }
}

// Usage
let network = FederatedNetworkBuilder::new()
    .with_discovery(DiscoveryConfig::default())
    .with_consensus(ConsensusConfig::bft())
    .build()?;
```

### Advanced: Type-State Builder

```rust
// State markers
pub struct NoDiscovery;
pub struct HasDiscovery;

// Builder parameterized by completeness
pub struct FederatedNetworkBuilder<D> {
    discovery: Option<DiscoveryConfig>,
    _marker: PhantomData<D>,
}

impl FederatedNetworkBuilder<NoDiscovery> {
    pub fn new() -> Self { /* ... */ }

    // Transition to HasDiscovery state
    pub fn with_discovery(self, config: DiscoveryConfig)
        -> FederatedNetworkBuilder<HasDiscovery>
    {
        FederatedNetworkBuilder {
            discovery: Some(config),
            _marker: PhantomData,
        }
    }
}

impl FederatedNetworkBuilder<HasDiscovery> {
    // Only available when discovery is set
    pub fn build(self) -> FederatedNetwork {
        FederatedNetwork {
            discovery: self.discovery.unwrap(), // Safe: guaranteed by type
        }
    }
}
```

---

## 6. Error Propagation Patterns

### Pattern 1: Error Context with `thiserror`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Failed to parse input: {input}")]
    ParseError {
        input: String,
        #[source]
        source: ParseError,
    },

    #[error("Network timeout after {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
    },

    #[error("Resource {resource} not found")]
    NotFound {
        resource: String,
    },
}

// Usage with context
fn process(input: &str) -> Result<Output, ProcessingError> {
    let parsed = parse(input)
        .map_err(|e| ProcessingError::ParseError {
            input: input.to_string(),
            source: e,
        })?;

    Ok(transform(parsed))
}
```

### Pattern 2: Error Conversion Chain

```rust
// External error → Internal error
impl From<oxigraph::store::StoreError> for RdfCompositionError {
    fn from(err: oxigraph::store::StoreError) -> Self {
        RdfCompositionError::TripleInsertion(err.to_string())
    }
}

// Internal error → Frontier error
impl From<RdfCompositionError> for FrontierError {
    fn from(err: RdfCompositionError) -> Self {
        FrontierError::RdfComposition(err)
    }
}

// Usage: Automatic conversion with ?
fn example() -> Result<(), FrontierError> {
    let store = Store::new()?; // StoreError → RdfCompositionError → FrontierError
    Ok(())
}
```

### Pattern 3: Early Return with Context

```rust
// Macro for context-aware early return
macro_rules! context {
    ($result:expr, $msg:expr) => {
        $result.map_err(|e| {
            FrontierError::Configuration(
                format!("{}: {}", $msg, e)
            )
        })?
    };
}

// Usage
fn load_config(path: &Path) -> Result<Config, FrontierError> {
    let contents = context!(
        fs::read_to_string(path),
        format!("Failed to read config from {}", path.display())
    );

    let config = context!(
        serde_json::from_str(&contents),
        "Failed to parse JSON"
    );

    Ok(config)
}
```

---

## 7. Testing Patterns

### Pattern 1: Chicago TDD (State-Based Testing)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_finds_capabilities() {
        // Arrange: Set up initial state
        let mut engine = DiscoveryEngine::new();
        engine.register_capability("user.create");
        engine.register_capability("user.delete");

        // Act: Perform operation
        let results = engine.discover("user.*");

        // Assert: Verify observable output
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"user.create"));
        assert!(results.contains(&"user.delete"));
    }
}
```

### Pattern 2: Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_optimizer_always_improves(
        initial_fitness in 0.0..1.0,
        iterations in 1..100usize,
    ) {
        let mut optimizer = PsoOptimizer::new();
        let final_fitness = optimizer.optimize(iterations);

        // Property: Optimization never makes things worse
        prop_assert!(final_fitness >= initial_fitness);
    }
}
```

### Pattern 3: Integration Testing with Feature Combinations

```rust
#[cfg(all(feature = "meta-framework", feature = "rdf-composition"))]
#[test]
fn test_meta_framework_rdf_integration() {
    // Test that meta-framework and rdf-composition work together
    let introspector = MetaFramework::new();
    let capabilities = introspector.capabilities();

    let mut store = RdfStore::new();
    for cap in capabilities {
        store.insert_triple(cap.to_rdf_triple()).unwrap();
    }

    let query_results = store.query("SELECT * WHERE { ?s ?p ?o }").unwrap();
    assert!(!query_results.is_empty());
}
```

### Pattern 4: Benchmark Testing

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_discovery_engine(c: &mut Criterion) {
    let mut engine = DiscoveryEngine::new();
    for i in 0..500 {
        engine.register_capability(&format!("cap.{}", i));
    }

    c.bench_function("discover 500 capabilities", |b| {
        b.iter(|| {
            let results = engine.discover("cap.*");
            black_box(results);
        })
    });
}

criterion_group!(benches, bench_discovery_engine);
criterion_main!(benches);
```

---

## 8. Performance Optimization Patterns

### Pattern 1: Const Generics for Zero-Cost

```rust
// Zero-cost array size specification
pub struct FixedCapacityQueue<T, const N: usize> {
    items: [Option<T>; N],
    head: usize,
}

impl<T, const N: usize> FixedCapacityQueue<T, N> {
    pub const fn new() -> Self {
        Self {
            items: [const { None }; N],
            head: 0,
        }
    }

    // Compile-time capacity checking
    pub fn push(&mut self, item: T) -> Result<(), QueueFullError> {
        if self.head >= N {
            return Err(QueueFullError);
        }
        self.items[self.head] = Some(item);
        self.head += 1;
        Ok(())
    }
}

// Usage: Size known at compile time, no runtime overhead
let mut queue: FixedCapacityQueue<Task, 100> = FixedCapacityQueue::new();
```

### Pattern 2: Inline Hints

```rust
// Force inlining for hot paths
#[inline(always)]
pub fn fast_path(&self, input: u64) -> u64 {
    input.wrapping_mul(2).wrapping_add(1)
}

// Suggest inlining (compiler decides)
#[inline]
pub fn moderate_path(&self, input: &str) -> String {
    format!("processed: {}", input)
}

// Prevent inlining (force function call)
#[inline(never)]
pub fn cold_path(&self, error: &Error) {
    log::error!("Error: {}", error);
}
```

### Pattern 3: Lazy Initialization

```rust
use once_cell::sync::Lazy;

// Initialize once, reuse forever
static GLOBAL_REGISTRY: Lazy<CapabilityRegistry> = Lazy::new(|| {
    let mut registry = CapabilityRegistry::new();
    registry.scan_capabilities();
    registry
});

// Usage: First access initializes, subsequent accesses are instant
fn get_registry() -> &'static CapabilityRegistry {
    &GLOBAL_REGISTRY
}
```

### Pattern 4: SmallVec for Stack Optimization

```rust
use smallvec::SmallVec;

// Store up to 8 items on stack, spill to heap if more
pub struct Capabilities {
    items: SmallVec<[Capability; 8]>,
}

// Benefits:
// - Most use cases: 0 heap allocations
// - Rare large cases: Automatic heap promotion
// - No runtime overhead for common case
```

### Pattern 5: Copy-on-Write with `Cow`

```rust
use std::borrow::Cow;

pub fn process_string(input: &str) -> Cow<str> {
    if input.contains("special") {
        // Need to modify: allocate new string
        Cow::Owned(input.replace("special", "SPECIAL"))
    } else {
        // No modification needed: borrow original
        Cow::Borrowed(input)
    }
}

// Caller benefits from zero allocation in common case
let result = process_string("normal input"); // No allocation!
```

---

## Summary: Pattern Selection Guide

### When to Use Each Pattern

| Pattern | Performance Impact | Type Safety | Ergonomics | Complexity |
|---------|-------------------|-------------|------------|------------|
| Zero-Cost Adapter | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | Medium |
| Feature-Gate | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | Low |
| Type-State | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Medium |
| Builder | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Low |
| Const Generics | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Medium |
| Lazy Init | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | Low |

### Pattern Combinations

**Best Practices:**
1. **External Integration:** Zero-Cost Adapter + Feature-Gate
2. **State Machines:** Type-State + Builder
3. **Configuration:** Builder + Lazy Init
4. **Performance-Critical:** Const Generics + Inline Hints
5. **Error Handling:** thiserror + Context Propagation

---

**End of Integration Patterns Guide**

These patterns should be applied consistently across all frontier features to ensure:
- Zero-cost abstractions where possible
- Type safety at compile time
- Ergonomic APIs for users
- Maintainable codebase for developers
