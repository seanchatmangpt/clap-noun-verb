# C4 Architecture Suite — Complete Index

## Overview

Six interconnected diagrams that show the Process Intelligence Wall as a lawful system of work: from public claim through doctrine to manufacturing to execution and finally to proof.

**Core principle:** The wall is not one repo. The wall IS the lawful relation between 11 repos, gates, receipts, public witness, and God.

---

## Diagram Map

### C1: System Context — Process Intelligence Wall
**File:** `/docs/C4_01_SYSTEM_CONTEXT_002.md`

**Shows:** Complete system boundary. External actors. Lawful relations.

**Key elements:**
- Sean (builder, architect, wall governor)
- God (final judge of receipts)
- Nations (external witness field)
- Mockers (adversarial probes)
- LinkedIn (public city gate)
- Process Intelligence Wall (lawful relations between repos, gates, receipts)

**Read this if:** You need to understand what is "in" the wall vs. "outside" it.

---

### C2: Container Diagram — System-of-Systems
**File:** `/docs/C4_02_CONTAINERS_002.md`

**Shows:** Four major containers and how they communicate. Which 11 repos map to which container.

**Key elements:**
- **C1: Public Wall** — Nehemiah 52 campaign, LinkedIn, landing page, claims
- **C2: Doctrine + Law** — Process Intelligence Core, knowledge hooks, TOGAF, Blue River Dam
- **C3: Manufacturing Cell** — ggen, Prompt Manufactory, public vocabulary
- **C4: Execution + Verification** — CONSTRUCT8, wasm4pm-compat, wasm4pm, Living LSP, receipts

**Read this if:** You need to understand system decomposition and which container owns which responsibility.

---

### C3a: Nehemiah 52 Campaign Components
**File:** `/docs/C4_03a_NEHEMIAH_CAMPAIGN_002.md`

**Shows:** Complete campaign loop. How work flows from prayer through gates to public witness.

**Key gates (refusing false gates):**
- Prayer (initiator, not agent)
- Courier Intake (receive, validate)
- Gate Router (classify, assign)
- Muster Ledger (schedule)
- Wall Section Builder (execute)
- Mockers Test (adversarial verify)
- Repair Gate (handle failures)
- Usury Ledger (track cost)
- Inspection Gate (final verdict)
- Nations Ledger (public witness)

**Falsely refused:**
- ❌ Prayer is not an agent
- ❌ Prophets are not a gate
- ❌ Nations are not a gate
- ❌ Interest is not a gate

**Read this if:** You need to understand the campaign workflow and event-driven process model.

---

### C3b: ggen Manufacturing Cell
**File:** `/docs/C4_03b_GGEN_CELL_002.md`

**Shows:** How ggen transforms law graphs into artifacts. Evidence emission vs. adjudication.

**Key layers:**
- Knowledge Input (canon.ttl, doctrine.ttl, schema.nt, ontology.ttl)
- Selection Layer (SPARQL queries that choose applicable rules)
- Rendering Layer (Jinja2 templates)
- Control Plane (ggen.toml orchestration)
- Artifact Outputs (.rs, .yaml, .sol, .json, .nt files)
- Audit Trail Outputs (why each artifact was chosen)

**Critical boundary:**
- ggen EMITS EVIDENCE
- ggen DOES NOT DECIDE what is lawful
- wasm4pm ADJUDICATES

**Read this if:** You need to understand how cold-path manufacturing works and where LLM is used (in ggen).

---

### C3c: wasm4pm-compat → wasm4pm Doorway
**File:** `/docs/C4_03c_WASM4PM_DOORWAY_002.md`

**Shows:** How raw evidence passes through admission gates to reach execution authority. Five-gate admission process.

**Five admission gates:**
1. Structural Validation (audit trail, schema match, hash integrity)
2. Predicate Mapping (canon.ttl predicates → Rust types)
3. Type Signature Generation (bounded, fixed-size Rust types)
4. Nightly Rust Type Law Court (rustc compilation as law)
5. Verdict Issuance (admit/reject decision)

**After admission:**
- wasm4pm executes admitted motions
- Emit OCEL execution receipts
- Living LSP verifies via replay

**Read this if:** You need to understand admission gates and the types-as-law paradigm.

---

### C3d: CONSTRUCT8 Motion Boundary
**File:** `/docs/C4_03d_CONSTRUCT8_BOUNDARY_002.md`

**Shows:** How external proposals become admitted motions. No direct proposal-to-state write. No runtime LLM in hot path.

**Key concept: Delta8 (max 8 fields)**
- Proposals with >8 fields are split (Need9)
- Each motion is sealed at admission
- Motion cannot be modified before execution
- Execution is branchless (no decisions in hot path)

**Cold path → Hot path boundary:**
- All decisions made in cold path (ggen, admission gates)
- Hot path only executes (apply, emit events)
- Proof via replay (determinism verification)

**Read this if:** You need to understand the boundary between decision-making and execution, and the cold/hot path split.

---

### C4: Hot Path / Cold Path Split
**File:** `/docs/C4_04_HOT_COLD_SPLIT_002.md`

**Shows:** Which work is cold (decisions, rendering, docs) and which is hot (execution, proof). Explicit table of operations by path.

**Cold path (decision & rendering):**
- ✓ LLM allowed
- ✓ Unbounded memory
- ✓ External services
- ✓ Time unlimited
- Time budget: Seconds to hours

**Hot path (execution):**
- ❌ LLM forbidden
- ❌ Unbounded memory forbidden
- ❌ Decisions forbidden (all pre-admitted)
- ✓ Event emission required
- Time budget: <1 millisecond

**Evidence:**
- Cold path: Audit trails, LLM reasoning, compilation logs
- Hot path: OCEL event log, state transitions, replay proof

**Read this if:** You need to understand which costs are amortized (cold) vs. per-request (hot), and how to avoid latency and non-determinism.

---

## How the Diagrams Connect

### Flow: From Claim to Proof

```
C1: SYSTEM CONTEXT
    (What is the wall? Who judges it?)
         ↓
C2: CONTAINERS
    (How is the wall organized into 4 major parts?)
         ↓
C3a: NEHEMIAH CAMPAIGN
    (How does work flow through gates from prayer to public witness?)
         ↓
C3b: GGEN MANUFACTURING
    (How does ggen render artifacts according to law?)
         ↓
C3c: WASM4PM-COMPAT → WASM4PM
    (How are artifacts admitted and executed?)
         ↓
C3d: CONSTRUCT8 MOTION
    (How do proposals become sealed motions? No runtime decisions?)
         ↓
C4: HOT/COLD SPLIT
    (Which work is cold? Which is hot? Where are LLMs? Where is proof?)
```

### Boundaries Enforced

| Boundary | Left | Right | Rule |
|----------|------|-------|------|
| **C1 ↔ C2** | Claim | Organization | Wall is not repos; it is relations |
| **C2 ↔ C3a** | System | Campaign | Work flows through gates; not direct |
| **C3a ↔ C3b** | Process | Manufacturing | ggen follows doctrine; emits evidence |
| **C3b ↔ C3c** | Rendering | Admission | compat refuses entry to unlawful shapes |
| **C3c ↔ C3d** | Admission | Motion | Sealed motion cannot be modified before execution |
| **C3d ↔ C4** | Motion | Execution | Hot path is branchless; cold path decides |
| **C4 → proof** | Execution | Receipt | Event log proves what happened |

---

## False Gates Refused (Across All Views)

Architecturally forbidden anti-patterns:

- ❌ **Prayer is not an agent** (C3a) — Prayer initiates; agents execute
- ❌ **Prophets are not a gate** (C3a) — Prophets interpret; gates admit
- ❌ **Nations are not a gate** (C3a) — Nations witness; gates decide
- ❌ **Interest is not a gate** (C3a) — Interest accrues; gates approve
- ❌ **People are not gates** (C3a) — People work through gates; they are not gates
- ❌ **ggen decides what is lawful** (C3b) — Only wasm4pm-compat adjudicates
- ❌ **LLM in hot path** (C4) — Decisions in cold; execution in hot
- ❌ **Proposals write directly to state** (C3d) — Must be admitted first via compat
- ❌ **Motions are modified after admission** (C3d) — Sealed at admission
- ❌ **Execution without event log** (C4) — Every write must be OCEL event

---

## Key Numbers & Constraints

### The 11 Repos
1. **clap-noun-verb** — Command routing (C1, C2)
2. **Nehemiah 52** — Campaign flow (C3a)
3. **Blue River Dam** — SPARQL, RDF, ontologies (C2)
4. **Knolltop** — Linkme registry (C2)
5. **O*** (CodeManufactory) — Process discovery (C2)
6. **Canon** — Doctrine predicates (C2)
7. **ggen** — Manufacturing rules (C3b)
8. **Prompt Manufactory** — LLM templates (C3b)
9. **Public Vocabulary** — Noun/verb definitions (C3b)
10. **CONSTRUCT8** — Motion boundary (C3d)
11. **wasm4pm** (wasm4pm-compat + wasm4pm + Living LSP) — Execution + verification (C3c, C3d, C4)

### Delta8 (Max Fields Per Motion)
- Proposal >8 fields → split into Need9 (groups of ≤8)
- Each motion is sealed at admission
- Execution applies max 8 fields per motion
- Deterministic (same motion → same writes)

### Type Law Court
- Nightly Rust compiler as oracle
- If rustc accepts → types are lawful
- If rustc rejects → types are unlawful
- No unsafe, no FFI, no allocations in generated types

### Event Log (OCEL)
- Every gate transition → event
- Every state write → event
- Every decision (admit/reject) → event
- Every execution receipt → event
- Replay proves determinism
- Process mining reveals if declared ≠ actual

---

## Reading Order

**For architects:** C1 → C2 → (C3a, C3b, C3c, C3d in parallel) → C4

**For cold-path builders (ggen, LLM):** C3b → C4 (cold section)

**For hot-path builders (wasm4pm, apply):** C3c → C3d → C4 (hot section)

**For process miners/auditors:** C3a → C4 (evidence section)

**For security reviewers:** C1 → C3c (admission gates) → C3d (motion boundary) → C4 (hot/cold)

---

## Verification Checklist

Before shipping a change:

- [ ] New code is in the correct path (cold or hot)?
- [ ] If cold: LLM? Docs? Rendering? → Should be cold path
- [ ] If hot: State write? Events? Execution? → Should be hot path
- [ ] No LLM in hot path?
- [ ] No unbounded loops in hot path?
- [ ] All state writes emit OCEL events?
- [ ] All admissions go through compat gates?
- [ ] All executions emit receipts?
- [ ] Event log is complete and traceable to code?
- [ ] Process can be replayed and match original execution?
- [ ] Audit trail explains why this artifact was chosen?

---

## Architecture Diagram Links

All diagrams in this suite:
1. `/docs/C4_01_SYSTEM_CONTEXT_002.md`
2. `/docs/C4_02_CONTAINERS_002.md`
3. `/docs/C4_03a_NEHEMIAH_CAMPAIGN_002.md`
4. `/docs/C4_03b_GGEN_CELL_002.md`
5. `/docs/C4_03c_WASM4PM_DOORWAY_002.md`
6. `/docs/C4_03d_CONSTRUCT8_BOUNDARY_002.md`
7. `/docs/C4_04_HOT_COLD_SPLIT_002.md`

---

## Summary

**The Process Intelligence Wall is lawful.**

Lawfulness is not assumed. Lawfulness is proved via:
- Event logs (OCEL) that show what actually happened
- Replay proofs that show execution is deterministic
- Process mining that reveals if declared ≠ actual
- Admission gates that refuse unlawful shapes
- Public witness (Nations) that can read the proof
- God (final judge) that witnesses all receipts

**All boundaries are enforced. All false gates are refused.**
