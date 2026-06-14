# Performance Benchmarking Methodology

**Version:** 1.0.0
**Date:** 2026-01-05
**Author:** Performance Benchmarker Agent

---

## Overview

This document defines the **comprehensive benchmarking methodology** for clap-noun-verb performance validation across all 5 development phases. It provides:

- **Standardized measurement techniques** for reproducible results
- **Tool configuration** for Criterion, Flamegraph, and profiling
- **Statistical rigor** with confidence intervals and regression detection
- **Baseline management** for tracking performance over time
- **Best practices** for writing effective benchmarks

---

## 1. Benchmarking Tools

### 1.1 Criterion.rs (Primary Tool)

**Purpose**: Statistical benchmarking with automatic outlier detection and HTML reports

**Configuration**: Located in each benchmark file
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_operation(c: &mut Criterion) {
    c.bench_function("operation_name", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(expensive_operation());
        });
    });
}

criterion_group!(benches, bench_operation);
criterion_main!(benches);
```

**Key Settings**:
- **Measurement Time**: 10 seconds (15s for complex operations)
- **Sample Size**: 100 iterations (auto-adjusted for fast/slow operations)
- **Warmup**: 3 seconds before measurement
- **Significance Level**: 5% (p = 0.05)

### 1.2 Rust Test Framework (SLO Validation)

**Purpose**: Pass/fail validation against Service Level Objectives

**Configuration**: Using `#[test]` attribute
```rust
#[test]
fn slo_operation_under_target() {
    let start = Instant::now();
    perform_operation();
    let duration = start.elapsed();

    assert!(duration.as_millis() < TARGET,
            "SLO VIOLATION: took {}ms (target: <{}ms)",
            duration.as_millis(), TARGET);
}
```

**When to Use**:
- Binary pass/fail validation (operation meets SLO or doesn't)
- CI/CD gates (fail pipeline if SLO violated)
- Quick smoke tests before detailed Criterion benchmarks

### 1.3 Flamegraph (Profiling)

**Purpose**: Visual profiling to identify hot paths

**Installation**:
```bash
cargo install flamegraph
```

**Usage**:
```bash
# Profile specific benchmark
cargo flamegraph --bench phase3_optimization_benchmarks -- bench_pso

# Open generated flamegraph
open flamegraph.svg
```

**Interpretation**:
- **Width**: Time spent in function (wider = more time)
- **Height**: Call stack depth
- **Color**: No semantic meaning (random differentiation)

---

## 2. Benchmark Categories

### 2.1 Microbenchmarks

**Definition**: Measure individual operations in isolation

**Examples**:
- RDF triple creation
- JSON serialization
- Type-state transition

**Best Practices**:
- Use `black_box()` to prevent compiler optimization
- Minimize setup overhead (move outside measurement)
- Test one thing at a time

**Template**:
```rust
fn bench_micro(c: &mut Criterion) {
    c.bench_function("operation", |b| {
        b.iter(|| {
            black_box(single_operation());
        });
    });
}
```

### 2.2 Throughput Benchmarks

**Definition**: Measure operations per second

**Examples**:
- Economic simulation step (agents per second)
- RDF triple creation (triples per second)

**Best Practices**:
- Use `Throughput::Elements(count)` or `Throughput::Bytes(size)`
- Report both latency and throughput
- Test at scale (10, 100, 1000, 10000 elements)

**Template**:
```rust
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| process_batch(black_box(count)));
            },
        );
    }

    group.finish();
}
```

### 2.3 End-to-End Benchmarks

**Definition**: Measure complete workflows with real dependencies

**Examples**:
- Full SPARQL federation query across 3 peers
- Complete economic simulation step (100K agents)

**Best Practices**:
- Include setup/teardown in measurement if realistic
- Test with production-like data sizes
- Measure variance across runs

**Template**:
```rust
fn bench_e2e(c: &mut Criterion) {
    c.bench_function("workflow", |b| {
        b.iter(|| {
            let system = setup_system();
            let result = system.execute_workflow();
            black_box(result)
        });
    });
}
```

### 2.4 Comparative Benchmarks

**Definition**: Compare different implementations or algorithms

**Examples**:
- PSO vs Genetic Algorithm vs Differential Evolution
- Custom RDF implementation vs Oxigraph

**Best Practices**:
- Use identical input data
- Run all variants in same group
- Calculate speedup ratio

**Template**:
```rust
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithm_comparison");

    let input = setup_test_data();

    group.bench_function("algorithm_a", |b| {
        b.iter(|| algorithm_a(black_box(&input)));
    });

    group.bench_function("algorithm_b", |b| {
        b.iter(|| algorithm_b(black_box(&input)));
    });

    group.finish();
}
```

---

## 3. Statistical Rigor

### 3.1 Measurement Accuracy

**Criterion Methodology**:
1. **Warmup** (3 seconds): Stabilize CPU frequency, fill caches
2. **Sample Collection** (100 iterations): Measure actual performance
3. **Outlier Detection**: Remove outliers using Tukey's fences
4. **Statistical Analysis**: Calculate mean, median, std dev
5. **Confidence Intervals**: 95% CI for mean estimate

**Ensuring Accuracy**:
```bash
# Close other applications
# Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance

# Run with longer measurement time
cargo bench -- --measurement-time 30

# Increase sample size
cargo bench -- --sample-size 200
```

### 3.2 Variance Control

**Sources of Variance**:
- CPU frequency scaling
- Background processes
- Thermal throttling
- OS scheduler
- Garbage collection

**Mitigation**:
1. **Dedicated benchmark machine**: No other workloads
2. **Fixed CPU frequency**: Disable turbo boost
3. **Multiple runs**: Average across 3+ runs
4. **Long measurement time**: 10+ seconds per benchmark

### 3.3 Regression Detection

**Criterion Built-In**:
- Compares against saved baseline
- Reports % change with confidence interval
- Flags regressions > 5% threshold

**Custom Thresholds**:
```rust
// In benchmark code
group.significance_level(0.01);  // 1% significance
group.confidence_level(0.99);    // 99% confidence
```

**Interpretation**:
```
change: [-2.1% +0.5% +3.2%] (p = 0.58 > 0.05)
        └───┬───┘ └─┬─┘ └─┬─┘      └────┬────┘
            │      │    │               │
         Lower   Estimate Upper      P-value
         bound             bound      (not significant)
```

---

## 4. Baseline Management

### 4.1 Establishing Baseline

```bash
# 1. Checkout main branch
git checkout main

# 2. Clean build
cargo clean
cargo build --release --all-features

# 3. Run benchmarks and save as baseline
cargo make bench-baseline
```

**Baseline Storage**: `target/criterion/*/base/`

### 4.2 Comparing Against Baseline

```bash
# 1. Make changes on feature branch
git checkout feature/optimization

# 2. Run benchmarks with comparison
cargo make bench-compare

# 3. Review results
open target/criterion/report/index.html
```

### 4.3 Updating Baseline

**When to Update**:
- After merging optimization PR
- After major refactoring
- Monthly baseline refresh

**How to Update**:
```bash
git checkout main
git pull origin main
cargo make bench-baseline
```

---

## 5. Writing Effective Benchmarks

### 5.1 Use `black_box()`

**Purpose**: Prevent compiler from optimizing away code

**Bad** (Compiler eliminates operation):
```rust
b.iter(|| {
    let result = expensive_computation();
    // Compiler sees result unused, eliminates computation
});
```

**Good** (Compiler must perform operation):
```rust
b.iter(|| {
    let result = expensive_computation();
    black_box(result);  // Forces computation
});
```

### 5.2 Separate Setup from Measurement

**Bad** (Setup included in measurement):
```rust
b.iter(|| {
    let data = vec![1, 2, 3];  // Allocation measured
    process(data);
});
```

**Good** (Only process measured):
```rust
let data = vec![1, 2, 3];  // Setup outside iter
b.iter(|| {
    process(black_box(&data));
});
```

**With Dynamic Setup** (Use `iter_batched`):
```rust
use criterion::BatchSize;

b.iter_batched(
    || setup_test_data(),  // Setup (not measured)
    |data| process(data),  // Operation (measured)
    BatchSize::SmallInput,
);
```

### 5.3 Choose Appropriate Input Sizes

**Realistic Sizes**:
- **Small**: 10-100 items (common case)
- **Medium**: 100-1000 items (typical load)
- **Large**: 1000-100000 items (stress test)

**Scalability Testing**:
```rust
for size in [10, 100, 1000, 10000].iter() {
    group.bench_with_input(
        BenchmarkId::from_parameter(size),
        size,
        |b, &size| b.iter(|| process(black_box(size))),
    );
}
```

### 5.4 Document Expected Performance

```rust
/// Benchmark PSO optimization
///
/// **Expected Performance**: 35-45 ms for 500 combinations
/// **Target**: < 45 ms (SLO-O1)
/// **Baseline**: 450 ms (custom implementation)
/// **Improvement**: 10x faster
fn bench_pso_optimization(c: &mut Criterion) {
    // ... benchmark code ...
}
```

---

## 6. Interpreting Results

### 6.1 Criterion Output

```
operation_name          time:   [18.234 µs 18.756 µs 19.301 µs]
                        change: [-2.1% +0.5% +3.2%] (p = 0.58 > 0.05)
                        No change in performance detected.
                        thrpt:  [51.82 Kelem/s 53.31 Kelem/s 54.84 Kelem/s]
```

**Reading**:
- **time**: [lower bound, estimate, upper bound] with 95% confidence
- **change**: % difference from baseline [-lower, estimate, +upper]
- **p-value**: Statistical significance (p > 0.05 = not significant)
- **thrpt**: Throughput in elements/sec (if configured)

### 6.2 HTML Reports

**Location**: `target/criterion/report/index.html`

**Sections**:
- **Summary**: All benchmarks at a glance
- **Violin Plots**: Distribution of measurements
- **Line Plots**: Performance over time
- **Change Tables**: Comparison with baseline

### 6.3 Identifying Regressions

**Automatic Detection**:
- Criterion flags changes with p < 0.05
- Change > 5% threshold triggers warning

**Manual Analysis**:
1. Check violin plots for bimodality (indicates variance issues)
2. Review line plots for trend over time
3. Investigate outliers in raw data
4. Re-run suspect benchmarks with longer measurement time

---

## 7. SLO Test Methodology

### 7.1 Writing SLO Tests

**Pattern**:
```rust
#[test]
fn slo_operation_under_target() {
    // Arrange
    let input = setup_test_data();

    // Act (with timing)
    let start = Instant::now();
    let result = perform_operation(input);
    let duration = start.elapsed();

    // Assert (SLO compliance)
    assert!(duration.as_millis() < TARGET_MS,
            "SLO VIOLATION: {} took {}ms (target: <{}ms)",
            "operation_name", duration.as_millis(), TARGET_MS);

    // Optional: Verify correctness
    assert!(result.is_valid());
}
```

### 7.2 Running SLO Tests

```bash
# Run all SLO tests
cargo make slo-check

# Run specific SLO test
cargo test --test phase2_rdf_benchmarks slo_triple_creation

# Verbose output
cargo test --test phase2_rdf_benchmarks slo_triple_creation -- --nocapture
```

### 7.3 SLO Test Environment

**Consistency**:
- Run on same hardware for all tests
- Close background applications
- Use release mode: `cargo test --release`
- Single-threaded: `RUST_TEST_THREADS=1`

---

## 8. Profiling Workflow

### 8.1 Flamegraph Analysis

```bash
# 1. Generate flamegraph for slow operation
cargo flamegraph --bench phase3_optimization_benchmarks -- bench_pso

# 2. Analyze flamegraph.svg
open flamegraph.svg

# 3. Identify hot paths (widest rectangles)
# 4. Optimize those functions
# 5. Regenerate flamegraph to verify
```

### 8.2 Perf Integration

```bash
# Record performance data
perf record --call-graph=dwarf cargo bench

# Analyze report
perf report

# Generate flamegraph from perf data
perf script | stackcollapse-perf.pl | flamegraph.pl > perf-flamegraph.svg
```

### 8.3 Cachegrind (Cache Profiling)

```bash
# Profile cache usage
valgrind --tool=cachegrind cargo bench

# Analyze cache misses
cg_annotate cachegrind.out.<pid>
```

---

## 9. Best Practices Summary

### 9.1 Do's

✅ **Use `black_box()` for all benchmark inputs/outputs**
✅ **Separate setup from measurement**
✅ **Test at multiple scales (10, 100, 1000, 10000)**
✅ **Document expected performance in docstrings**
✅ **Run on dedicated hardware (disable frequency scaling)**
✅ **Use `iter_batched` for dynamic setup**
✅ **Save baseline before making changes**
✅ **Profile before optimizing (flamegraph)**

### 9.2 Don'ts

❌ **Don't include I/O in hot path measurements**
❌ **Don't test unrealistic input sizes**
❌ **Don't optimize without profiling first**
❌ **Don't trust single-run measurements**
❌ **Don't ignore statistical significance (p-value)**
❌ **Don't benchmark debug builds**
❌ **Don't compare across different machines**

---

## 10. Troubleshooting

### 10.1 High Variance in Results

**Symptoms**: Wide confidence intervals, inconsistent measurements

**Solutions**:
1. Increase measurement time: `--measurement-time 30`
2. Increase sample size: `--sample-size 200`
3. Check CPU frequency scaling: `cpupower frequency-info`
4. Close background applications
5. Use dedicated benchmark machine

### 10.2 Benchmarks Take Too Long

**Solutions**:
```bash
# Quick mode (less accurate)
cargo bench -- --quick

# Run specific benchmark
cargo bench --bench phase1_foundation_benchmarks -- json_serialization

# Reduce sample size
cargo bench -- --sample-size 50
```

### 10.3 SLO Tests Failing Intermittently

**Causes**:
- System load variation
- Thermal throttling
- OS scheduler preemption

**Solutions**:
1. Run multiple times, verify consistency
2. Add margin to SLO target (10-20%)
3. Use percentile targets (p95, p99) instead of absolute

---

## 11. References

- **Criterion.rs User Guide**: https://bheisler.github.io/criterion.rs/book/
- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **Flamegraph GitHub**: https://github.com/flamegraph-rs/flamegraph
- **Cachegrind Manual**: https://valgrind.org/docs/manual/cg-manual.html

---

**Methodology Version**: 1.0.0
**Last Updated**: 2026-01-05
**Next Review**: 2026-03-01

---

END OF PERFORMANCE METHODOLOGY
