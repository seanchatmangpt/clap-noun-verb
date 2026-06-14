# C4-04: Hot Path / Cold Path Split

## Mission

Show which work is cold (decisions, rendering, documentation) and which is hot (execution, proof). LLMs are cold only. Hot is bounded and deterministic.

## Hot/Cold Split Table

| Category | Activity | Cold Path | Hot Path | Why |
|----------|----------|-----------|----------|-----|
| **Input** | Proposal arrives | Acceptance | Execution | User sees execution latency, not decision latency |
| **Input** | External request | Accept (slow) | Apply (fast) | Admit/reject can be slow; execution must be fast |
| **Decision** | LLM context generation | ✓ Allowed | ❌ Forbidden | LLM latency must not affect user experience |
| **Decision** | Doctrine check | ✓ Allowed | ❌ Forbidden | Admin work; not user-blocking |
| **Decision** | Type law court | ✓ Allowed | ❌ Forbidden | Compile at CI; not at runtime |
| **Rendering** | ggen artifact creation | ✓ In CI | ❌ Not at runtime | Artifacts pre-rendered; hot path uses them |
| **Rendering** | Template expansion | ✓ In CI | ❌ Not at runtime | .tera templates rendered offline |
| **Rendering** | LLM-assisted code gen | ✓ In CI | ❌ Not at runtime | LLM assists offline; runtime uses result |
| **Execution** | Apply state change | ❌ Not here | ✓ Required | Hot path does actual work |
| **Execution** | Emit receipt event | ❌ Not here | ✓ Required | Proof must be synchronous with execution |
| **Verification** | Event log generation | ❌ Not here | ✓ Required | OCEL log must be complete and live |
| **Verification** | Conformance check | ✓ After fact | ✓ (Live LSP) | Replay/check happens post-execution (Living LSP) |
| **Storage** | Docs, canon, ARD | ✓ Cold | ❌ Not hot | Documentation is reference material |
| **Storage** | PRD (Product Requirements) | ✓ Cold | ❌ Not hot | Requirements are reference material |
| **Storage** | LLM audit explanations | ✓ Cold | ❌ Not hot | Explanations help humans understand decisions |
| **Storage** | Receipt ledger | ✓ Add post-exec | ✓ Record exec | Receipts are written during/after execution |
| **Proof** | Audit trail | ✓ Recorded in CI | ✓ Linked to exec | Trail explains why artifact was chosen |
| **Proof** | Event log (OCEL) | ❌ Not here | ✓ Required | Event log is proof of execution |
| **Proof** | Replay determinism | ✓ (After fact) | ✓ (Via Living LSP) | Replay proves execution was lawful |

## Hot Path (Execution Authority — wasm4pm)

### Permitted Operations
- ✓ Read fixed-size typed state (u32, [u8; 64], enums with known variants)
- ✓ Write to a bounded set of fields (Delta8: max 8 fields per motion)
- ✓ Emit OCEL event (one event per write)
- ✓ Return verdict (success/failure)
- ✓ Update state hash (Merkle root)

### Forbidden Operations
- ❌ Call LLM or external service
- ❌ Allocate unbounded memory (Vec, String, HashMap)
- ❌ Spawn threads or tasks
- ❌ Use floating-point or arbitrary precision
- ❌ Make decisions (all decisions are pre-admitted in cold path)
- ❌ Loop unbounded times
- ❌ Use randomness (determinism required for replay)
- ❌ Call syscalls or I/O
- ❌ Panic or abort (must return Result or error code)

### Type Guarantees
```rust
// Hot path types MUST be Copy, Clone, PartialEq, Eq, Hash
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReceiptId(u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GateVerdict {
    Admit = 0,
    Reject = 1,
    RequireRepair = 2,
}

// Hot path structures MUST be fixed-size
#[repr(C)]
pub struct Motion {
    receipt: ReceiptId,           // 4 bytes
    verdict: GateVerdict,         // 1 byte
    timestamp: [u8; 8],           // 8 bytes (unix_nanos)
    // Total: 13 bytes, known at compile-time
}

// Hot path CANNOT use these:
// ❌ Vec<T>           (unbounded)
// ❌ String           (unbounded)
// ❌ HashMap<K, V>    (unbounded)
// ❌ Box<T>           (heap allocation)
// ❌ Rc<T>            (reference counting)
// ❌ Arc<T>           (atomic reference counting)
// ❌ &'_ str          (slice, not owned)
// ❌ async fn         (cannot suspend)
// ❌ trait objects    (dyn Trait)
```

### CPU/Memory Budget
- **CPU time:** <1 millisecond per motion (bounded iterations, no unbounded loops)
- **Memory:** <1 KB per motion (fixed-size types only)
- **Cache behavior:** Predictable (no pointer chasing, no cache-unfriendly allocations)

### Example Hot Path Code

```rust
pub fn apply_motion(motion: &Motion) -> Result<(), NounVerbError> {
    // Read state (bounded, fixed-size)
    let current_state = STATE_TABLE.read(motion.receipt);
    
    // No decision: already admitted in cold path
    if motion.verdict != GateVerdict::Admit {
        return Err(NounVerbError::MotionRejected);
    }
    
    // Write (bounded: max 8 fields)
    for (key, value) in motion.delta8.iter() {  // delta8: &[Field; 8]
        STATE_TABLE.write(key, value);
        emit_write_event(motion.receipt, key, value);
    }
    
    // Deterministic: same motion → same writes
    Ok(())
}
```

---

## Cold Path (Decision & Rendering)

### Permitted Operations
- ✓ Call LLM (Claude, local models, etc.)
- ✓ Read entire graph (RDF, ontologies)
- ✓ Run SPARQL queries (complex, expensive)
- ✓ Generate code with ggen
- ✓ Write audit trails and explanations
- ✓ Perform type checking and compilation
- ✓ Log debugging information
- ✓ Make decisions (admission, splitting, routing)
- ✓ Allocate unbounded memory (lists, maps, strings)
- ✓ Use randomness or non-determinism (decisions are made offline)
- ✓ Call external services (SPARQL endpoint, code generator)
- ✓ Spawn threads/processes for CI/CD
- ✓ Write to files and databases

### Time Budget
- Unlimited (seconds, minutes, hours per decision)
- Batch processing OK (process many proposals together)
- Retry logic OK (re-render if law changes)

### Cost Model
- Cost is amortized (cold path cost / number of served hot-path calls)
- No per-request charge (decisions batch across users)
- LLM costs are infrastructure (like CI/CD)

### Example Cold Path Code (ggen rendering)

```python
# Cold path: ggen with LLM context
def render_audit_trail(artifact_id: str, rules: List[DoctrineRule]):
    # LLM is allowed here
    context = generate_llm_context(
        artifact=artifact_id,
        rules=rules,
        max_tokens=2000  # Can be large; not user-blocking
    )
    
    # SPARQL queries are allowed here
    matching_predicates = sparql_query(
        """
        SELECT ?p ?rule
        WHERE {
            ?p a ?type .
            ?rule doctrine:appliesTo ?type .
        }
        """
    )
    
    # Complex rendering is allowed here
    audit_trail = {
        "artifact_id": artifact_id,
        "rules_applied": matching_predicates,
        "llm_reasoning": context,
        "timestamp": now(),
    }
    
    return audit_trail
```

### Example Cold Path Code (Type Law Court)

```rust
// Cold path: rustc type checking (CI/CD)
fn validate_types_with_nightly_rust(generated_types: &str) -> Result<()> {
    // Write to temp file
    let temp_file = "/tmp/types_check.rs";
    fs::write(temp_file, generated_types)?;
    
    // Invoke rustc with expensive checks
    let output = std::process::Command::new("rustc")
        .args([
            "--crate-type", "lib",
            "--edition", "2024",
            "-Z", "stable-mir",  // Expensive; only in CI
            temp_file,
        ])
        .output()?;
    
    if !output.status.success() {
        Err(NounVerbError::TypeCheckFailed(
            String::from_utf8(output.stderr)?
        ))
    } else {
        Ok(())
    }
}
```

---

## Cold → Hot Boundary

### What Crosses the Boundary
- **From cold to hot:** Admitted motion (Delta8: typed, sealed, approved)
- **Signature:** All decisions made. All proof of correctness established. Ready for deterministic execution.

### What Does NOT Cross
- ❌ Decisions (made in cold)
- ❌ LLM context (generated in cold)
- ❌ Audit trails (recorded in cold)
- ❌ Code generation (done in cold)
- ❌ Type checking (done in cold)

### Seal at Admission
```rust
// Proposal (untyped, cold path)
{
    "change": { "field_1": "?", "field_2": "?", ... }  // 25 fields, unknown types
}

// COLD PATH: Admission + splitting
// Verdict: SPLIT (>8 fields) + types validated + rules checked

// Motion (typed, sealed, hot path)
{
    "motion_id": "uuid",
    "delta8": [
        { "field_1": 42u32, "field_2": "bytes[64]", ... },  // Max 8 fields
    ],
    "sealed_at": "2026-06-02T10:30:00Z",
    "ready_for_hot_path": true,
}

// RULE: Once sealed, motion cannot be modified.
// Hot path reads it as immutable.
```

---

## Evidence & Proof

### Cold Path Evidence
- **LLM audit trails:** Explain why ggen chose each rule
- **Compilation logs:** Prove types are valid
- **SPARQL traces:** Show which predicates matched
- **ggen explanations:** "We rendered this because predicate X applies"

Location: `docs/`, audit files, code comments

### Hot Path Evidence (OCEL Event Log)
- **Every write event:** "At timestamp T, wrote value V to field F"
- **Every verdict:** "Motion M was admitted/rejected"
- **State snapshots:** Hash of state before/after each motion
- **Replay proof:** Event log can be re-executed to match original

Location: Receipt ledger, event.log file, Living LSP

---

## Compliance Checklist

### For Cold Path Code
- [ ] Is this decision-making code? (Route to cold)
- [ ] Does it call LLM? (Must be cold)
- [ ] Does it run in CI/CD? (Probably cold)
- [ ] Can it take unlimited time? (Probably cold)
- [ ] Does it generate audit trails? (Yes, keep in cold)

### For Hot Path Code
- [ ] Is this applying a motion? (Route to hot)
- [ ] Does it emit events? (Must be hot)
- [ ] Does it modify state? (Must be hot)
- [ ] All types Copy + Clone + Eq? (Required for hot)
- [ ] No allocations? (Required for hot)
- [ ] No LLM calls? (Forbidden in hot)
- [ ] Deterministic? (Required for hot, proof via replay)

---

## Architecturally Forbidden

- ❌ LLM in hot path (latency, non-determinism)
- ❌ Unbounded loops in hot path (determinism, latency)
- ❌ Allocations in hot path (latency, GC)
- ❌ Decisions in hot path (all decisions in cold)
- ❌ State write without event (every write must be logged)
- ❌ Event without state write (no phantom events)
- ❌ Execution without admission (admission gates are mandatory)
- ❌ Audit trail in hot path (trails are cold; events are hot)
