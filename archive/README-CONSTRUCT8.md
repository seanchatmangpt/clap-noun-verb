# Construct8: Market Astrophysics Engine

**Construct8** is a deterministic, bounded-mutation market state representation system built on market astrophysics principles. It models market dynamics not as branchy rule engines or probabilistic simulations, but as causal graph transformations with event horizon detection and collision-based hidden structure inference.

## Core Philosophy

Markets are not just fast; they are *geometrically* different from human perception. The typical trading engine reasons about spreads, volumes, and features—2D or 3D observation spaces. Hidden market bodies (large capital flows, structural liquidity) exist below the orderbook horizon and only become visible through causal inference.

**Construct8** provides instruments for detecting these structures:

1. **MarketPlanckCell** — The smallest indivisible unit of market state change (relation topology, capital pressure, settlement constraints)
2. **Construct8Delta** — Bounded 8-triple mutations for deterministic graph state updates
3. **GraphField** — Accumulating causal graph state with state hash verification
4. **Instruments** — Telescope (observation), EventHorizon (boundary detection), Collider (hypothesis testing)
5. **Receipts** — Tamper-evident proof chains for state transitions

## Workspace Structure

```
crates/
├── c8-core/              # Base types: InstrumentId, VenueId, TickRelation, vector clocks
├── c8-graph/             # Construct8Delta: 8-triple bounded mutations
├── c8-market/            # MarketPlanckCell: relation classification (liquidity, capital, waves, settlement)
├── c8-instruments/       # Telescope, EventHorizonTelescope, MarketCollider
├── c8-receipts/          # Receipt types, receipt chains, tamper detection
├── c8-time/              # Monotonic time, causal vector clocks, lane management
├── c8-adversary/         # LogicPlayer vs. GraphPlayer game trees, representation gaps
└── c8-bench/             # Microbenchmarks for hot paths
```

## Build Instructions

All builds use `cargo make`. See `Makefile.toml` for available tasks.

### Quick Build
```bash
cargo make build
```

### Run Tests
```bash
cargo make test
```

### Run Examples
```bash
cargo run --example market_planck_demo
cargo run --example event_horizon_demo
cargo run --example collider_demo
cargo run --example adversary_gap_demo
```

### Validation
```bash
./scripts/validate.sh
```

### Benchmarks
```bash
./scripts/bench.sh
```

## Examples

### 1. Market Planck Cell Demo
**File:** `examples/market_planck_demo.rs`

Demonstrates:
- Synthetic tick generation with realistic market patterns
- Conversion to MarketPlanckCells (atomic relation changes)
- Emission of Construct8Deltas (bounded 8-triple mutations)
- Application to GraphField for state accumulation
- State hash verification and receipt generation

**Run:** `cargo run --example market_planck_demo`

**Output:** JSON receipt with final state hash, receipt chain, and graph field state.

### 2. Event Horizon Demo
**File:** `examples/event_horizon_demo.rs`

Demonstrates:
- Synthetic liquidity collapse scenario (orderbook degradation)
- Event horizon boundary detection via MarketEventHorizonTelescope
- Graph field state updates as boundaries cross
- Causal time tracking at the boundary
- Evidence of market astrophysics principles

**Run:** `cargo run --example event_horizon_demo`

**Output:** JSON receipt with liquidity cliff heights, boundary crossing times, and market astrophysics interpretation.

### 3. Market Collider Demo
**File:** `examples/collider_demo.rs`

Demonstrates:
- Two competing market hypotheses (liquidity vs. capital models)
- Collision of these hypotheses through shared observations
- Inference of hidden market body characteristics
- Gravity signature extraction (influence markers)
- Conformance analysis of visible vs. hidden dynamics

**Run:** `cargo run --example collider_demo`

**Output:** JSON receipt with collision analysis, hidden market body inference, and gravity signature.

### 4. Adversary Gap Demo (Existing)
**File:** `crates/c8-adversary/examples/adversary_gap_demo.rs`

Demonstrates:
- Two players observing the same market stream
- LogicPlayer: reasons over feature vectors only
- GraphPlayer: reasons over graph topology + causal relations
- Representation gap quantification
- Why graph-aware players discover more game tree nodes

**Run:** `cargo run --example adversary_gap_demo`

**Output:** Game tree comparison, missing state basis, prophecy illusion explanation.

## Key Concepts

### MarketPlanckCell
The smallest indivisible unit of market state change. Each cell represents a single atomic relation change:

- **LiquidityTopologyChange** — Connectivity or depth topology alteration
- **CapitalPressureShift** — Buy/sell imbalance pressure
- **RelationBreak** — Sudden loss of connectivity (event horizon candidate)
- **WavePhaseTransition** — Elliott wave phase change
- **SettlementConstraint** — Margin or settlement activation
- **LatencyGeometry** — Order propagation delay shift

### Construct8Delta
A bounded 8-triple RDF delta container. Every market state mutation fits within 8 RDF triples, guaranteeing:

- **Stack allocation** — No heap fragmentation
- **Constant space** — O(1) memory per delta regardless of graph size
- **Deterministic composition** — Deltas compose linearly

### GraphField
Accumulating causal graph state with:

- **Triple accumulation** — RDF triples are added/removed deterministically
- **State hash computation** — Verification of consistent state evolution
- **Causal ordering** — Monotonic time ensures causality preservation

### Event Horizon Detection
Marks causal boundaries where liquidity vanishes. Below the horizon:

- Orderbook connectivity is severed geometrically (not temporally)
- Causal time continues; causal graphs do not
- Capital structures (hidden orders, hedges) become invisible

### Market Collider
Tests hypothesis pairs by colliding market states to infer hidden structures:

1. Two models observe the same visible market traces
2. Their divergence encodes the gravity of hidden structures
3. The collision result (bounded delta, causal ordering) indicates hypothesis fitness

## Output Format: Receipts

All examples emit JSON receipts containing:

- **timestamp** — Execution time (UNIX epoch)
- **ticks_processed** — Number of input market ticks
- **cells_generated** — Number of Planck cells created
- **deltas_applied** — Number of Construct8Deltas applied
- **final_state_hash** — Deterministic hash of accumulated graph state
- **receipt_chain** — Tamper-evident sequence of state transitions

Each receipt entry includes:
```json
{
  "pre_state": "0x...",
  "post_state": "0x...",
  "delta_mask": "0b........",
  "causal_time": 1000,
  "receipt_hash": "0x..."
}
```

## Validation

Run the full validation suite:

```bash
./scripts/validate.sh
```

This checks:
1. Code formatting (rustfmt)
2. Linting (clippy)
3. Test suite (all features)
4. Example compilation

## Benchmarking

Run benchmarks and generate receipts:

```bash
./scripts/bench.sh
```

Benchmarks cover:
- Delta mutation time (should be O(1))
- State hash computation (constant time)
- Receipt generation overhead
- Vector clock lane operations
- Graph field accumulation

Results are written to `target/criterion/` with a summary receipt at `target/bench-results/benchmark_receipt.yaml`.

## Architecture Principles

### 1. Branchless Representational Math
Market dynamics are computed via graph mutations, not branchy decision trees. This enables:

- **Deterministic replay** — Given a receipt chain, graph state is fully recoverable
- **Bounded complexity** — Mutations are always O(1) regardless of accumulated state
- **Parallel safety** — Vector clocks ensure causal consistency across concurrent observations

### 2. Event Horizon Recognition
Liquidity doesn't just disappear; it crosses a causal boundary. The orderbook topology breaks before price moves. This enables:

- **Early detection** — Event horizons mark causal boundaries before market dislocation
- **Geometry-based inference** — Hidden bodies reveal themselves through light bending

### 3. Collision-Based Inference
Hidden market structures (capital flows, structural supply) are inferred by colliding two models of visible state. Divergence encodes gravitational effect:

- **No omniscience** — The collider doesn't see the hidden body; it infers it
- **Bounded error** — Collision results provide bounds on divergence

### 4. Proof Gates Over Rule Engines
State transitions are proven via receipts, not validated through branchy rule engines:

- **Tamper-evident** — Each receipt contains hash of previous state
- **Replay-safe** — Deterministic computation from receipt chain
- **Linear verification** — Validate receipt chain in O(n) time

## Future Work (See `docs/future/`)

- **Quantum-ready cryptography** — Post-quantum receipt hashing
- **SPARQL integration** — Query market causal graphs with standard RDF tools
- **Federated networks** — Multi-venue causal consistency
- **Autonomous discovery** — Self-optimizing relation classifiers
- **Reflexive testing** — Tests that prove their own correctness

## Documentation

- **`docs/MARKET_PHYSICS_THEORY.md`** — 10-section theoretical foundation
- **`docs/explanation/`** — Architecture and design patterns
- **`docs/howto/`** — Practical guides
- **`docs/future/`** — Frontier research directions

## License

Copyright (c) 2024 Sean Chatman
SPDX-License-Identifier: MIT OR Apache-2.0

## Quick Start

```bash
# Clone the repository
cd /Users/sac/clap-noun-verb

# Build all crates
cargo make build

# Run examples
cargo run --example market_planck_demo
cargo run --example event_horizon_demo
cargo run --example collider_demo
cargo run --example adversary_gap_demo

# Validate
./scripts/validate.sh

# Benchmark
./scripts/bench.sh
```

---

**Construct8: Where market physics meets deterministic computation.**
