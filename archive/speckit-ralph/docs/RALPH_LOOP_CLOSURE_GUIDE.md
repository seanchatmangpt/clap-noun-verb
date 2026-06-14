# Ralph Loop Closure: Implementation Guide

This guide outlines the finalized architecture for the Ralph loop, closing the gap between MCPP plan emission and unibit kinetic execution.

## 1. Loop Architecture
The Ralph loop follows a strict epistemic and kinetic sequence:
1. **Emission (`speckit-ralph run`)**: Generates a machine-readable `RalphPlan` JSON, defining the atomic steps for the swarm.
2. **Diagnosis (`mcpp doctor`)**: Validates the structural and epistemic integrity of the plan against the current workspace state.
3. **Receipting (`mcpp receipt`)**: Wraps the validated plan in a signed MCPP receipt (via `ggen`) to establish immutable provenance.
4. **Verification (`mcpp chain verify`)**: Ensures the signed receipt is valid within the broader swarm ledger.
5. **Advancement**: Updates the state to signify plan kinetic readiness.

## 2. Execution Protocol (`scripts/sr_loop.sh`)
The automation script orchestrates these steps as follows:

```bash
#!/bin/bash
set -e
# 1. Emission
cargo run -p speckit-ralph -- main run --goal "Implement Ralph Loop"
# 2. Validation
cargo run -- doctor
# 3. Receipting
cargo run -- receipt create
cargo run -- receipt sign
# 4. Verification
cargo run -- receipt verify
cargo run -- chain verify
# 5. State Transition
sed -i '' 's/phase: none/phase: plan/g' .chatmangpt/state.yaml
```

## 3. Epistemic Requirements
*   **RalphPlan Specification**: Must contain `id`, `goal`, `steps`, and `status`.
*   **Doctor Diagnosis**: The `doctor` command must verify field presence and structural validity before any signature is applied.
*   **Audit Trail**: Every loop closure must be logged in `PORTFOLIO_RALPH_LOOP_RECEIPT.md` and hashed into `EVIDENCE.ralph_loop_closure.hash`.

## 4. Troubleshooting
*   **Duplicate Commands**: If `mcpp` panics, verify that `playground` or other workspace members do not define overlapping command structures (e.g., duplicate `doctor` definitions).
*   **State Drift**: If the loop fails, the state in `.chatmangpt/state.yaml` must be manually reset to `phase: none` to prevent cascading errors in subsequent runs.
