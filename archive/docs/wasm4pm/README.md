# wasm4pm Repository Documentation Index

**Repository Location:** `/Users/sac/wasm4pm`

**Current Status:** Stable v26.5.29 on branch `fix/debt-markers-and-gap-close`

This directory contains comprehensive metadata and analysis of the wasm4pm workspace structure, features, and crate organization.

## Quick Reference

| Document | Purpose | Details |
|---|---|---|
| **[METADATA.md](./METADATA.md)** | Complete repository snapshot | Workspace members, git state, feature system, build profiles, deployment sizing |
| **[FEATURE_DEPENDENCY_MAP.md](./FEATURE_DEPENDENCY_MAP.md)** | Feature activation flow | Profile → canonical → internal feature mappings, forbidden combinations, size impact |
| **[WORKSPACE_CRATES.md](./WORKSPACE_CRATES.md)** | Crate inventory & details | 13 crate profiles, publication order, dependency graph, interop layers |

---

## Key Findings

### Repository
- **Location:** `/Users/sac/wasm4pm`
- **Branch:** `fix/debt-markers-and-gap-close`
- **Latest Commit:** `65169e62` — fix(debt): resolve debt markers blocking pre-push hook
- **Status:** Clean workspace (no uncommitted changes)
- **Version:** 26.5.29 (workspace-wide, except ocel-core: 26.5.30; cognition/macros: 26.5.28)

### Workspace Structure
**13 Crates** (all public, published to crates.io):
1. **wasm4pm** — Main library (WASM + Rust)
2. **wasm4pm-types** — Data structures, OCEL bindings
3. **wasm4pm-algos** — Algorithm implementations
4. **wasm4pm-cli** — Binary `wpm`
5. **wasm4pm-utils** — Utilities
6. **miniml-core** — Machine learning
7. **wasm4pm-cognition** — Cognition kernel
8. **prolog8** — Prolog interpreter
9. **wasm4pm-macros** — Proc-macros
10. **ocel-core** — Object-centric event logs
11. **ocpq** — OCEL query processor
12. **pm-core** — Core abstractions
13. **tps-metrics** — Benchmarking metrics

### Feature System

**6 Deployment Profiles:**
| Profile | Size | Use Case |
|---|---|---|
| mobile | ~500KB | Mobile devices |
| iot | ~1MB | IoT sensors |
| edge | ~1.5MB | Edge servers/CDN workers |
| fog | ~2MB | Fog computing |
| browser | ~2.78MB | Web browsers (DEFAULT) |
| cloud | — | Deprecated alias for browser |

**8 Canonical Features** (user-facing):
- `feature-conformance-basic`, `feature-conformance-full`
- `feature-discovery-advanced`
- `feature-ml`
- `feature-ocel`, `feature-powl`
- `feature-streaming-basic`, `feature-streaming-full`

**40+ Internal Features** (module gating):
- Algorithms: `alpha_plus_plus`, `heuristic_miner`, `inductive_miner`, `genetic`, `ilp`, `a_star`, `aco`, `pso`, `simulated_annealing`
- ML: `ml_classify`, `ml_cluster`, `ml_forecast`, `ml_anomaly`, `ml_regress`, `ml_pca`
- Streaming: `streaming_dfg`, `simd`, `streaming_full`
- Conformance: `conformance_basic`, `conformance_full`, `alignment_fitness`, `align_etconformance`, `petri_net_playout`, `extensive_playout`, `montecarlo`
- Meta: `bcinr`, `cognition`, `hand_rolled_stats`, `poc_gate_validator` (PoC only), `automl_experimental` (forward guard only)

### Forbidden Combinations
| Do NOT | Reason |
|---|---|
| `poc_gate_validator` in deployment | PoC only, use SPARQL-backed validator when ready |
| `automl_experimental` in deployment | Forward guard, await real src/automembrane.rs |
| `feature-gpu` on wasm32 | GPU native only; WASM is single-threaded |
| `feature-rayon` on wasm32 | Rayon for native parallelism; WASM uses streaming |

### Build Profiles
```toml
[profile.release]
opt-level = "z"     # Size optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen for best optimization
panic = "abort"     # Smaller binary
strip = "debuginfo" # Strip debug symbols

[profile.bench]
opt-level = 3       # Speed optimization
lto = false         # Faster rebuilds
```

### Publication Order
Strict order required to avoid circular dependencies:
1. ocel-core, wasm4pm-types, pm-core, wasm4pm-utils, miniml-core, prolog8, wasm4pm-macros
2. ocpq, wasm4pm-algos, wasm4pm-cognition
3. wasm4pm, wasm4pm-cli, tps-metrics

### Interop Layers
- **JavaScript/TypeScript:** wasm-bindgen, js-sys, serde-wasm-bindgen
- **Rust Library:** All crates on crates.io
- **CLI:** Binary `wpm` (via wasm4pm-cli)
- **Proc-macros:** Distributed slice registration via wasm4pm-macros

---

## Algorithms Provided

### Discovery
- **Basic:** Alpha++, Heuristic Miner, Inductive Miner
- **Advanced/Metaheuristic:** Genetic Algorithm, ILP, A*, ACO, PSO, Simulated Annealing

### Conformance
- **Token Replay:** Basic fitness checking
- **Alignment-based:** Alignment Fitness, ETconformance
- **Playout:** Petri net playout, Extensive playout, Monte Carlo

### Machine Learning
- Classification, Clustering, Forecasting, Anomaly Detection, Regression, PCA

### Streaming
- DFG (Directly-Follows Graph)
- SIMD-accelerated (requires `simd` feature)

### Analytics
- Fitness, Precision, Generalization metrics
- POWL (Partial-Order Workflow) discovery
- OCEL (Object-Centric Event Log) support

---

## Known Gaps (Forward Guards)

| Gap | Status | Location |
|---|---|---|
| PoC Gate Validator | In-memory HashSet only | `src/gate_validator_poc.rs` (cfg: `poc_gate_validator`) |
| AutoML Membrane | Forward guard, not implemented | `src/automembrane.rs` (cfg: `automl_experimental`) |
| GPU Acceleration | Dependencies not yet added | Requires wgpu 0.19 + pollster 0.3 |
| Rayon Parallelism | Dependency not yet added | Rayon crate pending |

---

## Benchmarks

**39 total benchmarks** covering:
- Fast algorithms (alpha++, heuristic, inductive)
- Medium algorithms (genetic, ILP)
- Slow algorithms (ACO, PSO, SA)
- Analytics (fitness, precision, generalization)
- Conformance (basic, full, ETconformance, OCEL v2)
- ML (classification, clustering, forecasting, PCA, anomaly)
- Streaming (DFG, SIMD)
- Extended discovery
- POWL discovery
- Hot kernels (SIMD inner loops)
- Scalability, latency, drift detection, MTTR recovery
- Prediction (accuracy, latency, baseline comparison)
- RL convergence, RDTSC validation
- Cache efficiency, real-data benchmarks

---

## Usage Examples

### Build for Specific Profile
```bash
cd /Users/sac/wasm4pm

# Browser (default, all features)
cargo build --release

# Mobile (minimal)
cargo build --release --features mobile

# Edge (intermediate)
cargo build --release --features edge

# Fog (nearly full, except POWL)
cargo build --release --features fog
```

### CLI Usage
```bash
wpm discover --input event_log.xes --algorithm heuristic-miner
wpm conformance --input event_log.xes --model petri_net.pnml
wpm ml-predict --input data.json --model classifier
```

### Rust Library Usage
```rust
use wasm4pm::{LogData, DiscoveryAlgorithm, AlphaPlusPlus};

let algorithm = AlphaPlusPlus::new();
let model = algorithm.discover(&log)?;
```

---

## References

- **Main Crate:** https://docs.rs/wasm4pm
- **GitHub:** https://github.com/seanchatmangpt/wasm4pm
- **crates.io:** https://crates.io/crates/wasm4pm

---

## Notes

- **Size optimization:** Release profile uses `opt-level = "z"` + LTO + strip for minimal WASM binary
- **WASM support:** Dual cdylib (JS) + rlib (Rust library) build
- **OCEL v2:** Full conformance validation support in v26.5.29
- **Default features:** `browser` profile (2.78MB, all algorithms + ML + POWL)
- **Feature interaction:** See FEATURE_DEPENDENCY_MAP.md for complete flow charts
- **Publication order:** Strictly enforced; see WORKSPACE_CRATES.md for details
