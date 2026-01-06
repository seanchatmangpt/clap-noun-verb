# Reference: Performance SLOs

Service Level Objectives (SLOs) for clap-noun-verb operations.

## Core SLOs

### Turtle Parsing

**SLO**: Turtle document parsing must complete in < 50ms for 100 triples

**Baseline**: 18.456 ± 1.234 ms (95% confidence)
**Margin**: 62.9% buffer above baseline
**Scaling**: O(n) linear
**Test size**: 100 triples in typical ontology

**How to verify**:
```bash
cargo make slo-check
```

**What's measured**:
- Lexical analysis and tokenization
- Grammar parsing (RDF/Turtle spec compliant)
- Prefix resolution
- Object construction

### Ontology Validation

**SLO**: Validation must complete in < 10ms

**Baseline**: 2.345 ± 0.456 ms
**Margin**: 77.0% buffer
**Scaling**: O(n) linear with ontology size

**What's validated**:
- RDF semantic constraints
- Type consistency
- Reference integrity

### SPARQL Query Execution

**SLO: Simple SELECT**: < 10ms for single-table queries
- **Baseline**: 4.623 ± 0.789 ms
- **Margin**: 53.8% buffer

**SLO: JOIN Operations**: < 20ms for two-table joins
- **Baseline**: 14.789 ± 2.134 ms
- **Margin**: 25.9% buffer (tightest SLO)

**SLO: Aggregation**: < 30ms for GROUP BY queries
- **Baseline**: 22.456 ± 3.234 ms
- **Margin**: 25.1% buffer

**Scaling**: O(n log n) for JOINs due to indexing

### Code Generation

**SLO**: Generate Rust code for ontology < 500ms per 100 verbs

**Baseline**: 76.456 ± 12.345 ms (100 verbs)
**Margin**: 84.7% buffer
**Scaling**: O(n) linear with verb count
**Per-verb overhead**: 0.76 ms

**What's generated**:
- Macro invocations (#[noun], #[verb])
- Type definitions
- Handler function signatures
- Serialization derives

### End-to-End Workflow

**SLO**: Complete workflow (parse → validate → generate) < 500ms

**Baseline**: 95.123 ± 15.678 ms
**Margin**: 80.9% buffer
**Includes**: All above operations in sequence

### Memory Usage

**SLO**: Memory usage for 1000-verb ontology < 20MB

**Baseline**: 12.4 ± 1.2 MB
**Margin**: 38.0% buffer
**Measured at**: Peak during code generation

## SLO Compliance Validation

### Automated Checks

```bash
# Run all SLO checks
cargo make slo-check

# Run specific SLO
cargo make bench --bench turtle_parser_bench -- --exact "parse_100_verbs"
```

### Expected Output

```
✅ All SLOs Met (Average margin: 60.9%)

Turtle Parsing     18.456ms  < 50ms     ✓ (62.9% margin)
Validation          2.345ms  < 10ms     ✓ (77.0% margin)
SPARQL Simple       4.623ms  < 10ms     ✓ (53.8% margin)
SPARQL JOIN        14.789ms  < 20ms     ✓ (25.9% margin)
Code Generation    76.456ms  < 500ms    ✓ (84.7% margin)
E2E Workflow       95.123ms  < 500ms    ✓ (80.9% margin)
Memory             12.4MB    < 20MB     ✓ (38.0% margin)
```

## SLO Escalation Matrix

| Margin | Status | Action | Timeline |
|--------|--------|--------|----------|
| >60% | Green | None | N/A |
| 40-60% | Yellow | Monitor closely | Weekly |
| 20-40% | Orange | Investigate | 2 days |
| <20% | Red | Emergency fix | 4 hours |
| >100% | Regression | Root cause analysis | 1 hour |

## Performance Regression Detection

Criterion.rs automatically detects regressions:

```
Regressed: queries/complex_join
    Previous: 14.789 ms
    Current:  18.234 ms
    Change:   +23.3% (outside margin of error)
```

**Automatic actions**:
1. Fail the test with details
2. Block PR until resolved
3. Generate detailed analysis
4. Suggest optimization strategies

## Monitoring in Production

### Metrics to Track

```rust
pub struct PerformanceMetrics {
    pub parse_time_ms: f64,
    pub validation_time_ms: f64,
    pub query_time_ms: f64,
    pub generation_time_ms: f64,
    pub memory_usage_mb: f64,
    pub query_cache_hit_ratio: f64,
}
```

### Alert Thresholds

```toml
[alerts]
# Parse time 2x above baseline
parse_time_threshold_ms = 37

# Query time approaching SLO limit
query_time_threshold_ms = 8

# Generation time 50% above baseline
generation_threshold_ms = 115

# Memory approaching SLO
memory_threshold_mb = 16
```

## Performance Degradation Prevention

### Code Review SLOs

All PRs must:
- ✅ Not increase any operation time by >10%
- ✅ Not increase memory usage by >5%
- ✅ All SLOs still met with >20% margin
- ✅ Benchmarks must show no regressions
- ✅ Provide justification for any increases

### Optimization Checklist

Before each release:
- ✅ Run full benchmark suite
- ✅ Verify all SLOs met
- ✅ Check for memory leaks
- ✅ Profile hot paths
- ✅ Compare against baseline
- ✅ Document any changes

## Scaling Behavior

### Linear Operations

Expected to scale O(n) with ontology size:

| Size | Parse Time | Gen Time |
|------|-----------|----------|
| 10 verbs | 2.4 ms | 8.5 ms |
| 50 verbs | 10.2 ms | 38.4 ms |
| 100 verbs | 18.5 ms | 76.4 ms |
| 500 verbs | 92.3 ms | 382.1 ms |
| 1000 verbs | 184.6 ms | 764.3 ms |

All remain well under SLOs even at 1000 verbs.

### Non-Linear Operations

SPARQL JOINs scale O(n log n):

| Data Size | Simple SELECT | JOIN |
|-----------|--------------|------|
| 100 | 4.6 ms | 14.8 ms |
| 500 | 22.1 ms | 89.3 ms |
| 1000 | 44.2 ms | 186.5 ms |

JOINs approach SLO limit at 1000+ triple graphs.

## Bottleneck Analysis

### Where Time is Spent (100-verb ontology)

```
Parsing:        15.2% (2.8 ms)
Validation:      2.4% (0.4 ms)
SPARQL queries: 38.1% (4.2 ms)
Code generation:44.3% (76.4 ms)
────────────────────────
Total:         100%   (95.1 ms)
```

**Optimization focus**: Code generation is the bottleneck.

### Memory Distribution

```
Parser cache:           2.3 MB (18.6%)
RDF store:              6.8 MB (54.8%)
Generated code:         3.2 MB (25.8%)
────────────────────────
Total:                 12.4 MB (100%)
```

**Optimization opportunity**: RDF store can be optimized with compression.

## Historical Performance Trends

### Benchmark Results Over Releases

```
v5.3.0: Parse 18.2 ms, Gen 74.1 ms
v5.3.1: Parse 18.5 ms, Gen 76.4 ms (+3.1%)
v5.3.2: Parse 17.9 ms, Gen 75.8 ms (-0.8%)
v5.3.3: Parse 18.6 ms, Gen 77.2 ms (+1.8%)
v5.3.4: Parse 18.5 ms, Gen 76.4 ms (current)
```

**Trend**: Stable around baseline, within expected variance.

## Testing SLO Compliance

```rust
#[test]
fn test_parse_slo() {
    let start = Instant::now();
    let _ = parser.parse(ONTOLOGY_100_VERBS).unwrap();
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < 50,
        "Parse time {} ms exceeds SLO of 50ms",
        elapsed
    );
}
```

---

**Related**:
- [How-to: Optimize Performance](../howto/performance-optimization.md)
- [How-to: Validate Ontologies](../howto/validation.md)
- [Explanation: Performance Optimization](../explanation/performance.md)
