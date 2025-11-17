# clap-noun-verb Benchmark Suite

Comprehensive performance benchmarking for v4.0.0 systems and autonomic kernel.

## Benchmark Files

### v4.0.0 Systems

1. **`v4_system_benchmarks.rs`** - Plugin, Middleware, Telemetry, Startup
   - Plugin loading (cold start, discovery, registry)
   - Middleware chain execution (0-5 layers)
   - Telemetry overhead (metrics, tracing, spans)
   - Startup sequence simulation
   - Memory allocation patterns

2. **`io_performance_benchmarks.rs`** - I/O System Performance
   - Pipeline construction and configuration
   - Buffer operations (1KB, 4KB, 16KB)
   - Processing patterns (line-by-line, chunked, transformed)
   - Async I/O simulation (backpressure, buffering)
   - Large file handling (1MB, 10MB)
   - Concurrent I/O patterns

3. **`config_startup_benchmarks.rs`** - Configuration & Startup
   - Config graph construction (10-100 nodes)
   - JSON/YAML parsing (small, medium, large)
   - Hot reload performance
   - Phase-by-phase startup breakdown
   - Memory footprint analysis
   - Configuration validation

### Autonomic Kernel

4. **`hot_path_benchmarks.rs`** - Zero-Allocation Hot Paths
   - InvocationQueue throughput
   - ContextPool allocation
   - HotPathContext creation
   - InvocationArena allocation
   - EffectFlags operations
   - CapabilityId creation

5. **`graph_benchmarks.rs`** - Capability Graph Operations
   - Graph construction (10-200 nodes)
   - Reachability queries
   - Shortest path algorithms
   - Graph statistics

## Quick Start

### Run All Benchmarks
```bash
cargo bench
```

### Run Specific Benchmark Suite
```bash
# v4.0.0 systems
cargo bench --bench v4_system_benchmarks
cargo bench --bench io_performance_benchmarks
cargo bench --bench config_startup_benchmarks

# Autonomic kernel
cargo bench --bench hot_path_benchmarks
cargo bench --bench graph_benchmarks
```

### Run Specific Benchmark Group
```bash
# Plugin loading only
cargo bench --bench v4_system_benchmarks -- plugin_loading

# Middleware chain only
cargo bench --bench v4_system_benchmarks -- middleware_chain

# I/O pipelines only
cargo bench --bench io_performance_benchmarks -- io_pipeline
```

## Benchmark Configuration

All benchmarks use:
- **Tool**: Criterion.rs v0.5
- **Measurement Time**: 10 seconds per benchmark
- **Sample Size**: 100 iterations (50 for startup benchmarks)
- **Warmup**: 3 seconds
- **Output**: HTML reports in `target/criterion/`

## Viewing Results

### Terminal Output
Benchmark results are displayed in the terminal with:
- Mean time and standard deviation
- Throughput (for applicable benchmarks)
- Change detection (vs previous run)

### HTML Reports
Open `target/criterion/report/index.html` in a browser for:
- Detailed statistics
- Violin plots
- Performance history
- Regression detection

### CI Integration
```bash
# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

## Expected Performance Ranges

### Plugin System
- Single plugin load: **15-25 µs**
- 10 plugins load: **200-350 µs**
- Registry lookup (10 plugins): **300-400 ns**

### Middleware Chain
- 1 layer overhead: **180-250 ns**
- 3 layers overhead: **550-680 ns**
- 5 layers overhead: **920-1150 ns**

### Telemetry
- Record command: **120-180 ns**
- Record error: **150-220 ns**
- Span creation: **35-55 ns**

### I/O System
- Buffer copy (1KB): **550 ns** (~1.8 GB/s)
- Buffer copy (16KB): **7.0 µs** (~2.3 GB/s)
- Process 1MB: **830 µs** (~1.2 GB/s)

### Configuration
- Small JSON parse: **1.2-1.8 µs**
- Medium JSON parse: **8-12 µs**
- Graph construction (50 nodes): **48-72 µs**

### Startup
- Full cold start: **680-1035 µs**
- Plugin loading (5): **125-180 µs**
- Middleware setup (5): **95-140 µs**

## Performance Regression Thresholds

Alert if any benchmark degrades by:
- **> 10%** for hot paths (middleware, telemetry)
- **> 20%** for plugin loading
- **> 30%** for startup sequence
- **> 50%** for I/O operations (may vary with system load)

## Benchmark Development

### Adding New Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Your code here
            black_box(my_function());
        });
    });
}

criterion_group!(my_benches, bench_my_feature);
criterion_main!(my_benches);
```

Add to `Cargo.toml`:
```toml
[[bench]]
name = "my_benchmarks"
harness = false
```

### Best Practices

1. **Use `black_box()`** - Prevents compiler optimization of benchmarked code
2. **Separate setup from measurement** - Use `iter_batched()` for setup
3. **Control input size** - Use `Throughput::Elements()` or `Throughput::Bytes()`
4. **Avoid I/O in hot path** - Mock or pre-load data
5. **Document expected ranges** - Add comments with target performance

## Troubleshooting

### Benchmarks Won't Compile
```bash
# Check for compilation errors
cargo bench --no-run

# Update dependencies
cargo update
```

### Inconsistent Results
- Close other applications
- Disable CPU frequency scaling
- Run multiple times and average
- Use `--measurement-time 30` for longer measurements

### Benchmarks Too Slow
- Reduce `--sample-size 50`
- Use `--quick` for rapid iteration
- Focus on specific benchmarks with `-- filter_pattern`

## Related Documentation

- **Performance Report**: `/docs/PERFORMANCE_ASSESSMENT_V4.md`
- **Summary**: `/docs/PERFORMANCE_SUMMARY.md`
- **Criterion Docs**: https://bheisler.github.io/criterion.rs/

---

**Last Updated**: 2025-11-16
**Benchmark Suite Version**: v4.0.0
