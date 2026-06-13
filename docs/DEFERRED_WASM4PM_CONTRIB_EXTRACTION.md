# Deferred wasm4pm Contributions — v26.6.3+ Roadmap

**Authority:** Inspection Gate (Process Intelligence Core)  
**Date Composed:** 2026-06-02  
**Status:** CONTRIBUTOR ROADMAP FOR v26.6.3+

---

## MISSION

This document maps all **DEFER_CONTRIB** capabilities that are **blocked on wasm4pm-compat** for v26.6.2. Each capability is:

1. **Named** — Specific contribution
2. **Blocked By** — What must be delivered first
3. **Expected Behavior** — What v26.6.3+ will do
4. **Extraction Requirements** — What contributors must do
5. **Acceptance Criteria** — How we know it's done

**Target Release:** v26.6.3 (estimated 2026-07-15)

---

## DEFERRED CONTRIBUTION 1: Motion Execution from OCEL Events

**Capability Name:** `wasm4pm::execute_motion_from_ocel`

**Category:** DEFER_CONTRIB

**Why Deferred:**
- Requires stable Motion struct from wasm4pm-compat
- Requires stable Receipt struct from wasm4pm-compat
- Requires nightly Rust type-law court (unstable in v26.6.2)
- Witness lattice registration incomplete

**Blocked On:**
1. wasm4pm-compat type-law court nightly test suite = ALIVE ✓
2. Motion struct definition = immutable signature
3. Receipt struct definition = immutable signature
4. Witness lattice = audit-certified

**Expected v26.6.3+ Behavior:**

```rust
// cargo-cicd/src/integrations/wasm4pm_future.rs
use wasm4pm::{Motion, Receipt, GateVerdict};
use ocel_core::Event;

/// Execute a motion derived from OCEL event log
pub fn execute_motion_from_ocel(
    event: &Event,
    audit_trail: &OcelAuditTrail,
) -> Result<Receipt, WasmError> {
    // 1. Validate event against schema
    let motion = Motion::from_ocel_event(event)?;

    // 2. Validate motion against witness lattice
    wasm4pm::validate_motion_signature(&motion)?;

    // 3. Execute motion in wasm4pm
    let receipt = wasm4pm::execute_motion(&motion)?;

    // 4. Emit receipt to ledger
    audit_trail.record_execution_receipt(&receipt)?;

    Ok(receipt)
}
```

**Extraction Requirements:**
1. Stabilize Motion struct in wasm4pm-compat
   - Location: `wasm4pm-compat/src/motion.rs`
   - Requirement: Copy, bounded size, no generics
   - Test: `test_motion_struct_is_copy` passes

2. Stabilize Receipt struct in wasm4pm-compat
   - Location: `wasm4pm-compat/src/receipt.rs`
   - Requirement: Copy, bounded size, immutable hash
   - Test: `test_receipt_struct_serializable` passes

3. Audit witness lattice registration
   - Location: `wasm4pm-compat/src/witness_lattice.rs`
   - Requirement: All Motion signatures registered
   - Test: `test_witness_lattice_100pct_coverage` passes

4. Compile nightly type signatures
   - Requirement: `rustc -Z stable-mir -C opt-level=3 types_stub.rs` succeeds
   - Test: `test_nightly_type_law_court_passes` passes

5. Implement OCEL → Motion converter
   - Location: `cargo-cicd/src/integrations/ocel_to_motion.rs`
   - Input: OCEL Event
   - Output: Motion struct (validated)
   - Test: Round-trip: OCEL event → Motion → Receipt → OCEL event matches

**Acceptance Criteria:**
- [ ] Motion struct is Copy and derives Serialize/Deserialize
- [ ] Motion struct is immutable (no mut fields)
- [ ] Receipt struct is Copy and has immutable hash
- [ ] Witness lattice has 100% Motion signature coverage
- [ ] Nightly type-law court passes with zero warnings
- [ ] OCEL → Motion converter round-trips correctly
- [ ] Integration tests with wasm4pm execute without deadlock/panic
- [ ] Process conformance oracle (pm4py) validates execution results

**Version Estimate:** v26.6.3-alpha (2026-06-30)

---

## DEFERRED CONTRIBUTION 2: Receipt Issuance & Ledger Maintenance

**Capability Name:** `wasm4pm::issue_receipt_to_ledger`

**Category:** DEFER_CONTRIB

**Why Deferred:**
- Receipt ledger schema not finalized (may change during v26.6.3 admission gate refinement)
- Receipt signing mechanism depends on type-law court certification
- Ledger immutability guarantees not yet audited

**Blocked On:**
1. Receipt struct immutable signature (see Contribution 1)
2. Receipt ledger schema = finalized and immutable
3. Cryptographic signing infrastructure (TBD)
4. Inspection Gate ledger certification

**Expected v26.6.3+ Behavior:**

```rust
// cargo-cicd/src/integrations/receipt_ledger.rs
use wasm4pm::Receipt;
use ocel_core::ReceiptLedger;

/// Issue receipt to immutable ledger
pub fn issue_receipt_to_ledger(
    receipt: &Receipt,
    ledger: &mut ReceiptLedger,
    authority: &GateAuthority,
) -> Result<LedgerEntry, LedgerError> {
    // 1. Validate receipt is properly formed
    receipt.validate_signature()?;

    // 2. Create ledger entry with timestamp
    let entry = LedgerEntry {
        receipt: receipt.clone(),
        timestamp: Instant::now(),
        authority_signature: authority.sign(receipt)?,
    };

    // 3. Append to ledger (immutable)
    ledger.append_entry(&entry)?;

    // 4. Return ledger entry proof
    Ok(entry)
}
```

**Extraction Requirements:**
1. Finalize Receipt ledger schema
   - Location: `docs/RECEIPT_LEDGER_SCHEMA.md`
   - Requirement: OCEL JSON schema, timestamped entries, signature proof
   - Test: `test_receipt_ledger_schema_valid` passes

2. Implement Receipt signing
   - Location: `wasm4pm-compat/src/receipt_signing.rs`
   - Requirement: Deterministic HMAC-SHA256 signature
   - Test: `test_receipt_signature_deterministic` passes

3. Implement Receipt ledger (immutable append-only)
   - Location: `cargo-cicd/src/receipt_ledger.rs`
   - Requirement: No mutation after append, cryptographic proof of order
   - Test: `test_ledger_immutability_enforced` passes

4. Implement ledger replay for conformance
   - Location: `cargo-cicd/src/ledger_conformance.rs`
   - Requirement: Replay receipt ledger and verify conformance with pm4py
   - Test: `test_ledger_replay_conforms_to_declared_process` passes

5. Implement authority signature validation
   - Requirement: Verify Inspection Gate signature on each ledger entry
   - Test: `test_authority_signature_verified` passes

**Acceptance Criteria:**
- [ ] Receipt ledger schema is published and immutable (Inspection Gate signed)
- [ ] Receipt signing is deterministic (same receipt → same signature)
- [ ] Receipt ledger append-only invariant enforced (no deletions, no mutations)
- [ ] Ledger entries include authority signatures
- [ ] Ledger replay produces identical process model (vs. declared model)
- [ ] Zero entries in ledger without valid authority signature
- [ ] Nightly test suite: `test_receipt_ledger_100pct_signed` passes

**Version Estimate:** v26.6.3-beta (2026-07-05)

---

## DEFERRED CONTRIBUTION 3: Type-Law Court Verdict Generation

**Capability Name:** `wasm4pm::generate_type_law_verdict`

**Category:** DEFER_CONTRIB

**Why Deferred:**
- Nightly Rust `-Z stable-mir` is unstable (not production-ready in v26.6.2)
- Type signatures still in flux during v26.6.2
- Compilation court not yet audited by Inspection Gate

**Blocked On:**
1. Type signatures stabilized (Contribution 1, Motion & Receipt structs)
2. Nightly Rust stable-mir stabilization (or Rust 1.85+ if promoted to stable)
3. Type-law court audit by Inspection Gate
4. Witness lattice proof-of-work completed

**Expected v26.6.3+ Behavior:**

```rust
// cargo-cicd/src/integrations/type_law_court.rs
use std::process::Command;

/// Compile type stubs with nightly Rust type law court
pub fn generate_type_law_verdict(
    type_stubs: &str,
    witness_lattice: &WitnessLattice,
) -> Result<TypeVerdict, TypeError> {
    // 1. Prepare nightly Rust compiler invocation
    let output = Command::new("rustc")
        .arg("--edition=2024")
        .arg("-Z").arg("stable-mir")
        .arg("-C").arg("opt-level=3")
        .arg(type_stubs)
        .output()?;

    if !output.status.success() {
        return Err(TypeError::CompilationFailed(
            String::from_utf8(output.stderr)?,
        ));
    }

    // 2. Verify witness lattice coverage
    witness_lattice.verify_100pct_coverage(type_stubs)?;

    // 3. Issue verdict (types are lawful)
    Ok(TypeVerdict {
        timestamp: Instant::now(),
        verdict: VerdictType::TypesLawful,
        witness_proof: witness_lattice.proof(),
    })
}
```

**Extraction Requirements:**
1. Define type-law court interface
   - Location: `wasm4pm-compat/src/type_law_court.rs`
   - Requirement: Accept type stubs, compile with stable-mir, return verdict
   - Test: `test_type_law_court_compilation_succeeds` passes

2. Implement verdict issuance
   - Location: `wasm4pm-compat/src/verdict.rs`
   - Requirement: Immutable TypeVerdict struct with timestamp and proof
   - Test: `test_verdict_struct_immutable` passes

3. Audit nightly Rust stable-mir feature
   - Requirement: Verify `-Z stable-mir` is stable enough for production
   - Test: `test_stable_mir_deterministic` (run 100x, same output every time)

4. Create deterministic type stub generator
   - Location: `cargo-cicd/src/type_stub_generator.rs`
   - Requirement: Same type signature → same stub code (hash-deterministic)
   - Test: `test_type_stub_generator_deterministic` passes

5. Implement verdict ledger entry
   - Requirement: Record each type-law court verdict with timestamp and proof
   - Test: `test_verdict_ledger_100pct_signed` passes

**Acceptance Criteria:**
- [ ] Type-law court compiles with nightly `-Z stable-mir` without warnings
- [ ] Verdict generation is deterministic (same types → same verdict hash)
- [ ] Verdict includes proof of witness lattice coverage
- [ ] Verdict timestamp is immutable (signed by Inspection Gate)
- [ ] Nightly test suite runs 24/7 (automated, gated to `#[cfg(nightly)]`)
- [ ] All verdicts recorded in immutable verdict ledger
- [ ] Zero verdicts issued without witness lattice proof

**Version Estimate:** v26.6.3-rc (2026-07-10)

---

## DEFERRED CONTRIBUTION 4: Witness Lattice Certification

**Capability Name:** `wasm4pm::certify_witness_lattice`

**Category:** DEFER_CONTRIB

**Why Deferred:**
- Witness lattice structure incomplete (Motion/Receipt signatures not finalized)
- Lattice audit trail not yet implemented
- Certification mechanism depends on type-law court (Contribution 3)

**Blocked On:**
1. Motion & Receipt struct signatures finalized (Contribution 1)
2. Type-law court audit passing (Contribution 3)
3. Witness lattice proof-of-work algorithm designed
4. Inspection Gate certification protocol finalized

**Expected v26.6.3+ Behavior:**

```rust
// cargo-cicd/src/integrations/witness_lattice.rs
use wasm4pm::{WitnessLattice, WitnessProof};

/// Certify witness lattice with Inspection Gate signature
pub fn certify_witness_lattice(
    lattice: &WitnessLattice,
    authority: &InspectionGate,
) -> Result<CertificationReceipt, CertError> {
    // 1. Verify 100% signature coverage
    let coverage = lattice.verify_coverage()?;
    if coverage != 1.0 {
        return Err(CertError::IncompleteCoverage(coverage));
    }

    // 2. Verify lattice structure (no cycles, no contradictions)
    lattice.verify_consistency()?;

    // 3. Generate proof-of-work witness
    let proof = lattice.generate_witness_proof()?;

    // 4. Sign certification by Inspection Gate
    let receipt = authority.sign_certification(&proof)?;

    Ok(receipt)
}
```

**Extraction Requirements:**
1. Design witness lattice structure
   - Location: `docs/WITNESS_LATTICE_SPEC.md`
   - Requirement: DAG structure mapping all Motion signatures to Receipt signatures
   - Test: `test_witness_lattice_is_dag` passes

2. Implement witness lattice as immutable data structure
   - Location: `wasm4pm-compat/src/witness_lattice.rs`
   - Requirement: Fixed-size array (max 1024 signatures), Copy derives
   - Test: `test_witness_lattice_copy` passes

3. Implement coverage verification
   - Requirement: Verify 100% of Motion signatures have corresponding Receipt witnesses
   - Test: `test_witness_lattice_coverage_100pct` passes

4. Implement consistency checking
   - Requirement: No cycles in witness DAG; all predicates in canon.ttl
   - Test: `test_witness_lattice_consistent` passes

5. Implement certification receipt
   - Location: `wasm4pm-compat/src/certification_receipt.rs`
   - Requirement: Immutable proof with Inspection Gate signature and timestamp
   - Test: `test_certification_receipt_immutable` passes

**Acceptance Criteria:**
- [ ] Witness lattice structure is DAG (no cycles)
- [ ] 100% of signatures covered by witnesses
- [ ] Lattice is Copy and serializable
- [ ] Coverage verified by automated nightly tests
- [ ] Certification receipt signed by Inspection Gate
- [ ] Lattice immutable after certification
- [ ] Zero new signatures added after certification (enforced by type system)

**Version Estimate:** v26.6.3-beta (2026-07-05)

---

## DEFERRED CONTRIBUTION 5: Cross-System Conformance Validation

**Capability Name:** `wasm4pm::validate_conformance_with_pm4py`

**Category:** DEFER_CONTRIB

**Why Deferred:**
- Requires complete process mining output from cargo-cicd
- Requires receipt ledger replay (Contribution 2)
- Requires pm4py integration (depends on Truex + Blue River Dam)

**Blocked On:**
1. Receipt ledger populated (Contribution 2)
2. Process event collection complete in cargo-cicd
3. pm4py process discovery working (external dependency)
4. Conformance oracle benchmarks established

**Expected v26.6.3+ Behavior:**

```rust
// cargo-cicd/src/integrations/conformance_validation.rs
use wasm4pm::ProcessModel;
use pm4py::DiscoveredModel;

/// Validate cargo-cicd execution against discovered process model
pub fn validate_conformance_with_pm4py(
    receipt_ledger: &ReceiptLedger,
    declared_model: &ProcessModel,
) -> Result<ConformanceMetrics, ConformError> {
    // 1. Extract OCEL from receipt ledger
    let ocel_log = receipt_ledger.to_ocel_log()?;

    // 2. Discover actual process model
    let discovered_model = pm4py::discover_process_model(&ocel_log)?;

    // 3. Compare declared vs. discovered
    let metrics = pm4py::conformance_check(
        &declared_model,
        &discovered_model,
        &ocel_log,
    )?;

    // 4. Verify fitness > 0.95
    if metrics.fitness < 0.95 {
        return Err(ConformError::LowFitness(metrics.fitness));
    }

    Ok(metrics)
}
```

**Extraction Requirements:**
1. Implement OCEL export from receipt ledger
   - Location: `cargo-cicd/src/ledger_to_ocel.rs`
   - Requirement: Receipt ledger → OCEL JSON (pm4py-compatible)
   - Test: `test_ocel_export_valid` passes

2. Integrate pm4py process discovery
   - Location: `cargo-cicd/src/pm4py_integration.rs`
   - Requirement: Invoke pm4py; parse discovered model
   - Test: `test_pm4py_discovery_convergence` (process discovery is deterministic)

3. Implement conformance check
   - Requirement: Compare declared vs. discovered; measure fitness/precision
   - Test: `test_conformance_high_fitness` (fitness > 0.95)

4. Define SLOs for process mining
   - Requirement: Discovery latency <500ms, conformance check <1000ms
   - Test: `test_process_mining_slos_met` passes

5. Emit conformance metrics to artifact store
   - Requirement: Fitness, precision, generalization, simplicity scores
   - Test: `test_metrics_all_populated` passes

**Acceptance Criteria:**
- [ ] OCEL export from ledger is pm4py-compatible
- [ ] Process discovery runs in <500ms
- [ ] Conformance check runs in <1000ms
- [ ] Fitness metric > 0.95 on all test cases
- [ ] Declared process model is always discovered (process conforms)
- [ ] Zero false negatives in conformance detection
- [ ] Metrics recorded in immutable artifact ledger

**Version Estimate:** v26.6.3 (2026-07-15)

---

## DEFERRED CONTRIBUTION 6: Performance Metrics Collection

**Capability Name:** `wasm4pm::collect_performance_metrics`

**Category:** DEFER_CONTRIB

**Why Deferred:**
- Requires complete cargo-cicd event collection
- Requires OCEL output from pipeline
- Requires profiling infrastructure (TBD)

**Blocked On:**
1. Receipt issuance complete (Contribution 2)
2. Conformance validation working (Contribution 5)
3. Performance profiling library integrated (external dependency)
4. SLO framework established

**Expected v26.6.3+ Behavior:**

```rust
// cargo-cicd/src/integrations/performance_metrics.rs
use wasm4pm::PerformanceMetrics;

/// Collect performance metrics from executed pipeline
pub fn collect_performance_metrics(
    receipt_ledger: &ReceiptLedger,
    declared_model: &ProcessModel,
) -> Result<PerformanceMetrics, MetricsError> {
    // 1. Extract timing data from receipts
    let stage_timings = receipt_ledger.extract_stage_timings()?;

    // 2. Calculate CPU cycles / memory usage per stage
    let resource_usage = receipt_ledger.extract_resource_usage()?;

    // 3. Measure process mining latency
    let discovery_latency = measure_discovery_latency(&declared_model)?;
    let conformance_latency = measure_conformance_latency(&declared_model)?;

    // 4. Compile metrics
    Ok(PerformanceMetrics {
        stage_timings,
        resource_usage,
        discovery_latency,
        conformance_latency,
        timestamp: Instant::now(),
    })
}
```

**Extraction Requirements:**
1. Add performance counters to receipt struct
   - Location: `wasm4pm-compat/src/receipt_with_metrics.rs`
   - Requirement: CPU cycles, memory bytes, wall-clock time (Copy types)
   - Test: `test_receipt_metrics_deterministic` passes

2. Implement stage timing extraction
   - Location: `cargo-cicd/src/metrics/stage_timings.rs`
   - Requirement: Extract µ₁-µ₅ timing data from receipt ledger
   - Test: `test_stage_timings_correct_order` passes

3. Implement SLO tracking
   - Requirement: Discovery <500ms, conformance <1000ms
   - Test: `test_slos_reported` passes

4. Emit metrics to monitoring system
   - Requirement: JSON export; compatible with Prometheus/OpenTelemetry
   - Test: `test_metrics_export_valid` passes

5. Generate performance dashboard
   - Requirement: Historical metrics; trend analysis; SLO violations flagged
   - Test: `test_dashboard_updates_hourly` passes

**Acceptance Criteria:**
- [ ] All stage timings collected (µ₁-µ₅)
- [ ] CPU cycles and memory usage recorded per stage
- [ ] Discovery latency <500ms (SLO met)
- [ ] Conformance latency <1000ms (SLO met)
- [ ] Metrics immutable after pipeline completion
- [ ] Metrics exported in standard format
- [ ] Performance regressions detected by CI/CD

**Version Estimate:** v26.6.3 (2026-07-15)

---

## CONTRIBUTOR ROADMAP SUMMARY

| Contribution | Owner | Blocked By | Expected Delivery | Acceptance Tests |
|--------------|-------|-----------|-------------------|------------------|
| 1. Motion Execution | wasm4pm-compat team | Motion/Receipt structs | v26.6.3-alpha | 5 tests + round-trip |
| 2. Receipt Ledger | cargo-cicd team | Receipt struct + schema | v26.6.3-beta | 7 tests + immutability |
| 3. Type-Law Court | wasm4pm-compat team | Type signatures + nightly | v26.6.3-rc | 5 tests + determinism |
| 4. Witness Lattice | wasm4pm-compat team | Structs + type-law court | v26.6.3-beta | 7 tests + 100% coverage |
| 5. Conformance Check | cargo-cicd + pm4py team | Ledger + discovery | v26.6.3 | 5 tests + high fitness |
| 6. Performance Metrics | cargo-cicd team | Ledger + metrics structs | v26.6.3 | 5 tests + SLO met |

**Critical Path:** 1 (Motion) → 2 (Ledger) → 5 (Conformance)

**Parallel Tracks:** 3 (Type-Law) + 4 (Witness) can proceed in parallel with 1/2/5.

---

## CONTRIBUTOR EXTRACTION CHECKLIST (for v26.6.3)

Before claiming a contribution, verify:

### Phase 1: Prerequisites (Before Starting)
- [ ] All blocked-on dependencies are ALIVE (Inspection Gate certified)
- [ ] Acceptance criteria are understood and testable
- [ ] Test infrastructure exists (unit + integration tests)
- [ ] Documentation is clear (no ambiguity in specification)

### Phase 2: Implementation
- [ ] Code is written and passes unit tests
- [ ] Code is integrated with existing modules (no orphaned code)
- [ ] Error handling is complete (no unwrap/expect/panic)
- [ ] Performance SLOs are met (if applicable)

### Phase 3: Testing & Certification
- [ ] All acceptance tests pass
- [ ] Integration tests pass (with dependent modules)
- [ ] Nightly test suite passes (if applicable)
- [ ] Code review by module owner completed

### Phase 4: Documentation & Receipt
- [ ] Design document updated
- [ ] API documentation complete
- [ ] CHANGELOG.md entry added
- [ ] Receipt issued by module owner

---

## VERSION TIMELINE

| Version | Date Est. | Contributions | Status |
|---------|-----------|---------------|--------|
| v26.6.2 | 2026-06-02 | None (deferred) | ✓ RELEASED |
| v26.6.3-alpha | 2026-06-30 | 1 (Motion Execution) | In Planning |
| v26.6.3-beta | 2026-07-05 | 2 (Ledger), 4 (Witness) | In Planning |
| v26.6.3-rc | 2026-07-10 | 3 (Type-Law Court) | In Planning |
| v26.6.3 | 2026-07-15 | 5 (Conformance), 6 (Metrics) | In Planning |

---

## INSPECTION GATE AUTHORITY SIGN-OFF

**Authority:** Inspection Gate (Process Intelligence Core)

```
Inspection Gate Verdict:

This roadmap correctly identifies all wasm4pm contributions as DEFER_CONTRIB
for v26.6.3+. All contributions are blocked on wasm4pm-compat stabilization.
Prerequisites are documented. Acceptance criteria are testable. Critical path
is clear. Extraction requirements are specific and measurable.

Inspection Gate certifies this as SAFE FOR CONTRIBUTION in v26.6.3.

Signed (Inspection Gate Authority):
_________________________________
Process Intelligence Core Lead / CTO

Date: 2026-06-02

Attestation: This roadmap becomes IMMUTABLE upon signature. Contributions
must follow this roadmap exactly. No deviations without Inspection Gate
approval.
```

---

**Document Version:** 001  
**Status:** CONTRIBUTOR ROADMAP LOCKED FOR v26.6.3  
**Next Review:** 2026-06-15 (dependencies checkpoint)
