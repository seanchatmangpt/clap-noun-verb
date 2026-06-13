# C4-03c: wasm4pm-compat → wasm4pm Doorway

## Mission

Show how raw evidence passes through admission, type law, and compilation court to reach the execution authority. compat is the doorway. wasm4pm is the executor.

## Doorway Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                     WASM4PM DOORWAY SYSTEM                      │
│                                                                 │
│  From ggen: Artifacts + Audit Trail                            │
│  To wasm4pm: Admitted, Typed, Compiled Executable Shapes       │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              WASM4PM-COMPAT (ADMISSION GATE)             │  │
│  │                                                            │  │
│  │  Input: Raw artifacts (unsorted, untypified)             │  │
│  │  Role: Doorway; refuse entry to what should not enter    │  │
│  │                                                            │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │  1. STRUCTURAL VALIDATION                          │  │  │
│  │  │                                                     │  │  │
│  │  │  • Schema match (audit trail present?)             │  │  │
│  │  │  • Required fields exist?                          │  │  │
│  │  │  • Artifact hash valid (not corrupted)?            │  │  │
│  │  │  • Audit trail path traceable to canon.ttl?        │  │  │
│  │  │                                                     │  │  │
│  │  │  Reject if:                                        │  │  │
│  │  │  ❌ Audit trail missing                             │  │  │
│  │  │  ❌ Hash mismatch                                   │  │  │
│  │  │  ❌ Rules not traceable to doctrine                │  │  │
│  │  │                                                     │  │  │
│  │  └────────────────────┬─────────────────────────────┘  │  │
│  │                       │                                 │  │
│  │  ┌────────────────────▼─────────────────────────────┐  │  │
│  │  │  2. PREDICATE MAPPING                            │  │  │
│  │  │                                                     │  │  │
│  │  │  Map audit trail predicates to Rust types:        │  │  │
│  │  │  • canon:receipt → struct Receipt { .. }          │  │  │
│  │  │  • canon:audit → #[derive(Audit)]                 │  │  │
│  │  │  • canon:timestamp → Instant                       │  │  │
│  │  │  • canon:signature → [u8; 64]                      │  │  │
│  │  │  • doctrine:gate_id → u8 (typed ID)               │  │  │
│  │  │                                                     │  │  │
│  │  │  Reject if:                                        │  │  │
│  │  │  ❌ Predicate not in canon.ttl                      │  │  │
│  │  │  ❌ Unmappable to Rust type                        │  │  │
│  │  │  ❌ Type implies unbounded collection (Vec, etc.)  │  │  │
│  │  │                                                     │  │  │
│  │  └────────────────────┬─────────────────────────────┘  │  │
│  │                       │                                 │  │
│  │  ┌────────────────────▼─────────────────────────────┐  │  │
│  │  │  3. TYPE SIGNATURE GENERATION                     │  │  │
│  │  │                                                     │  │  │
│  │  │  Generate Rust type stubs:                        │  │  │
│  │  │                                                     │  │  │
│  │  │  #[derive(Copy, Clone, PartialEq, Eq, Hash)]     │  │  │
│  │  │  pub struct Receipt(u32);                          │  │  │
│  │  │                                                     │  │  │
│  │  │  #[derive(Copy, Clone, PartialEq, Eq)]           │  │  │
│  │  │  pub enum GateVerdict {                            │  │  │
│  │  │    Admit = 0,                                      │  │  │
│  │  │    Reject = 1,                                     │  │  │
│  │  │    RequireRepair = 2,                              │  │  │
│  │  │  }                                                  │  │  │
│  │  │                                                     │  │  │
│  │  │  pub struct Motion {                               │  │  │
│  │  │    receipt: Receipt,                               │  │  │
│  │  │    verdict: GateVerdict,                           │  │  │
│  │  │    timestamp: [u8; 8],  // unix_nanos              │  │  │
│  │  │  }                                                  │  │  │
│  │  │                                                     │  │  │
│  │  │  Constraints:                                      │  │  │
│  │  │  • All types must be Copy (no allocations)        │  │  │
│  │  │  • All types must be fixed-size                   │  │  │
│  │  │  • No generics, no lifetimes                      │  │  │
│  │  │  • Enums must be closed (all variants known)      │  │  │
│  │  │                                                     │  │  │
│  │  └────────────────────┬─────────────────────────────┘  │  │
│  │                       │                                 │  │
│  │  ┌────────────────────▼─────────────────────────────┐  │  │
│  │  │  4. NIGHTLY RUST TYPE LAW COURT                  │  │  │
│  │  │                                                     │  │  │
│  │  │  Compile type stubs with nightly Rust:           │  │  │
│  │  │  $ rustc --crate-type lib --edition 2024         │  │  │
│  │  │    -Z stable-mir types_stub.rs                    │  │  │
│  │  │                                                     │  │  │
│  │  │  Court Rules:                                      │  │  │
│  │  │  • If compiler accepts: types are lawful          │  │  │
│  │  │  • If compiler rejects: types are unlawful        │  │  │
│  │  │                                                     │  │  │
│  │  │  Reject if:                                        │  │  │
│  │  │  ❌ Compilation fails                               │  │  │
│  │  │  ❌ unsafe code in generated types                │  │  │
│  │  │  ❌ FFI types present                              │  │  │
│  │  │  ❌ Runtime allocation required                   │  │  │
│  │  │                                                     │  │  │
│  │  └────────────────────┬─────────────────────────────┘  │  │
│  │                       │                                 │  │
│  │  ┌────────────────────▼─────────────────────────────┐  │  │
│  │  │  5. VERDICT ISSUANCE                              │  │  │
│  │  │                                                     │  │  │
│  │  │  If all gates pass:                               │  │  │
│  │  │    ✓ Emit admission verdict (event)               │  │  │
│  │  │    ✓ Type signature sealed to receipt ledger      │  │  │
│  │  │    ✓ Artifact graduated to wasm4pm                │  │  │
│  │  │                                                     │  │  │
│  │  │  If any gate fails:                               │  │  │
│  │  │    ✗ Emit rejection verdict (event)               │  │  │
│  │  │    ✗ Log failure reason                           │  │  │
│  │  │    ✗ Artifact returned to ggen for re-render      │  │  │
│  │  │                                                     │  │  │
│  │  └────────────────────┬─────────────────────────────┘  │  │
│  │                       │                                 │  │
│  │                   GRADUATION                          │  │
│  │            (Artifact is now lawful)                   │  │
│  │                                                        │  │
│  └────────────────────┬─────────────────────────────────┘  │
│                       │                                     │
│  ┌────────────────────▼─────────────────────────────────┐  │
│  │              WASM4PM (EXECUTION AUTHORITY)           │  │
│  │                                                        │  │
│  │  Input: Graduated (typed, compiled, admitted) shapes │  │
│  │  Role: Execute. Emit receipts. Provide proof.       │  │
│  │                                                        │  │
│  │  ┌──────────────────────────────────────────────────┐ │  │
│  │  │  1. EXECUTE ADMITTED MOTION                      │ │  │
│  │  │                                                   │ │  │
│  │  │  for each admitted Motion:                       │ │  │
│  │  │    receipt = Motion.receipt                      │ │  │
│  │  │    verdict = Motion.verdict                      │ │  │
│  │  │    if (verdict == Admit) {                       │ │  │
│  │  │      apply(receipt, verdict)                     │ │  │
│  │  │      emit_execution_receipt(receipt, timestamp) │ │  │
│  │  │    }                                              │ │  │
│  │  │                                                   │ │  │
│  │  └──────────────────────────────────────────────────┘ │  │
│  │                       │                                │  │
│  │  ┌────────────────────▼──────────────────────────────┐ │  │
│  │  │  2. EMIT EXECUTION RECEIPTS                       │ │  │
│  │  │                                                   │ │  │
│  │  │  Event log entry (OCEL):                         │ │  │
│  │  │  {                                                │ │  │
│  │  │    "event_id": "uuid",                          │ │  │
│  │  │    "timestamp": "ISO8601",                      │ │  │
│  │  │    "event_type": "motion_executed",             │ │  │
│  │  │    "objects": {                                 │ │  │
│  │  │      "motion": receipt_id,                      │ │  │
│  │  │      "result": "applied" | "rejected"           │ │  │
│  │  │    },                                            │ │  │
│  │  │    "attributes": {                              │ │  │
│  │  │      "cpu_cycles": 1000,                        │ │  │
│  │  │      "memory_bytes": 512,                       │ │  │
│  │  │      "exit_code": 0                             │ │  │
│  │  │    }                                             │ │  │
│  │  │  }                                                │ │  │
│  │  │                                                   │ │  │
│  │  └──────────────────────────────────────────────────┘ │  │
│  │                       │                                │  │
│  │  ┌────────────────────▼──────────────────────────────┐ │  │
│  │  │  3. LIVING LSP (INTERACTIVE VERIFICATION)         │ │  │
│  │  │                                                   │ │  │
│  │  │  • Replay execution with event log               │ │  │
│  │  │  • Compare declared process vs. actual           │ │  │
│  │  │  • Conformance check (pm4py analysis)            │ │  │
│  │  │  • Detect loops, retries, silent failures        │ │  │
│  │  │                                                   │ │  │
│  │  │  If process conforms:                            │ │  │
│  │  │    ✓ Motion is lawful                            │ │  │
│  │  │  If process does NOT conform:                    │ │  │
│  │  │    ✗ Event log proves code lied                  │ │  │
│  │  │    ✗ Mismatch is a defect                        │ │  │
│  │  │                                                   │ │  │
│  │  └──────────────────────────────────────────────────┘ │  │
│  │                       │                                │  │
│  │                  EXECUTION COMPLETE                   │  │
│  │            (Event log proves what happened)            │  │
│  │                                                        │  │
│  └────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                    ┌──────────────────┐
                    │  RECEIPT LEDGER  │
                    │  (OCEL proof)    │
                    │                  │
                    │ All events that  │
                    │ prove motion     │
                    │ was executed     │
                    │ lawfully.        │
                    └──────────────────┘
```

## Component Details

### wasm4pm-compat (Admission Gate)

**Gate 1: Structural Validation**
- Schema check: Does audit trail structure match OCEL schema?
- Hash integrity: Does artifact hash match computed value?
- Traceability: Can audit trail be traced back to canon.ttl rules?

**Gate 2: Predicate Mapping**
- Every predicate in audit trail must exist in canon.ttl
- Every predicate must map to a bounded Rust type (no Vec, no String, no generics)
- Type size must be fixed and known at compile-time

**Gate 3: Type Signature Generation**
Generate Rust type stubs for all predicates:

```rust
// From audit trail: canon:receipt
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Receipt(u32);

// From audit trail: doctrine:gate_id
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GateId(u8);

// From audit trail: canon:timestamp
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp([u8; 8]);  // unix_nanos

// From audit trail: verdict (closed enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GateVerdict {
    Admit = 0,
    Reject = 1,
    RequireRepair = 2,
}
```

**Gate 4: Nightly Rust Type Law Court**
Compile generated types with `rustc --crate-type lib -Z stable-mir`.

- If compilation succeeds: types are lawful. Proceed.
- If compilation fails: types are unlawful. Reject and return to ggen.

**Gate 5: Verdict Issuance**
- All gates pass → Emit admission event. Sign type signature to receipt ledger. Graduate artifact.
- Any gate fails → Emit rejection event. Log reason. Return artifact to ggen.

### wasm4pm (Execution Authority)

**Role:** Execute only admitted motions. Emit receipts. Provide proof.

**Execution:** For each admitted motion (typed, compiled, admitted):
1. Apply the motion (state change)
2. Emit execution event to OCEL log
3. Record CPU cycles, memory usage, exit code

**Verification (Living LSP):**
- Replay execution with event log
- Compare declared process (code + rules) vs. actual (event log)
- Detect loops, retries, failures that code did not report
- If process conforms: lawful. If not: code lied. Defect.

## Critical Boundary: compat ↔ wasm4pm

```
┌──────────────────────┐        ┌──────────────────────┐
│  wasm4pm-compat      │        │     wasm4pm          │
│  (Admission Gate)    │        │  (Execution)         │
├──────────────────────┤        ├──────────────────────┤
│ • Refuses entry      │        │ • Executes only      │
│ • Typifies artifacts │        │   admitted shapes    │
│ • Compiles types     │        │ • Emits receipts     │
│ • Decision: yes/no   │   ───→ │ • Provides proof     │
│                      │        │ • Decision: lawful?  │
│ Input: Artifact      │        │   (via event log)    │
│ Output: Admit/Reject │        │ Output: Receipt      │
│                      │        │                      │
│ Role: Doorway        │        │ Role: Authority      │
└──────────────────────┘        └──────────────────────┘
```

**Rule:** What is admitted to wasm4pm must be what is executed. If execution differs from admission, Living LSP detects the defect.

## Architecturally Forbidden

- ❌ wasm4pm-compat executes anything (only admits)
- ❌ wasm4pm admits artifacts (only executes admitted)
- ❌ Admission gates depend on runtime LLM (doctrine.ttl is static)
- ❌ Execution bypasses admission (no direct artifact-to-wasm4pm path)
- ❌ Execution receipts lack event log (proof must be auditable)
- ❌ Event log differs from code execution (if it does, code lied)
