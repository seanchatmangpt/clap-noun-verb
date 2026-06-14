# wasm4pm Leverage Matrix — Integration Opportunity Assessment

**Authority:** Water Gate (Delivery Lead)  
**Secondary Authority:** Inspection Gate (Process Intelligence Core)  
**Date:** 2026-06-02  
**Status:** LEVERAGE ASSESSMENT & OPPORTUNITY MATRIX

---

## MISSION

This document evaluates the leverage opportunity of each wasm4pm capability for cargo-cicd v26.6.2 and v26.6.3+. It answers: "What integration gives maximum benefit with minimum risk?"

---

## LEVERAGE OPPORTUNITY MATRIX

### Matrix Dimensions

- **X-Axis (Benefit):** How much value to cargo-cicd? (0-10 scale)
- **Y-Axis (Risk):** Integration complexity & stability risk? (0-10 scale)
- **Bubble Size:** Prerequisite blocking count (larger = more blocked)
- **Color:** Verdict category (green=FILE_EXCHANGE, yellow=SHELL_OUT, red=DEFER_CONTRIB, gray=FEATURE_GATE)

---

## OPPORTUNITY PLOTPOINTS

### HIGH-BENEFIT, LOW-RISK (Preferred for v26.6.3)

**CAP-001: OCEL JSON File Exchange**
- **Benefit:** 8/10 (enables process mining, no API coupling)
- **Risk:** 3/10 (file format only, decoupled)
- **Blocked By:** 1 prerequisite (Receipt ledger schema immutable)
- **Leverage Ratio:** 8:3 = 2.67 ✓ GOOD
- **Verdict:** FILE_EXCHANGE
- **Recommendation:** PRIMARY INTEGRATION PATH for v26.6.3
- **Timeline:** v26.6.3 (after receipt ledger finalized)
- **ROI:** High (unblocks process mining, no API risk)

### MEDIUM-BENEFIT, MEDIUM-RISK (Fallback for v26.6.3)

**CAP-002: CLI Event Emission**
- **Benefit:** 6/10 (works, but fragile to CLI changes)
- **Risk:** 6/10 (CLI contracts brittle; subject to change)
- **Blocked By:** 1 prerequisite (CLI documentation stable)
- **Leverage Ratio:** 6:6 = 1.0 ⚠ MARGINAL
- **Verdict:** SHELL_OUT
- **Recommendation:** FALLBACK ONLY if FILE_EXCHANGE blocked
- **Timeline:** v26.6.3+ (late, only if needed)
- **ROI:** Medium (works, but high maintenance cost)

### LOW-BENEFIT, HIGH-RISK (Deferred to v26.6.3+)

**CAP-003: Motion Struct API**
- **Benefit:** 7/10 (core functionality, but unstable)
- **Risk:** 9/10 (type signature may change; high refactor cost)
- **Blocked By:** 3 prerequisites (Motion struct stable, witness lattice, type-law audit)
- **Leverage Ratio:** 7:9 = 0.78 ✗ POOR
- **Verdict:** DEFER_CONTRIB 1
- **Recommendation:** DEFER until Motion struct immutable
- **Timeline:** v26.6.3-alpha (core dependency)
- **ROI:** Medium-Low (needed later, but prerequisite for other work)

**CAP-004: Receipt Struct API**
- **Benefit:** 8/10 (core to ledger, but incomplete)
- **Risk:** 9/10 (schema TBD; signing mechanism missing)
- **Blocked By:** 4 prerequisites (Receipt schema, signing, type-law audit, witness lattice)
- **Leverage Ratio:** 8:9 = 0.89 ✗ POOR
- **Verdict:** DEFER_CONTRIB 2
- **Recommendation:** DEFER until Receipt struct stable + signing infrastructure ready
- **Timeline:** v26.6.3-beta (critical dependency)
- **ROI:** Medium-Low (blocks ledger, but prerequisites must stabilize first)

**CAP-005: GateVerdict Enum**
- **Benefit:** 6/10 (needed for admission logic)
- **Risk:** 8/10 (enum variants may expand; unstable)
- **Blocked By:** 2 prerequisites (Type-law audit, enum finalization)
- **Leverage Ratio:** 6:8 = 0.75 ✗ POOR
- **Verdict:** DEFER_CONTRIB 3
- **Recommendation:** DEFER until enum finalized by Inspection Gate
- **Timeline:** v26.6.3-rc (type-law audit prerequisite)
- **ROI:** Low (needed, but only after type system settles)

**CAP-006: Type-Law Court Compilation**
- **Benefit:** 4/10 (test-only; not production value)
- **Risk:** 9/10 (nightly features; unstable; not production-ready)
- **Blocked By:** 2 prerequisites (Nightly stable-mir deterministic, type-law audit complete)
- **Leverage Ratio:** 4:9 = 0.44 ✗ VERY POOR
- **Verdict:** FEATURE_GATE (nightly, test-only)
- **Recommendation:** NEVER use in production; test-only for v26.6.3-rc
- **Timeline:** v26.6.3-rc+ (late, tests only)
- **ROI:** Very Low (test infrastructure; no production value)

**CAP-007: Witness Lattice Registration**
- **Benefit:** 7/10 (core to admission gate)
- **Risk:** 9/10 (central mechanism; not yet certified)
- **Blocked By:** 3 prerequisites (Lattice structure stable, audit complete, witness corpus certified)
- **Leverage Ratio:** 7:9 = 0.78 ✗ POOR
- **Verdict:** FEATURE_GATE (witness-lattice) + DEFER_CONTRIB 4
- **Recommendation:** DEFER until witness lattice audit-certified by Inspection Gate
- **Timeline:** v26.6.3-beta (late in critical path)
- **ROI:** Medium-Low (needed for admission, but prerequisites must stabilize)

**CAP-008: Process Model Discovery**
- **Benefit:** 9/10 (core to process mining)
- **Risk:** 8/10 (depends on ledger + pm4py; multi-system)
- **Blocked By:** 4 prerequisites (Receipt ledger, pm4py integration, OCEL export, discovery latency <500ms)
- **Leverage Ratio:** 9:8 = 1.13 ≈ MARGINAL
- **Verdict:** DEFER_CONTRIB 5
- **Recommendation:** DEFER until receipt ledger + pm4py integration working
- **Timeline:** v26.6.3 (late, critical dependency)
- **ROI:** High (unblocks process mining insights, but prerequisites take time)

**CAP-009: Conformance Checking**
- **Benefit:** 9/10 (critical for process validation)
- **Risk:** 9/10 (oracle not defined; multi-system integration)
- **Blocked By:** 5 prerequisites (Process discovery, pm4py fitness check, conformance SLO >0.95, declared model match)
- **Leverage Ratio:** 9:9 = 1.0 ≈ MARGINAL
- **Verdict:** DEFER_CONTRIB 5
- **Recommendation:** DEFER until process discovery working + conformance oracle defined
- **Timeline:** v26.6.3 (very late, critical path blocker)
- **ROI:** Very High (validates execution; but takes longest to implement)

**CAP-010: Performance Metrics Collection**
- **Benefit:** 7/10 (useful for optimization)
- **Risk:** 7/10 (SLO framework TBD; profiling infrastructure missing)
- **Blocked By:** 3 prerequisites (Profiling framework, SLO definitions, metrics instrumentation)
- **Leverage Ratio:** 7:7 = 1.0 ≈ NEUTRAL
- **Verdict:** DEFER_CONTRIB 6
- **Recommendation:** DEFER until profiling infrastructure integrated
- **Timeline:** v26.6.3 (parallel track, not on critical path)
- **ROI:** Medium (nice-to-have; deferred for simplicity)

---

## LEVERAGE RANKINGS

### By Leverage Ratio (Benefit:Risk)

| Rank | Capability | Benefit | Risk | Ratio | Verdict |
|------|------------|---------|------|-------|---------|
| 1 | OCEL JSON Exchange | 8 | 3 | **2.67** ✓ | FILE_EXCHANGE → PRIMARY PATH |
| 2 | CLI Event Emission | 6 | 6 | 1.0 | SHELL_OUT → Fallback |
| 3 | Process Discovery | 9 | 8 | 1.13 | DEFER_CONTRIB 5 → High value, late |
| 4 | Conformance Checking | 9 | 9 | 1.0 | DEFER_CONTRIB 5 → Critical, very late |
| 5 | Performance Metrics | 7 | 7 | 1.0 | DEFER_CONTRIB 6 → Neutral, defer |
| 6 | Receipt Struct | 8 | 9 | 0.89 | DEFER_CONTRIB 2 → Core, but risky |
| 7 | Motion Struct | 7 | 9 | 0.78 | DEFER_CONTRIB 1 → Core, risky |
| 8 | Witness Lattice | 7 | 9 | 0.78 | FEATURE_GATE 4 → Core, risky |
| 9 | GateVerdict Enum | 6 | 8 | 0.75 | DEFER_CONTRIB 3 → Unstable |
| 10 | Type-Law Court | 4 | 9 | **0.44** | FEATURE_GATE (nightly) → Avoid |

---

## CRITICAL PATH ANALYSIS

### v26.6.2 Critical Path
```
[ No integration ] ← CLEAN DECISION
    ↓
v26.6.3-alpha: Motion struct stabilize
v26.6.3-beta:  Receipt struct + witness lattice
v26.6.3-rc:    Type-law court audit
v26.6.3:       OCEL file exchange + process discovery
```

### v26.6.3 Integration Sequence (Recommended)

```
PHASE 1 (Parallel) — Stabilize Prerequisites:
├── Motion struct = immutable (v26.6.3-alpha)
├── Receipt schema = finalized (v26.6.3-alpha)
├── Witness lattice = audit-certified (v26.6.3-beta)
└── Type-law court = nightly tests pass (v26.6.3-rc)

PHASE 2 (Sequential) — Implement File Exchange:
└── OCEL JSON export from cargo-cicd
    └── OCEL JSON import to wasm4pm (v26.6.3)

PHASE 3 (Sequential) — Process Mining:
└── Process discovery from receipt ledger (v26.6.3)
    └── Conformance checking (v26.6.3)

PHASE 4 (Parallel) — Performance:
└── Metrics collection & SLO tracking (v26.6.3)
```

---

## RISK MITIGATION STRATEGY

### For HIGH-LEVERAGE Capabilities (OCEL)

**Leverage:** 2.67 (HIGH)  
**Risk:** 3 (LOW)

**Mitigation:**
1. ✓ Lock receipt ledger schema in v26.6.3-alpha
2. ✓ Cross-validate OCEL JSON with pm4py reference implementation
3. ✓ Gate integration on schema immutability (Inspection Gate signed)
4. ✓ Add round-trip tests (cargo-cicd → OCEL → wasm4pm → cargo-cicd)

**Success Criteria:**
- Receipt ledger schema immutable
- OCEL JSON validates against pm4py schema
- Round-trip tests 100% passing

---

### For MEDIUM-LEVERAGE Capabilities (CLI)

**Leverage:** 1.0 (MEDIUM)  
**Risk:** 6 (MEDIUM-HIGH)

**Mitigation:**
1. ✓ Document CLI contract in wasm4pm (human-readable manual)
2. ✓ Add versioning to CLI (e.g., `wasm4pm --version`)
3. ✓ Create adapter layer to isolate CLI changes (cargo-cicd/src/wasm4pm_shell.rs)
4. ✓ Fallback to file exchange if CLI unavailable

**Success Criteria:**
- CLI reference documentation published
- CLI versioning stable
- Adapter layer absorbs minor CLI changes
- Integration tests verify fallback behavior

---

### For LOW-LEVERAGE Capabilities (Core APIs, Type System)

**Leverage:** 0.44-0.89 (LOW)  
**Risk:** 8-9 (HIGH)

**Mitigation:**
1. ✓ DEFER all direct API usage to v26.6.3+
2. ✓ Use file exchange (CAP-001) as API buffer
3. ✓ Never wrap unstable types directly
4. ✓ Require Inspection Gate certification before integration

**Success Criteria:**
- Zero API imports in v26.6.2
- File exchange isolates API changes
- Refactor cost minimized by not coupling to unstable types

---

## OPPORTUNITY SUMMARY

### Best Integration Path for v26.6.3

**Recommended:** FILE_EXCHANGE (CAP-001: OCEL JSON)

**Why:**
- Highest leverage ratio (2.67)
- Lowest risk (3/10)
- Unblocks process mining without API coupling
- File format more stable than APIs
- Round-trip validation possible (OCEL → wasm4pm → discovery)

**Timeline:** v26.6.3 (after receipt ledger finalized)  
**Critical Dependency:** Receipt ledger schema immutable

### Fallback Path

**Conditional:** SHELL_OUT (CAP-002: CLI) if FILE_EXCHANGE blocked

**Why:**
- Still functional (leverage = 1.0)
- Requires CLI stabilization
- Higher maintenance (CLI brittleness)
- Only use if file exchange prerequisites blocked

**Timeline:** v26.6.3+ (late, only if needed)

### Path to Avoid

**Never:** WRAP_LOCAL (thin Rust adapters around unstable APIs)

**Why:**
- Risk is too high (API churn)
- Refactor cost too high
- File exchange is better isolation
- Wrapper becomes obsolete when APIs change

---

## STAKEHOLDER IMPACT

### For Delivery Lead (Water Gate)

**Impact:** v26.6.2 ships CLEAN (zero wasm4pm coupling)  
**Benefit:** No scope creep; clear v26.6.3 roadmap  
**Risk:** None (deferral is intentional)  
**Timeline:** Ready for v26.6.2 release

### For Process Intelligence (Inspection Gate)

**Impact:** wasm4pm-compat can evolve freely until v26.6.3  
**Benefit:** No coupling lock; time to stabilize core APIs  
**Risk:** None (cargo-cicd is decoupled)  
**Timeline:** Nightly test suite must reach ALIVE status by v26.6.3-alpha

### For Contributors

**Impact:** Clear contribution roadmap with 6 deferred contributions  
**Benefit:** Well-defined prerequisites; testable acceptance criteria  
**Risk:** Dependencies must stabilize (not contributor risk)  
**Timeline:** v26.6.3-alpha through v26.6.3 (7-week window)

---

## FINANCIAL IMPACT ESTIMATE

### v26.6.2 (Deferral Strategy)
- **Integration Cost:** 0 (deferred)
- **Risk Cost:** 0 (clean separation)
- **Testing Cost:** 0 (no integration testing)
- **Total:** $0

### v26.6.3 (File Exchange Primary Path)
- **Integration Cost:** ~2 FD (OCEL export + JSON round-trip tests)
- **Risk Cost:** ~0.5 FD (schema validation + cross-check with pm4py)
- **Testing Cost:** ~1 FD (integration tests, SLO verification)
- **Total:** ~3.5 FD

### v26.6.3 (Process Mining Secondary Path)
- **Integration Cost:** ~4 FD (process discovery + conformance oracle)
- **Risk Cost:** ~1 FD (process model validation, fitness SLO >0.95)
- **Testing Cost:** ~2 FD (pm4py integration, conformance tests)
- **Total:** ~7 FD (parallel track, not critical path)

---

**Document Version:** 001  
**Status:** LEVERAGE ASSESSMENT COMPLETE  
**Distribution:** cargo-cicd/docs/wasm4pm/WASM4PM_LEVERAGE_MATRIX.md
