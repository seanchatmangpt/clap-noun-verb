# wasm4pm Integration Recommendation for v26.6.2

**Authority:** Water Gate (Delivery Lead)  
**Gate Authority:** Secondary: Inspection Gate  
**Date Composed:** 2026-06-02  
**Status:** INTEGRATION PATH DECISION

---

## EXECUTIVE SUMMARY

**Recommended Integration Path: PATH D (DEFER)**

v26.6.2 does **not integrate wasm4pm** into cargo-cicd. The integration path is **deferred to v26.6.3+** with a clear roadmap for safe, staged adoption.

**Rationale:**
- wasm4pm-compat (admission gate) is still in development
- Type-law doorway is not yet stable
- Nightly test suite is incomplete (witness lattice unstable)
- Integration at this stage would couple cargo-cicd to a moving target
- Risk is LOW (defer-cost) vs. MEDIUM-HIGH (early integration risk)

---

## CAPABILITY VERDICT ANALYSIS

### Verdict Tallies (Based on Architectural Doorway Assessment)

| Verdict | Count | Capabilities |
|---------|-------|--------------|
| **USE_AS_IS** | 0 | None (all core APIs in flux) |
| **SHELL_OUT** | 1 | CLI event emission (if stable) |
| **FILE_EXCHANGE** | 1 | OCEL JSON import/export (if stable) |
| **FEATURE_GATE** | 2 | Type-law court compile (nightly feature), witness lattice registration |
| **WRAP_LOCAL** | 0 | No thin adapters recommended |
| **PATCH_SMALL** | 0 | No surface is small enough |
| **DEFER_CONTRIB** | 6 | See detailed list below |
| **DO_NOT_USE** | 0 | Nothing explicitly forbidden |

**Total Assessed Capabilities:** 10

### Stability Assessment by Surface

#### File Exchange (OCEL JSON)
- **Status:** Documented in C4_03c_WASM4PM_DOORWAY_002.md, but receipt ledger format is still fluid
- **Verdict:** FILE_EXCHANGE (conditional on schema finalization)
- **Risk:** Medium — JSON schema may change in v26.6.3
- **Note:** This surface has the highest stability potential

#### Shell-Out (CLI Commands)
- **Status:** Mentioned in doorway doc but no stable CLI signature documented
- **Verdict:** SHELL_OUT (conditional on CLI documentation)
- **Risk:** Medium-High — CLI may change; not a stable surface
- **Note:** Avoid for v26.6.2

#### Type-Law Court (Nightly Rust Compilation)
- **Status:** Requires `-Z stable-mir` (nightly feature), witness lattice still in development
- **Verdict:** FEATURE_GATE (requires `nightly` feature, only for tests)
- **Risk:** High — Nightly rust features are unstable
- **Note:** Not suitable for production integration

#### Witness Lattice (Type Signature Registration)
- **Status:** Central to admission gate but not yet audited/certified
- **Verdict:** FEATURE_GATE (behind `witness-lattice` feature)
- **Risk:** High — Core mechanism is incomplete
- **Note:** Deferring is correct choice

#### Core API Surfaces (Motion, Receipt, GateVerdict enums)
- **Status:** Defined in doorway doc but not yet compiled/tested at scale
- **Verdict:** DEFER_CONTRIB — These APIs will stabilize during v26.6.3
- **Risk:** High — Fundamental types may change
- **Note:** Defer all direct API usage

---

## INTEGRATION PATHS EVALUATED

### PATH A: File Exchange First (PREFERRED for v26.6.3+)

**Condition:** wasm4pm has stable OCEL/JSON import AND receipt ledger schema is finalized

```rust
// cargo-cicd/src/pipeline.rs
let ocel_events = cicd_stage.emit_ocel_json()?;
let receipt = wasm4pm::client::import_ocel(ocel_events)?;
process_models = wasm4pm::client::discover_models(&receipt)?;
```

**Verdict if true:** FILE_EXCHANGE  
**Risk:** Low (file-based, no runtime coupling)  
**Timeline:** v26.6.3+ (after receipt ledger stabilizes)

**Why deferred from v26.6.2:**
- Receipt ledger schema not finalized (may change in v26.6.3 admission gate refinement)
- OCEL JSON output format not yet audited for correctness
- wasm4pm-compat nightly test suite incomplete

---

### PATH B: CLI Shell-Out (FALLBACK for v26.6.3+)

**Condition:** wasm4pm has stable CLI commands documented in manual/CLI.md

```rust
// cargo-cicd/src/integrations/wasm4pm_shell.rs
let output = Command::new("wasm4pm")
    .args(&["discover", "--input", events_file, "--output", models_file])
    .status()?;
```

**Verdict if true:** SHELL_OUT  
**Risk:** Medium (CLI may change; fragile to version updates)  
**Timeline:** v26.6.3+ (only if FILE_EXCHANGE path not ready)

**Why deferred from v26.6.2:**
- No stable CLI signature exists yet
- CLI contracts are more brittle than file formats
- wasm4pm CLI may change between minor versions

---

### PATH C: Thin Rust Adapter (NOT RECOMMENDED)

**Condition:** wasm4pm exposes stable USE_AS_IS core APIs

```rust
// cargo-cicd/src/integrations/wasm4pm_current.rs (hypothetical)
// FORBIDDEN FOR v26.6.2 — This couples cargo-cicd to wasm4pm internals
```

**Verdict if true:** WRAP_LOCAL  
**Risk:** Medium-High (couples to unstable API; high refactor cost in v26.6.3)  
**Why REJECTED:** Type signatures (Motion, Receipt, GateVerdict) are not yet stable. Wrapping them creates a replaceable layer, but replacement cost is high when wrapped types change.

---

### PATH D: Defer (CHOSEN FOR v26.6.2) ✓

**Condition:** Most capabilities are DEFER_CONTRIB or incomplete

**Action:** v26.6.2 ships WITHOUT wasm4pm integration. Create clear deferral artifacts:
1. WASM4PM_INTEGRATION_RECOMMENDATION.md (this document)
2. DEFERRED_WASM4PM_CONTRIB_EXTRACTION.md (contribution roadmap)
3. Feature gate `wasm4pm_future` for future integration point
4. Placeholder module `src/integrations/wasm4pm_future.rs` (stub only)

**Risk:** None (no integration risk; deferral is safe)  
**Timeline:** v26.6.2 ships without wasm4pm; v26.6.3+ integration road begins

**Why chosen:**
- All core APIs still in flux (type law court not yet audited)
- Witness lattice registration incomplete
- Nightly Rust features required (not production-ready)
- Receipt ledger schema not finalized
- Integration at this stage creates high refactor cost in v26.6.3

---

## v26.6.2 SCOPE (What We Ship)

### INCLUDED in v26.6.2

- ✓ Process event emission (OCEL-style, internal format)
- ✓ Event collection during pipeline stages (μ₁-μ₅)
- ✓ Event log export to JSON (internal schema)
- ✓ Artifact receipt generation (internal types)
- ✓ Documentation of OCEL structure (for future import)

### EXPLICITLY EXCLUDED from v26.6.2

- ✗ Direct wasm4pm API imports (defer to v26.6.3)
- ✗ Motion/Receipt struct usage from wasm4pm (defer to v26.6.3)
- ✗ Type-law court compilation (defer to v26.6.3)
- ✗ Witness lattice registration (defer to v26.6.3)
- ✗ Process model discovery integration (defer to v26.6.3)

### DOCUMENT PLACEHOLDER

Create `/src/integrations/wasm4pm_future.rs`:

```rust
//! wasm4pm Integration Placeholder
//!
//! DEFERRED TO v26.6.3+
//!
//! This module is a placeholder for future wasm4pm integration.
//! In v26.6.3, this will implement FILE_EXCHANGE path (Path A).
//!
//! Integration prerequisite:
//! - wasm4pm-compat type-law court nightly test suite ALIVE
//! - Receipt ledger schema finalized and audited
//! - OCEL JSON schema cross-validated with wasm4pm
//!
//! See: WASM4PM_INTEGRATION_RECOMMENDATION.md
//! See: DEFERRED_WASM4PM_CONTRIB_EXTRACTION.md

#![cfg(feature = "wasm4pm_future")]

// Placeholder: no implementation for v26.6.2
```

---

## v26.6.3+ SCOPE (Deferred Work)

### v26.6.3 Pre-Integration Checklist

Before integrating wasm4pm in v26.6.3, verify:

- [ ] wasm4pm-compat nightly test suite is 100% passing (Inspection Gate certification)
- [ ] Receipt ledger schema is IMMUTABLE (signed by Inspection Gate)
- [ ] OCEL JSON schema is cross-validated with pm4py
- [ ] Witness lattice corpus is audit-certified
- [ ] Type-law audit trail is complete (zero violations)
- [ ] Performance SLOs established and documented

### v26.6.3 Integration Tasks

1. **Finalize Receipt Ledger Schema**
   - Lock OCEL JSON format
   - Publish schema documentation
   - Certify schema with Inspection Gate

2. **Implement FILE_EXCHANGE Path (A)**
   - Add OCEL JSON export from cargo-cicd
   - Implement wasm4pm JSON import client
   - Add process discovery integration

3. **Stabilize Type-Law Court**
   - Move `-Z stable-mir` to stable Rust (if possible)
   - Audit nightly type compilation path
   - Certify witness lattice registration

4. **Add Process Mining Output**
   - Emit conformance metrics
   - Integrate pm4py analysis
   - Publish process models to artifact store

---

## DEFERRED CONTRIBUTIONS (See Separate Document)

See: `DEFERRED_WASM4PM_CONTRIB_EXTRACTION.md`

**Deferred Capabilities:**
1. Motion execution from OCEL events
2. Receipt issuance and ledger maintenance
3. Type-law court verdict generation
4. Witness lattice certification
5. Cross-system conformance validation
6. Performance metrics collection

Each capability is mapped to v26.6.3+ owner + acceptance criteria.

---

## RISK ASSESSMENT

### Integration Risk: PATH D (DEFER) = LOW

| Risk Factor | Impact | Mitigation |
|------------|--------|-----------|
| Scope creep | Low | Clear deferral doc + feature gate |
| API changes in v26.6.3 | None (not integrated) | Placeholder preserves integration point |
| Stakeholder confusion | Medium | Document clearly in CHANGELOG.md |
| Missed deadline | None | No deadline (deferred is intentional) |

### Integration Risk: PATH A (FILE_EXCHANGE) = MEDIUM

**If we attempted Path A in v26.6.2:**
| Risk Factor | Impact | Mitigation |
|------------|--------|-----------|
| Receipt schema changes | High | Re-export logic must change |
| OCEL JSON format instability | High | Import failures in wasm4pm |
| Nightly type-law court failures | High | Admit gate fails; motions rejected |
| Witness lattice incomplete | High | Cannot certify type signatures |

---

## RECOMMENDATION SUMMARY

| Aspect | Decision |
|--------|----------|
| **Recommended Path** | PATH D (DEFER to v26.6.3+) |
| **v26.6.2 Integration** | None (clean separation) |
| **v26.6.3+ Path** | PATH A (FILE_EXCHANGE) |
| **Fallback for v26.6.3** | PATH B (SHELL_OUT) if Path A blocked |
| **Never Use** | PATH C (Thin Rust Adapter) — too much coupling |
| **Primary Risk** | Premature integration (AVOIDED by deferring) |
| **Primary Benefit** | Clean v26.6.2 release + unblocked v26.6.3 planning |

---

## INSPECTION GATE AUTHORITY SIGN-OFF

**Authority:** Inspection Gate + Water Gate (Delivery Authority)

```
Water Gate Verdict (Delivery Authority):

This recommendation correctly identifies wasm4pm as a deferred integration
for v26.6.3+. v26.6.2 ships without wasm4pm coupling. Integration roadmap
is documented. Risk is minimized through clear deferral artifacts.

Water Gate certifies this as SAFE DEFAULT for v26.6.2 delivery.

Signed (Water Gate Authority):
_________________________________
Delivery Lead

Date: 2026-06-02

Secondary Authority (Inspection Gate):
This matrix aligns with Inspection Gate doctrine. Type-law court is not
yet production-ready. Receipt ledger schema not immutable. Deferral is
the correct choice. v26.6.3 integration prerequisites documented.

Signed (Inspection Gate Authority):
_________________________________
Process Intelligence Core / CTO

Date: 2026-06-02
```

---

**Document Version:** 001  
**Status:** INTEGRATION PATH LOCKED FOR v26.6.2  
**Next Review:** Before v26.6.3 planning (estimated 2026-07-01)
