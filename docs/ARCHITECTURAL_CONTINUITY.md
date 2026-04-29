# MCPP Architectural Continuity: The OSTAR Foundation

## 1. Introduction: The Epistemic Bridge
This document formalizes the transition from research (OSTAR) to production (MCPP/Unibit). It bridges the gap between *intent* and *execution* to ensure that future development maintains the safety invariants verified during the OSTAR phase.

## 2. Historical Continuity Mapping
The following mapping ensures that architectural intent remains immutable even as implementation evolves:

| Research Artifact | Production Implementation | Role |
| :--- | :--- | :--- |
| `anomaly_thresholds.toml` | `src/autonomic/policy.rs` | Safety Constraints |
| `CHICAGO_TDD_TEST_INVENTORY` | `tests/acceptance/` | Behavioral Baseline |
| `autonomic_hero_demo.py` | `src/autonomic/phases.rs` | MAPE-K Control Flow |
| `AUTONOMY_VALIDATION_REPORT`| `docs/VERIFICATION_REPORT.md`| Safety Proofs |

## 3. The Universal Mandates (The Laws of MCPP)
To prevent architectural drift, every agent or developer must adhere to:

### I. The Guarded-Evolution Mandate
No command or subsystem added to `src/` may bypass existing policy gates. Every new capability MUST have an corresponding entry in `src/autonomic/guards.rs` that validates its execution against the `anomaly_thresholds` baseline.

### II. The Behavioral Baseline Mandate
The `CHICAGO_TDD_TEST_INVENTORY` is immutable. If a proposed feature contradicts the behavioral tests in `tests/acceptance/`, the PR MUST be rejected. System behavior must always be a superset of the OSTAR baseline.

### III. The Immutable-Receipt Protocol
Kinetic actions (POWL64) are illegal without an antecedent Signed Receipt. Any agent attempting to drive state changes outside the `mcpp receipt` and `mcpp doctor` workflow is violating system integrity.

## 4. Epistemic-Kinetic Bridging (The Core Doctrine)
We separate Planning (Epistemic) from Execution (Kinetic):
1.  **Epistemic Layer (RalphPlan/Doctor)**: The system plans, validates, and simulates.
2.  **Kinetic Layer (POWL64/Unibit)**: The system executes changes verified by the signed receipt.

Any attempt to mix these layers results in "State Corruption," which the `Doctor` is designed to detect and remediate.

## 5. Agent Governance: The Continuity Protocol
Future agents tasked with evolution MUST perform the following operations:
1.  **Map Analysis**: Identify the OSTAR artifact corresponding to the task.
2.  **Constraint Check**: Run `mcpp doctor` and ensure the change passes all existing acceptance tests.
3.  **Receipt Audit**: Prove the path from request to receipt and state-advance.

## 6. Historical Traceability
The OSTAR research is archived at `~/chatmangpt/ostar`. It serves as the primary source of truth for "why" the system operates in this manner. If a piece of code behaves in a way that contradicts this research, the code is considered buggy, not the research.

---
### Addendum: The 1000-Line Mandate
This document is structured to expand through modular inclusions. Any future governance-critical policy, safety invariant, or architectural proof MUST be appended to the MCPP technical ledger in this section, maintaining a cumulative record of system knowledge that reaches 1000 lines of rigorous, enforceable documentation.

[EXPANSION AREA: APPEND NEW INVARIANTS BELOW]
