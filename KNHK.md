# KNHK: Kinetic Knowledge Hypergraph

## What is KNHK?

**KNHK** stands for **Kinetic Knowledge Hypergraph**—the ontology layer (Σ) in the graph-universe thesis.

- **Kinetic** = The graph evolves over time; change is native
- **Knowledge** = Rich semantic relationships; not just raw data
- **Hypergraph** = Relationships can connect any number of nodes, not just pairs

**KNHK is the source of truth for the entire system.** Everything else—code, tests, configs, CLI—is derived from KNHK.

```
KNHK (Kinetic Knowledge Hypergraph)
 ↓
μ-Kernel execution (A = μ(O))
 ↓
Observables: receipts, effects, metrics (Γ)
 ↓
AHI analyzes and proposes ΔΣ (ontology changes)
 ↓
Loop back to KNHK (graph evolves)
```

---

## Why "Hypergraph"?

### Graphs vs. Hypergraphs

**Traditional Graph**: Edges connect exactly 2 nodes
```
Capability: storage.create
    ↓
  Requires: {Agent, Resource}
    ↓
  Guards: {ReadQuota, WriteQuota}
```

Limitations:
- N-ary relationships must be "flattened" into multiple binary edges
- Complex constraints require elaborate workarounds
- Composition is unclear

**Hypergraph**: Edges (called "hyperedges") connect any number of nodes

```
Hyperedge: "ExecContext"
  ├─ Agent (executor)
  ├─ Capability (action)
  ├─ Resource (target)
  ├─ Guard (constraint)
  ├─ Effect (what changes)
  └─ Receipt (audit trail)

All related in one semantic unit.
```

Benefits:
- Natural representation of complex relationships
- Constraint handling is explicit
- Composition is algebraic (overlays)
- Scales to trillion-agent systems

---

## The Four Planes: O, Σ, Q, ΔΣ

KNHK operates across four semantic planes:

### O: Observations
Runtime events, metrics, receipts from executed actions.

```json
{
  "timestamp": "2025-11-17T12:34:56.789Z",
  "agent_id": "swarm-001",
  "action": "invoke storage.create",
  "duration_ns": 47,
  "memory_used_bytes": 2048,
  "cpu_cycles": 188,
  "effects": ["MutateState", "observational"],
  "quota_remaining": { "cpu": 999812, "memory": 1046528 },
  "receipt_hash": "sha256:..."
}
```

### Σ: Ontology (KNHK Base)
Schema, types, relationships, policies, rules.

```rust
{
  "command_id": "storage.create",
  "noun": "storage",
  "verb": "create",
  "signature": {
    "input": ["Key: String", "Value: Bytes"],
    "output": "Result<StorageRef>"
  },
  "effects": ["MutateState"],
  "requirements": {
    "auth": ["StorageAdmin"],
    "latency_ns": 100,
    "memory_bytes": 4096
  },
  "invariants": [
    "key must be non-empty",
    "value must be <= 1MB",
    "must be idempotent"
  ]
}
```

### Q: Invariants / Queries
Constraints, guards, verification rules that must always be true.

```rust
{
  "invariant_id": "quota_enforcement",
  "predicate": "∀ execution: quota_used(execution) ≤ quota_allocated",
  "enforcement": "strict",
  "consequences": "kill_session_if_violated"
}

{
  "invariant_id": "timing_bound",
  "predicate": "∀ op ∈ μ-ops: latency(op) ≤ 100ns",
  "enforcement": "strict",
  "consequences": "mark_receipt_as_timeout"
}
```

### ΔΣ: Ontology Changes (Overlays)
Proposed modifications to the ontology, represented as graph overlays.

```rust
{
  "delta_id": "add_ttl_to_storage",
  "timestamp": "2025-11-17T13:00:00Z",
  "proposed_by": "ahi-optimization-loop",
  "changes": [
    {
      "type": "modify_field",
      "target": "storage.create",
      "field": "parameters",
      "operation": "add",
      "value": { "name": "ttl", "type": "Duration", "required": false }
    }
  ],
  "justification": "Optimization: avoid storage bloat by auto-expiring old entries",
  "affected_invariants": [],
  "proof_of_safety": "..."
}
```

---

## KNHK Structure: Nodes and Hyperedges

### Node Types

**1. Type Nodes**
```
Type: String
Type: Bytes
Type: StorageRef
Type: Duration
```

**2. Capability Nodes**
```
Capability: storage.create
Capability: storage.read
Capability: storage.delete
Capability: storage.list
```

**3. Entity Nodes**
```
Entity: Agent
Entity: Resource
Entity: Session
Entity: Request
```

**4. Policy Nodes**
```
Policy: QuotaEnforcement
Policy: AuthorizationCheck
Policy: TimingBound
Policy: IsolationConstraint
```

**5. Effect Nodes**
```
Effect: ReadOnly
Effect: MutateState
Effect: MutateConfig
Effect: MutateSecurity
Effect: observational
```

### Hyperedge Types

**1. ExecContext Hyperedge**
Connects: Agent, Capability, Resource, Guard, Effect, Receipt

```
ExecContext {
  executor: Agent,
  action: Capability,
  target: Resource,
  constraint: Guard,
  consequence: Effect,
  proof: Receipt
}
```

**2. AuthContext Hyperedge**
Connects: Agent, Credential, Policy, AccessLevel, TimeRange

```
AuthContext {
  principal: Agent,
  proof: Credential,
  rule: Policy,
  grant: AccessLevel,
  expires: TimeRange
}
```

**3. Invariant Hyperedge**
Connects: Guard, Variable, Predicate, Enforcement, Consequence

```
InvariantCheck {
  constraint: Guard,
  over: Variable,
  rule: Predicate,
  mode: Enforcement,
  failure_action: Consequence
}
```

**4. CompositionHyperedge**
Connects: Capabilities, Policies, Effects (shows how to compose)

```
Composition {
  left: Capability,
  right: Capability,
  combined: Capability,
  algebra: "overlay_merge"
}
```

---

## Kinetic: Temporal Evolution

KNHK is **kinetic** because the graph changes over time:

### Version Control
```
KNHK_v1.0.0 (Nov 1, 2025)
├─ Capability: storage.create (string key, bytes value)
└─ Effect: MutateState

  ↓ ΔΣ_001 applied (Nov 10, 2025)

KNHK_v1.1.0 (Nov 10, 2025)
├─ Capability: storage.create (string key, bytes value, ttl:Duration)
└─ Effect: MutateState

  ↓ ΔΣ_002 applied (Nov 15, 2025)

KNHK_v1.2.0 (Nov 15, 2025)
├─ Capability: storage.create (string key, bytes value, ttl:Duration)
├─ Capability: storage.create_with_options (object with flags)
└─ Effect: MutateState
```

### Change Tracking
Each ΔΣ (delta) includes:
- What changed
- Who proposed it
- Why (justification)
- Proof of safety
- Timestamp
- Parent version hash

This creates a **causal chain** of ontology evolution.

### Rollback Capability
Because changes are tracked:
```bash
# Rollback to previous version if ΔΣ causes problems
knhk_restore --version=1.1.0
# Regenerates all code from old schema
# Existing receipts (O) are still valid (immutable audit trail)
```

---

## Knowledge: Semantic Richness

KNHK is "knowledge" (not just "data") because it captures:

### Type Relationships
```
Type: StorageRef {
  inherits_from: Reference,
  has_properties: [key, value, ttl],
  constraints: [key_non_empty, value_size <= 1MB]
}
```

### Semantic Links
```
Capability: storage.create
  causes_effect: MutateState
  requires_guard: WriteQuota
  on_type: StorageRef
  with_latency_bound: 100ns
  in_context: ExecContext
```

### Rules and Policies
```
Rule: AuthorizationRule {
  if: agent.has_role("StorageAdmin"),
  then: can_invoke(storage.create),
  else: deny_with_error("insufficient_permissions")
}

Rule: ConsistencyRule {
  for_all: executions,
  maintain: "no_orphaned_storage_refs"
}
```

### Constraints and Invariants
```
Constraint: QuotaConstraint {
  over: [cpu_budget, memory_budget, io_quota],
  enforced_by: kernel,
  violated_when: usage > allocation,
  consequence: kill_session
}

Invariant: TimingInvariant {
  statement: "all_μ_operations_complete_in_<=_100ns",
  proven_by: [benchmark_suite, property_tests]
}
```

---

## Current Implementation in Codebase

### Where KNHK is Implemented

**src/autonomic/graph.rs** (794 lines)
- Capability graph structure (nodes, edges, hyperedges)
- Reachability queries
- Equivalence and dominance analysis
- Policy constraint evaluation
- This is the **base KNHK implementation**

**src/autonomic/schema.rs**
- Type definitions
- Command schemas
- Composition metadata
- This is the **KNHK type layer**

**src/autonomic/planes.rs**
- O: Observation plane (runtime events)
- Σ: Ontology plane (schema)
- Q: Invariants plane (constraints)
- ΔΣ: Overlay plane (changes)
- This is the **four-plane KNHK model**

**src/autonomic/contracts.rs** (592 lines)
- Execution contracts (what's allowed)
- Concurrency models
- Temporal specs
- These are **KNHK invariants in code**

**src/autonomic/governance.rs** (687 lines)
- Governance events
- Policy ledger
- This is the **KNHK change tracking**

**src/kernel/ahi_policy.rs** (13.6K lines)
- Policy engine
- ΔΣ proposal evaluation
- This is the **KNHK evolution engine**

### Limitations of Current Implementation

The current graph system is a **foundation for KNHK**, but lacks:

1. **Explicit hypergraph semantics**
   - Current: Binary edges (node → node)
   - Needed: N-ary hyperedges (ExecContext, AuthContext)

2. **Kinetic versioning**
   - Current: Single version of graph
   - Needed: Version history, rollback capability, change tracking

3. **Knowledge representation**
   - Current: Capability graph
   - Needed: Rich semantic types, rules, constraints, policies

4. **ΔΣ algebra**
   - Current: Overlay proposals exist
   - Needed: Formal algebra proving overlays always compose without conflict

---

## Evolution: From "Graph" to "KNHK"

### Phase 1: Naming & Terminology (Current)
- Rename `capability_graph` → `knhk_graph`
- Document that this is a "Kinetic Knowledge Hypergraph"
- Update comments to use KNHK terminology

### Phase 2: Hyperedge Support (3-5 days effort)
```rust
// Current (binary edges)
edge: Capability → Agent

// New (hyperedges)
hyperedge ExecContext {
  executor: Agent,
  action: Capability,
  target: Resource,
  guard: Guard,
  effect: Effect
}
```

### Phase 3: Versioning & Change Tracking (1-2 weeks effort)
```rust
pub struct KNHK {
  version: Version,
  nodes: BTreeMap<NodeId, Node>,
  hyperedges: Vec<Hyperedge>,
  history: Vec<(Version, Timestamp, ΔΣ, Proof)>,  // Change history
  invariants: Vec<Invariant>,  // Knowledge (rules)
}

pub fn apply_delta(&mut self, delta: ΔΣ) -> Result<()> {
  // Verify delta doesn't violate invariants
  // Apply to graph
  // Record in history
}

pub fn rollback(&mut self, version: Version) -> Result<()> {
  // Restore from history
}
```

### Phase 4: Rich Semantics (2-3 weeks effort)
```rust
pub struct Node {
  id: NodeId,
  kind: NodeKind,  // Type, Capability, Entity, Policy, Effect
  properties: Map<String, Value>,  // Rich typed properties
  relationships: Vec<SemanticLink>,  // "has_constraint", "requires_auth", etc.
  rules: Vec<Rule>,  // Knowledge: "if X then Y"
  invariants: Vec<Invariant>,  // Must-always-be-true statements
}
```

### Phase 5: ΔΣ Algebra Formalization (3-4 weeks effort)
```rust
// Prove composition is always valid
pub fn compose(delta1: ΔΣ, delta2: ΔΣ) -> Result<ΔΣ> {
  let combined = delta1.merge_with(delta2);

  // Verify combined doesn't create conflicts
  verify_no_text_conflicts(&combined)?;  // nomrg property
  verify_no_invariant_violations(&combined)?;  // KNHK property
  verify_deterministic_application(&combined)?;  // μ property

  Ok(combined)
}
```

---

## The Role of KNHK in the Thesis

The graph-universe thesis states: **A = μ(O)**

- **O = Ontology = KNHK** — The semantic foundation
- **μ = Kernel** — The deterministic execution engine
- **A = Application** — The observable behavior

Without KNHK, there is no thesis. KNHK is the **source of truth** that the entire system is built upon.

---

## Future: KNHK as AI-Readable Knowledge

At trillion-agent scale, KNHK becomes more than documentation—it becomes the **machine-readable knowledge base** that agents use to:

1. **Self-organize**: Agents read KNHK to understand relationships and constraints
2. **Reason about change**: Agents propose ΔΣ based on observations and invariants
3. **Verify safety**: Agents check that proposed changes don't violate invariants
4. **Self-heal**: Agents rollback ΔΣ that cause problems
5. **Improve continuously**: AHI drives DFLSS optimization loops using KNHK insights

At that point, KNHK is not just "ontology" or "schema"—it's the **collective knowledge** of the entire system.

---

## References

- **PHILOSOPHY.md** — Why ontology (Σ) is primary
- **MU_KERNEL.md** — How μ applies KNHK (A = μ(O))
- **src/autonomic/graph.rs** — Current KNHK implementation
- **src/autonomic/planes.rs** — O/Σ/Q/ΔΣ plane model
- **AHI.md** (TODO) — How AHI uses KNHK to govern evolution
- **concept_gaps.json** — Specific gaps in KNHK formalization
