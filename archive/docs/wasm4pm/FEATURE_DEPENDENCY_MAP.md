# wasm4pm Feature Dependency Map

## Feature Activation Flow

### Deployment Profile → Canonical Features → Internal Features

```
browser (DEFAULT)
├─ feature-conformance-full
│  └─ conformance_full
│     ├─ conformance_basic
│     ├─ alignment_fitness
│     └─ align_etconformance
├─ feature-discovery-advanced
│  └─ discovery_advanced
│     ├─ alpha_plus_plus
│     ├─ heuristic_miner
│     ├─ inductive_miner
│     ├─ genetic
│     ├─ ilp
│     ├─ a_star
│     ├─ aco
│     ├─ pso
│     └─ simulated_annealing
├─ feature-ml
│  └─ ml
│     ├─ ml_classify
│     ├─ ml_cluster
│     ├─ ml_forecast
│     ├─ ml_anomaly
│     ├─ ml_regress
│     └─ ml_pca
├─ feature-streaming-full
│  └─ streaming_full
│     ├─ streaming_basic
│     │  └─ streaming_dfg
│     └─ simd
├─ feature-ocel
│  └─ ocel (dep:ocel-core)
├─ feature-powl
│  └─ powl
├─ feature-statrs
│  └─ statrs (dep:statrs)
├─ discovery_advanced (see above)
├─ conformance_full (see above)
├─ ml (see above)
├─ streaming_full (see above)
├─ petri_net_playout
├─ extensive_playout
├─ montecarlo
├─ statrs (see above)
├─ ocel (see above)
├─ powl (see above)
├─ console_error_panic_hook (dep:console_error_panic_hook)
└─ import (wasm4pm-types/import → quick-xml + flate2)
```

## Profile Details

### mobile (~500KB)
- feature-conformance-basic
  - conformance_basic
- feature-hand-rolled-stats
  - hand_rolled_stats
- bcinr (branchless algorithms)

### iot (~1MB)
- feature-conformance-basic
  - conformance_basic
- feature-hand-rolled-stats
  - hand_rolled_stats
- discovery_basic
  - alpha_plus_plus
  - heuristic_miner
  - inductive_miner
- conformance_basic
- bcinr

### edge (~1.5MB)
- feature-conformance-basic
  - conformance_basic
- feature-discovery-advanced
  - discovery_basic + all metaheuristics
- feature-streaming-basic
  - streaming_basic → streaming_dfg
- feature-hand-rolled-stats
  - hand_rolled_stats
- discovery_advanced (see above)
- conformance_basic
- bcinr

### fog (~2MB)
- feature-conformance-full
  - conformance_full → conformance_basic + alignment_fitness + align_etconformance
- feature-discovery-advanced
  - discovery_advanced
- feature-ml
  - ml + all ML classifiers
- feature-streaming-full
  - streaming_full → streaming_basic + simd
- feature-ocel
  - ocel (dep:ocel-core)
- feature-statrs
  - statrs (dep:statrs)
- discovery_advanced
- conformance_full
- ml
- streaming_full
- statrs
- ocel
- bcinr

### browser (~2.78MB, DEFAULT)
- feature-conformance-full → conformance_full
- feature-discovery-advanced → discovery_advanced
- feature-ml → ml
- feature-streaming-full → streaming_full
- feature-ocel → ocel
- feature-powl → powl
- feature-statrs → statrs
- discovery_advanced
- conformance_full
- ml
- streaming_full
- petri_net_playout
- extensive_playout
- montecarlo
- statrs
- ocel
- powl
- console_error_panic_hook
- import → wasm4pm-types/import

### cloud (DEPRECATED ALIAS)
- Same as browser

## Canonical vs Internal Feature Mapping

| Canonical | Internal Deps | Purpose |
|---|---|---|
| `feature-conformance-basic` | `conformance_basic` | Token replay, basic fitness checking |
| `feature-conformance-full` | `conformance_basic, alignment_fitness, align_etconformance` | Advanced alignment-based conformance |
| `feature-discovery-advanced` | `discovery_basic, genetic, ilp, a_star, aco, pso, simulated_annealing` | Metaheuristic process discovery |
| `feature-ml` | `ml, ml_classify, ml_cluster, ml_forecast, ml_anomaly, ml_regress, ml_pca` | Machine learning suite |
| `feature-ocel` | `ocel` | Object-centric event logs |
| `feature-powl` | `powl` | Partial-order workflows |
| `feature-streaming-basic` | `streaming_basic, streaming_dfg` | DFG streaming without SIMD |
| `feature-streaming-full` | `feature-streaming-basic, streaming_full, simd` | SIMD-accelerated streaming |
| `feature-hand-rolled-stats` | `hand_rolled_stats` | Custom statistics (space optimized) |
| `feature-statrs` | `statrs, dep:statrs` | Full statrs library (precision) |
| `feature-gpu` | *(empty, wgpu/pollster missing)* | GPU acceleration (reserved) |
| `feature-rayon` | *(empty, rayon crate missing)* | Parallel processing (reserved) |

## Discovery Algorithm Hierarchy

```
discovery_basic (Core)
├─ alpha_plus_plus
├─ heuristic_miner
└─ inductive_miner

discovery_advanced = discovery_basic + metaheuristics
├─ genetic
├─ ilp (Integer Linear Programming)
├─ a_star
├─ aco (Ant Colony Optimization)
├─ pso (Particle Swarm Optimization)
└─ simulated_annealing
```

## Conformance Algorithm Hierarchy

```
conformance_basic (Token Replay)
└─ conformance_full
   ├─ alignment_fitness (Optimal alignment)
   └─ align_etconformance (ETconformance, event-transition alignment)

Playout modules (used by conformance):
├─ petri_net_playout
├─ extensive_playout
└─ montecarlo (simulation-based)
```

## ML Classifier Hierarchy

```
ml (Enable all)
├─ ml_classify (Classification)
├─ ml_cluster (Clustering)
├─ ml_forecast (Time series forecasting)
├─ ml_anomaly (Anomaly detection)
├─ ml_regress (Regression)
└─ ml_pca (Dimensionality reduction)
```

## Streaming Architecture

```
streaming_basic
└─ streaming_dfg (Directly-Follows Graph)

streaming_full = streaming_basic + SIMD
├─ streaming_dfg
└─ simd (SIMD acceleration)
```

## External Crate Dependencies via Features

| Feature | External Crate | Version | Optional |
|---|---|---|---|
| `dep:statrs` | `statrs` | 0.17 | Yes |
| `dep:miniml` | `miniml` | 26.5.29 | Yes |
| `dep:wasm4pm-cognition` | `wasm4pm-cognition` | 26.5.28 | Yes |
| `dep:ocel-core` | `ocel-core` | 26.5.30 | Yes |
| `wasm4pm-types/import` | quick-xml, flate2 | 0.37, 1.0 | Yes (via `import` feature) |

## Forbidden Combinations

| Do NOT Enable | Reason | Alternatives |
|---|---|---|
| `poc_gate_validator` in any deployment profile | PoC only, in-memory HashSet insufficient for production | Await SPARQL-backed validator |
| `automl_experimental` in any deployment profile | Forward guard only, src/automembrane.rs not created | Await real AutoML model implementation |
| `gpu` feature on wasm32 | GPU not available in WASM; GPU is native only (Vulkan/Metal/DX12) | Use browser/cloud/fog/edge profiles instead |
| `rayon` feature on wasm32 | WASM is single-threaded; rayon is for native parallel execution | Streaming algorithms for WASM |
| `feature-gpu` without wgpu + pollster deps | Compilation will fail (crate not yet added) | Wait for wgpu/pollster workspace dependency |
| `feature-rayon` without rayon dep | Compilation will fail (crate not yet added) | Wait for rayon workspace dependency |

## Feature Size Impact (Approximate)

Based on release profile (opt-level=z, LTO, strip):

| Profile | Size | Critical Features |
|---|---|---|
| mobile | ~500KB | conformance_basic + hand_rolled_stats |
| iot | ~1MB | + discovery_basic |
| edge | ~1.5MB | + discovery_advanced + streaming_basic |
| fog | ~2MB | + ml + streaming_full + ocel |
| browser | ~2.78MB | all features (default) |

## Conditional Module Compilation

Each internal feature gates one or more Rust modules:

```rust
#[cfg(feature = "alpha_plus_plus")]
pub mod alpha_plus_plus;

#[cfg(feature = "genetic")]
pub mod genetic;

#[cfg(feature = "ml_classify")]
pub mod ml_classify;

#[cfg(feature = "simd")]
pub mod simd_streaming;

#[cfg(feature = "alignment_fitness")]
pub mod alignment_fitness;

#[cfg(feature = "poc_gate_validator")]
pub mod gate_validator_poc;  // NEVER in production

#[cfg(feature = "automl_experimental")]
pub mod automembrane;  // Forward guard only
```

## Tests Requiring Features

| Test | Requires | Rationale |
|---|---|---|
| `agentic_jtbd_tests` | `cloud` | JTBD validation requires full feature set |
| `agentic_wasm_export_tests` | `cloud` | WASM export validation requires full feature set |
| `e2e_agentic_pipeline` | `cloud` | End-to-end pipeline requires all features |
| `ocel_v2` | `ocel` | OCEL v2 format tests |

## Benchmarks Requiring Features

Most benchmarks work without special features. Two require `cloud`:

- `autonomy_jtbd_validation` → `cloud`
- `agentic_bench` → `cloud`
- `autoprocess_latency` → `cloud`
- `autoprocess_criterion` → `cloud`

## Changelog by Feature Type

**Recent (v26.5.29):**
- OCEL v2 conformance validation
- ETconformance alignment module
- Streaming SIMD acceleration
- AutoML membrane (forward guard)
- Cognition kernel re-export

**Stabilized:**
- Core discovery algorithms (alpha++, heuristic, inductive)
- Token replay conformance
- Basic ML (classify, cluster, forecast, anomaly)
- DFG streaming
- POWL support
