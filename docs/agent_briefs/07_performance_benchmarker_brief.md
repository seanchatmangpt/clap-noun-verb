# Performance Benchmarker Brief - v6.0.0 SLO Validation

**Agent ID**: performance-benchmarker-v6
**Memory Key**: performance_validation
**Dependencies**: Independent work (parallel)
**Timeline**: Complete within 25 minutes

## Mission
Establish v6.0.0 performance SLOs, benchmark against v5.5.0, validate zero-cost abstraction claims, and ensure no performance regressions.

## v6.0.0 Performance Targets (SLOs)

```
CLI execution end-to-end:      ≤ 100ms
Compilation (incremental):     ≤ 2s
Test execution (unit):         ≤ 10s
Test execution (integration):  ≤ 30s
Memory usage at runtime:       ≤ 10MB
Binary size (release):         ≤ 5MB
```

## Work Steps

1. **Establish Baseline** (5 min)
   - Create comprehensive benchmark suite
   - Measure v5.5.0 performance (current state)
   - Tools: `cargo bench`, `cargo build --release --timings`
   - Record: Execution time, memory usage, binary size

2. **Identify Performance-Critical Paths** (5 min)
   - Code paths that execute most frequently
   - Memory-intensive operations
   - String allocations
   - Regex compilation (if used)
   - Clap parsing overhead

3. **Create v6.0.0 Performance Benchmarks** (10 min)
   - Write realistic benchmark scenarios
   - Use criterion.rs for statistical rigor
   - Benchmark CLI execution with various inputs
   - Benchmark compilation speed
   - Measure memory usage patterns

4. **Compare vs v5.5.0** (5 min)
   - v5.5.0 baseline: [Record measurements]
   - v6.0.0 performance: [Measure after implementation]
   - Identify regressions (>5% is concerning)
   - Identify improvements

5. **Validate Zero-Cost Claims** (3 min)
   - Verify zero-cost abstractions actually have zero cost
   - Check codegen for generics/const generics
   - Ensure trait objects only where documented
   - Confirm inlining works as expected

6. **Store Results in Memory** (2 min)
   - Save performance_validation findings
   - Include benchmark code, baseline data, SLO status
   - Ready for Release Manager

## Benchmark Code Structure

```rust
// benches/v6_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn cli_parsing_benchmark(c: &mut Criterion) {
    c.bench_function("parse_complex_command", |b| {
        b.iter(|| {
            // Realistic CLI parsing scenario
            let result = parse_command(black_box("noun verb --flag value".to_string()));
            result
        });
    });
}

fn compilation_speed_benchmark(c: &mut Criterion) {
    // Measure incremental compilation time
    // Run actual cargo builds and measure
}

criterion_group!(benches, cli_parsing_benchmark, compilation_speed_benchmark);
criterion_main!(benches);
```

## Deliverables

### Performance Benchmark Report
```
# v6.0.0 Performance SLO Status

## CLI Execution Benchmarks
- Parse simple command: 0.5ms (✅ target ≤100ms)
- Parse complex command: 2.3ms (✅ target ≤100ms)
- Full CLI execution: 45ms (✅ target ≤100ms)

## Compilation Benchmarks
- Clean build: 8s (⚠️ target ≤2s for incremental)
- Incremental build: 1.2s (✅ target ≤2s)

## Memory Usage
- Parser state: 256KB (✅ target ≤10MB)
- Full CLI: 2.3MB (✅ target ≤10MB)

## Regressions from v5.5.0
- CLI parsing: -3% (improvement ✅)
- Compilation: +1% (acceptable ✅)
- Memory: -2% (improvement ✅)

## Zero-Cost Abstraction Validation
- Generic monomorphization: ✅ Zero overhead
- Const generics: ✅ Compile-time only
- Type-level features: ✅ No runtime cost
```

### Benchmark Code
- `/benches/v6_performance_slos.rs` - SLO benchmarks
- `/benches/v6_comparison.rs` - v5.5.0 vs v6.0.0 comparison

### Performance Optimization Guide (if needed)
- Identified bottlenecks
- Optimization recommendations
- Trade-offs analyzed

## Constraints
- **SLOs are targets**: 100ms CLI, 2s compile (incremental), 10MB memory
- **No regressions**: Performance should match or improve v5.5.0
- **Zero-cost validation**: All claimed zero-cost abstractions verified
- **Reproducible benchmarks**: Results must be consistent

## Success Criteria
- ✅ Baseline performance measured (v5.5.0)
- ✅ All SLO targets achievable
- ✅ No significant regressions from v5.5.0
- ✅ Zero-cost abstractions validated
- ✅ Benchmark code added to repo
- ✅ Memory key performance_validation populated
- ✅ Performance report complete

## Andon Signal
**IF PERFORMANCE REGRESSION > 5%**: Alert for investigation
- This blocks release if not addressed
- May indicate new code inefficiency
- Needs optimization pass before release

## Notes
- Benchmarks should simulate real usage patterns
- Use criterion.rs for statistical rigor
- Run benchmarks in release mode
- Disable debug optimizations during benchmarking
- Compare with v5.5.0 to establish delta
