# Performance Monitoring & Regression Detection

**Quick Start**: Run `./scripts/measure_performance.sh` to validate all SLOs locally.

This guide covers continuous performance monitoring, regression detection, and decision-making.

---

## Table of Contents

1. [Local Performance Validation](#local-performance-validation)
2. [CI/CD Integration](#cicd-integration)
3. [Regression Detection & Alerting](#regression-detection--alerting)
4. [Baseline Management](#baseline-management)
5. [Performance Profiling](#performance-profiling)
6. [Troubleshooting](#troubleshooting)

---

## Local Performance Validation

### Before Every Commit

```bash
# Quick validation (< 1 minute)
cargo make slo-check

# Or use the measurement script
./scripts/measure_performance.sh

# Expected output:
# ✅ Incremental compile: 660ms (threshold: 2000ms)
# ✅ Binary size: 2MB (threshold: 10MB)
# ✅ Test suite: 160ms (threshold: 1000ms)
# ✅ Doc build: 4s (threshold: 15s)
```

### Before Major Changes

```bash
# Save a baseline before significant refactoring
cargo make bench-baseline

# Work on your changes...

# After changes, compare
cargo make bench-compare

# If acceptable, update baseline
cargo make bench-baseline
```

### Full Performance Audit

```bash
# Comprehensive check (5-10 minutes)
cargo make release-validate

# This runs:
# - format-check
# - andon-check (compiler warnings, test failures)
# - test-frontier-matrix (all feature combinations)
# - coverage-report
# - bench-compare
# - slo-check
# - security-scan
# - build-release
# - doc
```

---

## CI/CD Integration

### GitHub Actions Workflow

The project includes `.github/workflows/performance.yml` which:

1. **Measures on every PR**: Compilation time, binary size, test speed
2. **Saves baselines on main**: Stores benchmark results for comparison
3. **Detects regressions**: Warns on significant performance drops
4. **Analyzes dependencies**: Flags duplicate/heavy dependencies

### Viewing CI Results

```bash
# Check workflow runs
gh run list --branch main --workflow performance.yml

# View latest run
gh run view --log

# Download artifact
gh run download <run-id> -n performance-results
open performance-results/report/index.html
```

### Expected CI Time

```
Performance (ubuntu-latest):
  - Incremental compile: 1-2s
  - Release build: 3-5s
  - Tests: 0.5-1s
  - Benchmarks: 30-40s
  - Total: ~2-3 minutes
```

---

## Regression Detection & Alerting

### 1. Compile-Time Regressions

**Threshold**: > 2.0s incremental compile

**Detection**:
```bash
# In CI
./scripts/measure_performance.sh

# Or manually
touch src/lib.rs
time cargo build --quiet
# If > 2000ms, investigate
```

**Root Cause Analysis**:
```bash
# 1. Check dependency changes
git diff Cargo.toml | grep version

# 2. Identify slow crate
cargo build -v 2>&1 | grep "Compiling" | sort -k4 -rn | head -5

# 3. Check for new features
git diff Cargo.toml | grep features

# 4. Profile with flamegraph
cargo install flamegraph
RUSTFLAGS="-Z time-passes" cargo build 2>&1 | grep "time:"
```

**Fix Strategies**:
- ✅ Gate new dependency behind feature flag
- ✅ Use optional dependencies with `dep:name` syntax
- ✅ Split into separate crate if possible
- ❌ Never add unoptimized dependencies to default features

### 2. Binary Size Regressions

**Threshold**: > 10MB release binary

**Detection**:
```bash
# Automated in CI
cargo build --release --quiet
SIZE=$(du -b target/release/clap_noun_verb | cut -f1)
if [ $SIZE -gt $((10 * 1024 * 1024)) ]; then
    echo "REGRESSION: $((SIZE / 1024 / 1024))MB > 10MB"
    exit 1
fi
```

**Root Cause Analysis**:
```bash
# 1. Check feature impact
cargo build --release --no-default-features
du -h target/release/clap_noun_verb  # Baseline

cargo build --release --all-features
du -h target/release/clap_noun_verb  # With all features

# 2. Identify heavy features
for feature in repl otel federated-network; do
    cargo build --release --features $feature --quiet
    du -h target/release/clap_noun_verb
done

# 3. Check for debug symbols
file target/release/clap_noun_verb
# If "not stripped", symbols are included
```

**Fix Strategies**:
- ✅ Enable `strip = true` in `[profile.release]`
- ✅ Remove unused features
- ✅ Gate heavy features behind flags
- ❌ Never use full LTO just to reduce size (too slow)

### 3. Test Execution Regressions

**Threshold**: > 1.0s full test suite

**Detection**:
```bash
# Time test execution
time cargo test --quiet

# If > 1000ms, investigate
# Expected: 160ms (parallel)
```

**Root Cause Analysis**:
```bash
# 1. Find slow test
cargo test --quiet -- --nocapture --test-threads=1 2>&1 | \
    grep -E "test result:|test .* ok" | tail -20

# 2. Check for I/O in tests
grep -r "fs::" tests/ | head -10
grep -r "sleep\|delay" tests/ | head -10

# 3. Check for heavy dependencies
grep -r "expensive_crate::" tests/ | head -10

# 4. Profile test execution
cargo test --lib -- --nocapture --test-threads=1
```

**Fix Strategies**:
- ✅ Mock file I/O with test fixtures
- ✅ Use `black_box()` to prevent compiler optimizations
- ✅ Mark slow tests with `#[ignore]` (run separately)
- ❌ Never use `std::thread::sleep` in tests

### 4. Benchmark Regressions

**Threshold**: > 5% slower than baseline

**Detection**:
```bash
# Compare to baseline
cargo make bench-compare

# Criterion output shows change percentage
# Example: change: [-2% +1% +4%]
# If lower bound > 5%, regression detected
```

**Root Cause Analysis**:
```bash
# 1. Check recent commits
git log --oneline -10

# 2. Profile with perf (Linux)
cargo build --release
perf record -F 99 target/release/clap_noun_verb
perf report  # Shows hotspots

# 3. Check for added allocations
git diff HEAD~1..HEAD -- src/
# Look for Vec::new, String::from, etc. in hot paths

# 4. Verify measurement noise
cargo bench --bench dispatch -- --baseline main --verbose
# Show confidence intervals
```

**Fix Strategies**:
- ✅ Reduce allocations in hot paths
- ✅ Cache computed values
- ✅ Use references instead of clones
- ❌ Never sacrifice correctness for speed

---

## Baseline Management

### Saving Baselines

```bash
# After optimizations or before major changes
cargo make bench-baseline

# This creates: target/criterion/*/base/raw.json
# Baseline is named "main" by default

# List available baselines
ls target/criterion/dispatch/
# Shows: base/, main/
```

### Comparing Baselines

```bash
# Compare current code to "main" baseline
cargo make bench-compare

# Compare to specific baseline (if multiple exist)
cargo bench -- --baseline my_baseline

# Output format:
# dispatch_verb_lookup              time:   [10.2 µs 10.3 µs 10.5 µs]
#                                   change: [-2.1% +0.5% +3.2%]
#                                   (within noise margin, no regression)
```

### Resetting Baselines

```bash
# Delete baseline (useful after intentional optimization)
rm -rf target/criterion/*/main/

# Save new baseline after changes
cargo make bench-baseline

# Commit the new baseline (optional)
# git add target/criterion/
# git commit -m "Baseline update after optimization"
```

### CI Baseline Updates

On the `main` branch, CI automatically saves benchmarks:

```yaml
# .github/workflows/performance.yml
- name: Save benchmark baseline
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  run: cargo bench --all-features -- --save-baseline main
```

This ensures the baseline stays current.

---

## Performance Profiling

### Compile-Time Profiling

**Goal**: Identify which crates/files take longest to compile.

```bash
# Time each compilation phase
RUSTFLAGS="-Z time-passes" cargo build 2>&1 | grep "time:" | sort -k3 -rn | head -20

# Example output:
# parse: 2ms [==================]
# macro_expand: 8ms [==================]
# codegen_mono: 450ms [==================]

# Profile specific crate
cargo build -p clap --release -v 2>&1 | grep "Compiling\|Finished"
```

### Runtime Profiling

**Goal**: Identify hot paths and bottlenecks.

```bash
# Using flamegraph (Linux/macOS)
cargo install flamegraph

# Profile benchmark
cargo flamegraph --bench dispatch -- --profile-time 10

# Output: flamegraph.svg (open in browser)
```

### Memory Profiling

**Goal**: Detect memory leaks and peak usage.

```bash
# Using valgrind (Linux)
valgrind --leak-check=full \
    --show-leak-kinds=all \
    --track-origins=yes \
    target/debug/myapp command arg

# Using /usr/bin/time
/usr/bin/time -v cargo build 2>&1 | grep "Maximum resident"
# Output: Maximum resident set size: 450000 Kbytes
```

### Dependency Profiling

**Goal**: Understand which dependencies are expensive.

```bash
# Build times per dependency
cargo build --release -v 2>&1 | \
    awk '/Compiling/ {print $2}' | \
    while read dep; do
        echo -n "$dep: "
        cargo build -p "$dep" --release 2>&1 | grep "Finished" || echo "N/A"
    done

# Dependency tree with sizes
cargo tree --depth 1

# Check for duplicates
cargo tree --duplicates
```

---

## Troubleshooting

### "Compile time is 3s instead of 0.66s"

**Diagnosis**:
```bash
# Check for heavy feature
cargo build --no-default-features --quiet
# If fast, default features are the issue

# Identify which
cargo tree --depth 1
cargo build -v 2>&1 | grep "Compiling" | sort -k4 -rn | head -5
```

**Solutions**:
1. Check Cargo.toml for new dependencies (revert if not needed)
2. Gate heavy dependencies behind features
3. Update dependencies (newer versions may be optimized)
4. Check for `cargo update` adding new versions

### "Binary is 15MB instead of 2.2MB"

**Diagnosis**:
```bash
# Check if debug symbols present
file target/release/clap_noun_verb
# If "not stripped", symbols are the issue

# Check with features
cargo build --release --no-default-features --quiet
du -h target/release/clap_noun_verb
```

**Solutions**:
1. Add `strip = true` to `[profile.release]` in Cargo.toml
2. Disable unnecessary features
3. Check for vendored dependencies (bloat)

### "Tests take 5s instead of 0.16s"

**Diagnosis**:
```bash
# Find slow test
RUST_TEST_THREADS=1 cargo test --lib --quiet 2>&1 | \
    grep -oP 'test \K[^ ]+' | \
    while read test; do
        echo -n "$test: "
        time cargo test "$test" --lib --quiet 2>&1
    done
```

**Solutions**:
1. Remove I/O from tests (use fixtures)
2. Mock external dependencies
3. Mark slow tests with `#[ignore]`
4. Run tests in parallel (RUST_TEST_THREADS not set)

### "Benchmarks show unexpected variance"

**Diagnosis**:
```bash
# Run with more samples
cargo bench -- --verbose --sample-size 1000

# Check for system load
top -b -n 1 | head -5
# If high load, reschedule benchmark

# Run multiple times
for i in {1..5}; do
    cargo bench -- --save-baseline run$i 2>&1 | tail -3
done
```

**Solutions**:
1. Close other applications
2. Increase sample size (more stable)
3. Use isolated system (if critical)
4. Rerun if variance > 10%

---

## References

- [PERFORMANCE_STANDARDS.md](../PERFORMANCE_STANDARDS.md) — Detailed standards
- [PERFORMANCE_GUIDE.md](PERFORMANCE_GUIDE.md) — Practical how-to
- [Criterion.rs docs](https://bheisler.github.io/criterion.rs/)
- [Cargo build cache](https://doc.rust-lang.org/cargo/guide/build-cache.html)
- [Flamegraph guide](https://www.brendangregg.com/flamegraphs.html)

---

**Last Updated**: 2026-08-20  
**Version**: 26.9.1  
**Maintained by**: clap-noun-verb contributors
