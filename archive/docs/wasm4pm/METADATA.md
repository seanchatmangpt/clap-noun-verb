# wasm4pm Repository Metadata

**Repository Location:** `/Users/sac/wasm4pm`

**Current Branch:** `fix/debt-markers-and-gap-close`

**Latest Commit:** `65169e62` — fix(debt): resolve debt markers blocking pre-push hook

**Git Status:** Clean (no uncommitted changes)

## Workspace Structure

### Root Workspace
- **Resolver:** 2
- **Version:** 26.5.29 (workspace-wide)
- **Edition:** 2021
- **License:** MIT OR Apache-2.0

### Workspace Members
1. **wasm4pm** — Main library crate (cdylib + rlib)
2. **tps-metrics** — Throughput/latency metrics
3. **crates/wasm4pm-types** — Binary data structures (public)
4. **crates/wasm4pm-algos** — Process mining algorithms (public)
5. **crates/wasm4pm-cli** — Command-line interface `wpm` (public)
6. **crates/wasm4pm-utils** — Utilities (public)
7. **crates/miniml-core** — Machine learning core (public)
8. **crates/wasm4pm-cognition** — Cognition kernel (public, optional)
9. **crates/prolog8** — Prolog interpreter (public)
10. **crates/wasm4pm-macros** — Proc-macros (public)
11. **crates/ocel-core** — Object-centric event logs (public, v26.5.30)
12. **crates/ocpq** — OCEL query processor (public, v26.5.30)
13. **crates/pm-core** — Core PM abstractions (public)

**Public vs Internal:** All crates are published to crates.io

## Workspace Dependencies

Shared dependencies across all crates:

| Dependency | Version | Features |
|---|---|---|
| `serde` | 1.0 | derive |
| `serde_json` | 1.0 | preserve_order |
| `chrono` | 0.4 | serde |
| `uuid` | 1.16 | v4, serde, js |
| `hashbrown` | 0.17 | inline-more |
| `smallvec` | 1.13 | serde |
| `itertools` | 0.14.0 | — |
| `rustc-hash` | 2 | — |
| `blake3` | 1.5 | — |
| `sha2` | 0.10.8 | — |
| `bcinr` | 26.4.22 | — |
| `fake` | 2.9 | — (dev) |
| `proptest` | 1.4 | — (dev) |
| `tempfile` | 3.8 | — (dev) |

**Path-based crate deps:**
- `wasm4pm-types`, `wasm4pm-algos`, `wasm4pm-utils`, `wasm4pm-cognition`, `wasm4pm-macros`, `prolog8`, `miniml`, `ocel-core`, `ocpq`

## Main Crate: wasm4pm

**Type:** WebAssembly library for JavaScript/TypeScript

### Direct Dependencies (Non-Workspace)
- `wasm-bindgen` 0.2.92
- `js-sys` 0.3
- `wasm-bindgen-futures` 0.4.42
- `serde-wasm-bindgen` 0.6
- `thiserror` 1
- `once_cell` 1.19
- `roxmltree` 0.19
- `fastrand` 2.1.0
- `rand` 0.8
- `rand_distr` 0.4
- `tracing` 0.1
- `console_error_panic_hook` (optional)
- `statrs` 0.17 (optional)

### Platform-Specific Dependencies
- **wasm32:** `getrandom` 0.2 with js feature
- **native (non-wasm32):** `tracing-subscriber` 0.3 with fmt + registry

### Lib Configuration
- **crate-type:** `["cdylib", "rlib"]`
- **Description:** High-performance process mining algorithms in WebAssembly

### Benchmarks (20 total)
- Fast algorithms (alpha++, heuristic, inductive)
- Medium algorithms (genetic, ILP)
- Slow algorithms (ACO, PSO, SA)
- Analytics (fitness, precision, generalization)
- Conformance (basic, full, ETconformance)
- ML (classification, clustering, forecasting, regression, PCA)
- Streaming (DFG, SIMD)
- Extended discovery
- POWL discovery
- Tier 2 metaheuristic
- Tier 1 discovery
- ML streaming simulation
- Hot kernels
- Closed Claw (custom harness)
- JTBD validation
- Autonomy/JTBD validation (requires `cloud`)
- SIMD inner loops
- Scalability
- Agentic (requires `cloud`)
- AutoProcess latency (requires `cloud`)
- Constant latency loops
- AutoProcess criterion (requires `cloud`)
- AutoML profiling
- RDTSC validation
- Drift detection
- MTTR recovery
- RL convergence
- Prediction accuracy
- Prediction latency
- Streaming vs batch
- Oracle rank validation
- Prediction baseline comparison
- OCEL flattening
- Cache efficiency
- Drift detection (detailed)
- Real-data
- Autonomic real-data

### Tests (3 feature-gated)
- `agentic_jtbd_tests` (requires `cloud`)
- `agentic_wasm_export_tests` (requires `cloud`)
- `e2e_agentic_pipeline` (requires `cloud`)
- `ocel_v2` (requires `ocel`)

## Feature System

### Deployment Profiles

| Profile | Size Target | Includes |
|---|---|---|
| **mobile** | ~500KB | conformance-basic, hand-rolled-stats, bcinr |
| **iot** | ~1MB | conformance-basic, discovery-basic, hand-rolled-stats, bcinr |
| **edge** | ~1.5MB | conformance-basic, discovery-advanced, streaming-basic, hand-rolled-stats, bcinr |
| **fog** | ~2MB | conformance-full, discovery-advanced, ml, streaming-full, ocel, statrs, bcinr |
| **browser** | ~2.78MB | All features including POWL, statrs, ocel, console_error_panic_hook (DEFAULT) |
| **cloud** | — | Alias for browser (deprecated) |

### Canonical Feature Flags

| Feature | Dependencies | Purpose |
|---|---|---|
| `feature-conformance-basic` | `conformance_basic` | Token replay fitness checking |
| `feature-conformance-full` | `conformance_basic`, `alignment_fitness`, `align_etconformance` | Advanced alignment-based conformance |
| `feature-discovery-advanced` | `discovery_basic`, `genetic`, `ilp`, `a_star`, `aco`, `pso`, `simulated_annealing` | Metaheuristic discovery |
| `feature-ml` | `ml`, `ml_classify`, `ml_cluster`, `ml_forecast`, `ml_anomaly`, `ml_regress`, `ml_pca` | Machine learning suite |
| `feature-ocel` | `ocel` | Object-centric event logs |
| `feature-powl` | `powl` | Partial-order workflows |
| `feature-streaming-basic` | `streaming_basic`, `streaming_dfg` | DFG streaming |
| `feature-streaming-full` | `feature-streaming-basic`, `streaming_full`, `simd` | SIMD-accelerated streaming |
| `feature-hand-rolled-stats` | `hand_rolled_stats` | Custom statistics (size-optimized) |
| `feature-statrs` | `statrs`, `dep:statrs` | Full-precision statistics |
| `feature-gpu` | *(prerequisites missing)* | GPU-accelerated LinUCB (NOT wasm32, requires wgpu + pollster) |
| `feature-rayon` | *(prerequisites missing)* | Parallel processing (NOT wasm32, rayon crate not yet added) |
| `bcinr` | `dep:bcinr` | Branchless algorithms (branch count invariant reduction) |
| `cognition` | `dep:wasm4pm-cognition` | Cognition kernel substrate |

### Internal Feature Flags (Module Gating)

**Discovery:**
- `discovery_basic` = `[alpha_plus_plus, heuristic_miner, inductive_miner]`
- `discovery_advanced` = `[discovery_basic, genetic, ilp, a_star, aco, pso, simulated_annealing]`
- Individual: `alpha_plus_plus`, `heuristic_miner`, `inductive_miner`, `genetic`, `ilp`, `a_star`, `aco`, `pso`, `simulated_annealing`

**Conformance:**
- `conformance_basic` = Token replay
- `conformance_full` = `[conformance_basic, alignment_fitness, align_etconformance]`
- Module gating: `alignment_fitness`, `petri_net_playout`, `extensive_playout`, `align_etconformance`, `montecarlo`

**ML:**
- `ml` = All ML modules
- Individual: `ml_classify`, `ml_cluster`, `ml_forecast`, `ml_anomaly`, `ml_regress`, `ml_pca`

**Streaming:**
- `streaming_basic` = `[streaming_dfg]`
- `streaming_full` = `[streaming_basic, simd]`
- Module gating: `streaming_dfg`, `simd`

**Dependencies & Gates:**
- `statrs` = `[dep:statrs]`
- `miniml` = `[dep:miniml]`
- `gpu` = No-op (wgpu/pollster not added)
- `rayon` = No-op (rayon crate not yet added)
- `powl` = POWL algorithms
- `ocel` = `[dep:ocel-core]`
- `serde` = Serialization gate
- `hand_rolled_stats` = Custom statistics
- `simd` = SIMD acceleration
- `console_error_panic_hook` = `[dep:console_error_panic_hook]`
- `poc_gate_validator` = In-memory HashSet gate validator (PoC, NEVER in deployment profiles)
- `automl_experimental` = AutoML membrane layer (forward guard, NEVER until implemented)
- `import` = `[wasm4pm-types/import]` — XES import via quick-xml + flate2

## Supporting Crates

### wasm4pm-types
- **Features:** `import` (enables quick-xml + flate2 for XES import)
- **Purpose:** Binary data structures (OCEL bindings, event types)
- **Public:** Yes

### wasm4pm-cli
- **Bin:** `wpm` (the CLI executable)
- **Features:** Enables `cloud` profile on main `wasm4pm` crate
- **Dependencies:** clap 4 (derive + env + cargo), anyhow, thiserror 2.0, dialoguer, colored, indicatif

### ocel-core (v26.5.30)
- **Purpose:** Object-centric event log model
- **Public:** Yes
- **Exported via:** `feature-ocel`

### ocpq (v26.5.30)
- **Purpose:** OCEL query processor (SPARQL-like)
- **Public:** Yes

## Build Profiles

```toml
[profile.release]
opt-level = "z"      # Size optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for best optimization
panic = "abort"      # Smaller binary (no unwinding)
strip = "debuginfo"  # Strip debug symbols

[profile.bench]
opt-level = 3        # Speed optimization
lto = false          # Disable LTO for faster rebuilds
```

## Source Organization

- **Main library:** `/Users/sac/wasm4pm/wasm4pm/src/`
- **CLI:** `/Users/sac/wasm4pm/crates/wasm4pm-cli/src/main.rs`
- **Types:** `/Users/sac/wasm4pm/crates/wasm4pm-types/src/`
- **Algorithms:** `/Users/sac/wasm4pm/crates/wasm4pm-algos/src/`
- **Cognition:** `/Users/sac/wasm4pm/crates/wasm4pm-cognition/src/`
- **ML:** `/Users/sac/wasm4pm/crates/miniml-core/src/`
- **OCEL:** `/Users/sac/wasm4pm/crates/ocel-core/src/`
- **Prolog:** `/Users/sac/wasm4pm/crates/prolog8/src/`
- **Macros:** `/Users/sac/wasm4pm/crates/wasm4pm-macros/src/`

## Key Design Points

1. **WebAssembly-first:** Dual cdylib + rlib build for JS/TS interop and Rust library use
2. **Feature-based deployment profiles:** Mobile/IoT/Edge/Fog/Browser/Cloud sizing
3. **WASM size optimization:** Release profile uses `-z` opt-level + LTO + strip
4. **Process mining focus:** Conformance, discovery (alpha++, heuristic, inductive, genetic, ILP, ACO, PSO, SA), ML, streaming
5. **Object-centric:** OCEL-native event log model via `ocel-core`
6. **Extensible:** Distributed slice registration for plugins (via `wasm4pm-macros`)
7. **Conditional compilation:** GPU (Vulkan/Metal/DX12) and Rayon (parallel) reserved but not yet added

## Notes

- **Status:** Stable production release (v26.5.29)
- **No uncommitted changes** in wasm4pm repo
- **Gate validators:** PoC in-memory HashSet only; SPARQL-backed validation not yet integrated
- **AutoML membrane:** Forward guard; src/automembrane.rs not created until real model available
- **Default profile:** `browser` (full feature set for web)
- **Deployment:** Separate feature flags per platform (mobile: 500KB, fog: 2MB, browser: 2.78MB)
