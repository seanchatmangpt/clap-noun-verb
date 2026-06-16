# Performance Decision Framework

**Purpose**: Standardized decision-making for performance-related changes, feature additions, and trade-offs.

**Audience**: Maintainers deciding whether to accept features, dependencies, or optimizations.

---

## Table of Contents

1. [Feature Addition Decision Tree](#feature-addition-decision-tree)
2. [Dependency Addition Decision Tree](#dependency-addition-decision-tree)
3. [Performance Trade-off Matrix](#performance-trade-off-matrix)
4. [Optimization Prioritization](#optimization-prioritization)
5. [Headroom Management](#headroom-management)
6. [Acceptable Degradation](#acceptable-degradation)

---

## Feature Addition Decision Tree

**Start**: "We want to add a new feature"

```
┌─────────────────────────────────────────────┐
│ Measure Impact                              │
│ - Compile time delta                        │
│ - Binary size delta                         │
│ - Runtime overhead                          │
│ - Dependencies added                        │
└─────────┬───────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────┐
│ Compile Time Impact?                        │
└──────────────┬──────────────────────────────┘
               │
     ┌─────────┼─────────┐
     │         │         │
     ▼         ▼         ▼
  ≤50ms   50-100ms  >100ms
    │         │        │
    ▼         ▼        ▼
  GREEN    YELLOW     RED
    │         │        │
    └─┬───────┼────────┘
      │       │
      ▼       ▼
┌─────────────────────────────────────────────┐
│ Binary Size Impact?                         │
└──────────────┬──────────────────────────────┘
               │
     ┌─────────┼─────────────────┐
     │         │                 │
     ▼         ▼                 ▼
  ≤100KB  100-200KB         >200KB
    │         │               │
    ▼         ▼               ▼
  GREEN    YELLOW           RED
    │         │               │
    └─┬───────┼───────────────┘
      │       │
      ▼       ▼
┌─────────────────────────────────────────────┐
│ Runtime Overhead?                           │
└──────────────┬──────────────────────────────┘
               │
     ┌─────────┼──────────┐
     │         │          │
     ▼         ▼          ▼
  <1%µs    1-5µs      >5µs
    │         │        │
    ▼         ▼        ▼
  GREEN    YELLOW     RED
    │         │        │
    └─┬───────┴────────┘
      │
      ▼
┌─────────────────────────────────────────────┐
│ Decision                                    │
├─────────────────────────────────────────────┤
│ 3 GREEN   → ACCEPT (low impact)             │
│ 2 GREEN   → ACCEPT (no concerns)            │
│ 1 GREEN   → REVIEW (might block)            │
│ 0 GREEN   → REJECT (too expensive)          │
│            unless strong justification      │
└─────────────────────────────────────────────┘
```

### Decision Details

#### Accept Immediately (3-2 GREEN)
- Feature is orthogonal to performance budgets
- Feature adds value without cost
- No further discussion needed

**Example**: Adding a `--version` flag that prints a constant string

#### Review Required (1 GREEN + trade-off)
- Feature has one significant cost
- Benefit justifies the cost
- Requires maintainer review
- Document the trade-off in PR

**Example**: Adding `repl` feature (200ms compile, 600KB binary → high value for REPL users)

#### Reject (0 GREEN or contradicts SLO)
- Feature violates SLOs
- Benefit does not justify cost
- Propose alternative approach

**Example**: Adding heavy ML library to default features (violates compile-time SLO)

---

## Dependency Addition Decision Tree

**Start**: "We want to add a dependency"

```
┌──────────────────────────────────────────────┐
│ Measure Dependency Impact (in isolation)     │
│ - Compile time delta                         │
│ - Binary size delta                          │
│ - Transitive dependencies                    │
│ - Security/maintenance status                │
└──────────────┬───────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────┐
│ Is dependency maintained & secure?           │
└──────────────┬────────────────────────────────┘
               │
         YES   │   NO
          ┌────┴─────┐
          │           │
          ▼           ▼
        CONT        REJECT
                    (no exceptions)
          │
          ▼
┌──────────────────────────────────────────────┐
│ Compile Impact?                              │
├──────────────┬──────────────────────────────┤
│ <50ms        │ ACCEPT (auto-gate)          │
│ 50-100ms     │ REQUIRE FEATURE GATE         │
│ >100ms       │ REJECT (find alternative)    │
└──────────────┴──────────────────────────────┘
```

### Process

1. **Measure baseline**:
   ```bash
   cargo build --release --no-default-features
   # Record time and size
   ```

2. **Add dependency**:
   ```bash
   cargo add new_dep
   cargo build --release --no-default-features
   # Compare times
   ```

3. **Check security**:
   ```bash
   cargo audit
   cargo deny check licenses
   ```

4. **Make decision**:
   - ≤50ms compiled time → add to default features
   - 50-100ms → gate behind feature flag
   - >100ms → find alternative or reject

### Examples

**✅ Accept**: `once_cell` (minimal, widely-used, fast)
```toml
[dependencies]
once_cell = "1.19"
```

**✅ Gate**: `rustyline` (good but optional, 200ms)
```toml
[dependencies]
rustyline = { version = "14.0", optional = true }

[features]
repl = ["dep:rustyline"]
```

**❌ Reject**: `heavy-ml-library` (1000ms, not needed by all users)
- Either find lightweight alternative
- Or create separate crate with dependency

---

## Performance Trade-off Matrix

| Trade-off | Acceptable? | Budget | Condition |
|-----------|-------------|--------|-----------|
| Compile time for features | ✅ Yes | ≤100ms | Feature-gated |
| Binary size for code clarity | ✅ Yes | ≤500KB | Worthwhile abstraction |
| Runtime for safety | ✅ Yes | ≤10% slowdown | High-value safety guarantee |
| Macro expansion for ergonomics | ✅ Yes | ≤1ms per item | Major DX improvement |
| Memory for speed | ⚠️ Maybe | ≤50MB peak | Transient (during build/test) |
| Test speed for determinism | ✅ Yes | Serial tests OK | Uses RUST_TEST_THREADS=1 |
| Doc build for comprehensiveness | ✅ Yes | ≤20s | Includes examples |
| Startup latency for features | ✅ Yes | ≤5ms overhead | Feature-specific startup |

### Decision Process

For each trade-off:

1. **Quantify both sides**:
   - Cost: "Compile time +50ms"
   - Benefit: "Adds safety guarantees for X use case"

2. **Apply headroom rules**:
   - Is headroom > 30%? → Automatically accept
   - Is headroom 10-30%? → Requires review
   - Is headroom < 10%? → Requires strong justification

3. **Document the decision**:
   - PR description or code comment
   - Link to this framework
   - Explain why trade-off is worth it

---

## Optimization Prioritization

### Priority Levels

**Priority 1 (Defend SLOs)** — Blocker if violated
- Incremental compile ≤ 2.0s
- Binary size ≤ 10 MB
- Test suite ≤ 1.0s
- Dispatch overhead < 5%

**Priority 2 (Maintain Headroom)** — Critical if < 20% headroom
- Incremental compile < 1.5s (25% headroom)
- Binary size < 5 MB (50% headroom)
- Test suite < 0.5s (50% headroom)

**Priority 3 (Improve)** — Nice-to-have optimizations
- Reduce micro-allocations
- Optimize hot paths
- Cache computations
- Profile and fix bottlenecks

**Priority 4 (Polish)** — Low-priority
- Reduce build below 0.66s
- Reduce binary below 2MB
- Optimize non-critical paths

### When to Prioritize

```
Current Metric      Headroom    Action
─────────────────────────────────────────
1.0s compile        50%         Monitor, no action
1.5s compile        25%         Plan optimization
1.8s compile        10%         Optimize now
2.1s compile        0%          BLOCKER - must fix

2.2MB binary        78%         Monitor, no action
5.0MB binary        50%         Plan optimization
8.0MB binary        20%         Optimize now
11MB binary         0%          BLOCKER - must fix
```

### Optimization Workflow

1. **Baseline measurement**: Record current metrics
2. **Identify bottleneck**: Profile to find root cause
3. **Implement fix**: Make targeted change
4. **Measure result**: Verify improvement
5. **Regression test**: Ensure no side effects
6. **Document**: Update CHANGELOG and comments

---

## Headroom Management

### Headroom Rules

```
Headroom > 50%  (Green Zone)
  → Add features freely
  → Monitor performance
  → Document any regressions

Headroom 20-50% (Yellow Zone)
  → New features require optimization review
  → Measure impact before merging
  → Consider trade-offs carefully

Headroom < 20%  (Red Zone)
  → Treat as critical
  → Optimize before new features
  → Require strong justification for any changes
```

### Current Status (2026-06-14)

| Metric | Budget | Current | Headroom | Status |
|--------|--------|---------|----------|--------|
| Incremental Compile | 2.0s | 0.66s | 67% | 🟢 Green |
| Binary Size | 10MB | 2.2MB | 78% | 🟢 Green |
| Test Suite | 1.0s | 0.16s | 84% | 🟢 Green |
| All Comfortable | — | — | — | ✅ Safe to add features |

### Monitoring Plan

**Quarterly review**:
1. Run `cargo make release-validate`
2. Update metrics table in PERFORMANCE_STANDARDS.md
3. If any yellow, create optimization issue
4. If any red, immediate action required

**Per-PR review**:
1. Review for performance impact
2. Run benchmarks on significant changes
3. Alert if headroom decreases

---

## Acceptable Degradation

### What We Accept

✅ **Intentional trade-offs with documentation**:
- "Feature X adds 100ms compile time, enabled by default because Y"
- "Optional dependency Z improves experience by 50%, gated behind feature"

✅ **Hardware-dependent variance**:
- Build time varies by CPU/disk speed
- Memory usage depends on system resources
- Network latency for CI affects test time

✅ **Transient degradation**:
- First build after clean (warming caches is normal)
- Doc build includes examples (one-time per release)
- Full CI run slower than incremental (parallel jobs)

### What We Don't Accept

❌ **Unexplained regressions**:
- Compile time increases without documentation
- Binary size grows without justification
- Test failures that block merge

❌ **Ignored SLO violations**:
- Accepting 2.5s compile time when target is 2.0s
- Ignoring binary size that exceeds 10MB
- Merging test failures with comment "fix later"

❌ **Performance cliffs**:
- Feature that's fine in isolation but breaks when combined
- Dependency that interacts badly with others
- Change that only manifests under specific conditions

### Regression Response Process

**On detection**:
1. Run full `./scripts/measure_performance.sh`
2. Identify which metric(s) regressed
3. Create issue with `performance` label
4. Block release if SLO violated

**Investigation**:
1. `git bisect` to find commit
2. Analyze change (code, dependencies, features)
3. Determine root cause

**Resolution**:
1. Revert if no simple fix
2. Optimize if fixable
3. Document trade-off if intentional
4. Update SLO if raising baseline

---

## Case Studies

### Case 1: Adding OpenTelemetry Support

**Proposal**: "Add optional `otel` feature for OpenTelemetry"

**Impact measurement**:
- Compile: +400ms (to 1.1s)
- Binary: +1.2MB (to 3.4MB)
- Runtime: <1µs overhead (gates on env var)

**Decision**: ✅ **Accept with feature gate**
- Compile time still < 2.0s threshold
- Binary size well under 10MB
- Optional feature, no default impact
- Value: Production observability

**Approval**: Merge with documentation

---

### Case 2: Switching to Hash-Based Routing

**Proposal**: "Optimize verb lookup from O(n) to O(1) with hashmap"

**Measurement**:
- Compile: No change (no new deps)
- Binary: +100KB (hashmap overhead)
- Runtime: 50µs → 10µs (80% improvement)

**Decision**: ✅ **Accept immediately**
- No compile regression
- Binary size negligible (1% increase)
- Significant runtime improvement
- Affects critical dispatch path

**Approval**: Merge as optimization

---

### Case 3: Adding Data Validation Library

**Proposal**: "Add `validator` crate for field validation"

**Measurement**:
- Compile: +150ms (to 0.82s incremental)
- Binary: +250KB (to 2.45MB)
- Runtime: <1µs (used only in validators)
- Dependencies: validator + 8 transitive

**Decision**: 🔴 **Reject (unconditional)**
- Compile time still acceptable (under budget)
- **But**: 150ms is substantial for feature with low adoption
- **Recommendation**: Use `serde` validators instead (already included)

**Approval**: Suggest alternative (serde_json schema validation)

---

### Case 4: Feature Request: Async Support

**Proposal**: "Add `async_verb` macro for async handlers"

**Measurement**:
- Compile: No change (linkme-based, no new deps)
- Binary: +50KB (async runtime is already included via tokio workspace dep)
- Runtime: Depends on use case
- Feature gate: Yes (`async` feature)

**Decision**: ✅ **Accept with feature gate**
- Minimal default impact
- Value for async-heavy CLIs
- Clean feature gate (no surprise in baseline)

**Approval**: Merge with good documentation

---

## Summary Checklist

Before proposing a performance-impacting change:

- [ ] Measured compile time impact
- [ ] Measured binary size impact
- [ ] Measured runtime overhead
- [ ] Checked dependency security status
- [ ] Determined feature gate (if appropriate)
- [ ] Verified no SLO violations
- [ ] Documented trade-off
- [ ] Obtained maintainer approval for trade-offs

---

## References

- [PERFORMANCE_STANDARDS.md](../PERFORMANCE_STANDARDS.md) — Detailed standards
- [PERFORMANCE_GUIDE.md](../howto/PERFORMANCE_GUIDE.md) — Practical how-to
- [PERFORMANCE_MONITORING.md](../howto/PERFORMANCE_MONITORING.md) — Monitoring guide
- [CLAUDE.md](../../CLAUDE.md) — Project SLOs

---

**Last Updated**: 2026-06-14  
**Version**: 26.6.14  
**Maintained by**: clap-noun-verb contributors
