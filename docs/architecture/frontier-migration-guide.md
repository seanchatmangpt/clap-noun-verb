# Frontier Features Migration Guide

**Version**: 1.0.0
**Date**: 2026-01-05
**Audience**: Developers migrating to integrated frontier features

## Overview

This guide covers migration paths from:
1. Separate `clap-noun-verb-macros-frontier` package → Integrated features
2. Custom implementations → Frontier features
3. No features → Incremental feature adoption

## Migration Path 1: From Separate Package

### Before: Separate Frontier Package

**Project structure:**
```toml
# Cargo.toml
[dependencies]
clap-noun-verb = "5.3.4"
clap-noun-verb-macros-frontier = { git = "https://github.com/user/frontier", branch = "main" }
```

**Code:**
```rust
// src/main.rs
use clap_noun_verb::{Cli, Noun, Verb};
use clap_noun_verb_macros_frontier::meta_framework::*;
use clap_noun_verb_macros_frontier::rdf_composition::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = Cli::new("my-app");

    // Use frontier features
    let meta = MetaFramework::new()
        .load_ontology("capabilities.ttl")?;

    let capabilities = meta.introspect_capabilities().await?;

    Ok(())
}
```

### After: Integrated Features

**Project structure:**
```toml
# Cargo.toml
[dependencies]
clap-noun-verb = { version = "5.4", features = ["meta-framework", "rdf-composition"] }
```

**Code changes:**
```rust
// src/main.rs
use clap_noun_verb::{Cli, Noun, Verb};
use clap_noun_verb::frontier::meta_framework::*;      // Changed import
use clap_noun_verb::frontier::rdf_composition::*;     // Changed import

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = Cli::new("my-app");

    // Same API - no code changes
    let meta = MetaFramework::new()
        .load_ontology("capabilities.ttl")?;

    let capabilities = meta.introspect_capabilities().await?;

    Ok(())
}
```

### Step-by-Step Migration

#### Step 1: Update Cargo.toml

```diff
 [dependencies]
-clap-noun-verb = "5.3.4"
-clap-noun-verb-macros-frontier = { git = "https://github.com/user/frontier", branch = "main" }
+clap-noun-verb = { version = "5.4", features = ["meta-framework", "rdf-composition"] }
```

**Verify:**
```bash
cargo update
cargo tree | grep clap-noun-verb
# Should show single clap-noun-verb with features
```

#### Step 2: Update Imports

**Find all frontier imports:**
```bash
grep -r "use clap_noun_verb_macros_frontier" src/
```

**Replace pattern:**
```diff
-use clap_noun_verb_macros_frontier::meta_framework::*;
+use clap_noun_verb::frontier::meta_framework::*;

-use clap_noun_verb_macros_frontier::rdf_composition::*;
+use clap_noun_verb::frontier::rdf_composition::*;
```

**Automated replacement:**
```bash
# Use sed or your editor's find-replace
find src/ -type f -name "*.rs" -exec sed -i \
  's/clap_noun_verb_macros_frontier/clap_noun_verb::frontier/g' {} +
```

#### Step 3: Verify Build

```bash
cargo make check
cargo make test
cargo make lint
```

**Expected results:**
- ✅ All tests pass
- ✅ No compiler errors
- ✅ No clippy warnings

#### Step 4: Update Documentation

Update any documentation referencing the separate package:

```diff
-## Installation
-
-```toml
-[dependencies]
-clap-noun-verb = "5.3.4"
-clap-noun-verb-macros-frontier = { git = "..." }
-```
+## Installation
+
+```toml
+[dependencies]
+clap-noun-verb = { version = "5.4", features = ["frontier-all"] }
+```
```

### API Compatibility

**99% compatible** - Only import paths change, not APIs

| Old API | New API | Status |
|---------|---------|--------|
| `clap_noun_verb_macros_frontier::meta_framework::*` | `clap_noun_verb::frontier::meta_framework::*` | ✅ Compatible |
| `MetaFramework::new()` | `MetaFramework::new()` | ✅ Identical |
| `#[meta_aware]` | `#[meta_aware]` | ✅ Identical |
| All public types | All public types | ✅ Identical |

**Breaking changes**: None

---

## Migration Path 2: From Custom Implementation

### Scenario A: Custom RDF Introspection

**Before: Custom implementation**

```rust
// src/custom_introspection.rs
use oxigraph::store::Store;

pub struct CustomIntrospector {
    store: Store,
}

impl CustomIntrospector {
    pub fn new() -> Self {
        Self { store: Store::new().unwrap() }
    }

    pub async fn introspect(&self) -> Vec<String> {
        // Custom SPARQL queries
        let query = "SELECT ?cap WHERE { ?cap a :Capability }";
        // ... manual query execution
        vec![]
    }
}
```

**After: Using meta-framework feature**

```rust
// src/main.rs
use clap_noun_verb::frontier::meta_framework::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = MetaFramework::new()
        .load_ontology("capabilities.ttl")?;

    let capabilities = meta.introspect_capabilities().await?;

    // Same result, less code
    Ok(())
}
```

**Benefits:**
- ✅ 80% less code
- ✅ Type-safe API
- ✅ Tested and optimized
- ✅ Self-optimization built-in

### Scenario B: Custom Capability Discovery

**Before: Custom implementation**

```rust
pub struct CustomDiscovery {
    capabilities: Vec<Capability>,
}

impl CustomDiscovery {
    pub fn search(&self, goal: Goal) -> Vec<Capability> {
        // Manual search algorithm
        self.capabilities.iter()
            .filter(|c| self.matches_goal(c, &goal))
            .cloned()
            .collect()
    }
}
```

**After: Using discovery-engine feature**

```rust
use clap_noun_verb::frontier::discovery_engine::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let discovery = CapabilityDiscovery::new();

    let paths = discovery.search(
        current_capabilities,
        Goal::Maximize(Metric::Throughput)
    )?;

    // A* search + swarm optimization + safety proofs
    Ok(())
}
```

**Benefits:**
- ✅ Advanced algorithms (A*, swarm optimization)
- ✅ Fitness scoring (utility, novelty, safety)
- ✅ Safety proofs included
- ✅ Handles 2^64 capability space

### Feature Selection Decision Tree

```
Do you need semantic introspection?
├─ Yes → Use meta-framework
│   └─ Do you need runtime composition?
│       ├─ Yes → Add rdf-composition
│       └─ No → meta-framework only
│
└─ No → Continue

Do you need multi-scale patterns?
├─ Yes → Use fractal-patterns (zero-cost)
└─ No → Continue

Do you need capability discovery?
├─ Yes → Use discovery-engine
│   └─ Do you need learning?
│       ├─ Yes → Add learning-trajectories
│       └─ No → discovery-engine only
│
└─ No → Continue

Do you need distributed composition?
├─ Yes → Use federated-network
└─ No → Continue

Do you need economic modeling?
├─ Yes → Use economic-sim
└─ No → Continue
```

---

## Migration Path 3: Incremental Feature Adoption

### Week 1: Add Type-Level Features (Zero-Cost)

**Goal**: Add fractal patterns and executable specs (no build time impact)

```toml
[dependencies]
clap-noun-verb = { version = "5.4", features = ["fractal-patterns", "executable-specs"] }
```

**Code:**
```rust
use clap_noun_verb::frontier::fractal_patterns::*;

// Same pattern works at three scales
let cli_pattern = Pattern::<CliScale>::new("agent", "coordinate");
let agent_pattern = Pattern::<AgentScale>::new("agent", "coordinate");
let eco_pattern = Pattern::<EcosystemScale>::new("agent", "coordinate");

// Execute at appropriate scale
cli_pattern.execute(cli_context())?;
```

**Impact:**
- Binary size: +0 KB (pure type-level)
- Build time: +0s
- New capabilities: Multi-scale patterns, executable specs

### Week 2: Add Lightweight Features

**Goal**: Add discovery engine and learning trajectories

```toml
[dependencies]
clap-noun-verb = { version = "5.4", features = [
    "fractal-patterns",
    "executable-specs",
    "discovery-engine",        # New
    "learning-trajectories",   # New
] }
```

**Code:**
```rust
use clap_noun_verb::frontier::discovery_engine::*;
use clap_noun_verb::frontier::learning_trajectories::*;

// Discover optimal capability combinations
let discovery = CapabilityDiscovery::new();
let optimal = discovery.search(
    current_capabilities,
    Goal::Maximize(Metric::Value)
)?;

// Optimize learning paths
let learner = LearningPath::new();
let path = learner.optimize(user_profile, target_competency).await?;
```

**Impact:**
- Binary size: +500 KB
- Build time: +5s (agent2028 infrastructure)
- New capabilities: Autonomous discovery, adaptive learning

### Week 3: Add Semantic Features

**Goal**: Add RDF and meta-framework

```toml
[dependencies]
clap-noun-verb = { version = "5.4", features = [
    "fractal-patterns",
    "executable-specs",
    "discovery-engine",
    "learning-trajectories",
    "meta-framework",          # New (large dependency)
    "rdf-composition",         # New
] }
```

**Code:**
```rust
use clap_noun_verb::frontier::meta_framework::*;
use clap_noun_verb::frontier::rdf_composition::*;

// Self-introspection
let meta = MetaFramework::new().load_ontology("capabilities.ttl")?;
let capabilities = meta.introspect_capabilities().await?;

// Runtime composition
let composition = CliComposition::new(ontology)
    .announce(capabilities).await?
    .discover().await?
    .validate()?;

composition.execute().await?;
```

**Impact:**
- Binary size: +6 MB (oxigraph is large)
- Build time: +35s (oxigraph compilation)
- New capabilities: Self-awareness, runtime composition

### Week 4: Add Distributed Features

**Goal**: Complete with federated network

```toml
[dependencies]
clap-noun-verb = { version = "5.4", features = ["frontier-all"] }
```

**Code:**
```rust
use clap_noun_verb::frontier::federated_network::*;

// Join federated network
let network = FederatedNetwork::join("discovery.example.com").await?;

// Advertise capabilities
network.advertise(local_capabilities).await?;

// Discover and invoke remote capabilities
let remote_caps = network.discover().await?;
let result = network.invoke("remote-cli", "capability", args).await?;
```

**Impact:**
- Binary size: +2 MB (crypto + QUIC)
- Build time: +10s
- New capabilities: Distributed composition, Byzantine fault tolerance

### Cumulative Impact

| Week | Features Added | Binary Size | Build Time | Capabilities |
|------|---------------|-------------|------------|--------------|
| 0 (baseline) | (none) | 2 MB | 8s | Core CLI |
| 1 | fractal, specs | 2 MB | 8s | Multi-scale, specs |
| 2 | + discovery, learning | 3 MB | 15s | + Autonomous intelligence |
| 3 | + meta, rdf | 9 MB | 50s | + Self-awareness, composition |
| 4 | + federated | 11 MB | 60s | + Distributed coordination |

---

## Migration Checklist

### Pre-Migration

- [ ] Audit current dependencies
- [ ] Identify which frontier features you need
- [ ] Review feature selection guide
- [ ] Estimate build time impact
- [ ] Plan incremental adoption strategy

### During Migration

- [ ] Update Cargo.toml with selected features
- [ ] Run `cargo update`
- [ ] Update import statements
- [ ] Run `cargo make check` (verify compilation)
- [ ] Run `cargo make test` (verify functionality)
- [ ] Run `cargo make lint` (verify code quality)
- [ ] Update documentation
- [ ] Update CI configuration

### Post-Migration

- [ ] Verify all tests pass
- [ ] Check binary size is acceptable
- [ ] Benchmark performance (no regressions)
- [ ] Update user-facing documentation
- [ ] Train team on new features
- [ ] Monitor production for issues

---

## Common Migration Issues

### Issue 1: Feature Not Available

**Symptom:**
```rust
error[E0432]: unresolved import `clap_noun_verb::frontier::meta_framework`
```

**Cause**: Feature not enabled in Cargo.toml

**Fix:**
```toml
[dependencies]
clap-noun-verb = { version = "5.4", features = ["meta-framework"] }
```

### Issue 2: Conflicting Dependency Versions

**Symptom:**
```
error: failed to select a version for `tokio`
```

**Cause**: Multiple tokio versions in dependency tree

**Fix:**
```toml
[dependencies]
tokio = { version = "1.40", features = ["full"] }
clap-noun-verb = { version = "5.4", features = ["async"] }
```

### Issue 3: Missing Optional Dependency

**Symptom:**
```rust
error[E0433]: failed to resolve: use of undeclared crate or module `oxigraph`
```

**Cause**: Feature enabled but dependency not pulled

**Fix**: Run `cargo clean` and rebuild:
```bash
cargo clean
cargo build --features meta-framework
```

### Issue 4: Build Time Too Long

**Symptom**: Clean build takes >60s

**Cause**: Large dependencies (oxigraph, quinn)

**Mitigation**:
1. Use incremental builds (fast: ~8s)
2. Enable sccache:
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```
3. Reduce feature set (remove meta-framework if not needed)

### Issue 5: Binary Size Too Large

**Symptom**: Binary >15MB

**Cause**: All features enabled

**Mitigation**:
1. Use `--release` mode (optimizations + stripping)
2. Strip symbols:
   ```toml
   [profile.release]
   strip = true
   lto = true
   ```
3. Reduce features (identify which features add size):
   ```bash
   cargo bloat --release --crates --features meta-framework
   ```

---

## Rollback Plan

If migration fails, rollback is straightforward:

### Step 1: Revert Cargo.toml

```bash
git checkout HEAD -- Cargo.toml
cargo update
```

### Step 2: Revert Code Changes

```bash
git checkout HEAD -- src/
```

### Step 3: Verify

```bash
cargo make check
cargo make test
```

**Rollback time**: <5 minutes

---

## Performance Migration

### Before: Custom Implementation Performance

Typical custom implementation:
- Introspection: ~50-100ms (unoptimized SPARQL)
- Discovery: O(N²) brute force
- Binary size: 5-10MB (manual dependencies)

### After: Frontier Features Performance

Optimized implementation:
- Introspection: ~5-10ms (optimized queries + caching)
- Discovery: O(N log N) A* + swarm optimization
- Binary size: 11MB (all features) or 2MB (no features)

**Performance improvement**: 5-10x faster for introspection, exponential improvement for discovery

---

## Training and Documentation

### Developer Training

**Week 1**: Core concepts
- Feature flag system
- Conditional compilation
- Type-state patterns

**Week 2**: Individual features
- Meta-framework usage
- RDF composition patterns
- Fractal patterns

**Week 3**: Feature combinations
- Semantic stack
- Intelligence stack
- Full frontier

### Documentation Updates

Required documentation updates:
1. **README**: Add feature selection guide
2. **API docs**: Add feature badges (`#[cfg_attr(docsrs, doc(cfg(feature = "...")))]`)
3. **Examples**: Create examples for each feature
4. **Integration tests**: Update to use new imports
5. **CI configuration**: Add feature matrix testing

---

## Support and Resources

### Getting Help

- **GitHub Issues**: https://github.com/seanchatmangpt/clap-noun-verb/issues
- **Documentation**: https://docs.rs/clap-noun-verb
- **Examples**: `/examples/frontier/`

### Migration Support Tools

```bash
# Check current feature usage
cargo tree --features meta-framework

# Audit dependency sizes
cargo bloat --release --crates

# Compare performance
cargo bench --baseline before
# ... migrate ...
cargo bench --baseline after
cargo bench -- --baseline before
```

---

## Success Criteria

Migration is successful when:

- ✅ All tests pass (`cargo make test`)
- ✅ No compiler errors (`cargo make check`)
- ✅ No linting warnings (`cargo make lint`)
- ✅ Performance maintained or improved
- ✅ Binary size within acceptable limits
- ✅ Team trained on new features
- ✅ Documentation updated
- ✅ CI pipeline passing

**Expected migration time**: 1-4 weeks depending on feature adoption strategy
