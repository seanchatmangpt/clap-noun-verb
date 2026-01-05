# Feature Composition Guide: Combining Advanced Features Effectively

**Version**: 5.3.4
**Date**: 2026-01-05
**Complexity**: Expert
**Prerequisites**: Understanding of all advanced features

---

## Table of Contents

1. [Feature Compatibility Matrix](#1-feature-compatibility-matrix)
2. [Progressive Adoption Path](#2-progressive-adoption-path)
3. [Common Composition Patterns](#3-common-composition-patterns)
4. [Architecture Patterns](#4-architecture-patterns)
5. [Performance Optimization](#5-performance-optimization)
6. [Testing Composed Systems](#6-testing-composed-systems)
7. [Production Deployment Strategies](#7-production-deployment-strategies)
8. [Migration Guides](#8-migration-guides)
9. [Troubleshooting](#9-troubleshooting)
10. [Best Practices](#10-best-practices)

---

## 1. Feature Compatibility Matrix

### 1.1 Feature Dependencies

```
┌──────────────┬─────┬─────────┬─────────┬──────────┬─────┬────────────┐
│ Feature      │async│agent2028│autonomic│ crypto   │ rdf │observability│
├──────────────┼─────┼─────────┼─────────┼──────────┼─────┼────────────┤
│ async        │  -  │    ✓    │    ✓    │    ○    │  ○  │     ○      │
│ agent2028    │  ✓  │    -    │    ✓    │    ✓    │  ○  │     ✓      │
│ autonomic    │  ✓  │    ○    │    -    │    ○    │  ○  │     ✓      │
│ crypto       │  ○  │    ○    │    ○    │    -    │  ✓  │     ○      │
│ rdf          │  ○  │    ○    │    ○    │    ✓    │  -  │     ○      │
│observability │  ○  │    ○    │    ○    │    ○    │  ○  │     -      │
└──────────────┴─────┴─────────┴─────────┴──────────┴─────┴────────────┘

Legend:
  ✓ = Required dependency
  ○ = Optional but recommended
  - = Self
```

### 1.2 Feature Flag Combinations

**Minimal Composition** (2 features):
```toml
# Basic CLI with async support
clap-noun-verb = { version = "5.3", features = ["async"] }
```

**Distributed System** (4 features):
```toml
# Agent coordination with trust and consensus
clap-noun-verb = { version = "5.3", features = [
    "async",
    "agent2028",
    "crypto",
    "observability"
] }
```

**Semantic AI System** (5 features):
```toml
# RDF ontology + Agent coordination + Observability
clap-noun-verb = { version = "5.3", features = [
    "async",
    "rdf",
    "agent2028",
    "crypto",
    "observability"
] }
```

**Self-Healing Distributed System** (6 features):
```toml
# Full autonomic system with distributed coordination
clap-noun-verb = { version = "5.3", features = [
    "async",
    "agent2028",
    "autonomic",
    "crypto",
    "rdf",
    "observability"
] }
```

**Complete Stack** (all features):
```toml
clap-noun-verb = { version = "5.3", features = ["full"] }
```

### 1.3 Compilation Time vs Features

| Feature Set | Compilation Time | Dependencies |
|-------------|------------------|--------------|
| Default (minimal) | ~30s | 10 crates |
| + async | ~45s | 15 crates |
| + agent2028 | ~90s | 25 crates |
| + autonomic | ~120s | 30 crates |
| + rdf | ~150s | 35 crates |
| full | ~180s | 40 crates |

**Recommendation**: Only enable features you actively use.

---

## 2. Progressive Adoption Path

### 2.1 Stage 1: Basic CLI (Week 1)

**Goal**: Get familiar with attribute macros and basic CLI patterns

```toml
[dependencies]
clap-noun-verb = "5.3"  # Minimal features
```

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    healthy: bool,
}

/// Show system status
#[verb("status", "system")]
fn show_status() -> clap_noun_verb::Result<Status> {
    Ok(Status { healthy: true })
}

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
```

**Verify**:
```bash
cargo make check
cargo make test
cargo make lint
```

### 2.2 Stage 2: Add Async Support (Week 2)

**Goal**: Enable concurrent operations

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["async"] }
tokio = { version = "1.40", features = ["full"] }
```

```rust
use clap_noun_verb_macros::verb;
use tokio;

#[verb("fetch", "data")]
async fn fetch_data(url: String) -> clap_noun_verb::Result<String> {
    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run_async().await
}
```

**Verify**:
```bash
cargo make test-async
```

### 2.3 Stage 3: Add Agent Coordination (Week 3-4)

**Goal**: Distributed multi-agent system

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["async", "agent2028", "crypto"] }
```

```rust
use clap_noun_verb::agent2028::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup agent registry
    let registry = Arc::new(AgentRegistry::new());

    // Register agents
    for i in 0..10 {
        registry.register(Agent {
            id: format!("agent-{}", i),
            capabilities: vec!["compute".to_string()],
            // ...
        }).await;
    }

    // Setup broker
    let broker = CommandBroker::new(registry, RoutingStrategy::BestFit);

    // Run CLI with agent coordination
    clap_noun_verb::run_async().await
}
```

**Verify**:
```bash
cargo make test-agent2028
cargo make bench-agents
```

### 2.4 Stage 4: Add RDF Semantics (Week 5)

**Goal**: AI-discoverable CLI with semantic metadata

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = [
    "async",
    "agent2028",
    "rdf",
    "crypto"
] }
```

```rust
use clap_noun_verb::rdf::{RdfStore, SparqlQuery};

async fn setup_semantic_layer() -> Result<RdfStore, Box<dyn std::error::Error>> {
    let store = RdfStore::new();

    // Load ontology
    let ontology = include_str!("../ontology/cli-capabilities.ttl");
    store.load_turtle(ontology)?;

    Ok(store)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rdf_store = setup_semantic_layer().await?;

    // CLI now has semantic metadata
    clap_noun_verb::run_async().await
}
```

**Verify**:
```bash
cargo make test-rdf
```

### 2.5 Stage 5: Add Autonomic Self-Healing (Week 6)

**Goal**: MAPE-K loop for automatic recovery

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = [
    "async",
    "agent2028",
    "autonomic",
    "observability",
    "crypto"
] }
```

```rust
use clap_noun_verb::autonomic::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup autonomic system
    let autonomic = Autonomic::new();

    autonomic.monitor.register("api-server").await;
    autonomic.monitor.register("database").await;

    // Start MAPE-K loop
    tokio::spawn(async move {
        loop {
            autonomic.run_cycle().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    clap_noun_verb::run_async().await
}
```

**Verify**:
```bash
cargo make test-autonomic
cargo make slo-check
```

### 2.6 Stage 6: Production (Week 7+)

**Goal**: Full production deployment

```toml
[dependencies]
clap-noun-verb = { version = "5.3", features = ["full"] }
```

**Complete stack with all integrations**

---

## 3. Common Composition Patterns

### 3.1 Pattern: Agent2028 + RDF (Semantic Agent Discovery)

**Use Case**: AI agents discover and invoke CLI commands semantically

```rust
use clap_noun_verb::agent2028::*;
use clap_noun_verb::rdf::*;

pub async fn semantic_agent_discovery() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup RDF ontology
    let rdf_store = RdfStore::new();
    rdf_store.load_turtle(include_str!("../ontology/cli-capabilities.ttl"))?;

    // 2. Setup agent registry
    let registry = Arc::new(AgentRegistry::new());

    // 3. Discover agents via SPARQL
    let query = r#"
        PREFIX cnv: <http://clap-noun-verb.org/ontology#>

        SELECT ?capability
        WHERE {
            ?command cnv:requiresCapability ?capability .
        }
    "#;

    let capabilities = rdf_store.execute(&SparqlQuery::parse(query)?)?;

    // 4. Register agents with semantic capabilities
    for capability in capabilities {
        let agent = Agent {
            id: format!("semantic-agent-{}", uuid::Uuid::new_v4()),
            capabilities: vec![capability.to_string()],
            // ...
        };
        registry.register(agent).await;
    }

    // 5. Route commands using both semantic and agent systems
    let broker = CommandBroker::new(registry, RoutingStrategy::BestFit);

    Ok(())
}
```

**Benefits**:
- AI agents can discover capabilities automatically
- Dynamic agent registration based on ontology
- Semantic capability matching

### 3.2 Pattern: Agent2028 + Autonomic (Self-Healing Agents)

**Use Case**: Agents self-heal and auto-scale based on load

```rust
use clap_noun_verb::agent2028::*;
use clap_noun_verb::autonomic::*;

pub async fn self_healing_agents() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup agent registry
    let registry = Arc::new(AgentRegistry::new());

    // 2. Setup autonomic monitor
    let autonomic = Autonomic::new();

    // 3. Monitor agent health
    for agent in registry.list_all().await {
        autonomic.monitor.register(&agent.id).await;
    }

    // 4. MAPE-K loop for agent recovery
    tokio::spawn(async move {
        loop {
            // Monitor agent health
            for agent in registry.list_all().await {
                autonomic.monitor.update_metric(
                    &agent.id,
                    SystemMetric::new("health_score", agent.health_score)
                ).await;
            }

            // Analyze: Detect unhealthy agents
            for agent in registry.list_all().await {
                if let Some(anomaly) = autonomic.anomaly_detector
                    .detect(&agent.id, agent.health_score)
                    .await
                {
                    // Plan: Recovery strategy
                    let action = autonomic.auto_recovery
                        .plan_recovery(&agent.id, "health_degradation")
                        .await;

                    // Execute: Restart or replace agent
                    if let Some(action) = action {
                        autonomic.auto_recovery.execute(&action.action_id).await.ok();
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    Ok(())
}
```

**Benefits**:
- Automatic agent recovery
- Proactive health monitoring
- Zero-downtime agent replacement

### 3.3 Pattern: RDF + Autonomic (Semantic Self-Optimization)

**Use Case**: System self-optimizes based on semantic relationships

```rust
pub async fn semantic_optimization() -> Result<(), Box<dyn std::error::Error>> {
    let rdf_store = RdfStore::new();
    let autonomic = Autonomic::new();

    // Query semantic relationships
    let query = r#"
        SELECT ?command ?effect
        WHERE {
            ?command cnv:hasEffect ?effect .
            FILTER(?effect = cnv:Expensive)
        }
    "#;

    let expensive_commands = rdf_store.execute(&SparqlQuery::parse(query)?)?;

    // Monitor expensive commands
    for command in expensive_commands {
        autonomic.monitor.register(&command).await;
    }

    // Auto-optimize expensive operations
    tokio::spawn(async move {
        loop {
            for command in &expensive_commands {
                let metrics = autonomic.monitor.get_metrics(command).await;

                // If too slow, cache results or optimize
                if metrics.avg_latency > 1000.0 {
                    autonomic.auto_recovery.execute("enable_caching").await.ok();
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    });

    Ok(())
}
```

### 3.4 Pattern: Complete Stack (All Features)

**Use Case**: Production trillion-agent system with semantic discovery and self-healing

```rust
use clap_noun_verb::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize RDF ontology
    let rdf_store = initialize_ontology().await?;

    // 2. Setup agent coordination
    let registry = setup_agent_registry().await?;
    let broker = CommandBroker::new(registry.clone(), RoutingStrategy::BestFit);
    let trust_calc = TrustScoreCalculator::new();

    // 3. Setup autonomic system
    let autonomic = setup_autonomic_system().await?;

    // 4. Start MCP server for AI integration
    tokio::spawn(async move {
        start_mcp_server(rdf_store).await.unwrap();
    });

    // 5. Start autonomic monitoring loop
    tokio::spawn(async move {
        autonomic.run_continuous_loop().await;
    });

    // 6. Start agent health monitoring
    tokio::spawn(async move {
        monitor_agent_health(registry, trust_calc).await;
    });

    // 7. Run CLI
    clap_noun_verb::run_async().await
}
```

---

## 4. Architecture Patterns

### 4.1 Layered Architecture

```
┌─────────────────────────────────────────────────┐
│              CLI Layer (User Interface)         │
│  clap-noun-verb attribute macros                │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│         Semantic Layer (RDF/SPARQL/MCP)         │
│  Command discovery, ontology reasoning          │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│    Coordination Layer (Agent2028)               │
│  Distributed agents, consensus, trust           │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│      Autonomic Layer (MAPE-K Loop)              │
│  Self-healing, auto-scaling, optimization       │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│    Observability Layer (Metrics/Logging)        │
│  Prometheus, tracing, structured logs           │
└─────────────────────────────────────────────────┘
```

### 4.2 Microkernel Architecture

**Core** (minimal):
- Attribute macro system
- Command registry
- Basic CLI parsing

**Plugins** (features):
- RDF: Semantic discovery plugin
- Agent2028: Distributed coordination plugin
- Autonomic: Self-healing plugin
- Observability: Monitoring plugin

**Integration**: Plugins communicate via event bus

---

## 5. Performance Optimization

### 5.1 Feature-Specific Optimizations

**RDF/SPARQL**:
```rust
// Cache SPARQL query results
use lru::LruCache;

pub struct CachedRdfStore {
    store: RdfStore,
    cache: Arc<Mutex<LruCache<String, Vec<String>>>>,
}

impl CachedRdfStore {
    pub async fn execute_cached(&self, query: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut cache = self.cache.lock().await;

        if let Some(cached) = cache.get(query) {
            return Ok(cached.clone());
        }

        let results = self.store.execute(&SparqlQuery::parse(query)?)?;
        cache.put(query.to_string(), results.clone());

        Ok(results)
    }
}
```

**Agent2028**:
```rust
// Optimize capability lookup with inverted index
pub struct OptimizedAgentRegistry {
    agents: HashMap<String, Agent>,
    capability_index: HashMap<String, Vec<String>>,
}

impl OptimizedAgentRegistry {
    pub fn find_by_capability_fast(&self, capability: &str) -> Vec<&Agent> {
        self.capability_index
            .get(capability)
            .map(|ids| ids.iter().filter_map(|id| self.agents.get(id)).collect())
            .unwrap_or_default()
    }
}
```

**Autonomic**:
```rust
// Batch metric updates
pub async fn batch_metric_updates(autonomic: &Autonomic, updates: Vec<(String, f64)>) {
    let futures = updates.into_iter().map(|(component, value)| {
        autonomic.monitor.update_metric(&component, SystemMetric::new("value", value))
    });

    futures::future::join_all(futures).await;
}
```

### 5.2 Compilation Time Optimization

**Strategy 1**: Conditional compilation

```rust
#[cfg(feature = "agent2028")]
use clap_noun_verb::agent2028::*;

#[cfg(feature = "agent2028")]
pub async fn setup_agents() {
    // Only compiled if agent2028 feature enabled
}
```

**Strategy 2**: Lazy initialization

```rust
use once_cell::sync::Lazy;

static AGENT_REGISTRY: Lazy<AgentRegistry> = Lazy::new(|| {
    AgentRegistry::new()
});
```

---

## 6. Testing Composed Systems

### 6.1 Integration Testing Strategy

```rust
#[cfg(all(feature = "agent2028", feature = "autonomic"))]
#[tokio::test]
async fn test_self_healing_agents() {
    // Setup
    let registry = Arc::new(AgentRegistry::new());
    let autonomic = Autonomic::new();

    // Register agent
    let agent = Agent {
        id: "test-agent".to_string(),
        health_score: 0.9,
        // ...
    };
    registry.register(agent).await;

    // Monitor agent
    autonomic.monitor.register("test-agent").await;

    // Simulate health degradation
    registry.update_health("test-agent", 0.3).await;

    // Run autonomic cycle
    autonomic.run_single_cycle().await;

    // Verify recovery
    let recovered_agent = registry.find("test-agent").await.unwrap();
    assert!(recovered_agent.health_score > 0.8);
}
```

### 6.2 Feature Flag Testing

```rust
#[cfg(feature = "rdf")]
#[tokio::test]
async fn test_rdf_integration() {
    // RDF-specific tests
}

#[cfg(all(feature = "rdf", feature = "agent2028"))]
#[tokio::test]
async fn test_semantic_agent_discovery() {
    // Composed RDF + Agent2028 tests
}
```

---

## 7. Production Deployment Strategies

### 7.1 Gradual Rollout

**Phase 1**: Deploy basic CLI
```bash
cargo build --release
# Only default features
```

**Phase 2**: Enable async
```bash
cargo build --release --features async
```

**Phase 3**: Add agent coordination
```bash
cargo build --release --features async,agent2028,crypto
```

**Phase 4**: Full stack
```bash
cargo build --release --features full
```

### 7.2 Feature Toggles

```rust
pub struct FeatureConfig {
    pub enable_rdf: bool,
    pub enable_agent2028: bool,
    pub enable_autonomic: bool,
}

impl FeatureConfig {
    pub fn from_env() -> Self {
        Self {
            enable_rdf: std::env::var("ENABLE_RDF").is_ok(),
            enable_agent2028: std::env::var("ENABLE_AGENT2028").is_ok(),
            enable_autonomic: std::env::var("ENABLE_AUTONOMIC").is_ok(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FeatureConfig::from_env();

    #[cfg(feature = "rdf")]
    if config.enable_rdf {
        setup_rdf().await?;
    }

    #[cfg(feature = "agent2028")]
    if config.enable_agent2028 {
        setup_agents().await?;
    }

    clap_noun_verb::run_async().await
}
```

---

## 8. Migration Guides

### 8.1 From Basic to Agent2028

**Before**:
```rust
fn main() {
    clap_noun_verb::run().unwrap();
}
```

**After**:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(AgentRegistry::new());
    let broker = CommandBroker::new(registry, RoutingStrategy::BestFit);

    clap_noun_verb::run_async().await
}
```

**Migration steps**:
1. Add `async` and `agent2028` features to Cargo.toml
2. Change `main` to `#[tokio::main] async fn main()`
3. Initialize agent registry before `run_async()`
4. Update all command handlers to async

### 8.2 From Agent2028 to Full Stack

**Before**:
```toml
features = ["async", "agent2028", "crypto"]
```

**After**:
```toml
features = ["full"]
```

**Migration steps**:
1. Add RDF ontology files
2. Initialize autonomic system
3. Setup observability
4. Update configuration files

---

## 9. Troubleshooting

### 9.1 Common Issues

**Issue 1**: Compilation errors with multiple features

```bash
error: cannot find value `AgentRegistry` in module `agent2028`
```

**Solution**: Ensure `agent2028` feature is enabled

```toml
clap-noun-verb = { version = "5.3", features = ["agent2028"] }
```

**Issue 2**: Runtime panics in autonomic system

```
thread 'tokio-runtime-worker' panicked at 'called `Option::unwrap()` on a `None` value'
```

**Solution**: Initialize all components before starting MAPE-K loop

```rust
autonomic.monitor.register("component").await;  // BEFORE running loop
autonomic.run_continuous_loop().await;
```

**Issue 3**: SPARQL queries return empty results

**Solution**: Verify ontology is loaded correctly

```rust
let store = RdfStore::new();
store.load_turtle(ontology)?;  // Check for errors
```

### 9.2 Performance Issues

**Symptom**: Slow compilation (> 5 minutes)

**Solution**: Reduce features to only what you need

```toml
# Instead of:
features = ["full"]

# Use:
features = ["async", "agent2028"]  # Only what you need
```

**Symptom**: High memory usage in production

**Solution**: Optimize caching and buffer sizes

```rust
// Limit cache size
let cache = LruCache::new(1000);  // Instead of unbounded

// Limit event buffer
let (tx, rx) = mpsc::channel(100);  // Instead of 10000
```

---

## 10. Best Practices

### 10.1 Feature Selection Checklist

- [ ] Start with minimal features
- [ ] Add features incrementally
- [ ] Test each feature addition
- [ ] Profile compilation time
- [ ] Benchmark runtime performance
- [ ] Verify observability
- [ ] Document feature dependencies

### 10.2 Code Organization

```
my-cli/
├── Cargo.toml              # Feature flags
├── src/
│   ├── main.rs            # Integration point
│   ├── cli/               # CLI layer
│   ├── semantic/          # RDF layer (feature = "rdf")
│   ├── coordination/      # Agent2028 (feature = "agent2028")
│   ├── autonomic/         # MAPE-K (feature = "autonomic")
│   └── observability/     # Metrics (feature = "observability")
├── ontology/              # RDF ontology files
├── tests/
│   ├── integration/       # Feature-specific tests
│   └── e2e/              # End-to-end tests
└── benches/              # Performance benchmarks
```

### 10.3 Configuration Management

```yaml
# config.yaml
features:
  rdf:
    enabled: true
    ontology_path: "./ontology/cli-capabilities.ttl"
    sparql_cache_size: 1000

  agent2028:
    enabled: true
    registry_size: 10000
    routing_strategy: "BestFit"

  autonomic:
    enabled: true
    monitor_interval_seconds: 60
    anomaly_threshold: 2.0

  observability:
    enabled: true
    metrics_port: 9090
    tracing_level: "info"
```

---

## Conclusion

Effective feature composition requires:
1. **Progressive adoption**: Start minimal, add incrementally
2. **Clear dependencies**: Understand feature relationships
3. **Testing strategy**: Test each composition thoroughly
4. **Performance monitoring**: Profile compilation and runtime
5. **Gradual deployment**: Roll out features in phases

**Key Recommendations**:
- Use feature flags judiciously (only enable what you need)
- Test composed systems at integration level
- Monitor performance metrics in production
- Document feature interactions clearly
- Have rollback plans for each feature

**Next Steps**:
1. Assess your requirements
2. Select minimum viable feature set
3. Follow progressive adoption path
4. Deploy incrementally
5. Monitor and optimize

**Related Guides**:
- [SEMANTIC_AGENT_COORDINATOR.md](./SEMANTIC_AGENT_COORDINATOR.md) - Complete system overview
- [ADVANCED_TYPE_STATE_PATTERNS.md](./ADVANCED_TYPE_STATE_PATTERNS.md) - Type-level safety
- [DISTRIBUTED_COORDINATION_GUIDE.md](./DISTRIBUTED_COORDINATION_GUIDE.md) - Agent2028 deep dive
- [AUTONOMIC_SYSTEMS_GUIDE.md](./AUTONOMIC_SYSTEMS_GUIDE.md) - Self-tuning implementation

---

**Generated**: 2026-01-05
**Framework Version**: clap-noun-verb 5.3.4
**Maintainer**: clap-noun-verb contributors
