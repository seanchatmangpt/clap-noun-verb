# OSTAR-MCPP: Master Research and Loop Closure Plan

## 1. Executive Summary
This document defines the research and closure roadmap to unify OSTAR (Research) and MCPP (Production). It formalizes all open loops, their dependencies, and the precise motions required for closure.

## 2. Open Loop Inventory
| Loop ID | Target Domain | Status | Dependency |
| :--- | :--- | :--- | :--- |
| **LOOP-001** | `speckit-ralph` | Closed | N/A |
| **LOOP-002** | `unibit-powl64` | Open | Receipt Integrity Chain |
| **LOOP-003** | `ggen` | Open | Typed Turtle Specification |
| **LOOP-004** | `telemetry` | Open | Autonomic Feedback Integration |

## 3. The 12-Month Research Plan (Composition of Correct Documents)

### Phase 1: Substrate Hardening (Q1)
*   **Goal**: Establish immutable provenance for all kinetic actions.
*   **Documentation Source**: `docs/VERIFICATION_REPORT.md`
*   **Motion**: Finalize receipt verification for `unibit-powl64` (LOOP-002).

### Phase 2: Ontology Canonicalization (Q2)
*   **Goal**: Ensure `ggen` generates perfectly typed Rust code from Turtle ontologies.
*   **Documentation Source**: `examples/turtle-specs/README.md`
*   **Motion**: Resolve OBO Foundry/W3C schema mapping gaps (LOOP-003).

### Phase 3: Autonomic Feedback (Q3)
*   **Goal**: Enable MAPE-K loops to adjust parameters dynamically without re-emission.
*   **Documentation Source**: `AUTONOMIC.md`
*   **Motion**: Integrate `telemetry` profile stream into the `Governor` (LOOP-004).

## 4. Closing the Gaps: Operational Procedure
To close a loop, every agent/developer must execute:
1.  **Map**: Locate the target in the *Open Loop Inventory*.
2.  **Verify**: Ensure the source documentation (e.g., `AUTONOMIC.md`) is current.
3.  **Execute**: Run the corresponding closure script (e.g., `scripts/sr_loop.sh`).
4.  **Audit**: Append the result to `PORTFOLIO_RALPH_LOOP_RECEIPT.md`.

## 5. Architectural Integrity Protocol
To maintain the 1000-line integrity of this technical ledger:
*   Any closure that fails `mcpp doctor` is a **Violation**.
*   Any loop closed without a `Receipt` is a **Security Risk**.
*   Any state advance without corresponding `PORTFOLIO` entry is **Invalid**.

---

### Expansion: Cumulative Invariant Ledger
[EXPANSION AREA: APPEND NEW INVARIANTS OR LOOP CLOSURES BELOW]

*   **Invariant 001**: Every kinetic action must have an antecedent receipt hash.
*   **Invariant 002**: Epistemic outputs (plans/receipts) must be validated by a Doctor node before kinetic triggers.
*   **Invariant 003**: All autonomic feedback loops must be logged in the centralized telemetry store.
