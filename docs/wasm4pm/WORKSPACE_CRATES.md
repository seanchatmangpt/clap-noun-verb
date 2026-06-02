# wasm4pm Workspace Crates — Detailed Inventory

## Summary Table

| Crate | Version | Type | Public | Features | Purpose |
|---|---|---|---|---|---|
| **wasm4pm** | 26.5.29 | Library | Yes | ~40 internal | Process mining algorithms (WASM/JS) |
| **wasm4pm-types** | 26.5.29 | Library | Yes | `import` | Binary data structures, OCEL bindings |
| **wasm4pm-algos** | 26.5.29 | Library | Yes | — | Algorithm implementations |
| **wasm4pm-cli** | 26.5.29 | Binary | Yes | — | Command-line interface `wpm` |
| **wasm4pm-utils** | 26.5.29 | Library | Yes | — | Utility functions |
| **miniml-core** | 26.5.29 | Library | Yes | — | Machine learning core |
| **wasm4pm-cognition** | 26.5.28 | Library | Yes | — | Cognition kernel substrate |
| **prolog8** | 26.5.29 | Library | Yes | — | Prolog interpreter |
| **wasm4pm-macros** | 26.5.28 | Proc-macro | Yes | — | `#[noun]`, `#[verb]`, `#[arg]` macros |
| **ocel-core** | 26.5.30 | Library | Yes | — | Object-centric event logs |
| **ocpq** | 26.5.30 | Library | Yes | — | OCEL query processor |
| **pm-core** | 26.5.29 | Library | Yes | — | Core PM abstractions |
| **tps-metrics** | — | Library | Yes | — | Throughput/latency metrics |

---

## Detailed Crate Profiles

### 1. wasm4pm (Main Library)

**Path:** `/Users/sac/wasm4pm/wasm4pm/`

**Purpose:** High-performance process mining algorithms compiled to WebAssembly for JavaScript/TypeScript

**Type:** Dual build (cdylib + rlib)

**Direct Dependencies:**
- Workspace: `wasm4pm-types`, `wasm4pm-algos`, `wasm4pm-cognition` (opt), `miniml` (opt), `ocel-core` (opt)
- External: `wasm-bindgen` 0.2.92, `js-sys` 0.3, `wasm-bindgen-futures` 0.4.42, `serde-wasm-bindgen` 0.6
- Platform: WASM32 has `getrandom` with js feature; native has `tracing-subscriber`
- Stats: `statrs` 0.17 (optional)

**Features:** ~40 internal + 6 deployment profiles
- Profiles: `mobile`, `iot`, `edge`, `fog`, `browser` (default), `cloud` (alias)
- Core: `feature-conformance-basic`, `feature-conformance-full`, `feature-discovery-advanced`, `feature-ml`, `feature-ocel`, `feature-powl`, `feature-streaming-basic`, `feature-streaming-full`
- Algorithms: `alpha_plus_plus`, `heuristic_miner`, `inductive_miner`, `genetic`, `ilp`, `a_star`, `aco`, `pso`, `simulated_annealing`
- ML: `ml_classify`, `ml_cluster`, `ml_forecast`, `ml_anomaly`, `ml_regress`, `ml_pca`
- Streaming: `streaming_dfg`, `simd`, `streaming_full`, `streaming_basic`
- Conformance: `conformance_basic`, `conformance_full`, `alignment_fitness`, `align_etconformance`, `petri_net_playout`, `extensive_playout`, `montecarlo`
- Meta: `bcinr`, `cognition`, `hand_rolled_stats`, `feature-hand-rolled-stats`, `feature-statrs`, `poc_gate_validator`, `automl_experimental`, `import`, `serde`

**Benchmarks:** 39 total (see METADATA.md)

**Tests:** 4 feature-gated (agentic_jtbd_tests, agentic_wasm_export_tests, e2e_agentic_pipeline, ocel_v2)

**Key Modules (cfg-gated):**
- `src/alpha_plus_plus.rs` (#[cfg(feature = "alpha_plus_plus")])
- `src/genetic.rs` (#[cfg(feature = "genetic")])
- `src/ml_*.rs` (#[cfg(feature = "ml_*")])
- `src/simd_streaming.rs` (#[cfg(feature = "simd")])
- `src/alignment_fitness.rs` (#[cfg(feature = "alignment_fitness")])
- `src/gate_validator_poc.rs` (#[cfg(feature = "poc_gate_validator")] — NEVER in production)
- `src/automembrane.rs` (#[cfg(feature = "automl_experimental")] — forward guard only)

**Packaging:**
- Keywords: process-mining, wasm, webassembly, petri-net, bpm
- Categories: wasm, algorithms, data-structures, science
- Homepage: https://github.com/seanchatmangpt/wasm4pm
- Docs: https://docs.rs/wasm4pm
- crate-type: `["cdylib", "rlib"]` (both WASM and Rust library)

---

### 2. wasm4pm-types (Data Structures)

**Path:** `/Users/sac/wasm4pm/crates/wasm4pm-types/`

**Purpose:** Binary data structures for wasm4pm platform, OCEL bindings, event type definitions

**Type:** Library

**Direct Dependencies:**
- Workspace: `ocel-core`
- External: `serde` (with derive), `serde_json`, `chrono`, `uuid`, `hashbrown`, `smallvec`, `blake3`, `rustc-hash`
- Optional: `quick-xml` 0.37, `flate2` 1.0 (both behind `import` feature)

**Features:**
- `import` — Enables XES import via quick-xml + flate2

**Purpose:** XES import allows reading eXtensible Event Stream files (XML-based process logs)

**Status:** Stable, publishes before main crate

---

### 3. wasm4pm-algos (Algorithms)

**Path:** `/Users/sac/wasm4pm/crates/wasm4pm-algos/`

**Purpose:** Process mining algorithm implementations (discovery, conformance, performance analysis)

**Type:** Library

**Algorithms Provided:**
- Discovery: Alpha++, Heuristic Miner, Inductive Miner, Genetic, ILP, A*, ACO, PSO, Simulated Annealing
- Conformance: Token Replay, Alignment Fitness, ETconformance
- Analytics: Fitness metrics, precision, generalization
- POWL: Partial-order workflow discovery
- ML: Classification, clustering, forecasting, anomaly detection, regression, PCA
- Streaming: DFG (directly-follows graph), SIMD-accelerated

**Status:** Stable, published before main crate

---

### 4. wasm4pm-cli (Command-line Interface)

**Path:** `/Users/sac/wasm4pm/crates/wasm4pm-cli/`

**Binary Name:** `wpm`

**Purpose:** Official command-line interface for wasm4pm process mining

**Direct Dependencies:**
- `clap` 4 (with derive, env, cargo features)
- `anyhow` 1.0, `thiserror` 2.0 (error handling)
- `serde`, `serde_json`
- `dialoguer` 0.11 (interactive prompts)
- `colored` 2.1 (colored output)
- `indicatif` 0.17 (progress bars)
- Workspace: `wasm4pm-types`, `wasm4pm-algos`, `wasm4pm` (with `cloud` feature), `ocel-core`

**Features:** Enables `cloud` profile on main `wasm4pm` crate

**Installation:**
```bash
cargo install wasm4pm-cli  # Installs as `wpm` binary
```

**Status:** Stable, published alongside main crate

---

### 5. wasm4pm-utils (Utilities)

**Path:** `/Users/sac/wasm4pm/crates/wasm4pm-utils/`

**Purpose:** Common utility functions (parsing, formatting, validation)

**Type:** Library

**Status:** Stable, published alongside main crate

---

### 6. miniml-core (Machine Learning)

**Path:** `/Users/sac/wasm4pm/crates/miniml-core/`

**Purpose:** Machine learning algorithms (classification, clustering, forecasting, PCA, anomaly detection)

**Type:** Library

**Status:** Stable, published alongside main crate

**Feature Integration:** Enabled via `feature-ml` → `ml` internal features on main crate

---

### 7. wasm4pm-cognition (Cognition Kernel)

**Path:** `/Users/sac/wasm4pm/crates/wasm4pm-cognition/`

**Purpose:** Cognition kernel substrate (reasoning, knowledge representation)

**Type:** Library

**Status:** Stable, published before main crate

**Feature Integration:** Optional via `cognition` feature on main crate

**Versioning:** 26.5.28 (pinned; main crate at 26.5.29)

---

### 8. prolog8 (Prolog Interpreter)

**Path:** `/Users/sac/wasm4pm/crates/prolog8/`

**Purpose:** Prolog interpreter for logical inference and rule-based processing

**Type:** Library

**Status:** Stable, published alongside main crate

**Use Case:** Rule-based discovery, conformance checking via logical queries

---

### 9. wasm4pm-macros (Proc-macros)

**Path:** `/Users/sac/wasm4pm/crates/wasm4pm-macros/`

**Purpose:** Procedural macros for declarative algorithm registration

**Provided Macros:**
- `#[noun]` — Noun command registration (deprecated no-op)
- `#[verb]` — Main verb/algorithm registration via distributed slice
- `#[arg]` — Parameter/argument attribute macros
- (Additional frontier macros: fractal patterns, federated network, semantic composition, executable specs, learning trajectories, reflexive testing)

**Type:** Proc-macro crate

**Status:** Stable, published before main crate

**Versioning:** 26.5.28 (pinned; main crate at 26.5.29)

**Usage Pattern:**
```rust
#[verb]
pub fn my_algorithm(input: &LogData) -> Result<ProcessModel> {
    // Algorithm implementation
}
```
The macro registers the function in a `linkme::distributed_slice`, allowing runtime discovery without explicit registration.

---

### 10. ocel-core (Object-Centric Event Logs)

**Path:** `/Users/sac/wasm4pm/crates/ocel-core/`

**Purpose:** Object-centric event log (OCEL) data model

**Type:** Library

**Status:** Stable, published before main crate

**Versioning:** 26.5.30 (one patch ahead of workspace version)

**OCEL Support:**
- OCEL v2 conformance validation
- Multi-object/multi-perspective event logs
- Object types, event types, relationships
- Temporal ordering with object context

**Feature Integration:** Enabled via `feature-ocel` → `ocel` → `dep:ocel-core` on main crate

---

### 11. ocpq (OCEL Query Processor)

**Path:** `/Users/sac/wasm4pm/crates/ocpq/`

**Purpose:** OCEL query processor (SPARQL-like query execution)

**Type:** Library

**Status:** Stable, published before main crate

**Versioning:** 26.5.30 (one patch ahead)

**Use Case:** Querying and filtering object-centric event logs

---

### 12. pm-core (Core PM Abstractions)

**Path:** `/Users/sac/wasm4pm/crates/pm-core/`

**Purpose:** Foundational process mining abstractions (traits, common types)

**Type:** Library

**Status:** Stable, published alongside main crate

---

### 13. tps-metrics (Throughput/Latency)

**Path:** `/Users/sac/wasm4pm/tps-metrics/`

**Purpose:** Benchmarking metrics (transactions per second, latency distribution)

**Type:** Library

**Status:** Stable, published alongside main crate

**Use Case:** Perf validation in CI/CD and release notes

---

## Publication Order

**CRITICAL:** Respect this order to avoid dependency resolution errors.

1. `wasm4pm-types` (no workspace crate deps, only `ocel-core`)
2. `ocel-core` (no workspace crate deps)
3. `ocpq` (depends on `ocel-core`)
4. `pm-core` (standalone)
5. `wasm4pm-utils` (standalone)
6. `miniml-core` (standalone)
7. `prolog8` (standalone)
8. `wasm4pm-macros` (standalone proc-macro)
9. `wasm4pm-algos` (depends on types, utils, miniml, prolog)
10. `wasm4pm-cognition` (standalone)
11. `wasm4pm` (depends on types, algos, cognition, miniml, ocel-core)
12. `wasm4pm-cli` (depends on wasm4pm + types + ocel-core)
13. `tps-metrics` (standalone)

---

## Workspace Dependency Graph

```
ocel-core (v26.5.30)
├─ ocpq (v26.5.30)
├─ wasm4pm-types (v26.5.29)
│  └─ ocel-core
├─ wasm4pm-algos (v26.5.29)
│  ├─ wasm4pm-types
│  ├─ miniml-core
│  └─ prolog8
├─ wasm4pm (v26.5.29, main lib)
│  ├─ wasm4pm-types
│  ├─ wasm4pm-algos
│  ├─ wasm4pm-cognition (v26.5.28, optional)
│  ├─ miniml-core (optional)
│  └─ ocel-core
├─ wasm4pm-cli
│  ├─ wasm4pm
│  ├─ wasm4pm-types
│  ├─ wasm4pm-algos
│  └─ ocel-core
└─ Standalone:
   ├─ wasm4pm-utils
   ├─ pm-core
   ├─ prolog8
   ├─ miniml-core
   ├─ wasm4pm-macros (proc-macro)
   └─ tps-metrics
```

---

## Feature Coverage by Crate

| Crate | Canonical Features | Internal Features |
|---|---|---|
| **wasm4pm** | 8 (conformance, discovery, ml, ocel, powl, streaming, stats, gpu/rayon) | 40+ (algorithms, modules) |
| **wasm4pm-types** | 1 (`import`) | 1 (`import`) |
| **wasm4pm-algos** | None (gates via parent) | None (no cfg-gating) |
| **wasm4pm-cli** | None (activates `cloud` on parent) | None |
| **miniml-core** | None (gates via parent) | None |
| **ocel-core** | None | None |
| **ocpq** | None | None |
| **prolog8** | None | None |
| **wasm4pm-macros** | None | None |
| **pm-core** | None | None |
| **wasm4pm-cognition** | None (gates via parent) | None |
| **wasm4pm-utils** | None | None |
| **tps-metrics** | None | None |

---

## Development Status

**v26.5.29 (Current Stable)**
- All 13 crates stable and published
- All algorithms implemented and tested
- OCEL v2 conformance validated
- WASM build passing CI
- Binary size optimized: browser profile 2.78MB

**Pinned to Earlier Versions:**
- `wasm4pm-cognition` v26.5.28
- `wasm4pm-macros` v26.5.28
- Likely due to API stability or circular dependency handling

**Known Gaps (Forward Guards):**
- `poc_gate_validator` (PoC only, in-memory HashSet)
- `automl_experimental` (forward guard, src/automembrane.rs pending)
- `feature-gpu` (wgpu/pollster not yet added as deps)
- `feature-rayon` (rayon not yet added as dep)

---

## Interop Layers

### JavaScript/TypeScript (via wasm-bindgen)
- `wasm4pm` exports to `wasm4pm` npm package
- Bindings: `wasm-bindgen`, `js-sys`, `wasm-bindgen-futures`, `serde-wasm-bindgen`

### Rust Library
- `wasm4pm` rlib exports to crates.io
- All crates available as Rust dependencies

### CLI
- `wasm4pm-cli` binary `wpm` for command-line usage
- Interactive prompts via `dialoguer`
- Colored output via `colored`

### Proc-macros
- `wasm4pm-macros` enables distributed slice registration
- Automatic algorithm discovery at compile-time
- No runtime registration needed
