# nomrg: No-Merge Graph Overlays

## The Problem: Textual Merges at Trillion-Agent Scale

In traditional version control:
- Developer A edits file X (lines 50-60)
- Developer B edits file X (lines 100-110)
- Merge is automatic (different sections)

But at **trillion-agent scale** with **millions of concurrent branches**:

```
Branch 1:        storage.create takes ttl parameter
Branch 2:        storage.create adds encryption option
Branch 3:        storage.create adds compression flag
Branch 4:        storage.delete adds force_purge option
Branch 5:        storage.list adds pagination
...
Branch 1M:       storage.create adds retry policy

Merge all into main:
  ❌ Text conflicts at every step
  ❌ Human intervention required for each branch
  ❌ Linear merge time O(n) for n branches
  ❌ Impossible at 1M+ branches
```

### Why Textual Merges Fail

1. **False conflicts**: Changes to different logical concepts appear to conflict textually
2. **Monotonic cost**: Each new branch takes time linear in prior branch count
3. **Human bottleneck**: All merges require human decision
4. **Order dependency**: Merge A→B→C differs from B→A→C

---

## The Solution: nomrg (No-Merge Graph Overlays)

**nomrg** eliminates textual merges entirely. Instead:

- All changes are **graph overlays** (ΔΣ)
- Overlays **compose algebraically** (no conflicts by design)
- Composition is **commutative and associative**
- All branches merge in **logarithmic time** (tree-based)

```
Branch 1: ΔΣ₁ (add ttl parameter)
Branch 2: ΔΣ₂ (add encryption)
Branch 3: ΔΣ₃ (add compression)

Merge tree:
                  Main + ΔΣ₁ + ΔΣ₂ + ΔΣ₃
                 /                          \
         [M + ΔΣ₁ + ΔΣ₂]           [ΔΣ₃ merged]
        /              \
    [M + ΔΣ₁]        [+ ΔΣ₂]
   /        \
  M        ΔΣ₁

Time: O(log n) instead of O(n²)
Merges: 0 conflicts (always valid)
Humans: 0 involved (automatic composition)
```

---

## How Graph Overlays Work

### 1. Base State (KNHK)

```
KNHK_main {
  storage.create: {
    parameters: [key: String, value: Bytes],
    effect: MutateState,
    latency_ns: 100
  }
}
```

### 2. Overlay 1: Add TTL

```
ΔΣ₁ {
  target: storage.create,
  operation: modify_field,
  field: parameters,
  change: add { ttl: Duration, optional: true }
}

Result: storage.create(key, value, ttl)
```

### 3. Overlay 2: Add Encryption

```
ΔΣ₂ {
  target: storage.create,
  operation: modify_field,
  field: parameters,
  change: add { encryption: EncryptionMode, optional: false, default: AES256 }
}

Result: storage.create(key, value, encryption)
```

### 4. Compose ΔΣ₁ + ΔΣ₂

```
ΔΣ₁ ⊕ ΔΣ₂ = ΔΣ₁₂

Result: storage.create(key, value, ttl, encryption)

Key property: ΔΣ₁ ⊕ ΔΣ₂ = ΔΣ₂ ⊕ ΔΣ₁ (commutative)
```

### 5. Apply Composed Overlay

```
KNHK_main ⊕ ΔΣ₁₂ = KNHK_main_merged

storage.create: {
  parameters: [key, value, ttl, encryption],
  effect: MutateState,
  latency_ns: 100
}
```

---

## Overlay Algebra: Why No Conflicts?

### Commutative Property

**Claim**: ΔΣ_A ⊕ ΔΣ_B = ΔΣ_B ⊕ ΔΣ_A

**Proof sketch**:
- ΔΣ_A adds parameter `ttl` to `storage.create`
- ΔΣ_B adds parameter `encryption` to `storage.create`
- Order of addition doesn't matter: {ttl, encryption} = {encryption, ttl}
- Both result in the same final type signature

**Why this works**:
- Overlays are **declarative**, not procedural
- They declare "what the final state should be", not "how to get there"
- The order doesn't affect the end result

### Associative Property

**Claim**: (ΔΣ_A ⊕ ΔΣ_B) ⊕ ΔΣ_C = ΔΣ_A ⊕ (ΔΣ_B ⊕ ΔΣ_C)

**Proof sketch**:
- Merging A+B first then C = composing A+(B+C) first
- Because all operations are independent additions to the parameter list
- The final result is the same regardless of grouping

---

## Types of Overlays

### Type 1: Add Field
```
ΔΣ {
  operation: add_field,
  target: storage.create,
  field: { name: "ttl", type: "Duration", optional: true }
}
```

### Type 2: Remove Field
```
ΔΣ {
  operation: remove_field,
  target: storage.create,
  field: "deprecated_param"
}
```

### Type 3: Modify Field
```
ΔΣ {
  operation: modify_field,
  target: storage.create,
  field: "ttl",
  changes: { optional: false, default: "5m" }
}
```

### Type 4: Add Constraint
```
ΔΣ {
  operation: add_constraint,
  target: storage.create,
  constraint: "key must be <= 256 characters"
}
```

### Type 5: Add Effect
```
ΔΣ {
  operation: add_effect,
  target: storage.create,
  effect: "LogAccess"
}
```

### Type 6: Add Guard
```
ΔΣ {
  operation: add_guard,
  target: storage.create,
  guard: "RequireAdminRole"
}
```

---

## Conflict Detection & Resolution

### When Do Overlays Conflict?

**Conflict 1: Incompatible Field Changes**
```
ΔΣ_A: storage.create.ttl = Duration
ΔΣ_B: storage.create.ttl = Timestamp

Compose: ❌ CONFLICT (ttl has two types)
```

**Conflict 2: Violated Invariant**
```
ΔΣ_A: add parameter (brings parameter count to 5)
ΔΣ_B: add parameter (brings parameter count to 6)
Invariant Q: max_parameters <= 5

Compose: ❌ CONFLICT (violates Q)
```

**Conflict 3: Incompatible Guards**
```
ΔΣ_A: add guard AdminRole
ΔΣ_B: add guard NoOneAllowed

Compose: ❌ CONFLICT (contradictory permissions)
```

### How Are Conflicts Resolved?

**Option 1: Reject Incompatible Overlays**
```
ahi_policy_engine.compose(ΔΣ_A, ΔΣ_B)?
  ↓
Verify no type conflicts
Verify no invariant violations
Verify no permission contradictions
  ↓
Ok: Return ΔΣ_AB
Err: Reject the merge, notify proposer
```

**Option 2: Manual Resolution (Rare)**
```
If automatic resolution fails:
1. Log the conflict
2. Create conflict resolution ticket
3. Propose three-way merge (main + A + B → resolution)
4. Human reviews, picks resolution
5. Apply resolution overlay ΔΣ_resolution

Note: This is the EXCEPTION, not the rule.
```

---

## Current Implementation in Codebase

### Where nomrg Exists

**src/autonomic/planes.rs** (ΔΣ plane)
```rust
pub struct DeltaSigma {  // ΔΣ: ontology overlays
    pub changes: Vec<Change>,
    pub justification: String,
    pub parent_hash: Hash,
    pub proposed_by: AgentId,
}
```

**src/kernel/ahi_policy.rs**
```rust
pub fn compose_deltas(delta_a: ΔΣ, delta_b: ΔΣ) -> Result<ΔΣ> {
    // Verify composition is valid
    // Apply overlay algebra
}
```

### Limitations of Current Implementation

1. **Implicit algebra**
   - Composition exists but algebra is not formalized
   - No formal proof that composition is commutative/associative

2. **Limited conflict detection**
   - Basic checks exist
   - Doesn't handle all conflict types (e.g., semantic conflicts)

3. **No CRDT semantics**
   - Current: arbitrary overlay application
   - Needed: CRDT (Conflict-Free Replicated Data Type) guarantees

---

## Formalization: Overlay Algebra

### Formal Definition

```
Let KNHK = (V, E, L) where:
  V = set of nodes (capabilities, types, effects)
  E = set of edges (relationships)
  L = set of labels (properties, constraints)

Let ΔΣ = (ΔV, ΔE, ΔL) where:
  ΔV = nodes to add/remove
  ΔE = edges to add/remove
  ΔL = labels to add/modify/remove

Apply operator: ⊕
  KNHK' = KNHK ⊕ ΔΣ

Compose operator: ⊙
  ΔΣ_AB = ΔΣ_A ⊙ ΔΣ_B
```

### Axioms

**Axiom 1: Commutativity**
```
∀ ΔΣ_A, ΔΣ_B:
  ΔΣ_A ⊙ ΔΣ_B = ΔΣ_B ⊙ ΔΣ_A
```

**Axiom 2: Associativity**
```
∀ ΔΣ_A, ΔΣ_B, ΔΣ_C:
  (ΔΣ_A ⊙ ΔΣ_B) ⊙ ΔΣ_C = ΔΣ_A ⊙ (ΔΣ_B ⊙ ΔΣ_C)
```

**Axiom 3: Identity**
```
∃ ΔΣ_∅ (empty overlay):
  KNHK ⊕ ΔΣ_∅ = KNHK
  ΔΣ_A ⊙ ΔΣ_∅ = ΔΣ_A
```

**Axiom 4: Invariant Preservation**
```
∀ KNHK, ΔΣ, Q (invariants):
  verify_invariants(KNHK, Q) ∧ verify_invariants(ΔΣ, Q)
  ⟹ verify_invariants(KNHK ⊕ ΔΣ, Q)
```

**Axiom 5: Determinism**
```
∀ KNHK, ΔΣ:
  KNHK ⊕ ΔΣ is deterministic
  (same input ⟹ same output)
```

---

## CRDT Integration

A **Conflict-Free Replicated Data Type (CRDT)** ensures that:
- All replicas can apply updates independently
- Updates automatically merge without conflict
- Final state is consistent across replicas

**nomrg overlays as CRDT**:

```rust
pub struct OverlayCRDT {
    state: KNHK,
    operations: Vec<ΔΣ>,  // append-only log
}

impl OverlayCRDT {
    pub fn apply(&mut self, delta: ΔΣ) {
        // ΔΣ operations are idempotent
        // Applying twice = applying once
        self.operations.push(delta);
        self.rebuild();  // Recompute state from log
    }

    pub fn merge(&mut self, other: OverlayCRDT) {
        // Merge another replica's operation log
        let mut combined = self.operations.clone();
        combined.extend(other.operations);
        // Deduplicate (same ΔΣ from different replicas)
        combined.sort_by_key(|d| d.id);
        combined.dedup_by_key(|d| d.id);
        // Recompute state
        self.operations = combined;
        self.rebuild();
    }
}
```

---

## Evolution Plan: From Current to Full nomrg

### Phase 1: Formalize Algebra (2-3 weeks)
- [ ] Document commutative/associative proofs
- [ ] Add formal tests for algebra properties
- [ ] Create overlay invariant checker

### Phase 2: Implement CRDT (3-4 weeks)
- [ ] Convert ΔΣ to CRDT operations
- [ ] Implement idempotent apply
- [ ] Add operation deduplication
- [ ] Test multi-replica merge

### Phase 3: Distributed Testing (2-3 weeks)
- [ ] Create test harness for parallel branch composition
- [ ] Verify commutative property with 1000+ overlays
- [ ] Benchmark merge time (should be O(log n))

### Phase 4: Production Hardening (1-2 weeks)
- [ ] Add conflict logging and telemetry
- [ ] Implement conflict resolution workflows
- [ ] Create runbooks for rare manual intervention

---

## Reference Implementations

### G-Counter (Simple CRDT)
```rust
// G-Counter: grow-only counter (no decrement)
struct GCounter {
    replicas: Map<ReplicaId, u64>,
}

impl GCounter {
    fn increment(&mut self, replica: ReplicaId) {
        self.replicas[replica] += 1;
    }

    fn value(&self) -> u64 {
        self.replicas.values().sum()
    }

    fn merge(&mut self, other: GCounter) {
        for (r, v) in other.replicas {
            self.replicas[r] = max(self.replicas[r], v);
        }
    }
}

// nomrg overlays work similarly:
// - Each overlay is a delta
// - Overlays compose via max operation (take both)
// - Merge is just concatenation + dedup
```

---

## Benefits at Scale

### Merge Time
- **Git**: O(n²) with n branches (each pair must be manually resolved)
- **nomrg**: O(log n) with n branches (tree-based automatic composition)
- **At 1M branches**: Git takes years, nomrg takes seconds

### Human Effort
- **Git**: 1M merges → 1M conflict resolutions (if 1% conflict rate)
- **nomrg**: 1M merges → 0-10 manual interventions (if algebra is sound)

### Scalability
- **Git**: hits wall around 10-50 concurrent branches
- **nomrg**: scales to millions of concurrent branches

---

## References

- **PHILOSOPHY.md** — Graph-universe thesis context
- **KNHK.md** — KNHK (ontology) that nomrg overlays modify
- **AHI.md** (TODO) — How AHI proposes ΔΣ changes
- **src/autonomic/planes.rs** — ΔΣ plane implementation
- **src/kernel/ahi_policy.rs** — Overlay composition logic
- CRDT references:
  - Shapiro et al. "A comprehensive study of CRDT" (2016)
  - https://crdt.tech/
