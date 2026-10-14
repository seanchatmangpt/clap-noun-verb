# Test Orchestration Guide

**clap-noun-verb** uses a comprehensive multi-tier test strategy powered by `cargo-make`. This guide covers running tests with different feature combinations, deterministic execution, benchmarking, CI coordination, coverage analysis, and flaky test detection.

---

## Quick Reference

| Scenario | Command | Use Case |
|----------|---------|----------|
| **Local development** | `cargo make test` | Fast feedback during coding |
| **Deterministic testing** | `cargo make test-lib-deterministic` | Eliminate race conditions, single-threaded |
| **All features** | `cargo make test-all` | Release validation, all features enabled |
| **Frontier features** | `cargo make test-frontier` | Test cutting-edge experimental features |
| **Feature matrix** | `cargo make test-feature-combinations` | Validate default/no features/repl combinations |
| **Benchmarks** | `cargo make bench` | Performance tracking with criterion |
| **CI full suite** | `cargo make ci` | Equivalent to GitHub Actions CI |
| **Release validation** | `cargo make release-validate` | Pre-release comprehensive check |

---

## 1. Running Tests with Different Feature Combinations

The framework supports **4 primary test scenarios** for feature validation:

### 1.1 Default Features (Baseline)

```bash
cargo make test-default
```

- Runs with workspace default features
- **Use case**: Day-to-day development, fastest feedback loop
- **CI location**: Runs in `ci` and `ci-success` jobs

### 1.2 No Features (Minimal)

```bash
cargo make test-no-features
```

- Isolates core library functionality without optional modules
- **Use case**: Validate core `#[verb]`, `#[noun]` macros work without features
- **CI location**: `ci` job, `test-feature-combinations` dependency

### 1.3 REPL Feature Only

```bash
cargo make test-repl
```

- Tests the interactive REPL feature (`rustyline` integration)
- **Use case**: Validate REPL-specific verb/noun handlers
- **CI location**: `ci` job, feature matrix testing

### 1.4 Feature Combinations Matrix

```bash
cargo make test-feature-combinations
```

Runs all three above sequentially:
- Default features
- No features
- REPL feature

**Output**: Aggregate status across combinations
```
Running tests under different feature flag combinations...
✅ PASS: default features
✅ PASS: no features  
✅ PASS: repl feature
✅ Feature matrix complete
```

### 1.5 All Features (Release Validation)

```bash
cargo make test-all
```

- Enables **every optional feature**: `async`, `federated-network`, `otel`, etc.
- **Use case**: Release validation, pre-publish verification
- **CI location**: `release-check` job (manual trigger before publish)
- **Performance note**: Slower; use for final checks only

---

## 2. Deterministic Single-Threaded Test Execution

### 2.1 Default Parallel Testing

By default, cargo runs tests in parallel using your CPU count:

```bash
cargo make test
# ≈ 0.1-0.5s for full test suite on modern hardware
```

**Problem**: Race conditions, test order dependencies, flaky failures (appear randomly).

### 2.2 Single-Threaded Deterministic Mode

```bash
cargo make test-lib-deterministic
```

**Environment variable**: `RUST_TEST_THREADS=1`

- Forces sequential test execution (no parallelism)
- Eliminates race conditions from shared state
- **Output is identical every run** — true determinism
- **Use case**: 
  - Pre-commit verification
  - Debugging flaky tests
  - CI "unfailable architecture" (must not flake randomly)

**Library tests only** (excludes integration tests):
```bash
cargo test --lib --quiet
RUST_TEST_THREADS=1
```

### 2.3 Integration Tests (Isolated)

```bash
cargo make test-integration-isolated
```

- Single-threaded integration tests
- Isolation prevents cross-test interference
- **Use case**: Tests that share file system, databases, or sockets

### 2.4 Unfailable Test Architecture

```bash
cargo make test-unfailable
```

Combines both:
- Library tests (deterministic)
- Integration tests (isolated)

**Result**: **All tests complete successfully 100% of the time** (no flakes).

**CI enforcement**: The `ci` job runs `test-unfailable`, not standard parallel testing.

---

## 3. Benchmark Running and Interpretation

The project uses **criterion** for statistical benchmarking. Located in `Cargo.toml`:

```toml
[[bench]]
name = "dispatch"
path = "benches/dispatch.rs"
harness = false  # Uses criterion, not cargo default
```

### 3.1 Basic Benchmark Run

```bash
cargo make bench
```

- Runs with `--all-features`
- Statistical measurements with criterion
- Generates HTML report in `target/criterion/`

**Output example**:
```
Running: benches/dispatch.rs

Dispatch Benchmark Results:
  dispatch_empty_command    time:   [12.345 ms 12.567 ms 12.789 ms]
  dispatch_nested_command   time:   [23.456 ms 23.678 ms 23.890 ms]
```

### 3.2 Save Baseline

```bash
cargo make bench-baseline
```

- Saves current measurements as `main` baseline
- **Use case**: Before major refactor, creating release checkpoint
- **File location**: `target/criterion/baseline_main/`

**Command**: `cargo bench --all-features -- --save-baseline main`

### 3.3 Compare Against Baseline

```bash
cargo make bench-compare
```

- Runs benchmarks and compares against previously saved `main` baseline
- Criterion automatically detects **regressions** (slower) or **improvements** (faster)
- **Use case**: Post-refactor validation, "did we get slower?"

**Output** (example regression detection):
```
dispatch_empty_command: time:   [15.000 ms 15.234 ms 15.468 ms]
                        change: [+21.7% +21.9% +22.1%] (regression)
                        ❌ Performance regressed!
```

### 3.4 Phase-Based Benchmarking

Project supports **phase-specific benchmarks**:

```bash
cargo make bench-phase1  # Foundation benchmarks
cargo make bench-phase2  # RDF/Semantic benchmarks (requires features)
cargo make bench-phase3  # Optimization & ML benchmarks
cargo make bench-phase4  # Advanced features benchmarks
```

Each phase tests specific subsystem performance.

### 3.5 Profiling-Enabled Runs

```bash
cargo make profile
```

- Runs benchmarks with `--profile-time 10`
- Enables time profiling (Linux `perf` integration)
- **Use case**: Identifying CPU hotspots, cache misses
- **Output**: Flamegraph data in `target/criterion/`

### 3.6 SLO Validation

```bash
cargo make slo-check
```

Verifies performance targets (Service Level Objectives):

```
Performance SLO Validation
============================
✅ Incremental Compilation: 0.66s (Target: ≤2s)
   Status: PASS (67% faster than target)

✅ Binary Size: 2.2MB (Target: ≤10MB)
   Status: PASS (78% under target)
```

**Configured SLOs**:
- **Incremental compilation**: ≤2 seconds
- **Binary size (release)**: ≤10 MB
- **CLI generation latency**: ≤100ms

---

## 4. CI Pipeline Coordination

### 4.1 Local CI Simulation

Run **exactly what CI runs**:

```bash
cargo make ci
```

**Dependencies** (executed in order):
1. `format-check` — Code formatting validation
2. `clippy` — Linter with strict warnings
3. `test-feature-combinations` — All 3 feature sets
4. `test-unfailable` — Library + integration (deterministic)
5. `build-examples` — Verify examples compile
6. `check-all` — Check with all features

**Runtime**: ~30-60 seconds (depending on hardware)

**Exit behavior**: Fails on **first error** (no partial passes)

### 4.2 GitHub Actions CI Jobs

#### Standard CI Pipeline (`.github/workflows/ci.yml`)

**Parallel jobs** (independent, run simultaneously):

| Job | Purpose | Failure Condition |
|-----|---------|-------------------|
| `fmt` | rustfmt check | Non-formatted code |
| `clippy` | Linting (deny warnings) | Any clippy warning |
| `test` | Multi-version tests (stable/beta/nightly) | Any failed test |
| `nextest` | Faster test runner | Failed nextest run |
| `msrv` | Minimum Rust 1.74 | Build fails on MSRV |
| `docs` | Documentation build | Doc errors |
| `audit` | Security vulnerabilities | Known CVE detected |
| `licenses` | Dependency license check | Non-permissive license |
| `typos` | Spell check | Typos found |

**Overall result**: `ci-success` — passes only if ALL jobs succeed

#### Frontier CI Pipeline (`.github/workflows/frontier-ci.yml`)

**21-point feature matrix** — tests all combinations:

```
Tier 0: Baseline (1)
  - Default features

Tier 1: Individual features (9)
  - meta-framework, rdf-composition, fractal-patterns, discovery-engine,
    federated-network, learning-trajectories, reflexive-testing,
    economic-sim, quantum-ready

Tier 2: Meta-features (3)
  - frontier-semantic, frontier-intelligence, frontier-quality

Tier 3: Critical combinations (5)
  - meta-framework + rdf-composition
  - discovery-engine + learning-trajectories
  - federated-network + rdf-composition
  - economic-sim + learning-trajectories
  - executable-specs

Tier 4: Extremes (3)
  - frontier-all, no-default-features, repl
```

**Additional checks**:
- Code coverage (tarpaulin, 80% threshold)
- Performance benchmarks (criterion)
- Security scanning (audit, deny, outdated)
- Compilation time tracking
- Binary size analysis
- Andon Protocol (6-point signal check)

### 4.3 Andon Signal Protocol

**"Stop-the-line" quality gate**:

```bash
cargo make andon-check
```

**6 signal checks** (all must pass):

1. **Compiler Errors** — `cargo check`
2. **Compiler Warnings** — Warnings in check output
3. **Test Failures** — Actual failed tests
4. **Clippy Warnings** — Linter issues
5. **Code Formatting** — `cargo fmt --check`
6. **Documentation** — Doc builds without errors

If ANY signal is red → **STOP THE LINE** (exit 1)

**Output**:
```
🚦 ANDON CHECK 1: Compiler Errors
✅ ANDON GREEN: No compiler errors

🚦 ANDON CHECK 2: Compiler Warnings
✅ ANDON GREEN: No compiler warnings
...
✅ ALL ANDON SIGNALS GREEN - PROCEED WITH CONFIDENCE
```

### 4.4 Security Scanning

```bash
cargo make security-scan
```

Runs 3 tools:

| Tool | Check | Output |
|------|-------|--------|
| `cargo-audit` | Known CVEs in dependencies | "No known vulnerabilities" |
| `cargo-deny` | License compliance, advisory bans | License check passed |
| `cargo-outdated` | Dependency freshness | Up-to-date report |

**CI runs on every push** to main/develop/PR.

---

## 5. Test Coverage Analysis

### 5.1 Coverage Report Generation

```bash
cargo make coverage-report
```

**Tools used**: `cargo-tarpaulin` (line coverage via instrumentation)

**Auto-installs** if missing:
```
Installing cargo-tarpaulin... (first run)
```

**Output**:
```
Generating code coverage report...
✅ Coverage report generated in ./coverage/
Open ./coverage/index.html to view
Coverage: 84.2%
✅ Coverage meets 80% threshold
```

**Report location**: `./coverage/index.html` (HTML) + `./coverage/cobertura.xml` (CI consumption)

### 5.2 Coverage Threshold

- **Target**: 80% line coverage minimum
- **Exclusions**: `tests/*`, `benches/*`, `examples/*` (don't count toward coverage)
- **CI enforcement**: `frontier-ci.yml` fails if <80%

**Check coverage locally**:
```bash
# After running coverage-report
COVERAGE=$(grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml | head -1)
echo "Coverage: $((COVERAGE * 100))%"
```

### 5.3 Interpreting Coverage Gaps

**High coverage (>85%)** → Healthy codebase, most paths tested

**Medium coverage (80-85%)** → Acceptable, focus on untested branches

**Low coverage (<80%)** → Gap analysis needed:
- Which modules uncovered?
- Are they critical or dead code?
- Can refactor to reduce untestable code?

**Not all code needs testing**:
- Error handling paths (hard to trigger)
- CLI help/usage strings
- Dead code branches
- Feature-gated stubs

---

## 6. Flaky Test Detection and Fixing

### 6.1 Detecting Flaky Tests

**Symptom**: Test passes in isolation, fails randomly in CI or under load.

#### Method 1: Local Loop Testing

Run test 50 times, detect non-determinism:

```bash
#!/bin/bash
TESTS=50
FAILURES=0

for i in {1..50}; do
    if ! cargo test test_name --quiet; then
        ((FAILURES++))
    fi
done

if [ $FAILURES -gt 0 ]; then
    echo "❌ Flaky: $FAILURES/$TESTS failures"
else
    echo "✅ Deterministic: 50/50 passes"
fi
```

#### Method 2: Deterministic Mode Detection

```bash
# Run once in parallel (may pass)
cargo make test

# Run once single-threaded (may still fail if logic flaky)
cargo make test-lib-deterministic
```

If test fails in deterministic mode → **logic flaky** (not race condition)
If test passes single-threaded → **likely race condition**

#### Method 3: Use `loom` for Race Condition Testing

Project includes `loom` dev-dependency:

```rust
#[cfg(test)]
mod tests {
    use loom::thread;
    use loom::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_no_data_races() {
        loom::model(|| {
            let flag = Arc::new(AtomicBool::new(false));
            let flag_clone = flag.clone();

            thread::spawn(move || {
                flag_clone.store(true, Ordering::SeqCst);
            });

            // Race detection here
        });
    }
}
```

Run with: `cargo test --test '*' -- --ignored`

### 6.2 Root Causes of Flaky Tests

| Cause | Detection | Fix |
|-------|-----------|-----|
| **Shared state** | Fails sometimes in parallel, always in sequential | Use `serial_test` crate or isolate state |
| **Timing assumptions** | Fails under load | Remove `sleep()`, use proper synchronization |
| **File system races** | Fails when tests touch same files | Use `tempfile` with unique names per test |
| **Non-deterministic collections** | HashSet iteration order differs | Use `BTreeSet`, sort before comparing |
| **External dependencies** | Network, services fail randomly | Mock with `mockito`, test without network |
| **Random data** | Seed not set | Use `proptest` with seed control |

### 6.3 Fixing Flaky Tests

#### Fix: Serialized Tests (Sequential Execution)

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_shared_database_access() {
    let db = setup_db();
    db.insert("key", "value");
    assert_eq!(db.get("key"), "value");
}
```

**Cost**: Slower (can't run in parallel), but reliable.

#### Fix: Isolated Temporary Resources

```rust
use tempfile::TempDir;

#[test]
fn test_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    std::fs::write(&file_path, "content").unwrap();
    assert!(file_path.exists());
    // Auto-cleanup on drop
}
```

#### Fix: Mock External Services

```rust
#[test]
fn test_api_call() {
    let mock = mockito::mock("GET", "/api")
        .with_status(200)
        .with_body("response")
        .create();

    // Test makes HTTP request to mock
    let result = call_api();
    assert_eq!(result, "response");
    
    mock.assert();
}
```

#### Fix: Deterministic Data Generation

```rust
// Bad: random iteration order
#[test]
fn test_with_hashset() {
    let set: HashSet<_> = vec![1, 2, 3].into_iter().collect();
    let vec: Vec<_> = set.into_iter().collect();
    // Vec order is non-deterministic!
}

// Good: consistent order
#[test]
fn test_with_btree() {
    let set: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
    let vec: Vec<_> = set.into_iter().collect();
    // Vec order is always [1, 2, 3]
}
```

### 6.4 Testing Fixes with CI Simulation

After fixing flaky test:

```bash
# 1. Verify locally (deterministic mode)
cargo make test-lib-deterministic

# 2. Run 10 times in parallel
for i in {1..10}; do cargo make test || exit 1; done

# 3. Simulate full CI
cargo make ci

# 4. Only then: push and verify in GH Actions
git push origin branch-name
```

### 6.5 Pre-Commit Hook to Catch Flakes

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running deterministic test suite..."
cargo make test-lib-deterministic || {
    echo "❌ Flaky tests detected!"
    exit 1
}

echo "✅ All checks passed"
```

---

## 7. Test Organization

### 7.1 Test Directory Structure

```
clap-noun-verb/
├── tests/                          # Integration tests
│   ├── cli/                       # CLI-specific integration tests
│   ├── acceptance/                # Acceptance/E2E tests
│   ├── cli_builder.rs             # CliBuilder tests
│   ├── cli_router.rs              # CommandRouter tests
│   └── ... (40+ test files)
│
├── src/                            # Library source
│   ├── lib.rs                     # Entry point (has #[cfg(test)] unit tests)
│   ├── cli/                       # CLI modules (may have inline #[test])
│   ├── builder.rs                 # Tests in same file
│   └── ... (units tests inline)
│
├── clap-noun-verb-macros/         # Proc-macro crate
│   ├── src/
│   │   ├── lib.rs                 # Macro definitions
│   │   └── tests.rs               # Macro tests
│   └── tests/
│       └── ... (integration tests)
│
└── benches/                        # Criterion benchmarks
    └── dispatch.rs                # Command dispatch benchmarking
```

### 7.2 Test Naming Convention

Project follows AAA (Arrange-Act-Assert) with descriptive names:

```rust
#[test]
fn test_verb_command_executes_successfully_with_required_args() {
    // Arrange
    let registry = CommandRegistry::new();
    
    // Act
    let result = registry.dispatch("command", vec!["arg"]);
    
    // Assert
    assert!(result.is_ok());
}
```

**Pattern**: `test_<what>_<condition>_<expected>`

### 7.3 Test Annotations

```rust
#[test]
#[ignore]  // Excluded from normal runs, use --ignored flag
fn test_slow_operation() { ... }

#[test]
#[should_panic]  // Test passes if code panics
fn test_invalid_args_panic() { ... }

#[serial_test::serial]  // Run sequentially (not parallel)
fn test_shared_state() { ... }

#[serial_test::file_serial("db")]  // Multiple tests share same resource
fn test_database_1() { ... }

#[serial_test::file_serial("db")]
fn test_database_2() { ... }
```

---

## 8. Practical Workflows

### 8.1 Daily Development Workflow

```bash
# Start: Run quick tests
cargo make test

# Change code...

# Validate (deterministic)
cargo make test-lib-deterministic

# Before committing
cargo make lint  # format-check + clippy

# Just before push
cargo make ci    # Full local CI simulation
```

### 8.2 Frontier Feature Development

```bash
# Develop feature behind gate
# Edit Cargo.toml with new feature flag

# Test all combinations
cargo make test-frontier-matrix

# Or test specific feature
cargo test --features "my-new-feature" --quiet

# Benchmark if performance-critical
cargo make bench
```

### 8.3 Release Preparation

```bash
# Comprehensive release validation
cargo make release-validate

# This runs:
# - andon-check (all signals green)
# - test-frontier-matrix (all 23 combinations)
# - coverage-report (80% threshold)
# - bench-compare (vs. baseline)
# - slo-check (performance targets)
# - security-scan (CVEs, licenses)
# - build-release (release binary)
# - doc (documentation builds)
```

### 8.4 CI/GitHub Actions Workflow

**On Push to main**:
1. `.github/workflows/ci.yml` runs (standard CI)
2. `.github/workflows/frontier-ci.yml` runs (extended frontier tests)
3. `.github/workflows/audit.yml` runs (security/license checks)
4. `.github/workflows/docs-validation.yml` runs (docs.rs compatibility)

**View results**: GitHub PR/Push page → Checks tab

### 8.5 Debugging a CI Failure

```bash
# 1. Reproduce locally
cargo make ci

# 2. Identify which task failed
# If ci fails, check output of:
cargo make format-check
cargo make clippy
cargo make test-feature-combinations
cargo make test-unfailable
cargo make build-examples
cargo make check-all

# 3. Fix locally
# ... edit code ...

# 4. Re-validate
cargo make ci

# 5. Push when green
git push origin branch-name
```

### 8.6 Troubleshooting Test Hangs

If `cargo test` hangs (infinite loop, deadlock):

```bash
# 1. Interrupt (Ctrl+C)

# 2. Check for deadlocks with timeout
timeout 30 cargo make test

# 3. Run single-threaded (reveals deadlocks faster)
cargo make test-lib-deterministic

# 4. If still hangs, identify test:
cargo test test_name --quiet -- --nocapture
```

---

## 9. Performance Expectations

### 9.1 Test Execution Times

| Task | Duration | Parallelism | Notes |
|------|----------|-------------|-------|
| `cargo make test` | 0.1-0.5s | Parallel | Default, fastest feedback |
| `cargo make test-lib-deterministic` | 0.3-1.0s | Single-threaded | Slower, deterministic |
| `cargo make test-feature-combinations` | 1-3s | Sequential (3 runs) | Tests 3 feature sets |
| `cargo make test-all` | 2-5s | Parallel | All features, slower |
| `cargo make ci` | 30-60s | Parallel + sequential | Full validation pipeline |
| `cargo make bench` | 10-30s | Varies | Criterion statistical runs |
| `cargo make coverage-report` | 20-40s | Varies | Instrumentation overhead |

### 9.2 Resource Usage

| Check | CPU | RAM | Disk |
|-------|-----|-----|------|
| Tests | 2-4 cores | 512MB-1GB | 100MB |
| Benchmarks | 2-4 cores | 1GB+ | 50MB |
| Coverage | 1-2 cores | 512MB | 100MB |

### 9.3 SLO Targets

From `Makefile.toml` `slo-check`:

```
✅ Incremental Compilation: 0.66s (Target: ≤2s)
✅ Binary Size: 2.2MB (Target: ≤10MB)
```

---

## 10. Advanced Topics

### 10.1 Criterion Benchmark Customization

Edit `benches/dispatch.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_group!(benches, dispatch_benchmarks);
criterion_main!(benches);

fn dispatch_benchmarks(c: &mut Criterion) {
    c.bench_function("empty_command", |b| {
        b.iter(|| {
            // Benchmark code
            CommandRegistry::new().dispatch(black_box("cmd"), black_box(vec![]))
        })
    });

    // Warm up, multiple iterations, confidence intervals
    c.bench_function("complex_command", |b| {
        b.iter(|| {
            // Benchmark code
        })
    });
}
```

### 10.2 Proptest Property-Based Testing

For randomized invariant testing:

```rust
use proptest::proptest;

proptest! {
    #[test]
    fn test_verb_dispatch_preserves_input(args in ".*") {
        let registry = CommandRegistry::new();
        let result = registry.dispatch(&args, vec![]);
        // Invariant: output should be deterministic for same input
        prop_assert_eq!(
            registry.dispatch(&args, vec![]),
            result
        );
    }
}
```

### 10.3 Custom Test Harness

For feature-gated tests:

```rust
// In Cargo.toml:
[[test]]
name = "custom_harness"
path = "tests/custom_harness.rs"
harness = false  # Use custom main()

// In tests/custom_harness.rs:
fn main() {
    #[cfg(feature = "frontier-semantic")]
    {
        // Only run if feature enabled
        test_semantic_features();
    }
}
```

### 10.4 Continuous Coverage Tracking

In CI (frontier-ci.yml), coverage is uploaded to Codecov:

```bash
codecov/codecov-action@v3
```

View historical coverage: https://app.codecov.io/gh/seanchatmangpt/clap-noun-verb

---

## 11. Reference: All Makefile Tasks

### Testing Tasks

| Task | Purpose | Feature Scope |
|------|---------|---------------|
| `test` | Quick parallel tests | Default features |
| `test-default` | Tests with default features | Default |
| `test-no-features` | Tests without any features | No features |
| `test-repl` | REPL feature tests | repl only |
| `test-feature-combinations` | All 3 feature combos | Default, No, REPL |
| `test-lib-deterministic` | Deterministic lib tests | Default (single-threaded) |
| `test-integration-isolated` | Isolated integration tests | Default (single-threaded) |
| `test-unfailable` | All tests deterministically | Default (unfailable arch) |
| `test-all` | All features enabled | all-features |
| `test-integration` | Integration examples (ignored) | all-features |
| `test-timeout` | Tests with 10ms timeout cap | all-features |
| `test-frontier` | All features (alias for test-all) | all-features |
| `test-frontier-all` | All frontier features | frontier-all |
| `test-frontier-minimal` | No features | no-default-features |
| `test-frontier-matrix` | 23-point feature matrix | All combinations |

### Benchmark Tasks

| Task | Purpose |
|------|---------|
| `bench` | Run all benchmarks (criterion) |
| `bench-baseline` | Save benchmarks as `main` baseline |
| `bench-compare` | Compare against `main` baseline |
| `bench-phase1` | Foundation benchmarks only |
| `bench-phase2` | RDF/Semantic benchmarks |
| `bench-phase3` | Optimization & ML benchmarks |
| `bench-phase4` | Advanced features benchmarks |
| `profile` | Benchmarks with profiling enabled |

### Quality & CI Tasks

| Task | Purpose |
|------|---------|
| `format-check` | Verify code formatting |
| `format` | Auto-format code |
| `clippy` | Run linter (deny warnings) |
| `lint` | Format check + clippy |
| `ci` | Full CI equivalent (local) |
| `verify` | Quick verification (format, clippy, test-timeout) |
| `andon-check` | 6-point signal validation |
| `security-scan` | Audit + deny + outdated checks |
| `coverage-report` | Generate coverage report (80% threshold) |
| `slo-check` | Performance SLO validation |

### Build Tasks

| Task | Purpose |
|------|---------|
| `build` | Debug build |
| `build-release` | Release build (optimized) |
| `build-examples` | Build all example binaries |
| `build-frontier-all` | Build with all frontier features |
| `check` | Compile check (no binary) |
| `check-all` | Check with all features |
| `check-frontier-all` | Check frontier features |

### Documentation

| Task | Purpose |
|------|---------|
| `doc` | Build docs (docs.rs compatible) |
| `doc-open` | Build and open in browser |

### Release & Publishing

| Task | Purpose |
|------|---------|
| `release-check` | Pre-release validation suite |
| `release-validate` | Comprehensive release validation |
| `publish-dry-run-macros` | Dry-run publish macros crate |
| `publish-macros` | Publish macros crate to crates.io |
| `publish-dry-run` | Dry-run publish main crate |
| `publish` | Publish main crate to crates.io |
| `publish-all` | Full publish workflow (both crates) |

### Utilities

| Task | Purpose |
|------|---------|
| `clean` | Remove build artifacts |
| `audit` | Security audit |
| `verify-frontier` | Verify frontier features compile |
| `verify-publish` | Search crates.io for published crate |

---

## 12. Checklist for Test Success

### Pre-Commit

- [ ] `cargo make test-lib-deterministic` passes
- [ ] `cargo make format-check` passes
- [ ] No new warnings from `cargo make clippy`

### Before Push

- [ ] `cargo make ci` passes (full local CI)
- [ ] Flaky test checks (`for i in {1..5}; do cargo make test || exit 1; done`)

### Before Release

- [ ] `cargo make release-validate` passes (comprehensive suite)
- [ ] Review `cargo make coverage-report` output (>80%)
- [ ] `cargo make bench-compare` shows no major regressions
- [ ] `cargo make security-scan` reports no CVEs

### In CI (GitHub Actions)

- [ ] All `ci.yml` jobs pass (fmt, clippy, test, nextest, msrv, docs, audit, licenses)
- [ ] `frontier-ci.yml` jobs pass (feature-matrix, coverage, benchmarks, security, andon, binary-size)
- [ ] Build artifacts available in GitHub Actions summary

---

## 13. Related Documentation

- **CLAUDE.md**: Project overview, architecture, critical rules
- **Makefile.toml**: Complete task definitions (this guide supplements)
- **.github/workflows/**: CI/CD pipeline implementations
- **tests/README.md**: Test structure and conventions
- **Cargo.toml**: Features, dependencies, dev-dependencies

---

**Last Updated**: 2025-06-14 | **Version**: 26.9.1

For questions or improvements, refer to the project's GitHub Issues or CLAUDE.md guidelines.
