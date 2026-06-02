# C4-03d: CONSTRUCT8 Motion Boundary

## Mission

Show how external proposals become admitted motions without direct proposal-to-state write. No runtime LLM in hot path.

## Motion Boundary Pipeline

```
┌────────────────────────────────────────────────────────────────┐
│                    CONSTRUCT8 MOTION BOUNDARY                  │
│                                                                │
│  External Proposal → Admit/Split → Apply (Branchless) →      │
│  Receipt + Replay (Proof that motion was lawful)             │
│                                                                │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │           EXTERNAL PROPOSAL (INPUT)                     │  │
│  │                                                          │  │
│  │  Proposal from external source:                         │  │
│  │  • User command (clap-noun-verb)                        │  │
│  │  • HTTP request (API)                                   │  │
│  │  • Event trigger (queue message)                        │  │
│  │  • Cronjob / scheduled task                             │  │
│  │                                                          │  │
│  │  Structure:                                              │  │
│  │  {                                                        │  │
│  │    "proposal_id": "uuid",                               │  │
│  │    "timestamp": "ISO8601",                              │  │
│  │    "requester": "user_id | agent_id",                  │  │
│  │    "action": "apply_motion",                            │  │
│  │    "scope": "noun_id",                                  │  │
│  │    "change": { ... },  // untyped dict                 │  │
│  │    "audit_trail": null  // not yet traced               │  │
│  │  }                                                        │  │
│  │                                                          │  │
│  └─────────────────────┬──────────────────────────────────┘  │
│                        │                                      │
│  ┌─────────────────────▼──────────────────────────────────┐  │
│  │       ADMISSION CHECK (GATE 1)                         │  │
│  │                                                          │  │
│  │  Rules (from doctrine.ttl):                            │  │
│  │  • Requester has authority for this noun?              │  │
│  │  • Action is allowed for this scope?                   │  │
│  │  • Change data contains required fields?               │  │
│  │  • Proposal does not violate usury limits?             │  │
│  │  • Proposal does not exceed Need9 threshold?           │  │
│  │                                                          │  │
│  │  Check: Change size > 8 fields?                        │  │
│  │  → If YES: split into separate motions (Need9)         │  │
│  │  → If NO: proceed as single motion                     │  │
│  │                                                          │  │
│  │  Verdict:                                                │  │
│  │  ✓ ADMIT: Proposal is lawful. Proceed.                │  │
│  │  ✗ REJECT: Proposal violates doctrine. Stop.          │  │
│  │  ⊕ SPLIT: Proposal too large. Divide and recurse.    │  │
│  │                                                          │  │
│  └─────────────────────┬──────────────────────────────────┘  │
│                        │                                      │
│        ┌───────────────┼───────────────┐                     │
│        │               │               │                     │
│   REJECT          SPLIT (Need9)      ADMIT                   │
│        │               │               │                     │
│   ┌────▼──┐      ┌─────▼────┐    ┌───▼──────┐               │
│   │ EVENT │      │ SPLIT     │    │DELTA8    │               │
│   │ LOG:  │      │ MOTION 1 │    │ CONTROL  │               │
│   │REJECT │      │ MOTION 2 │    │          │               │
│   │       │      │ ...      │    │ADMISSION │               │
│   │STOP.  │      │ MOTION N │    │CONTRACT  │               │
│   └───────┘      └─────┬────┘    └───┬──────┘               │
│                        │              │                      │
│                        │         ┌────▼──────────────────┐  │
│                        └────────→│  NEED9 RECURSION      │  │
│                                   │  (re-admit each split) │  │
│                                   └─────┬──────────────────┘  │
│                                         │                     │
│  ┌──────────────────────────────────────▼────────────────┐   │
│  │        GRADUATION CONTRACT (FOR EACH MOTION)           │   │
│  │                                                          │   │
│  │  Admitted motion now enters Delta8:                   │   │
│  │                                                          │   │
│  │  {                                                        │   │
│  │    "motion_id": "uuid",                                │   │
│  │    "delta8": {                                          │   │
│  │      "admitted": true,                                │   │
│  │      "requester": "user_id",                          │   │
│  │      "change": { field_1, field_2, ... },  // max 8   │   │
│  │      "timestamp": "ISO8601",                           │   │
│  │      "audit_trail": { ggen_rules, predicate_map, ... }│   │
│  │    },                                                   │   │
│  │    "sealed_at": "admission_timestamp",                │   │
│  │    "ready_for_apply": true                            │   │
│  │  }                                                        │   │
│  │                                                          │   │
│  │  CRITICAL: Motion is SEALED at admission.             │   │
│  │  It cannot be modified until wasm4pm executes.        │   │
│  │                                                          │   │
│  └──────────────────────┬───────────────────────────────┘   │
│                         │                                     │
│  ┌──────────────────────▼───────────────────────────────┐   │
│  │     BRANCHLESS APPLY (NO DECISION IN HOT PATH)        │   │
│  │                                                          │   │
│  │  wasm4pm executes motion WITHOUT further gating:      │   │
│  │                                                          │   │
│  │  apply(motion: Delta8) {                              │   │
│  │    // No decision: already admitted                    │   │
│  │    // No LLM: path is cold (already rendered by ggen) │   │
│  │    // No branching: delta is fixed at admission       │   │
│  │                                                          │   │
│  │    for field in motion.delta8.change {                │   │
│  │      state[field.key] = field.value;                  │   │
│  │      emit_write_event(field.key, field.value);        │   │
│  │    }                                                     │   │
│  │                                                          │   │
│  │    emit_completion_event(motion_id, SUCCESS);         │   │
│  │  }                                                       │   │
│  │                                                          │   │
│  │  Guarantees:                                            │   │
│  │  • No runtime decisions (all admitted earlier)        │   │
│  │  • No unbounded loops (max 8 fields written)          │   │
│  │  • Deterministic (same motion → same writes)          │   │
│  │  • Auditable (every write is an event)                │   │
│  │                                                          │   │
│  └──────────────────────┬───────────────────────────────┘   │
│                         │                                     │
│  ┌──────────────────────▼───────────────────────────────┐   │
│  │         EXECUTION RECEIPT (PROOF)                      │   │
│  │                                                          │   │
│  │  OCEL Event Log Entry:                                │   │
│  │  {                                                        │   │
│  │    "event_id": "uuid",                                │   │
│  │    "timestamp": "ISO8601",                            │   │
│  │    "event_type": "motion_applied",                    │   │
│  │    "objects": {                                        │   │
│  │      "motion": motion_id,                             │   │
│  │      "state_before": hash_before,                     │   │
│  │      "state_after": hash_after                        │   │
│  │    },                                                   │   │
│  │    "attributes": {                                    │   │
│  │      "fields_written": 8,                             │   │
│  │      "cpu_cycles": 500,                               │   │
│  │      "exit_code": 0,                                  │   │
│  │      "conformance": "PASS"                            │   │
│  │    }                                                     │   │
│  │  }                                                        │   │
│  │                                                          │   │
│  │  Receipt is signed to receipt ledger.                 │   │
│  │                                                          │   │
│  └──────────────────────┬───────────────────────────────┘   │
│                         │                                     │
│  ┌──────────────────────▼───────────────────────────────┐   │
│  │       REPLAY + CONFORMANCE CHECK (LIVING LSP)         │   │
│  │                                                          │   │
│  │  Can we re-execute the same motion and get the        │   │
│  │  same writes?                                          │   │
│  │                                                          │   │
│  │  $ living-lsp replay motion_id --log events.ocel      │   │
│  │                                                          │   │
│  │  If replay matches original execution:                │   │
│  │    ✓ Motion is lawful (deterministic, reproducible)  │   │
│  │                                                          │   │
│  │  If replay diverges:                                  │   │
│  │    ✗ Motion is unlawful (code is non-deterministic)  │   │
│  │                                                          │   │
│  │  Process Conformance:                                 │   │
│  │  • Declared process (admission rules) vs.             │   │
│  │  • Actual process (event log)                         │   │
│  │  • Mismatch = defect (event log proves code lied)     │   │
│  │                                                          │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                                │
│  PRINCIPLE:                                                   │
│  All decisions (admit/reject) are COLD PATH (before apply).  │
│  All execution is BRANCHLESS (no runtime decisions).         │
│  All proof is AUDITABLE (event log shows what happened).     │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## Cold Path vs. Hot Path

### Cold Path (Decision & Rendering)
- **Location:** CI, offline, before deployment
- **Work:** Admission checking, ggen rendering, type law court
- **LLM allowed:** Yes (generates audit trails, renders code)
- **Time:** Unlimited (seconds, minutes, hours)
- **Cost:** Budgeted; not part of user latency

Example: ggen renders artifacts with LLM context:
```
$ ggen render --llm-context "this is a receipt audit" \
   --source canon.ttl --query which_predicates_apply.rq
```

### Hot Path (Execution)
- **Location:** Runtime, production, in wasm4pm
- **Work:** Apply state changes (writes to fields)
- **LLM allowed:** No (decisions already made in cold path)
- **Time:** Bounded (milliseconds, microseconds)
- **Cost:** Part of user latency; must be fast and deterministic

Example: wasm4pm executes pre-admitted motion:
```rust
pub fn apply(motion: Delta8) -> Result<()> {
    // No decision here. Already admitted.
    // No LLM here. Already rendered.
    for field in motion.change.iter() {
        state[field.key] = field.value;
    }
    Ok(())
}
```

## Need9 Split Threshold

**Rule:** If a proposal has >8 fields, split it.

**Why:** 8 is Biblically lawful; 9 is Need (necessity, urgency). Proposals >8 must be broken into Need9 (groups of 9 or fewer).

**Example:**

Proposal with 25 fields:
- Motion 1: fields 1-8 (admitted, applied)
- Motion 2: fields 9-16 (admitted, applied)
- Motion 3: fields 17-25 (admitted, applied)

Each motion is independent. Each is admitted. Each is applied. All receipts are linked.

## Critical Boundary: Proposal ↔ Motion ↔ Execution

```
┌─────────────────┐      ┌────────────────┐      ┌──────────────┐
│    Proposal     │      │     Motion     │      │  Execution   │
│  (untyped,      │      │  (typed,       │      │  (admitted,  │
│  unverified)    │      │   admitted)    │      │   executed)  │
├─────────────────┤      ├────────────────┤      ├──────────────┤
│ • User input    │      │ • Graduated    │      │ • Branchless │
│ • Raw change    │  ──→ │   from compat  │  ──→ │   apply      │
│ • No decision   │      │ • Sealed at    │      │ • Event log  │
│                 │      │   admission    │      │ • Replay     │
│ Input: Proposal │      │ • Delta8       │      │ • Conformance│
│ Output: Reject/ │      │ • No runtime   │      │              │
│          Split/ │      │   decision     │      │ Output:      │
│          Admit  │      │                │      │ Receipt      │
└─────────────────┘      └────────────────┘      └──────────────┘
```

**Rule:** Proposal becomes motion only after admission. Motion is sealed and cannot be modified before execution.

## Architecturally Forbidden

- ❌ Proposal → direct state write (must be admitted first)
- ❌ Runtime decision in hot path (all decisions in cold path)
- ❌ LLM in apply() (only in ggen, only in cold path)
- ❌ Unbounded writes (max 8 fields per motion via Delta8)
- ❌ Unauditable execution (every write is an OCEL event)
- ❌ Silent failures (rejection is logged; applies are logged)
- ❌ Modification after admission (motion is sealed)
- ❌ Skip conformance check (Living LSP must verify process match)
