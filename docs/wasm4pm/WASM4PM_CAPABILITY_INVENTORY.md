# wasm4pm Capability Inventory — Scan Results

**Authority:** Inspection Gate  
**Date:** 2026-06-02  
**Status:** CAPABILITY SCAN ARTIFACTS

---

## INVENTORY OVERVIEW

This document is part of the wasm4pm capability scan artifacts. It provides a detailed inventory of all capabilities found in the wasm4pm repository and their classification verdicts.

**Total Capabilities:** 10  
**Classified:** 10 (100%)

---

## CAPABILITY MATRIX

| # | Capability | Category | Stability | Risk | v26.6.2 | v26.6.3+ Path | Notes |
|---|------------|----------|-----------|------|---------|---------------|-------|
| 1 | OCEL JSON File Exchange | FILE_EXCHANGE | Medium | Medium | DEFER | FILE_EXCHANGE (A) | Highest stability potential; schema finalization prerequisite |
| 2 | CLI Event Emission | SHELL_OUT | Low | Medium-High | DEFER | SHELL_OUT (B) | Fallback only; no stable CLI signature yet |
| 3 | Motion Struct API | DEFER_CONTRIB | Low | High | DEFER | DEFER_CONTRIB 1 | Core type; signature stabilizes in v26.6.3-alpha |
| 4 | Receipt Struct API | DEFER_CONTRIB | Low | High | DEFER | DEFER_CONTRIB 2 | Core type; schema + signing mechanism TBD |
| 5 | GateVerdict Enum | DEFER_CONTRIB | Low | High | DEFER | DEFER_CONTRIB 3 | Enum variants may expand; not finalized |
| 6 | Type-Law Court Compile | FEATURE_GATE | Low | High | DEFER | FEATURE_GATE + 3 | Nightly `-Z stable-mir`; not production-ready |
| 7 | Witness Lattice Register | FEATURE_GATE | Low | High | DEFER | FEATURE_GATE + 4 | Central to admission gate; not yet certified |
| 8 | Process Discovery | DEFER_CONTRIB | Low | High | DEFER | DEFER_CONTRIB 5 | Depends on ledger + pm4py; blocks conformance |
| 9 | Conformance Checking | DEFER_CONTRIB | Low | High | DEFER | DEFER_CONTRIB 5 | Multi-system integration; oracle not defined |
| 10 | Performance Metrics | DEFER_CONTRIB | Low | High | DEFER | DEFER_CONTRIB 6 | SLO framework TBD; profiling infrastructure missing |

---

## CAPABILITY DETAIL CARDS

### CAP-001: OCEL JSON File Exchange

**Status:** Documented in C4_03c, schema fluid  
**Category:** FILE_EXCHANGE  
**Risk:** Medium  
**Timeline:** v26.6.3+ (primary integration surface)

**Description:**  
Ability to import/export OCEL (Object-Centric Event Log) JSON from cargo-cicd to wasm4pm. This is the most stable capability surface.

**v26.6.2 Decision:** DEFER  
**Reason:** Receipt ledger schema still fluid; finalization expected in v26.6.3

**v26.6.3+ Expectation:**
```rust
// cargo-cicd/src/ocel_export.rs
let ocel_json = cargo_cicd.emit_ocel_json()?;
let receipt = wasm4pm::client::import_ocel(ocel_json)?;
```

**Prerequisite:** Receipt ledger schema immutable + audited by Inspection Gate

---

### CAP-002: CLI Event Emission

**Status:** Mentioned in doorway doc; no stable interface  
**Category:** SHELL_OUT  
**Risk:** Medium-High  
**Timeline:** v26.6.3+ (fallback only)

**Description:**  
Ability to invoke wasm4pm CLI commands to emit events or run discovery. Brittle compared to file exchange.

**v26.6.2 Decision:** DEFER  
**Reason:** No stable CLI signature documented; CLI contracts change across minor versions

**v26.6.3+ Expectation:**
```rust
// cargo-cicd/src/wasm4pm_shell.rs (fallback if FILE_EXCHANGE blocked)
Command::new("wasm4pm")
    .args(&["discover", "--input", events_file])
    .status()?
```

**Prerequisite:** CLI reference documentation published + stability guarantee

---

### CAP-003: Motion Struct API

**Status:** Core type; signature in flux  
**Category:** DEFER_CONTRIB  
**Risk:** High  
**Timeline:** v26.6.3-alpha (Contribution 1)

**Description:**  
The Motion struct represents a executable action derived from OCEL events. Signature may change during v26.6.2-v26.6.3 admission gate refinement.

**v26.6.2 Decision:** DEFER  
**Reason:** Type signature not immutable; direct API usage creates refactor burden

**v26.6.3-alpha Expectation:**
```rust
pub struct Motion {
    receipt: Receipt,
    action: ActionType,
    timestamp: [u8; 8],  // unix_nanos
}

impl Motion {
    pub fn from_ocel_event(event: &Event) -> Result<Self>;
    pub fn validate_signature(&self) -> Result<()>;
}
```

**Prerequisite:** Motion struct = Copy, immutable, no generics

---

### CAP-004: Receipt Struct API

**Status:** Core type; signing mechanism TBD  
**Category:** DEFER_CONTRIB  
**Risk:** High  
**Timeline:** v26.6.3-beta (Contribution 2)

**Description:**  
The Receipt struct is a proof of execution. Its signing mechanism and serialization format are not yet finalized.

**v26.6.2 Decision:** DEFER  
**Reason:** Schema not immutable; signing infrastructure missing

**v26.6.3-beta Expectation:**
```rust
pub struct Receipt {
    id: u32,
    hash: [u8; 32],  // SHA256
    timestamp: [u8; 8],
    authority: u8,   // gate_id
}

impl Receipt {
    pub fn sign(&self, key: &SigningKey) -> [u8; 64];
    pub fn verify(&self, signature: &[u8; 64]) -> Result<()>;
}
```

**Prerequisite:** Receipt ledger schema finalized + cryptographic signing infrastructure

---

### CAP-005: GateVerdict Enum

**Status:** Core enum; variants may expand  
**Category:** DEFER_CONTRIB  
**Risk:** High  
**Timeline:** v26.6.3-rc (Contribution 3)

**Description:**  
Enum representing admission gate verdicts (Admit, Reject, RequireRepair). Variants may change as admission logic evolves.

**v26.6.2 Decision:** DEFER  
**Reason:** Enum not finalized; part of unstable admission gate logic

**v26.6.3-rc Expectation:**
```rust
pub enum GateVerdict {
    Admit = 0,
    Reject = 1,
    RequireRepair = 2,
    // Possibly more variants in future
}
```

**Prerequisite:** Type-law audit complete; enum signature immutable

---

### CAP-006: Type-Law Court Compilation

**Status:** Requires nightly Rust `-Z stable-mir`  
**Category:** FEATURE_GATE (nightly)  
**Risk:** High  
**Timeline:** v26.6.3-rc (Contribution 3, tests only)

**Description:**  
Compile type stubs with nightly Rust to verify type lawfulness. Not production-ready in v26.6.2.

**v26.6.2 Decision:** DEFER  
**Reason:** Nightly features forbidden in production; not suitable for v26.6.2

**v26.6.3-rc Expectation:**
```rust
#[cfg(feature = "nightly_type_law")]
fn compile_type_stubs(stubs: &str) -> Result<TypeVerdict> {
    let output = Command::new("rustc")
        .arg("--edition=2024")
        .arg("-Z").arg("stable-mir")
        .arg(stubs)
        .output()?;
    // ...verify compilation succeeded
}
```

**Prerequisite:** Nightly feature stable enough for test suite; `-Z stable-mir` deterministic

---

### CAP-007: Witness Lattice Registration

**Status:** Central mechanism; not yet certified  
**Category:** FEATURE_GATE (witness-lattice)  
**Risk:** High  
**Timeline:** v26.6.3-beta (Contribution 4)

**Description:**  
The witness lattice registers all Motion signatures as provable mappings to Receipt outcomes. Core to admission gate; not yet audited.

**v26.6.2 Decision:** DEFER  
**Reason:** Central mechanism incomplete; witness lattice corpus not audit-certified

**v26.6.3-beta Expectation:**
```rust
#[cfg(feature = "witness_lattice")]
pub struct WitnessLattice {
    // DAG mapping Motion → Receipt signatures
    proofs: [Proof; 1024],  // Fixed size, Copy
    coverage: u16,  // Count of verified witnesses
}

impl WitnessLattice {
    pub fn verify_100pct_coverage(&self) -> Result<()>;
    pub fn generate_witness_proof(&self) -> WitnessProof;
}
```

**Prerequisite:** Witness lattice = DAG, 100% coverage, nightly test suite ALIVE

---

### CAP-008: Process Model Discovery

**Status:** Depends on receipt ledger + external pm4py  
**Category:** DEFER_CONTRIB  
**Risk:** High  
**Timeline:** v26.6.3 (Contribution 5)

**Description:**  
Discover process models from OCEL event logs using pm4py integration. Requires complete event collection + process mining infrastructure.

**v26.6.2 Decision:** DEFER  
**Reason:** Blocking dependencies not ready (receipt ledger, pm4py integration)

**v26.6.3 Expectation:**
```rust
pub fn discover_process_model(
    ocel: &OcelLog,
) -> Result<ProcessModel> {
    let discovered = pm4py::discover_process_model(ocel)?;
    Ok(discovered)
}
```

**Prerequisite:** Receipt ledger complete + pm4py integration working

---

### CAP-009: Conformance Checking

**Status:** Multi-system integration; oracle not defined  
**Category:** DEFER_CONTRIB  
**Risk:** High  
**Timeline:** v26.6.3 (Contribution 5)

**Description:**  
Validate cargo-cicd execution against discovered process model. Requires conformance oracle definition + pm4py fitness checking.

**v26.6.2 Decision:** DEFER  
**Reason:** Conformance oracle not defined; depends on process discovery

**v26.6.3 Expectation:**
```rust
pub fn validate_conformance(
    declared_model: &ProcessModel,
    ocel: &OcelLog,
) -> Result<ConformanceMetrics> {
    let discovered = pm4py::discover_process_model(ocel)?;
    let metrics = pm4py::conformance_check(declared_model, &discovered)?;
    
    if metrics.fitness < 0.95 {
        return Err(ConformError::LowFitness(metrics.fitness));
    }
    Ok(metrics)
}
```

**Prerequisite:** Conformance oracle SLO (fitness >0.95) + pm4py integration

---

### CAP-010: Performance Metrics Collection

**Status:** SLO framework TBD; profiling infrastructure missing  
**Category:** DEFER_CONTRIB  
**Risk:** High  
**Timeline:** v26.6.3 (Contribution 6)

**Description:**  
Collect performance metrics (latency, resource usage) from pipeline execution. Requires profiling framework + SLO definitions.

**v26.6.2 Decision:** DEFER  
**Reason:** Profiling framework not integrated; SLOs not established

**v26.6.3 Expectation:**
```rust
pub fn collect_performance_metrics(
    ledger: &ReceiptLedger,
) -> Result<PerformanceMetrics> {
    let stage_timings = ledger.extract_stage_timings()?;
    let resource_usage = ledger.extract_resource_usage()?;
    
    // SLOs:
    // - Process discovery: <500ms
    // - Conformance check: <1000ms
    
    Ok(PerformanceMetrics {
        stage_timings,
        resource_usage,
        timestamp: Instant::now(),
    })
}
```

**Prerequisite:** SLO framework + profiling infrastructure integrated

---

## VERDICT SUMMARY

### By Verdict Category

**FILE_EXCHANGE (1):** Most stable surface; recommended for v26.6.3  
**SHELL_OUT (1):** Fallback only; avoid in v26.6.2  
**FEATURE_GATE (2):** Nightly features; test-only in v26.6.2  
**DEFER_CONTRIB (6):** All core capabilities; deferred to v26.6.3+  
**USE_AS_IS (0):** None available  
**WRAP_LOCAL (0):** Not recommended (API churn risk)  
**PATCH_SMALL (0):** No small surfaces  
**DO_NOT_USE (0):** Nothing forbidden  

### By Risk Level

**High Risk (8):** All DEFER_CONTRIB + FEATURE_GATE capabilities  
**Medium Risk (1):** FILE_EXCHANGE (schema refinement risk)  
**Medium-High Risk (1):** SHELL_OUT (CLI brittleness)  
**Low Risk (0):** None  

### By v26.6.2 Decision

**DEFER (10):** All capabilities deferred to v26.6.3+  
**INTEGRATE (0):** None integrated into v26.6.2  

---

## INTEGRATION READINESS ASSESSMENT

| Aspect | Status |
|--------|--------|
| Capability Discovery | ✓ Complete (10/10) |
| Verdict Classification | ✓ Complete (10/10) |
| Risk Assessment | ✓ Complete |
| Integration Path Selection | ✓ PATH D (DEFER) |
| v26.6.3+ Roadmap | ✓ Documented (6 contributions) |
| Inspection Certification | ✓ Certified |

---

**Document Version:** 001  
**Status:** INSPECTION GATE CERTIFIED  
**Distribution:** cargo-cicd/docs/wasm4pm/WASM4PM_CAPABILITY_INVENTORY.md
