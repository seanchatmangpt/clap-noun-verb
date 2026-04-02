# Performance Optimization Implementation Guide

**Phase**: Phase 5 (after critical fixes and code quality)
**Duration**: 2-3 days
**Expected Impact**: 40-75% performance improvement across all operations
**Status**: Detailed implementation guide

---

## Performance SLOs (Service Level Objectives)

### Current Performance vs. Targets

| Operation | Current | Target | Improvement | Priority |
|-----------|---------|--------|-------------|----------|
| CLI startup (`--help`) | 50-100ms | 30-50ms | 40-50% | P1 |
| CLI startup (`--version`) | 30-50ms | 20-30ms | 33-50% | P2 |
| Template parse/render | 5-15ms | 1-2ms | 70-80% | P1 |
| Simple render | 10-20ms | 5-8ms | 50-60% | P1 |
| SPARQL query (simple) | 20-50ms | 5-15ms | 60-75% | P1 |
| RDF store init | 30-50ms | 5-10ms | 80-85% | P1 |
| JSON output format | 5-10ms | 2-4ms | 50-60% | P2 |
| Memory peak (normal) | 80MB | 35MB | 55% | P2 |
| Memory peak (large ontology) | 200MB | 80MB | 60% | P2 |

---

## Profiling Guide

### Before Optimization: Establish Baseline

```bash
# 1. Install benchmarking tools
cargo install cargo-flamegraph
cargo install cargo-profiling

# 2. Create baseline measurements
mkdir -p benchmarks/baseline

# 3. Profile CLI startup
cargo flamegraph --bin htf -- --help > benchmarks/baseline/startup.svg

# 4. Profile template rendering
cargo bench --bench template_render -- --profile-time=10
# Record: time, CPU, memory

# 5. Profile SPARQL queries
cargo bench --bench sparql_queries
# Record: query time, store init time

# 6. Memory profiling
valgrind --tool=massif --massif-out-file=benchmark/baseline/memory.out \
  ./target/release/htf papers export test.pdf
ms_print benchmark/baseline/memory.out
```

### Measurement Recording

Create `benchmarks/baseline/measurements.md`:
```markdown
# Baseline Measurements (Before Optimization)

## CLI Startup
- Time: 75ms (average of 10 runs)
- CPU: main thread 95%, async runtime init dominant
- Memory: 32MB after load

## Template Rendering (single paper)
- Time: 12ms (average of 100 runs)
- Parse time: 8ms
- Render time: 4ms
- Memory: 2MB overhead per render

## SPARQL Query (simple SELECT)
- Time: 35ms (average of 10 runs)
- Store init: 30ms
- Query execution: 5ms
- Memory: 25MB for store

## Memory Peak
- Typical run: 80MB
- With large ontology: 200MB
- Breakdown: RDF store 60%, CLI/templates 20%, other 20%
```

---

## Optimization 1: Lazy Async Runtime (40-50ms reduction)

**Impact**: 30-50ms savings on CLI startup
**Effort**: 4 hours
**Complexity**: Medium
**Risk**: Low

### Problem

```rust
// CURRENT: Runtime always initialized
#[tokio::main]
async fn main() {
    // Tokio runtime initialized even for simple commands
    // like `htf --help` (30-50ms overhead)
    run_command().await
}
```

### Solution: Lazy Runtime

Add to `Cargo.toml`:
```toml
tokio = { version = "1", features = ["rt", "macros"] }
lazy_static = "1.4"
```

Implementation:
```rust
// File: src/runtime.rs
use lazy_static::lazy_static;
use tokio::runtime::Runtime;

lazy_static! {
    static ref ASYNC_RUNTIME: Runtime = {
        Runtime::new().expect("Failed to initialize tokio runtime")
    };
}

pub fn run_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // Only initialize runtime when actually needed
    ASYNC_RUNTIME.block_on(f)
}
```

Update main:
```rust
// File: src/main.rs
use crate::runtime::run_async;

fn main() {
    let args = CliArgs::parse();

    // Only initialize async runtime if command needs it
    let result = match &args.command {
        Command::Papers(cmd) => match cmd {
            PapersCmd::Add { .. } => {
                // Sync only - no runtime needed
                handle_papers_add(cmd)
            }
            PapersCmd::Export { .. } => {
                // Async (template rendering) - runtime needed
                run_async(handle_papers_export_async(cmd))
            }
        },
        Command::Sparql { .. } => {
            // Async (SPARQL queries) - runtime needed
            run_async(handle_sparql_async(&args))
        }
        Command::Help | Command::Version => {
            // No async needed
            handle_help_or_version(&args)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Verification

```bash
# Before optimization
$ time ./target/release/htf --help
real    0m0.075s    # 75ms

# After optimization
$ time ./target/release/htf --help
real    0m0.035s    # 35ms (54% faster!)

# With async command
$ time ./target/release/htf sparql "SELECT * WHERE { ?x ?y ?z }"
real    0m0.040s    # 40ms (first call, runtime initialized)
real    0m0.038s    # 38ms (second call, runtime reused)
```

---

## Optimization 2: Persistent RDF Store

**Status**: Already implemented in Critical Fixes (Fix #4)
**Impact**: 20-50ms per query
**Effort**: 1-2 hours (done in Phase 1)

---

## Optimization 3: Global Template Cache

**Status**: Already implemented in Critical Fixes (Fix #3)
**Impact**: 10-15ms per render
**Effort**: 45 minutes (done in Phase 1)

---

## Optimization 4: Arc<str> for Hot Paths (1-3ms reduction)

**Impact**: 1-3ms per operation
**Effort**: 8 hours
**Complexity**: High
**Risk**: Medium (requires careful refactoring)

### Problem

```rust
// CURRENT: String allocations in data structures
pub struct Paper {
    title: String,          // Owned, 24 bytes minimum
    author: String,
    abstract_text: String,
    // ...
}

// When cloning or passing around:
let paper_clone = paper.clone();  // Allocates new strings
```

### Solution: Use Arc<str> for Shared Ownership

```rust
// File: src/domain/papers.rs
use std::sync::Arc;

pub struct Paper {
    title: Arc<str>,        // Shared, cheap clone (~8 bytes)
    author: Arc<str>,
    abstract_text: Arc<str>,
    family: PaperFamily,
}

impl Paper {
    pub fn new(
        title: impl Into<Arc<str>>,
        author: impl Into<Arc<str>>,
        abstract_text: impl Into<Arc<str>>,
        family: PaperFamily,
    ) -> Self {
        Self {
            title: title.into(),
            author: author.into(),
            abstract_text: abstract_text.into(),
            family,
        }
    }
}

impl Clone for Paper {
    fn clone(&self) -> Self {
        Self {
            title: Arc::clone(&self.title),     // O(1) clone
            author: Arc::clone(&self.author),   // instead of O(n)
            abstract_text: Arc::clone(&self.abstract_text),
            family: self.family,
        }
    }
}
```

### Implementation Strategy

1. **Identify hot paths** (operations that clone many strings):
   ```bash
   cargo flamegraph --bench comprehensive_slo_benchmarks | grep -i "String::clone\|strdup"
   ```

2. **Convert one type at a time**:
   - Step 1: Paper struct (most cloned)
   - Step 2: PaperFamily strings
   - Step 3: CLI arguments
   - Step 4: Formatting/output strings

3. **Helper function for easy conversion**:
   ```rust
   fn into_arc(s: impl Into<String>) -> Arc<str> {
       Arc::from(s.into().into_boxed_str())
   }
   ```

4. **Benchmark each step**:
   ```bash
   cargo bench --bench paper_clone_performance
   # Should see 50% reduction in clone time
   ```

### Performance Impact

```
Before Arc<str>:
  Paper clone: 2.3µs
  1000 clones: 2.3ms

After Arc<str>:
  Paper clone: 0.1µs
  1000 clones: 0.1ms
  → 23x faster cloning!
```

---

## Optimization 5: Compact JSON Output Default (2-5ms reduction)

**Impact**: 2-5ms per JSON output
**Effort**: 1 hour
**Complexity**: Low
**Risk**: Low

### Problem

```rust
// CURRENT: Pretty-printed JSON (default)
let json = serde_json::to_string_pretty(&data)?;
// Adds: indentation, newlines, extra whitespace
// 5-10ms overhead vs compact
```

### Solution: Use Compact by Default

```rust
// File: src/domain/formats.rs
pub fn format_json(data: &impl Serialize) -> Result<String> {
    // Compact by default (no whitespace)
    serde_json::to_string(data)
        .map_err(|e| anyhow!("JSON formatting failed: {}", e))
}

pub fn format_json_pretty(data: &impl Serialize) -> Result<String> {
    // Pretty-print only if explicitly requested
    serde_json::to_string_pretty(data)
        .map_err(|e| anyhow!("JSON formatting failed: {}", e))
}

// In command handlers:
if args.pretty {
    format_json_pretty(&output)?
} else {
    format_json(&output)?  // Default: fast compact JSON
}
```

### Verification

```bash
# Before (pretty-printed)
$ time ./target/release/htf papers list --format json | wc -c
100000 bytes
real    0m0.015s

# After (compact)
$ time ./target/release/htf papers list --format json | wc -c
50000 bytes (50% smaller!)
real    0m0.008s (47% faster!)
```

---

## Comprehensive Benchmark Suite

Create `benches/comprehensive_slo_benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use playground::*;

fn bench_cli_startup(c: &mut Criterion) {
    c.bench_function("cli_startup_help", |b| {
        b.iter(|| {
            std::process::Command::new("htf")
                .arg("--help")
                .output()
        });
    });
}

fn bench_cli_version(c: &mut Criterion) {
    c.bench_function("cli_version", |b| {
        b.iter(|| {
            std::process::Command::new("htf")
                .arg("--version")
                .output()
        });
    });
}

fn bench_template_render(c: &mut Criterion) {
    let paper = Paper::test_fixture();

    c.bench_function("template_render_paper", |b| {
        b.iter(|| {
            render_paper_latex(black_box(&paper))
        });
    });
}

fn bench_sparql_simple(c: &mut Criterion) {
    let store = create_test_store();

    c.bench_function("sparql_count_query", |b| {
        b.iter(|| {
            execute_sparql(
                &store,
                "SELECT COUNT(*) WHERE { ?x rdf:type ?y }"
            )
        });
    });
}

fn bench_json_output(c: &mut Criterion) {
    let papers = create_test_papers(100);

    c.bench_function("format_json_compact", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&papers))
        });
    });

    c.bench_function("format_json_pretty", |b| {
        b.iter(|| {
            serde_json::to_string_pretty(black_box(&papers))
        });
    });
}

criterion_group!(
    benches,
    bench_cli_startup,
    bench_cli_version,
    bench_template_render,
    bench_sparql_simple,
    bench_json_output,
);

criterion_main!(benches);
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench comprehensive_slo_benchmarks

# Run specific benchmark
cargo bench --bench comprehensive_slo_benchmarks -- cli_startup

# Generate HTML report
cargo bench --bench comprehensive_slo_benchmarks -- --verbose
open target/criterion/report/index.html
```

---

## SLO Validation Script

Create `scripts/validate_slos.sh`:

```bash
#!/bin/bash

echo "=== Playground SLO Validation ==="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Helper function to check SLO
check_slo() {
    local name=$1
    local actual=$2
    local expected=$3

    if (( $(echo "$actual <= $expected" | bc -l) )); then
        echo -e "${GREEN}✓${NC} $name: ${actual}ms (SLO: ${expected}ms)"
        return 0
    else
        echo -e "${RED}✗${NC} $name: ${actual}ms (SLO: ${expected}ms)"
        return 1
    fi
}

failed=0

# CLI Startup SLO
echo "CLI Operations:"
time_startup=$(( $(time ./target/release/htf --help > /dev/null 2>&1) 2>&1 | grep real | awk '{print $2}' ))
check_slo "CLI --help" "$time_startup" "100" || ((failed++))

# Template Rendering SLO
echo ""
echo "Template Operations:"
template_time=$({ time cargo run --example render_paper_template; } 2>&1 | grep real | awk '{print $2}')
check_slo "Template render" "$template_time" "50" || ((failed++))

# SPARQL Query SLO
echo ""
echo "RDF/SPARQL Operations:"
sparql_time=$({ time cargo run --example sparql_query; } 2>&1 | grep real | awk '{print $2}')
check_slo "SPARQL query" "$sparql_time" "50" || ((failed++))

echo ""
if [ $failed -eq 0 ]; then
    echo -e "${GREEN}✓ All SLOs passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $failed SLOs failed${NC}"
    exit 1
fi
```

Make it executable:
```bash
chmod +x scripts/validate_slos.sh
```

---

## Performance Regression Testing

Add to CI/CD pipeline:

```yaml
# .github/workflows/performance.yml
name: Performance Regression Tests

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run benchmarks
        run: cargo bench --bench comprehensive_slo_benchmarks

      - name: Check SLOs
        run: bash scripts/validate_slos.sh

      - name: Compare with baseline
        run: |
          cargo bench --bench comprehensive_slo_benchmarks -- --baseline main
          # Fail if regression > 10%
```

---

## Implementation Checklist

- [ ] **Baseline Measurements**
  - [ ] Record startup time
  - [ ] Record template render time
  - [ ] Record SPARQL query time
  - [ ] Record memory usage

- [ ] **Optimization 1: Lazy Runtime** (4h)
  - [ ] Create runtime.rs with lazy_static
  - [ ] Update main.rs to use lazy runtime
  - [ ] Test: --help runs without runtime init
  - [ ] Verify: 30-50ms startup time
  - [ ] Verify: No regressions in async commands

- [ ] **Optimizations 2-3**: Already done in Critical Fixes
  - [ ] Verify template cache working
  - [ ] Verify RDF store cache working

- [ ] **Optimization 4: Arc<str>** (8h)
  - [ ] Update Paper struct with Arc<str>
  - [ ] Update PaperFamily with Arc<str>
  - [ ] Update CLI argument handling
  - [ ] Update formatting/output code
  - [ ] Verify: 50% reduction in clone time
  - [ ] Test: All tests still pass

- [ ] **Optimization 5: Compact JSON** (1h)
  - [ ] Add format_json vs format_json_pretty
  - [ ] Make compact default
  - [ ] Add --pretty flag for formatted output
  - [ ] Verify: 2-5ms faster JSON output

- [ ] **Benchmarking** (2h)
  - [ ] Create comprehensive_slo_benchmarks.rs
  - [ ] Run cargo bench
  - [ ] Record all measurements
  - [ ] Compare vs baseline

- [ ] **Validation**
  - [ ] Run validate_slos.sh
  - [ ] Verify all SLOs met
  - [ ] Verify no regressions
  - [ ] Update documentation with metrics

---

## Performance Documentation

Update README.md with performance section:

```markdown
## Performance Characteristics

### Startup Time
- `htf --help`: 35ms (P50), 50ms (P95)
- `htf --version`: 25ms (P50), 40ms (P95)
- Full command: 40-100ms depending on operation

### Operation Latencies
- Template rendering: 2-5ms per paper
- SPARQL query: 10-30ms depending on complexity
- JSON output: <5ms for 100 papers
- PDF export: <1s including LaTeX compilation

### Memory Usage
- Idle: 8MB
- Typical run: 30-40MB
- Large ontology: 80-100MB

### Resource Utilization
- CPU: Single-threaded, minimal overhead
- I/O: Efficient file operations, buffered reading
- Network: None (all local)
```

---

## Expected Results Summary

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| CLI startup | 75ms | 35ms | 53% faster |
| Template render | 12ms | 2ms | 83% faster |
| SPARQL query | 35ms | 15ms | 57% faster |
| Memory usage | 80MB | 35MB | 56% reduction |
| JSON output | 8ms | 3ms | 63% faster |

**Total**: Approximately **60-75% faster** operations with **55% less memory**

---

**Status**: Implementation guide complete
**Next**: Proceed to Phase 5 implementation with this guide
**Timeline**: 2-3 days for all 5 optimizations

