# MCPP Research Documentation

**Research Context:** This directory contains theoretical and architectural research documents exploring MCPP (MCP Plus) as the "Universal Port" for UniverseOS—a unified substrate where ontologies, process models (POWL8), and tool protocols converge into a lawful, receipt-producing execution loop.

**Status:** Active Research Archive (v5.6.0+) — These documents inform future architectural decisions and serve as a knowledge base for the clap-noun-verb framework.

---

## File Inventory

| File | Purpose | Scope | Integration Status |
|------|---------|-------|-------------------|
| **RESEARCH_FINDINGS.md** | Executive summary of MCPP extraction strategy, core components mapping to codebase | Bridges theory ↔ implementation | Referenced in `ARCHITECTURAL_CONTINUITY.md` |
| **PHD_LEVEL_SYNTHESIS.md** | Formal mathematical framework: Chatman Equation, POWL8 as ISA, admissibility functions, receipts | Foundational theory | Philosophy for `MASTER_RESEARCH_PLAN.md` |
| **ARCHITECTURAL_ALIGNMENT.md** | Operational integration within UniverseOS: µ operator, feedback loop, "Tom" (Doctor/Wizard/Telco) layer | System coherence | Context for governance invariants |
| **POWL8_ISA.md** | Technical specification of POWL8 as executable control alphabet (micro-ops, hierarchy) | ISA definition | Informs future execution runtime design |

---

## Research Findings Summary

### Core Thesis
MCPP realizes the **Chatman Equation**: $A = \mu(O^*)$

- **O**: Raw observation / unconstrained state
- **O***: Semantic closure (bounded, typed ontology)
- **μ**: Lawful operator (MCPP control plane: rules, policies, workflows, proofs)
- **A**: Executable action

**Key Insight**: MCPP converts abstract semantic knowledge graphs into a lawful, autonomic runtime where tools, workflows, and economic settlements are isomorphic projections of the same operational ontology.

### Critical Components Identified

1. **RDF Engine** (`src/rdf/ontology.rs`, `sparql_executor.rs`, `src/semantic/capability.rs`)
   - Operational ontology store and query execution

2. **Universal Port (MCP)** (`src/rdf/mcp_server.rs`, `src/semantic/protocol.rs`)
   - Protocol substrate unifying tool invocation

3. **Law & Acceptance** (`src/rdf/guard_validation.rs`, `src/kernel/policy_governance.rs`)
   - SHACL-based validation and policy enforcement via admissibility predicate: `Accept(ΔO)`

4. **Memory & Audit** (`src/rdf/lockchain.rs`, `src/rdf/receipt.rs`)
   - Cryptographic receipts (Blake3) closing the cybernetic feedback loop

5. **POWL8 ISA**
   - Executable process geometry: sequence, choice, loop, partial-order concurrency, synchronization
   - Designed for AtomVM and concurrent runtime execution at programming-language speed

6. **Bootstrap Layer ("Tom")**
   - Three archetypal capabilities: **Doctor** (epistemology/validation), **Wizard** (transformation), **Telco** (connectivity)
   - Ensures every node in the system is initialized with minimal complete basis of lawful capabilities

---

## Recommendations

### (A) Keep as Research Archive — Move to `_internal/`

**Status:** ✓ RECOMMENDED

**Rationale:**
- These documents are **foundational but not user-facing**. They describe theoretical underpinnings that inform architectural decisions rather than procedural guidance.
- They reference future extraction targets (`src/rdf/`, `src/kernel/`) that are not yet stabilized in the public API.
- They are actively cited in internal research documents (`MASTER_RESEARCH_PLAN.md`, `ARCHITECTURAL_CONTINUITY.md`).

**Action:**
```bash
mkdir -p docs/_internal/research-mcpp
mv docs/mcpp/*.md docs/_internal/research-mcpp/
rmdir docs/mcpp
```

**Why not keep in `docs/mcpp/`?**
- The directory name implies active, stable documentation.
- Users browsing `docs/` should not encounter theoretical research without a clear integration path.
- Prevents confusion about whether MCPP is a user-facing feature (it is not, yet).

---

### (B) Link from `docs/explanation/` (Partial Integration)

**Status:** ✓ OPTIONAL (Deferred)

**Rationale:**
- Once MCPP stabilizes as a public feature (post-v6.0), create a new `docs/explanation/mcpp-architecture.md` that:
  - Summarizes the Chatman Equation and core concepts
  - Links to `_internal/research-mcpp/` for deep dives
  - References actual implemented modules (e.g., "See `src/rdf/` for ontology implementation")

**Defer this until:**
- MCPP is stabilized and ready for user adoption
- Core components (`RDF engine`, `Universal Port`, `POWL8`) are production-ready
- A clear user journey exists (e.g., "How to extend clap-noun-verb with MCPP control plane")

---

### (C) Do NOT Integrate into Main Docs Yet

**Status:** Not recommended (v5.6.0)

**Rationale:**
- Main documentation (`docs/reference/`, `docs/howto/`, `docs/tutorial/`) targets current users of clap-noun-verb as a CLI framework.
- MCPP research is aspirational: it describes a *future* evolution where clap-noun-verb becomes the execution substrate for UniverseOS.
- Integrating now would confuse users and overstate feature maturity.

---

## Archive Context

**Discovery Date:** v5.6.0 (minimalist-refactor-final branch)

**Related Documents:**
- `docs/MASTER_RESEARCH_PLAN.md` — Roadmap for OSTAR (research) ↔ MCPP (production) closure
- `docs/ARCHITECTURAL_CONTINUITY.md` — Governance invariants and safety proofs for MCPP transition
- `docs/COMBINATORIAL_MAP.md` — Feature-capability graph (includes MCPP components)

**Future Review Points:**
- v6.0: Assess readiness to stabilize MCPP and link from `docs/explanation/`
- v7.0: Evaluate extraction of MCPP into standalone system
- Ongoing: Use these documents to validate architectural decisions during frontier feature development

---

## Recommended Next Steps

1. **Move to `_internal/` for safe archival** (clears noise from main `docs/mcpp/`)
2. **Audit references** in MASTER_RESEARCH_PLAN.md and ARCHITECTURAL_CONTINUITY.md; update links if moved
3. **Tag in commit message** the research epoch (v5.6.0 foundation)
4. **Revisit at v6.0 feature lock** to determine if MCPP is ready for user-facing documentation
