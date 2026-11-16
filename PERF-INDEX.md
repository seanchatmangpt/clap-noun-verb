# Performance Documentation Index

**clap-noun-verb v26.9.1** — Complete performance guide and reference materials.

## Quick Links

| Document | Purpose | Length | Time to Read |
|----------|---------|--------|--------------|
| [PERFORMANCE-QUICK-REF.md](./PERFORMANCE-QUICK-REF.md) | **Start here** — Commands, SLO status, quick fixes | 296 lines | 5 min |
| [PERFORMANCE.md](./PERFORMANCE.md) | **Complete guide** — All strategies, tools, CI/CD | 1068 lines | 30 min |
| [benches/dispatch.rs](./benches/dispatch.rs) | Benchmark suite — Criterion.rs setup | 186 lines | 10 min |
| [PERF-INDEX.md](./PERF-INDEX.md) | This file — Navigation & overview | — | 3 min |

---

## What's Covered

### 1. Incremental Compilation Optimization

**Current: 0.66s** | **Target: ≤2s** | **Status: ✅ PASS (67% margin)**

**In PERFORMANCE.md:**
- Section 2: [Incremental Compilation Optimization](./PERFORMANCE.md#incremental-compilation-optimization)
- 6 proven strategies:
  1. Cargo incremental compilation (default, built-in)
  2. Codegen units configuration (256 in dev, 16 in release)
  3. Macro crate overhead reduction (~0.15s of build time)
  4. Dependency bloat detection (cargo-build-times, cargo-tree)
  5. Sccache distributed compilation cache (50-70% speedup on hits)
  6. Parallel compilation with explicit job counts

**Quick commands:**
```bash
cargo build                           # 0.66s expected
time cargo build --quiet              # Measure with timing
cargo make check-all                  # Check with all features
```

**Tools:**
- `cargo-build-times` — breakdown by crate
- `sccache` — distributed cache
- `mold` — ultra-fast linker (30-50% faster)

---

### 2. Binary Size Management

**Current: 2.2MB** | **Target: ≤10MB** | **Status: ✅ PASS (78% margin)**

**In PERFORMANCE.md:**
- Section 3: [Binary Size Management](./PERFORMANCE.md#binary-size-management)
- Measurement techniques with expected outputs
- Feature-by-feature size impact analysis
- LTO configuration (thin = fast, fat = slower but smaller)
- Stripping debug symbols for additional 10-15% reduction

**Quick commands:**
```bash
cargo build --release && ls -lh target/release/clap-noun-verb
cargo bloat --release -n 20           # Top 20 functions by size
strip target/release/clap-noun-verb   # Remove debug symbols
```

**Feature impact (expected):**
- No features: ~1.9MB
- +repl: ~2.1MB
- +otel: ~2.3MB
- All features: ~2.5-2.8MB

---

### 3. Test Execution Optimization

**Current: 0.3-0.6s** | **Target: <1s (parallel)** | **Status: ✅ PASS (40-70% margin)**

**In PERFORMANCE.md:**
- Section 4: [Test Execution Optimization](./PERFORMANCE.md#test-execution-optimization)
- Parallel test execution strategies
- Incremental test compilation
- Feature-specific test runs
- Coverage analysis with tarpaulin (80% threshold)

**Quick commands:**
```bash
cargo make test                       # Quick parallel tests (<1s)
cargo make test-lib-deterministic     # Single-threaded deterministic
cargo make test-feature-combinations  # Full feature matrix
cargo make coverage-report            # Generate coverage report
```

**Expected performance:**
- Quick tests (default): 0.3-0.6s
- With all features: 0.8-1.2s
- Feature matrix (23 combinations): 30-60s

---

### 4. Benchmarking Framework

**In PERFORMANCE.md:**
- Section 5: [Benchmarking Framework](./PERFORMANCE.md#benchmarking-framework)

**In benches/dispatch.rs:**
Criterion.rs benchmarks for:
- Command registry lookup (linear search, O(n))
- Argument parsing simulation
- Command dispatch paths
- Serialization/deserialization (JSON)
- Error handling paths
- String operations (noun::verb separation)

**Quick commands:**
```bash
cargo make bench                      # Run all benchmarks
cargo make bench-baseline             # Save baseline
cargo make bench-compare              # Compare to baseline
cargo bench --bench dispatch          # Run dispatch benchmarks only
```

**Understanding output:**
```
time: [125.43 us 126.51 us 127.61 us]     ← Confidence interval (95%)
change: [-2.34% -0.91% +0.49%]            ← Regression/improvement
R-square: 0.9834                          ← Goodness of fit (>0.98 = excellent)
```

---

### 5. Macro & Code Generation Profiling

**In PERFORMANCE.md:**
- Section 6: [Macro & Code Generation Profiling](./PERFORMANCE.md#macro--code-generation-profiling)

**Techniques:**
- Macro expansion analysis with `cargo expand`
- Per-function macro cost measurement
- Proc-macro compilation overhead (typically 15% of build time)
- Flamegraph visualization of hot paths

**Quick commands:**
```bash
cargo expand --lib | wc -l            # Expansion size (~500-800 lines)
time cargo build -p clap-noun-verb-macros --quiet  # Macro overhead
cargo install cargo-expand && cargo expand --lib   # Inspect expansion
cargo install flamegraph && cargo flamegraph --bench dispatch
```

**Expected overhead:**
- Macro crate compilation: ~0.06s
- Macro expansion per 10 verbs: ~0.02-0.03s
- Total macro overhead: ~15% of build time

---

### 6. Feature Compilation Impact Analysis

**In PERFORMANCE.md:**
- Section 7: [Feature Compilation Impact Analysis](./PERFORMANCE.md#feature-compilation-impact-analysis)

**23-Feature Combination Matrix:**
- Tier 0: Baseline (no features)
- Tier 1: Individual features (meta-framework, rdf-composition, etc.)
- Tier 2: Meta-features (frontier-semantic, frontier-intelligence, etc.)
- Tier 3: Critical combinations
- Tier 4: Extremes (frontier-all, no-default-features)

**Quick commands:**
```bash
cargo make test-frontier-matrix       # Test all 23 combinations
cargo check --features "meta-framework,rdf-composition"
cargo make test-frontier-all          # Test with all features

# Individual feature timing
for feature in meta-framework rdf-composition federated-network; do
  echo "Testing: $feature"
  time cargo build --features "$feature" --quiet
done
```

**Expected impact:**
- No features → +meta-framework: +0.01s
- No features → +federated-network: +0.01-0.02s
- No features → frontier-all: +0.08s total

---

### 7. Caching Strategies

**In PERFORMANCE.md:**
- Section 8: [Caching Strategies](./PERFORMANCE.md#caching-strategies)

**4-Level Caching Hierarchy:**

1. **Level 1: Cargo Incremental** (built-in, no setup)
   - Tracks file-level changes
   - Skips recompilation of unchanged files
   - Default: enabled

2. **Level 2: Sccache** (distributed, 50-70% speedup)
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   sccache -s  # Check statistics
   ```

3. **Level 3: mold Linker** (30-50% linking speedup)
   ```bash
   brew install mold  # macOS
   # Or uncomment in .cargo/config.toml
   ```

4. **Level 4: Cargo.lock** (dependency resolution)
   - Committed to repo (already done)
   - Prevents re-resolution on every build

**GitHub Actions example:**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: "."
```

---

### 8. Profiling Tools & Workflows

**In PERFORMANCE.md:**
- Section 9: [Profiling Tools & Workflows](./PERFORMANCE.md#profiling-tools--workflows)

**Available tools:**

| Tool | Purpose | Command |
|------|---------|---------|
| `cargo-build-times` | Crate-by-crate breakdown | `cargo build-times` |
| `cargo-bloat` | Function-level code size | `cargo bloat --release -n 20` |
| `cargo-expand` | Macro expansion inspection | `cargo expand --lib` |
| `cargo-tree` | Dependency graph | `cargo tree --depth 1` |
| `cargo-udeps` | Unused dependencies | `cargo +nightly udeps` |
| `perf` | Linux profiling | `perf record cargo build` |
| `flamegraph` | Visualization | `cargo flamegraph --bench dispatch` |

---

### 9. CI/CD Integration

**In PERFORMANCE.md:**
- Section 10: [CI/CD Integration](./PERFORMANCE.md#cicd-integration)

**GitHub Actions Example:**
- SLO validation step
- Binary size checking
- Benchmark comparison
- Artifact upload
- See PERFORMANCE.md for full workflow YAML

**Pre-commit Hook Example:**
- Validates incremental compile time
- Checks binary size
- Runs quick tests
- See PERFORMANCE.md for bash script

---

### 10. Troubleshooting Guide

**In PERFORMANCE.md:**
- End of document: [Troubleshooting](./PERFORMANCE.md#troubleshooting)

**Common issues & solutions:**

| Problem | Root Cause | Command | Expected Result |
|---------|-----------|---------|-----------------|
| Build >2s | New dependency? | `git diff Cargo.toml` | Identify bloat |
| Build slow | Incremental broken | `cargo clean && time cargo build` | Fresh baseline |
| Binary >10MB | New features? | `git diff Cargo.toml` | Feature impact |
| Tests >1s | Blocking ops? | `grep -r sleep tests/` | Find culprits |
| Macro slow | Complex derives? | `cargo expand --lib \| wc -l` | Measure expansion |

---

## SLO Status Dashboard

```
╔══════════════════════════════════════════════════════════════════╗
║           CLAP-NOUN-VERB PERFORMANCE SLO STATUS                  ║
╚══════════════════════════════════════════════════════════════════╝

Metric                 Current    Target     Status      Margin
─────────────────────────────────────────────────────────────────
Incremental Compile    0.66s      ≤2s        ✅ PASS     67%
Binary Size            2.2MB      ≤10MB      ✅ PASS     78%
Test Suite             0.3-0.6s   <1s        ✅ PASS     40-70%
─────────────────────────────────────────────────────────────────
Overall Status:        ALL SLOs PASSING ✅ (with comfortable margins)
```

**Verification Command:**
```bash
cargo make slo-check
```

---

## Quick Command Reference

### Development (Daily Use)

```bash
# Fast iterative build
cargo build                          # ~0.66s incremental

# Quick testing
cargo make test                      # ~0.3-0.6s

# Check without linking
cargo check                          # Fastest

# Single test
cargo test test_name --quiet         # Focused testing

# Lint & format
cargo make format-check && cargo make clippy
```

### Diagnostics (When Performance Degrades)

```bash
# Check what changed
git status && git diff Cargo.toml

# Measure build time
time cargo build --quiet

# Profile specific crate
cargo install cargo-build-times && cargo build-times

# Check binary size
ls -lh target/release/clap-noun-verb
cargo bloat --release -n 20

# Analyze macro overhead
cargo expand --lib | wc -l
```

### Benchmarking (Regular Cadence)

```bash
# Run benchmarks
cargo make bench

# Compare to baseline
cargo make bench-compare

# Save new baseline
cargo make bench-baseline
```

### Feature Testing (Before Releases)

```bash
# Test all 23 feature combinations
cargo make test-frontier-matrix

# Test with all features
cargo make test-frontier

# Test minimal features
cargo make test-frontier-minimal
```

### Caching (Team Optimization)

```bash
# Enable sccache
export RUSTC_WRAPPER=sccache

# Check cache statistics
sccache -s

# Clear cache if needed
cargo clean && rm -rf ~/.cargo/registry/cache/
```

---

## Integration with Existing Tools

The performance guide integrates seamlessly with existing project infrastructure:

### Makefile.toml Tasks

- `cargo make test` — Already optimized for parallelism
- `cargo make bench` — Now includes dispatch benchmarks
- `cargo make slo-check` — Validates all three SLOs
- `cargo make test-frontier-matrix` — Tests 23 feature combinations
- `cargo make coverage-report` — Tarpaulin-based coverage

### CLAUDE.md Integration

The [CLAUDE.md](./CLAUDE.md) critical rules align with performance:

- Error handling rules prevent panic overhead
- Logging restrictions (no println) reduce bloat
- Trait design (no async in traits) simplifies compilation
- Testing best practices support subsecond execution

### GitHub Actions CI

Example workflows provided in PERFORMANCE.md for:
- SLO validation on every PR
- Binary size checking
- Test execution monitoring
- Benchmark comparison on releases

---

## Maintenance & Review Schedule

### Quarterly (Every 3 months)

```bash
# Full performance review
cargo make release-validate

# Compare benchmarks
cargo make bench-compare

# Check SLO compliance
cargo make slo-check
```

**Next review date: 2026-09-14**

### Monthly (Spot checks)

```bash
# Quick SLO validation
cargo make slo-check

# Verify no regressions
cargo make test
```

### Per-PR (Automated in CI)

- Binary size check (must stay ≤10MB)
- Test execution time (must stay <1s)
- Compilation time validation

---

## Further Reading

### Official References

- [Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Criterion.rs Documentation](https://docs.rs/criterion/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

### Tools & Projects

- [mold — Fast Linker](https://github.com/rui314/mold)
- [sccache — Distributed Cache](https://github.com/mozilla/sccache)
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat)
- [cargo-expand](https://github.com/dtolnay/cargo-expand)

### Related Documentation

- [CLAUDE.md](./CLAUDE.md) — Architecture & critical rules
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Development guidelines
- [Makefile.toml](./Makefile.toml) — Build task definitions

---

## File Manifest

### Primary Documentation

- **PERFORMANCE.md** (28 KB)
  - Comprehensive 15-section guide
  - 1068 lines of detailed documentation
  - 50+ actionable commands
  - CI/CD integration examples
  - Troubleshooting guide

- **PERFORMANCE-QUICK-REF.md** (8 KB)
  - One-page developer reference
  - Quick command lookup
  - Scenario-based troubleshooting
  - SLO status at a glance

### Benchmark Suite

- **benches/dispatch.rs** (8 KB)
  - Criterion.rs benchmarks
  - 6 benchmark groups (50-200 operations each)
  - Registry lookup, parsing, dispatch, serialization
  - Error handling & string operations
  - Sample size: 100 iterations for statistical significance

### Configuration Examples

- **.cargo/config.example.toml** (in documentation)
  - mold linker setup
  - sccache configuration
  - Parallel job tuning
  - Profile settings reference

---

## Getting Started

1. **First time?** → Start with [PERFORMANCE-QUICK-REF.md](./PERFORMANCE-QUICK-REF.md) (5 min)
2. **Deep dive?** → Read [PERFORMANCE.md](./PERFORMANCE.md) (30 min)
3. **Contributing?** → Run benchmarks with [benches/dispatch.rs](./benches/dispatch.rs)
4. **Troubleshooting?** → Check the [Troubleshooting](./PERFORMANCE.md#troubleshooting) section

---

**Document Version**: 1.0  
**Last Updated**: 2026-06-14  
**Maintained By**: Sean Chatman (project author)

For updates or corrections, see [CONTRIBUTING.md](./CONTRIBUTING.md).
