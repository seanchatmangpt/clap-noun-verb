# CLAP-NOUN-VERB V26.6.2 AUTONOMIC POLICIES

**Date:** 2026-06-02  
**Feature:** autonomic (optional)  
**Status:** OPERATIONAL  

---

## Autonomic Architecture

The **autonomic** feature enables **suggest-mode policies** that evaluate CI/CD signals and emit recommendations without enforcing them.

**Design Principle:** Policies observe, recommend, never dictate. Human developers retain final decision authority.

---

## Policy Evaluation Framework

### Signal Detection

**4 Primary Signals:**
1. **Compile Status** — Cargo check result (success/failure)
2. **Test Results** — Test suite outcome (passed/failed/timeout)
3. **Code Quality** — fmt/clippy violations (none/warnings/errors)
4. **Benchmark Regressions** — Performance impact (±threshold %)

### Signal Sources
```rust
pub enum Signal {
    CompileStatus {
        success: bool,
        stderr: String,
    },
    TestResults {
        passed: u32,
        failed: u32,
        ignored: u32,
        duration_ms: u64,
    },
    CodeQuality {
        fmt_violations: u32,
        clippy_warnings: u32,
        clippy_errors: u32,
    },
    BenchmarkRegression {
        metric_name: String,
        baseline_ns: u64,
        current_ns: u64,
        threshold_percent: f64,
    },
}
```

### Recommendation Types

**3 Verdict Levels:**
1. **merge-ready** — All signals green, safe to merge
2. **needs-review** — Mixed signals, manual review recommended
3. **requires-fixes** — Blocker signals (compile failure, test failure, critical lints)

### Verdict Details
```rust
pub struct PolicyVerdic {
    pub status: VerdictStatus,  // merge-ready | needs-review | requires-fixes
    pub reasons: Vec<String>,    // Specific reasons
    pub recommendations: Vec<String>, // Actions to take
    pub signals_evaluated: Vec<Signal>,
    pub timestamp: SystemTime,
}
```

---

## Policy 1: Compile-Readiness Policy

**Enabled:** Yes (default)  
**Signal:** `CompileStatus`  
**Triggers:** On cargo check completion

### Evaluation Logic
```
if cargo check succeeds:
    → compile_ready = true
    → verdict = "proceed to tests"
elif cargo check fails with clippy only:
    → compile_ready = false
    → verdict = "needs-review (clippy violations)"
    → recommend: "Run 'cargo make clippy' to fix"
else (linker/semantic errors):
    → compile_ready = false
    → verdict = "requires-fixes"
    → recommend: "Fix compilation errors and re-run cargo check"
```

### Example Output
```rust
PolicyVerdic {
    status: MergeReady,
    reasons: vec!["Cargo check passed".to_string()],
    recommendations: vec!["Ready to run tests".to_string()],
    signals_evaluated: vec![
        Signal::CompileStatus { success: true, stderr: "".to_string() }
    ],
}
```

### Configuration (CLAUDE.md / cicd.toml)
```toml
[autonomic.policies.compile_readiness]
enabled = true
timeout_secs = 60
fail_on_deprecated_apis = false  # Warnings OK, errors block
```

---

## Policy 2: Test-Coverage Policy

**Enabled:** Yes (default)  
**Signal:** `TestResults`  
**Triggers:** After test suite completes

### Evaluation Logic
```
if all tests passed AND ignored <= threshold:
    → verdict = "merge-ready"
    → recommend: "All tests passed, ready for review"
elif some tests failed:
    → verdict = "requires-fixes"
    → recommend: "Fix failing tests and re-run test suite"
    → reason: "Failing tests: test_name_1, test_name_2"
elif all tests passed BUT high ignored count:
    → verdict = "needs-review"
    → recommend: "Consider enabling skipped tests"
    → reason: "{N} tests ignored (check for #[ignore] marks)"
```

### Example Output (failure case)
```rust
PolicyVerdic {
    status: RequiresFixes,
    reasons: vec![
        "2 tests failed: test_verb_routing, test_async_handler".to_string(),
    ],
    recommendations: vec![
        "Run: cargo make test --verbose".to_string(),
        "Fix failing tests and commit".to_string(),
    ],
    signals_evaluated: vec![
        Signal::TestResults {
            passed: 33,
            failed: 2,
            ignored: 18,
            duration_ms: 10290,
        }
    ],
}
```

### Configuration
```toml
[autonomic.policies.test_coverage]
enabled = true
min_passed_percent = 95
max_ignored_percent = 20
timeout_secs = 120
```

---

## Policy 3: Code-Quality Policy

**Enabled:** Yes (default)  
**Signal:** `CodeQuality` (fmt + clippy)  
**Triggers:** After clippy and fmt checks

### Evaluation Logic
```
if fmt_violations == 0 AND clippy_errors == 0:
    → verdict = "merge-ready"
    → recommend: "Code formatted and lint-clean"
elif fmt_violations > 0:
    → verdict = "needs-review"
    → recommend: "Run 'cargo make format'"
    → reason: "Code formatting issues detected"
elif clippy_errors > 0:
    → verdict = "requires-fixes"
    → recommend: "Run 'cargo make clippy' and fix"
    → reason: "Clippy errors (deny: warnings mode)"
elif clippy_warnings > threshold:
    → verdict = "needs-review"
    → recommend: "Address clippy warnings"
```

### Example Output (formatting issue)
```rust
PolicyVerdic {
    status: NeedsReview,
    reasons: vec![
        "Code formatting violations: 3 files".to_string(),
    ],
    recommendations: vec![
        "Run: cargo make format".to_string(),
        "Commit formatting changes".to_string(),
    ],
    signals_evaluated: vec![
        Signal::CodeQuality {
            fmt_violations: 3,
            clippy_warnings: 0,
            clippy_errors: 0,
        }
    ],
}
```

### Configuration
```toml
[autonomic.policies.code_quality]
enabled = true
deny_fmt_violations = true     # Format issues block merge
deny_clippy_errors = true      # Clippy errors block merge
warn_clippy_warnings = false   # Warnings OK
max_clippy_warnings = 5
```

---

## Policy 4: Performance Policy

**Enabled:** Yes (optional)  
**Signal:** `BenchmarkRegression`  
**Triggers:** After benchmark suite completes

### Evaluation Logic
```
for each benchmark:
    if current_ns <= baseline_ns * (1 + threshold_percent/100):
        → performance_ok = true
    else:
        → regression = current_ns - baseline_ns
        → regression_percent = (regression / baseline_ns) * 100
        → if regression_percent > threshold:
            → add to issues

if no regressions:
    → verdict = "merge-ready"
elif regressions <= acceptable threshold:
    → verdict = "needs-review"
    → recommend: "Benchmark regressions detected, verify acceptable"
else (severe regression):
    → verdict = "requires-fixes"
    → recommend: "Major performance regression detected"
```

### Example Output (regression detected)
```rust
PolicyVerdic {
    status: NeedsReview,
    reasons: vec![
        "Incremental compile: +15% (2.0s vs 1.74s baseline)".to_string(),
    ],
    recommendations: vec![
        "Investigate compile time impact".to_string(),
        "Run 'cargo make bench' to compare".to_string(),
    ],
    signals_evaluated: vec![
        Signal::BenchmarkRegression {
            metric_name: "incremental_compile".to_string(),
            baseline_ns: 1_740_000_000,
            current_ns: 2_000_000_000,
            threshold_percent: 10.0,
        }
    ],
}
```

### Configuration
```toml
[autonomic.policies.performance]
enabled = true
track_metrics = [
    "incremental_compile",
    "binary_size",
    "test_suite_duration"
]
regression_threshold_percent = 10.0  # Allow ±10% variance
warn_on_regression = true
block_on_severe = false  # Warnings only (not merge-blocking)
```

---

## Policy Composition

### Unified Verdict Algorithm

**Input:** 4 policy verdicts  
**Output:** Single composite verdict

```rust
pub fn composite_verdict(
    policies: Vec<PolicyVerdic>
) -> CompositeVerdic {
    let verdicts = policies.iter().map(|p| p.status);
    
    // Worst-case verdict wins
    if verdicts.any(|v| v == RequiresFixes) {
        CompositeVerdic {
            status: RequiresFixes,
            reasons: collect_all_reasons(&policies),
            recommendations: collect_all_recommendations(&policies),
        }
    } else if verdicts.any(|v| v == NeedsReview) {
        CompositeVerdic {
            status: NeedsReview,
            reasons: collect_needing_review(&policies),
            recommendations: collect_recommendations(&policies),
        }
    } else {
        CompositeVerdic {
            status: MergeReady,
            reasons: vec!["All policy checks passed".to_string()],
            recommendations: vec!["Ready to merge".to_string()],
        }
    }
}
```

---

## Usage Examples

### Example 1: All Checks Pass
```bash
$ cargo cicd autonomic suggest

PolicyVerdic:
  Status: merge-ready
  Reasons:
    - Compilation successful
    - 33 tests passed
    - Code formatting OK
    - No performance regressions
  Recommendations:
    - Ready to merge
    - Create PR to main branch

Exit Code: 0 (merge-ready)
```

### Example 2: Formatting Issue
```bash
$ cargo cicd autonomic suggest

PolicyVerdic:
  Status: needs-review
  Reasons:
    - Code formatting violations: 2 files
    - All tests passed
    - Compilation successful
  Recommendations:
    - Run: cargo make format
    - Review and commit formatting changes
    - Re-run: cargo cicd autonomic suggest

Exit Code: 1 (needs-review)
```

### Example 3: Test Failure
```bash
$ cargo cicd autonomic suggest

PolicyVerdic:
  Status: requires-fixes
  Reasons:
    - 2 tests failed: test_verb_dispatch, test_async_handler
    - Failed tests: src/tests/integration.rs lines 234, 567
  Recommendations:
    - Run: cargo make test --verbose
    - Fix failing test logic
    - Re-run: cargo make test
    - Once tests pass, re-run: cargo cicd autonomic suggest

Exit Code: 2 (requires-fixes)
```

---

## Exit Code Semantics

| Exit Code | Status | Merge Decision |
|-----------|--------|---|
| 0 | merge-ready | ✓ Safe to merge |
| 1 | needs-review | ? Human review required |
| 2 | requires-fixes | ✗ Fixes required before merge |

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Autonomic Policy Check
  run: cargo cicd autonomic suggest
  continue-on-error: true  # needs-review doesn't block CI

- name: Require Fixes (Hard Block)
  if: failure() && exit_code == 2
  run: exit 2  # Fail CI on requires-fixes verdict
```

---

## Configuration

### Enable/Disable Policies (CLAUDE.md or cicd.toml)
```toml
[autonomic]
enabled = true

[autonomic.policies]
compile_readiness.enabled = true
test_coverage.enabled = true
code_quality.enabled = true
performance.enabled = false  # Optional: disable expensive benchmarks

[autonomic.policies.code_quality]
deny_fmt_violations = true
deny_clippy_errors = true
warn_clippy_warnings = false
```

### Runtime Invoke
```bash
# Check all 4 policies
cargo cicd autonomic suggest

# Specific policy
cargo cicd autonomic suggest --policy compile_readiness

# With output format
cargo cicd autonomic suggest --format json > verdict.json
```

---

## Policy Signals in Detail

### Signal 1: CompileStatus
- **Source:** `cargo check` output
- **Captured Data:** Exit code, stderr, elapsed time
- **False Positives:** None (cargo is deterministic)
- **Caching:** None (always fresh)

### Signal 2: TestResults
- **Source:** `cargo test` output (test harness JSON format)
- **Captured Data:** Passed count, failed count, ignored count, duration
- **False Positives:** Flaky tests (if test is non-deterministic, verdict may vary)
- **Caching:** None (always fresh)

### Signal 3: CodeQuality
- **Source:** `cargo fmt --check` + `cargo clippy` (deny: warnings)
- **Captured Data:** Violation counts by category
- **False Positives:** Clippy false positives (rare, documented in #[allow])
- **Caching:** 1 hour (format and lints don't change rapidly)

### Signal 4: BenchmarkRegression
- **Source:** `cargo bench` with baseline stored in `.benchmarks/`
- **Captured Data:** Metric name, baseline, current, delta
- **False Positives:** CPU load variance (run on isolated machine)
- **Caching:** Baseline is immutable, current is fresh

---

## Limitations & Known Issues

1. **Flaky Tests:** Non-deterministic tests cause inconsistent verdicts
   - *Mitigation:* Mark flaky tests with `#[ignore]` during development

2. **Machine Variance:** Benchmark results vary by CPU load
   - *Mitigation:* Run benchmarks on isolated CI machines with stable clock

3. **Cache Invalidation:** CodeQuality cache doesn't invalidate on rustfmt version change
   - *Mitigation:* Manual `cargo make format-check` recommended on format tool updates

4. **Partial Signals:** If one policy fails to collect signal, verdict is incomplete
   - *Mitigation:* Emit partial verdict with missing-signal warning

---

## Future Policy Ideas

- **API Stability Policy** — Detect breaking changes in public API (semver)
- **Documentation Policy** — Verify all public items have doc comments
- **Security Policy** — Audit dependencies for known vulnerabilities
- **SLOC Policy** — Warn on excessive function/module size
- **Concurrency Policy** — Detect race conditions (miri linter)

---

## See Also

- **Autonomic Feature Code:** `src/autonomic/` directory
- **Policy Evaluator:** `src/autonomic/policy_evaluator.rs`
- **Configuration:** `/Users/sac/CLAUDE.md` (project rules)
- **Proof Gate Docs:** `receipts/CLAP_NOUN_VERB_V26_6_2_GGEN_MANUFACTURE.md` (proof gates section)
