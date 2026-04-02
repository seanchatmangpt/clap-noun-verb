# RDF Turtle CLI Generation - Benchmark Suite Guide

## Overview

A comprehensive benchmark suite using criterion.rs to measure performance of the MCP RDF Turtle CLI generation infrastructure. These benchmarks validate that all operations meet Service Level Objectives (SLOs) and provide detailed metrics for performance tuning.

## Benchmark Files

### 1. Turtle Parser Benchmarks (`benches/turtle_parser_bench.rs`)

Measures RDF 1.1 Turtle parsing performance with varying ontology sizes.

**Test Cases:**

| Benchmark | Size | Goal | Notes |
|-----------|------|------|-------|
| `turtle_parse_10_verbs` | 10 verbs | < 50ms | Small ontology |
| `turtle_parse_100_verbs` | 100 verbs | < 50ms | Medium ontology |
| `turtle_parse_1000_verbs` | 1000 verbs | < 200ms | Large ontology |
| `turtle_parsing` (parameterized) | 10-1000 | Analyze scaling | Full scalability curve |
| `turtle_validate_100_verbs` | Validation | < 10ms | Ontology validation |
| `prefix_resolution_100_verbs` | Prefix resolution | < 5ms | Namespace handling |

**What It Measures:**
- W3C Turtle 1.1 parsing speed (using oxigraph)
- Scaling behavior with increasing ontology size
- Validation overhead
- Prefix resolution efficiency

**Run It:**
```bash
cargo bench --bench turtle_parser_bench --features rdf-composition
```

---

### 2. SPARQL Executor Benchmarks (`benches/sparql_executor_bench.rs`)

Measures SPARQL 1.1 query execution performance for ontology introspection.

**Test Cases:**

| Benchmark | Query Type | Size | Goal | Notes |
|-----------|-----------|------|------|-------|
| `sparql_list_classes_10_verbs` | Class discovery | 10 | < 5ms | Small graph |
| `sparql_list_classes_100_verbs` | Class discovery | 100 | < 10ms | Medium graph |
| `sparql_list_classes_1000_verbs` | Class discovery | 1000 | < 50ms | Large graph |
| `sparql_list_properties_100_verbs` | Property discovery | 100 | < 10ms | All properties |
| `sparql_select_all_verbs_100` | SELECT * query | 100 | < 10ms | Simple query |
| `sparql_filter_query_100_verbs` | FILTER with regex | 100 | < 20ms | Pattern matching |
| `sparql_join_query_100_verbs` | Multi-pattern JOIN | 100 | < 50ms | Complex patterns |
| `sparql_queries` (parameterized) | SELECT queries | 10-1000 | Analyze scaling | Scalability |

**What It Measures:**
- Basic SPARQL class/property discovery
- Simple SELECT query execution
- Query complexity impact (filters, joins)
- Scaling with ontology size
- Result binding extraction

**Run It:**
```bash
cargo bench --bench sparql_executor_bench --features rdf-composition
```

---

### 3. Code Generator Benchmarks (`benches/code_generator_bench.rs`)

Measures CLI code generation performance from RDF ontologies.

**Test Cases:**

| Benchmark | Commands | Goal | Notes |
|-----------|----------|------|-------|
| `codegen_10_verbs` | 10 | < 100ms | Minimal CLI |
| `codegen_50_verbs` | 50 | < 200ms | Medium CLI |
| `codegen_100_verbs` | 100 | < 500ms | Large CLI |
| `codegen_500_verbs` | 500 | < 2s | XL CLI |
| `code_generation` (parameterized) | 10-250 | Analyze scaling | Full scalability |
| `noun_macro_generation` | N/A | < 100µs | Single noun macro |
| `verb_macro_generation` | N/A | < 100µs | Single verb macro |

**What It Measures:**
- Time to generate Rust code from ontology
- Per-noun and per-verb macro generation
- Code output size (lines of code)
- Scaling with CLI complexity
- TokenStream buffer efficiency

**Run It:**
```bash
cargo bench --bench code_generator_bench --features rdf-composition
```

---

### 4. MCP Workflow Benchmarks (`benches/mcp_workflow_bench.rs`)

Measures end-to-end MCP tool performance for complete agent workflows.

**Test Cases:**

| Benchmark | Workflow | Size | Goal | Notes |
|-----------|----------|------|------|-------|
| `e2e_parse_validate_generate_10_verbs` | Full pipeline | 10 | < 200ms | Small CLI |
| `e2e_parse_validate_generate_50_verbs` | Full pipeline | 50 | < 300ms | Medium CLI |
| `e2e_parse_validate_generate_100_verbs` | Full pipeline | 100 | < 500ms | Large CLI |
| `e2e_with_sparql_discovery_50_verbs` | + SPARQL | 50 | < 400ms | With discovery |
| `e2e_with_sparql_discovery_100_verbs` | + SPARQL | 100 | < 750ms | With discovery |
| `end_to_end_workflow` (parameterized) | Full pipeline | 10-100 | Analyze scaling | Scalability |
| `turtle_parser_creation` | Object creation | N/A | < 1ms | Cold start |
| `code_generator_creation` | Object creation | N/A | < 1ms | Cold start |
| `sparql_executor_creation_50_verbs` | Object creation | 50 | < 50ms | Executor init |

**What It Measures:**
- Complete agent workflow (parse → validate → generate)
- Impact of adding SPARQL discovery
- Component initialization costs
- Real-world usage patterns
- Pipeline latency from agent perspective

**Run It:**
```bash
cargo bench --bench mcp_workflow_bench --features rdf-composition
```

---

## Running Benchmarks

### Run All Benchmarks

```bash
cargo make bench --features rdf-composition
```

### Run Specific Benchmark

```bash
# Run only Turtle parser benchmarks
cargo bench --bench turtle_parser_bench --features rdf-composition

# Run only SPARQL benchmarks
cargo bench --bench sparql_executor_bench --features rdf-composition

# Run only code generator benchmarks
cargo bench --bench code_generator_bench --features rdf-composition

# Run only MCP workflow benchmarks
cargo bench --bench mcp_workflow_bench --features rdf-composition
```

### Run Specific Test

```bash
# Run single benchmark by name
cargo bench --bench turtle_parser_bench --features rdf-composition turtle_parse_100

# Run with filtering (partial match)
cargo bench --bench sparql_executor_bench --features rdf-composition list_classes
```

### Advanced Options

```bash
# Run benchmarks with verbose output
cargo bench --bench turtle_parser_bench --features rdf-composition -- --verbose

# Save results as baseline for comparison
cargo bench --bench turtle_parser_bench --features rdf-composition -- --save-baseline v5.3.4

# Compare against baseline
cargo bench --bench turtle_parser_bench --features rdf-composition -- --baseline v5.3.4

# Run with fewer samples (faster)
cargo bench --bench turtle_parser_bench --features rdf-composition -- --sample-size 10

# Generate HTML report
cargo bench --bench turtle_parser_bench --features rdf-composition -- --verbose
# Check: target/criterion/
```

---

## Understanding Criterion Output

### Sample Output

```
turtle_parse_100_verbs             time:   [18.234 ms 18.456 ms 18.703 ms]
                                   change: [-2.34% +0.12% +2.89%] (within noise)
                                   thrpt:  [5.345 MB/s 5.417 MB/s 5.484 MB/s]
                                   slope:  [1.0023 1.0047 1.0072] R²=0.9989
```

**Breaking it down:**

- **time**: Actual measured time (lower, middle, upper quartile)
- **change**: Comparison to baseline (if available)
- **thrpt**: Throughput in MB/s (inverse of time)
- **slope**: Linear regression slope (predictability)
- **R²**: Goodness of fit (how linear the measurements are)

### Interpreting Results

| Metric | Good | Concerning | Bad |
|--------|------|-----------|-----|
| Time | Stable, tight range | High variance | Exceeds SLO |
| Change | < ±5% vs baseline | ±5-10% drift | > 10% regression |
| Slope | ~1.0, R² > 0.99 | R² > 0.95 | R² < 0.95 (noise) |

---

## Performance SLOs

Service Level Objectives define acceptable performance targets:

### Critical SLOs (Must Pass)

| Component | Operation | Target |
|-----------|-----------|--------|
| Turtle Parser | Parse 100 triples | ≤ 50ms |
| Turtle Parser | Parse 1000 triples | ≤ 200ms |
| SPARQL | Simple SELECT query | ≤ 10ms |
| SPARQL | JOIN query | ≤ 50ms |
| Code Generator | Generate 10 commands | ≤ 100ms |
| MCP Tool | Full E2E (100 commands) | ≤ 500ms |
| Memory | Total system | ≤ 20MB |

### Secondary SLOs (Should Pass)

| Component | Operation | Target |
|-----------|-----------|--------|
| Turtle Parser | Validation overhead | ≤ 10ms |
| SPARQL | FILTER query | ≤ 20ms |
| Code Generator | 100 commands | ≤ 500ms |
| Code Generator | 500 commands | ≤ 2s |
| Parser Creation | Object init | ≤ 1ms |

---

## Analyzing Results

### Identify Bottlenecks

1. **Compare against SLOs**
   - Any metric exceeding SLO needs investigation

2. **Look for unexpected scaling**
   - O(n) operations should show linear growth
   - Superlinear growth indicates algorithm issue

3. **Check memory usage**
   - Should scale linearly with input size
   - Non-linear growth suggests memory leak

### Profiling

For deeper analysis beyond benchmark times:

```bash
# Generate flamegraph (requires flamegraph installation)
cargo flamegraph --bench turtle_parser_bench --features rdf-composition

# Use perf on Linux
perf record -g ./target/release/deps/turtle_parser_bench-*
perf report

# Use Instruments on macOS
instruments -t "System Trace" ./target/release/deps/turtle_parser_bench-*
```

---

## Regression Detection

Criterion automatically detects regressions when using baselines:

```bash
# First, establish baseline
cargo bench --bench turtle_parser_bench --features rdf-composition -- --save-baseline main

# Later, compare against baseline
cargo bench --bench turtle_parser_bench --features rdf-composition -- --baseline main
```

If any benchmark regresses > 5%, criterion will:
1. Alert with WARNING
2. Show detailed comparison
3. Exit with non-zero code (useful for CI)

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Performance Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo make bench --features rdf-composition -- --save-baseline ${{ github.ref }}

      - name: Comment results
        uses: actions/github-script@v6
        with:
          script: |
            # Parse criterion output and comment on PR
```

---

## Optimization Tips

### For Agents Using These Tools

1. **Cache ontologies**
   - Parsing is one-time cost
   - Reuse parsed results

2. **Batch operations**
   - Generate multiple CLIs together
   - Amortize executor creation cost

3. **Query optimization**
   - Use specific SPARQL patterns
   - Avoid wildcards when possible

### For maintainers

1. **Monitor regressions**
   - Run benchmarks before/after changes
   - Set up CI checks

2. **Profile hot paths**
   - Use flamegraph for unexplained slowness
   - Focus on algorithm improvements

3. **Consider caching**
   - SPARQL query result caching
   - Parsed ontology caching
   - Code generation caching

---

## Troubleshooting

### Benchmarks Won't Compile

```bash
# Ensure rdf-composition feature is enabled
cargo bench --bench turtle_parser_bench --features rdf-composition

# Clean and rebuild
cargo clean
cargo build --features rdf-composition
```

### Results Are Noisy

- Increase sample size: `-- --sample-size 100`
- Close other applications
- Use dedicated benchmark machine
- Look at criterion HTML reports: `target/criterion/index.html`

### Running on CI

- Use `--verbose` flag for debugging
- Save baseline on main branch
- Compare baseline to current branch
- Fail CI if regression > threshold

---

## Further Reading

- **Criterion.rs Book**: https://bheisler.github.io/criterion.rs/book/
- **Performance Benchmarks**: `docs/performance-benchmarks.md`
- **Architecture Guide**: `docs/rdf-turtle-cli-architecture.md`

---

Generated: 2026-01-06
Maintained by: Performance Engineering Team
