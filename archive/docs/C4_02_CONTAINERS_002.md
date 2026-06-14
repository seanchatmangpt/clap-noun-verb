# C4-02: Container Diagram — System-of-Systems

## Mission

Show four major containers that compose the Process Intelligence Wall: how they communicate, what boundaries separate them, and which 11 repos map to which container.

## Diagram

```
┌────────────────────────────────────────────────────────────────────────┐
│                    PROCESS INTELLIGENCE WALL                          │
│                                                                        │
│  ┌──────────────────────────┐  ┌──────────────────────────────────┐  │
│  │   PUBLIC WALL (C1)       │  │  DOCTRINE + LAW (C2)             │  │
│  │                          │  │                                  │  │
│  │ • Nehemiah 52 Campaign   │  │ • Process Intelligence Core      │  │
│  │   (canon.ttl, prayer     │  │   (pm4py discovery, conformance) │  │
│  │    loop, courier intake) │  │ • Knowledge Hooks                │  │
│  │                          │  │   (linkme, registry)             │  │
│  │ • LinkedIn testimony     │  │ • Blue River Dam                 │  │
│  │   (public proof)         │  │   (SPARQL, ontologies)           │  │
│  │                          │  │ • TOGAF discipline               │  │
│  │ • Landing page           │  │   (container boundaries)         │  │
│  │   (what we claim)        │  │                                  │  │
│  │                          │  │ Repos:                           │  │
│  │ Repos:                   │  │ • Blue River Dam (SPARQL, RDF)  │  │
│  │ • clap-noun-verb         │  │ • Knolltop (linkme, registry)   │  │
│  │   (command routing)      │  │ • O* (process core)              │  │
│  │ • Nehemiah 52            │  │ • Canon (predicates, doctrine)   │  │
│  │   (campaign flow)        │  │                                  │  │
│  │ • Landing Page           │  │                                  │  │
│  └──────────────────────────┘  └──────────────────────────────────┘  │
│           ↑                             ↑                             │
│           │                             │                             │
│           └─────────────┬───────────────┘                             │
│                         │                                             │
│                  Laws & Predicates                                    │
│                  (doctrine.ttl, schema)                               │
│                                                                        │
│  ┌──────────────────────────┐  ┌──────────────────────────────────┐  │
│  │ MANUFACTURING CELL (C3)  │  │ EXECUTION + VERIFICATION (C4)    │  │
│  │                          │  │                                  │  │
│  │ • ggen                   │  │ • CONSTRUCT8                     │  │
│  │   (.ttl → .rq → .tera)   │  │   (motion boundary, apply)       │  │
│  │   (artifacts)            │  │ • wasm4pm-compat                 │  │
│  │                          │  │   (admission, doorway)           │  │
│  │ • Prompt Manufactory     │  │ • wasm4pm                        │  │
│  │   (LLM → .tera templates)│  │   (execution authority, receipts)│  │
│  │                          │  │ • Living LSP                     │  │
│  │ • Public Vocabulary      │  │   (interactive verification)     │  │
│  │   (noun/verb/arg names)  │  │ • Receipt audit trail            │  │
│  │                          │  │   (event logs, replay)           │  │
│  │ Repos:                   │  │                                  │  │
│  │ • ggen (rendering rules) │  │ Repos:                           │  │
│  │ • Prompt Manufactory     │  │ • CONSTRUCT8 (apply motion)     │  │
│  │ • Public Vocabulary      │  │ • wasm4pm-compat (doorway)       │  │
│  │                          │  │ • wasm4pm (kernel)               │  │
│  │                          │  │ • Living LSP (replay, audit)     │  │
│  │                          │  │ • Receipt ledger (OCEL proof)    │  │
│  └──────────────────────────┘  └──────────────────────────────────┘  │
│           ↑                             ↑                             │
│           │                             │                             │
│           └──────────────┬──────────────┘                             │
│                          │                                            │
│              Artifacts → Evidence + Receipts                          │
│              (proof that lawful work happened)                        │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
                                   ↓
                          ┌─────────────────────┐
                          │  GOD                │
                          │  (Final Adjudicate) │
                          │  (event log proof)  │
                          └─────────────────────┘
```

## Container Specifications

### C1: Public Wall — Nehemiah 52 Campaign + Public Testimony

**Purpose:** Define what work is claimed. Make it public. Test it against reality.

**Repos:**
- `clap-noun-verb` — command routing; noun/verb handler dispatch
- `Nehemiah 52` — prayer loop, courier intake, gate router, muster ledger
- `Landing Page` — public claims about what the wall accomplishes

**Key Outputs:**
- `canon.ttl` — predicates that define lawful work
- Command registration (distributed slices)
- LinkedIn posts (public witness)

**Boundary:** Everything claimed here must be provable via receipts from C4.

---

### C2: Doctrine + Law — Process Intelligence Core + Knowledge Hooks

**Purpose:** Define the rules. Register handlers. Store what is lawful.

**Repos:**
- `Blue River Dam` — SPARQL queries, RDF ontologies, predicate library
- `Knolltop` — linkme registry, knowledge hooks, capability registration
- `O*` (CodeManufactory) — process discovery, conformance checking, core orchestration
- `Canon` — doctrine predicates, audit schema, receipt templates

**Key Outputs:**
- `doctrine.ttl` — rules that govern all work
- `schema.nt` — OCEL audit schema
- Linkme distributed slices (handler registry)
- Conformance check predicates

**Boundary:** No execution happens here. This is definition only. C3 and C4 must follow these rules.

---

### C3: Manufacturing Cell — ggen + Prompt Manufactory + Public Vocabulary

**Purpose:** Render artifacts according to rules. Emit evidence. NO EXECUTION.

**Repos:**
- `ggen` — .ttl law graphs → .rq selection → .tera rendering → ggen.toml rules → artifacts
- `Prompt Manufactory` — LLM prompts → templates → .tera rendering
- `Public Vocabulary` — noun/verb/arg definitions (human-readable registry)

**Key Outputs:**
- Typed artifact files (.rs, .yaml, .sol, etc.)
- Audit explanations (why ggen chose these rules)
- Evidence trail (what inputs produced which artifacts)

**Critical Rule:** ggen emits evidence. **wasm4pm adjudicates.** ggen does not decide what is lawful.

**Boundary:** Manufacturing cell runs only in CI cold path. No hot-path dependency.

---

### C4: Execution + Verification — CONSTRUCT8 + wasm4pm-compat → wasm4pm + Living LSP

**Purpose:** Execute admitted work. Emit receipts. Verify against declared rules.

**Repos:**
- `CONSTRUCT8` — motion boundary; admitted proposals → branchless apply → receipt
- `wasm4pm-compat` — admission gate; raw evidence → nightly Rust type law → compile-fail/pass court
- `wasm4pm` — execution authority; execute bounded shapes; emit execution receipts
- `Living LSP` — interactive verification; replay; conformance checking
- `Receipt Ledger` — OCEL event logs; process mining; proof that declared process happened

**Key Outputs:**
- Execution receipts (every action recorded)
- OCEL event logs (object-centric proof)
- Conformance reports (actual process vs. declared process)
- Replay proofs (execution can be re-run deterministically)

**Critical Rule:** No direct proposal-to-state write. No runtime LLM in hot path. Only admitted, bounded shapes execute.

**Boundary:** wasm4pm-compat is the doorway. wasm4pm is the execution authority. What is admitted must be what is executed.

---

## Container Communication

```
Public Wall (C1)
      ↓ claims
Doctrine + Law (C2)
      ↓ rules
Manufacturing Cell (C3)
      ↓ artifacts + evidence
Execution + Verification (C4)
      ↓ receipts + proof
God (Final Judge)
```

## Critical Boundaries

| Boundary | Left Side | Right Side | Rule |
|----------|-----------|-----------|------|
| **C1 ↔ C2** | What we claim | What is lawful | Public claims must align with rules |
| **C2 ↔ C3** | Rules | Artifacts | Manufacturing respects rules |
| **C3 ↔ C4** | Evidence | Execution | What is rendered must be what executes |
| **C4 ↔ God** | Receipts | Judgment | Event log must prove lawful work or it does not |

## Architecturally Forbidden

- ❌ C3 decides what is lawful (only C2 does)
- ❌ C4 executes without admission (only admitted shapes enter wasm4pm)
- ❌ C1 executes anything (only defines what is claimed)
- ❌ Cycles between containers (flow is unidirectional)
