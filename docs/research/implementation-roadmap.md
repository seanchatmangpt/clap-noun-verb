# Reflexive Testing Implementation Roadmap
## Actionable Plan for Integration of Proven Testing Frameworks

**Date**: 2026-01-05
**Status**: Ready for Implementation
**Timeline**: 5 weeks (phased rollout)

---

## Executive Summary

This roadmap provides a phased implementation plan to enhance the existing reflexive testing system with proven Rust testing frameworks. The project **already has excellent foundations** (proptest, criterion, insta, loom) - this roadmap focuses on:

1. **Formalizing RDF → Test mapping**
2. **Adding coverage analysis tooling**
3. **Automating test generation**
4. **Implementing SLO enforcement**

---

## Phase 1: Foundation & Audit (Week 1)

**Goal**: Verify current state and establish baselines

### Tasks

#### 1.1 Dependency Audit
```bash
cargo make check
cargo tree --all-features | grep -E "(proptest|criterion|insta|loom)"
```

**Deliverables**:
- ✅ Verify proptest 1.0 → upgrade to 1.5.0
- ✅ Verify criterion 0.5 is current
- ✅ Verify insta 1.0 → upgrade to 1.40.0
- ✅ Verify loom 0.7 is current

**Action Items**:
```toml
# Update Cargo.toml
[dev-dependencies]
proptest = "1.5"
criterion = { version = "0.5", features = ["html_reports"] }
insta = { version = "1.40", features = ["json", "yaml"] }
loom = "0.7"
```

#### 1.2 Add Coverage Tooling
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Add to Makefile.toml
```

**New Makefile Tasks**:
```toml
[tasks.coverage]
description = "Generate code coverage report (HTML + JSON)"
script = '''
cargo tarpaulin --all-features --workspace --timeout 120 \
    --out Html --out Json \
    --output-dir target/coverage \
    --exclude-files 'tests/*' 'benches/*' 'examples/*' 'playground/*' \
    --exclude 'clap-noun-verb-macros'
'''

[tasks.coverage-ci]
description = "Generate coverage for CI (LCOV format)"
script = '''
cargo tarpaulin --all-features --workspace --timeout 120 \
    --out Lcov \
    --output-dir target/coverage \
    --exclude-files 'tests/*' 'benches/*' 'examples/*' 'playground/*'
'''

[tasks.coverage-report]
description = "Open coverage report in browser"
dependencies = ["coverage"]
script = '''
open target/coverage/index.html || xdg-open target/coverage/index.html
'''
```

#### 1.3 Baseline Measurements
```bash
# Establish performance baselines
cargo make bench-baseline

# Establish coverage baseline
cargo make coverage
```

**Deliverables**:
- 📊 Baseline coverage report (current: ~XX%)
- 📊 Baseline performance metrics (criterion)
- 📊 Current RDF coverage (semantic combinations tested)

**Success Criteria**:
- ✅ All dependencies at latest stable versions
- ✅ Coverage tooling integrated into Makefile
- ✅ Baseline reports generated and saved

---

## Phase 2: RDF → Proptest Mapping (Week 2)

**Goal**: Implement automated test generation from RDF ontology

### Tasks

#### 2.1 Enhance SemanticTestGenerator

**File**: `/home/user/clap-noun-verb/clap-noun-verb-macros/src/macros/reflexive_testing.rs`

**Current State**:
- ✅ Extracts combinations from RDF
- ✅ Generates test placeholders
- ⚠️ No proptest strategy generation

**Enhancement**:
```rust
// Add to reflexive_testing.rs

/// Generates proptest strategies from RDF type information
pub struct ProptestStrategyMapper {
    type_strategies: HashMap<String, TokenStream>,
}

impl ProptestStrategyMapper {
    pub fn new() -> Self {
        let mut type_strategies = HashMap::new();

        // Map RDF types to proptest strategies
        type_strategies.insert(
            "String".to_string(),
            quote! { "[a-zA-Z0-9_]{1,100}" }
        );
        type_strategies.insert(
            "boolean".to_string(),
            quote! { any::<bool>() }
        );
        type_strategies.insert(
            "integer".to_string(),
            quote! { any::<i64>() }
        );
        type_strategies.insert(
            "UserId".to_string(),
            quote! { (1..=1000000u64).prop_map(UserId) }
        );

        Self { type_strategies }
    }

    pub fn strategy_for_type(&self, rdf_type: &str) -> Option<TokenStream> {
        self.type_strategies.get(rdf_type).cloned()
    }

    pub fn register_custom_strategy(&mut self, type_name: String, strategy: TokenStream) {
        self.type_strategies.insert(type_name, strategy);
    }
}

/// Enhanced test generation with proptest strategies
pub fn generate_proptest_tests<T>(
    combinations: &[Vec<String>],
    type_info: &HashMap<String, String>,
) -> TokenStream {
    let mapper = ProptestStrategyMapper::new();
    let test_functions: Vec<_> = combinations
        .iter()
        .enumerate()
        .map(|(idx, combo)| {
            let test_name = format_ident!("test_combination_{}", idx);
            let caps_str = combo.join(", ");

            // Generate strategies for each capability's input type
            let strategies = combo
                .iter()
                .filter_map(|cap| {
                    type_info.get(cap).and_then(|ty| mapper.strategy_for_type(ty))
                })
                .collect::<Vec<_>>();

            if strategies.is_empty() {
                // Fallback: simple assertion test
                quote! {
                    #[test]
                    fn #test_name() {
                        let result = execute_capability_combination(&[#(#combo),*]);
                        assert!(result.is_ok(), "Combination failed: {}", #caps_str);
                    }
                }
            } else {
                // Property-based test
                let strategy = if strategies.len() == 1 {
                    strategies[0].clone()
                } else {
                    quote! { (#(#strategies),*) }
                };

                quote! {
                    proptest! {
                        #[test]
                        fn #test_name(input in #strategy) {
                            let result = execute_capability_combination(
                                &[#(#combo),*],
                                input
                            );
                            prop_assert!(
                                result.is_ok(),
                                "Combination failed: {}",
                                #caps_str
                            );
                        }
                    }
                }
            }
        })
        .collect();

    quote! {
        #[cfg(test)]
        mod reflexive_proptest_generated {
            use super::*;
            use proptest::prelude::*;

            #(#test_functions)*
        }
    }
}
```

#### 2.2 Extract Type Information from RDF

```rust
/// Parse input/output types from RDF ontology
pub fn extract_type_mappings(ontology: &str) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for line in ontology.lines() {
        // Parse: cnv:inputType "String"
        if line.contains("cnv:inputType") || line.contains("cnv:outputType") {
            if let Some(cap_name) = extract_capability_from_context(line) {
                if let Some(type_name) = extract_quoted_value(line) {
                    types.insert(cap_name, type_name);
                }
            }
        }
    }

    types
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line[start + 1..].find('"')?;
    Some(line[start + 1..start + 1 + end].to_string())
}
```

#### 2.3 Add Exclusion Rule Support

```rust
/// Parse exclusion rules from RDF ontology
pub fn extract_exclusion_rules(ontology: &str) -> HashMap<String, Vec<String>> {
    let mut exclusions = HashMap::new();

    for line in ontology.lines() {
        // Parse: cli:cap-create cnv:excludes cli:cap-delete
        if line.contains("cnv:excludes") {
            if let (Some(cap1), Some(cap2)) = (
                extract_capability_before("cnv:excludes", line),
                extract_capability_after("cnv:excludes", line),
            ) {
                exclusions.entry(cap1).or_insert_with(Vec::new).push(cap2);
            }
        }
    }

    exclusions
}

/// Generate proptest filter for exclusion rules
pub fn generate_exclusion_filter(
    exclusions: &HashMap<String, Vec<String>>
) -> Option<TokenStream> {
    if exclusions.is_empty() {
        return None;
    }

    let filters: Vec<_> = exclusions
        .iter()
        .map(|(cap1, excluded)| {
            let checks: Vec<_> = excluded
                .iter()
                .map(|cap2| {
                    quote! {
                        !(caps.contains(&#cap1) && caps.contains(&#cap2))
                    }
                })
                .collect();

            quote! { #(#checks)&&* }
        })
        .collect();

    Some(quote! {
        .prop_filter("Exclusion rules", |caps| {
            #(#filters)&&*
        })
    })
}
```

**Deliverables**:
- ✅ Enhanced `SemanticTestGenerator` with proptest integration
- ✅ Type mapping extraction from RDF
- ✅ Exclusion rule support
- ✅ Automated test code generation

**Success Criteria**:
- ✅ Tests generated for all RDF capability combinations
- ✅ Proptest strategies respect RDF type constraints
- ✅ Exclusion rules enforced via `prop_filter`
- ✅ All generated tests pass

---

## Phase 3: Performance SLO Integration (Week 3)

**Goal**: Extract and enforce performance SLOs from RDF ontology

### Tasks

#### 3.1 Extract Performance Metadata

```rust
#[derive(Debug, Clone)]
pub struct PerformanceSLO {
    pub capability: String,
    pub expected_duration_ms: u64,
    pub expected_throughput: Option<u64>,
    pub max_memory_bytes: Option<usize>,
}

/// Extract performance SLOs from RDF ontology
pub fn extract_performance_slos(ontology: &str) -> Vec<PerformanceSLO> {
    let mut slos = Vec::new();

    for block in ontology.split("\n\n") {
        if let Some(capability) = extract_capability_from_block(block) {
            let expected_duration_ms = extract_duration_ms(block);
            let expected_throughput = extract_throughput(block);
            let max_memory_bytes = extract_memory_limit(block);

            if let Some(duration) = expected_duration_ms {
                slos.push(PerformanceSLO {
                    capability,
                    expected_duration_ms: duration,
                    expected_throughput,
                    max_memory_bytes,
                });
            }
        }
    }

    slos
}

fn extract_duration_ms(block: &str) -> Option<u64> {
    for line in block.lines() {
        if line.contains("cnv:expectedDuration") {
            // Parse: cnv:expectedDuration "100"^^xsd:long
            if let Some(value) = extract_quoted_value(line) {
                return value.parse().ok();
            }
        }
    }
    None
}
```

#### 3.2 Generate Criterion Benchmarks

```rust
/// Generate criterion benchmarks from performance SLOs
pub fn generate_criterion_benchmarks(slos: &[PerformanceSLO]) -> TokenStream {
    let bench_functions: Vec<_> = slos
        .iter()
        .map(|slo| {
            let func_name = format_ident!("bench_{}", slo.capability.replace('.', "_"));
            let cap_name = &slo.capability;
            let max_duration = slo.expected_duration_ms;
            let throughput = slo.expected_throughput;

            let throughput_setup = if let Some(ops) = throughput {
                quote! {
                    group.throughput(criterion::Throughput::Elements(#ops));
                }
            } else {
                quote! {}
            };

            quote! {
                fn #func_name(c: &mut criterion::Criterion) {
                    let mut group = c.benchmark_group(#cap_name);

                    #throughput_setup

                    group.bench_function("execute", |b| {
                        b.iter(|| {
                            criterion::black_box(execute_capability(#cap_name))
                        })
                    });

                    // SLO assertion
                    let mean_time = group.mean_estimate();
                    assert!(
                        mean_time.as_millis() <= #max_duration,
                        "Performance SLO violated for {}: {}ms > {}ms",
                        #cap_name,
                        mean_time.as_millis(),
                        #max_duration
                    );

                    group.finish();
                }
            }
        })
        .collect();

    let bench_names: Vec<_> = slos
        .iter()
        .map(|slo| format_ident!("bench_{}", slo.capability.replace('.', "_")))
        .collect();

    quote! {
        use criterion::{criterion_group, criterion_main, Criterion};

        #(#bench_functions)*

        criterion_group!(benches, #(#bench_names),*);
        criterion_main!(benches);
    }
}
```

#### 3.3 Add SLO Check Task

```toml
# Makefile.toml

[tasks.bench]
description = "Run all benchmarks"
command = "cargo"
args = ["bench", "--all-features"]

[tasks.bench-baseline]
description = "Save performance baseline"
script = '''
cargo bench --all-features -- --save-baseline main
echo "Baseline saved to target/criterion/*/base/estimates.json"
'''

[tasks.bench-compare]
description = "Compare against baseline"
script = '''
cargo bench --all-features -- --baseline main
'''

[tasks.slo-check]
description = "Verify performance SLOs (fails on violation)"
dependencies = ["bench"]
script = '''
#!/bin/bash
set -e

# Parse criterion output for SLO violations
VIOLATIONS=$(grep "Performance SLO violated" target/criterion/*/new/estimates.json || true)

if [ -n "$VIOLATIONS" ]; then
    echo "❌ Performance SLO violations detected:"
    echo "$VIOLATIONS"
    exit 1
else
    echo "✅ All performance SLOs met"
fi
'''
```

**Deliverables**:
- ✅ Performance SLO extraction from RDF
- ✅ Automated criterion benchmark generation
- ✅ SLO assertion checks in benchmarks
- ✅ CI integration for SLO verification

**Success Criteria**:
- ✅ All capability benchmarks generated from RDF
- ✅ SLO violations cause test failures
- ✅ Baseline comparison detects regressions
- ✅ CI fails on SLO violations

---

## Phase 4: Coverage Analysis (Week 4)

**Goal**: Combine structural + semantic coverage reporting

### Tasks

#### 4.1 Semantic Coverage Tracking

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SemanticCoverageReport {
    pub total_capabilities: usize,
    pub tested_capabilities: usize,
    pub capability_coverage_percent: f64,

    pub total_combinations: usize,
    pub tested_combinations: usize,
    pub combination_coverage_percent: f64,

    pub untested_capabilities: Vec<String>,
    pub untested_combinations: Vec<Vec<String>>,
}

impl SemanticCoverageReport {
    pub fn from_rdf_and_tests(
        ontology: &str,
        test_results: &TestResults,
    ) -> Self {
        let all_capabilities = extract_all_capabilities(ontology);
        let all_combinations = extract_all_combinations(ontology);

        let tested_capabilities: HashSet<_> = test_results
            .executed_tests
            .iter()
            .flat_map(|t| &t.capabilities)
            .cloned()
            .collect();

        let tested_combinations: HashSet<_> = test_results
            .executed_tests
            .iter()
            .map(|t| t.capabilities.clone())
            .collect();

        let untested_capabilities: Vec<_> = all_capabilities
            .iter()
            .filter(|cap| !tested_capabilities.contains(*cap))
            .cloned()
            .collect();

        let untested_combinations: Vec<_> = all_combinations
            .iter()
            .filter(|combo| !tested_combinations.contains(*combo))
            .cloned()
            .collect();

        Self {
            total_capabilities: all_capabilities.len(),
            tested_capabilities: tested_capabilities.len(),
            capability_coverage_percent:
                (tested_capabilities.len() as f64 / all_capabilities.len() as f64) * 100.0,

            total_combinations: all_combinations.len(),
            tested_combinations: tested_combinations.len(),
            combination_coverage_percent:
                (tested_combinations.len() as f64 / all_combinations.len() as f64) * 100.0,

            untested_capabilities,
            untested_combinations,
        }
    }
}
```

#### 4.2 Combined Coverage Report

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CombinedCoverageReport {
    // Structural coverage (tarpaulin)
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,

    // Semantic coverage (RDF)
    pub semantic: SemanticCoverageReport,

    // Overall score
    pub overall_score: f64,
    pub meets_threshold: bool,
}

impl CombinedCoverageReport {
    pub fn from_reports(
        tarpaulin_json: &str,
        semantic: SemanticCoverageReport,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let tarpaulin: TarpaulinReport = serde_json::from_str(tarpaulin_json)?;

        let overall_score = Self::calculate_score(
            tarpaulin.line_coverage,
            tarpaulin.branch_coverage,
            semantic.capability_coverage_percent,
            semantic.combination_coverage_percent,
        );

        Ok(Self {
            line_coverage: tarpaulin.line_coverage,
            branch_coverage: tarpaulin.branch_coverage,
            function_coverage: tarpaulin.function_coverage,
            semantic,
            overall_score,
            meets_threshold: overall_score >= 80.0,
        })
    }

    fn calculate_score(
        line_cov: f64,
        branch_cov: f64,
        cap_cov: f64,
        combo_cov: f64,
    ) -> f64 {
        // Weighted average
        0.30 * line_cov
            + 0.20 * branch_cov
            + 0.30 * cap_cov
            + 0.20 * combo_cov
    }

    pub fn generate_html_report(&self) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head><title>Coverage Report</title></head>
<body>
    <h1>Combined Coverage Report</h1>
    <h2>Structural Coverage (Tarpaulin)</h2>
    <ul>
        <li>Line Coverage: {:.2}%</li>
        <li>Branch Coverage: {:.2}%</li>
        <li>Function Coverage: {:.2}%</li>
    </ul>

    <h2>Semantic Coverage (RDF)</h2>
    <ul>
        <li>Capability Coverage: {:.2}%</li>
        <li>Combination Coverage: {:.2}%</li>
    </ul>

    <h2>Overall Score: {:.2}% {}</h2>

    <h3>Untested Capabilities ({}):</h3>
    <ul>{}</ul>

    <h3>Untested Combinations ({}):</h3>
    <ul>{}</ul>
</body>
</html>
"#,
            self.line_coverage,
            self.branch_coverage,
            self.function_coverage,
            self.semantic.capability_coverage_percent,
            self.semantic.combination_coverage_percent,
            self.overall_score,
            if self.meets_threshold { "✅ PASS" } else { "❌ FAIL" },
            self.semantic.untested_capabilities.len(),
            self.semantic
                .untested_capabilities
                .iter()
                .map(|c| format!("<li>{}</li>", c))
                .collect::<Vec<_>>()
                .join(""),
            self.semantic.untested_combinations.len(),
            self.semantic
                .untested_combinations
                .iter()
                .map(|combo| format!("<li>{}</li>", combo.join(", ")))
                .collect::<Vec<_>>()
                .join(""),
        )
    }
}
```

#### 4.3 Coverage Report Task

```toml
[tasks.coverage-combined]
description = "Generate combined coverage report (structural + semantic)"
script = '''
#!/bin/bash
set -e

# Run tarpaulin for structural coverage
cargo make coverage-ci

# Generate semantic coverage from RDF
cargo test --all-features -- --test-threads=1 --nocapture | \
    grep "reflexive_coverage" > target/semantic_coverage.txt || true

# Combine reports
cargo run --bin coverage-reporter -- \
    --tarpaulin target/coverage/lcov.info \
    --semantic target/semantic_coverage.txt \
    --output target/combined_coverage.html

echo "Combined coverage report: target/combined_coverage.html"
'''
```

**Deliverables**:
- ✅ Semantic coverage tracking
- ✅ Combined coverage report (structural + semantic)
- ✅ HTML report generation
- ✅ CI threshold enforcement

**Success Criteria**:
- ✅ Overall coverage score ≥ 80%
- ✅ Capability coverage ≥ 90%
- ✅ Combination coverage ≥ 75%
- ✅ All critical paths covered

---

## Phase 5: Quality Validation (Week 5)

**Goal**: Add mutation testing and finalize tooling

### Tasks

#### 5.1 Add Mutation Testing

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Add to Makefile.toml
```

```toml
[tasks.mutants]
description = "Run mutation testing (slow - run weekly)"
script = '''
cargo mutants --all-features --timeout 120 \
    --output target/mutants \
    --exclude 'tests/*' 'benches/*' 'examples/*'
'''

[tasks.mutants-fast]
description = "Run mutation testing on changed files only"
script = '''
# Only test files changed in current branch
CHANGED_FILES=$(git diff --name-only main...)
cargo mutants --all-features --timeout 60 \
    --file $CHANGED_FILES \
    --output target/mutants
'''
```

#### 5.2 Documentation

Create comprehensive documentation:

**Files to Create**:
1. `/home/user/clap-noun-verb/docs/testing/reflexive-testing-guide.md`
2. `/home/user/clap-noun-verb/docs/testing/property-based-testing.md`
3. `/home/user/clap-noun-verb/docs/testing/performance-slos.md`
4. `/home/user/clap-noun-verb/docs/testing/coverage-analysis.md`

#### 5.3 CI Integration

```yaml
# .github/workflows/test-coverage.yml
name: Test & Coverage

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      # Run all tests
      - name: Run tests
        run: cargo make test-all

      # Run property tests
      - name: Run property tests
        run: cargo test --all-features --test advanced_property_tests
        timeout-minutes: 30

      # Generate coverage
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo make coverage-ci

      - name: Upload to codecov
        uses: codecov/codecov-action@v4
        with:
          files: target/coverage/lcov.info
          fail_ci_if_error: true

      # Performance SLO check
      - name: Performance SLO check
        if: github.ref == 'refs/heads/main'
        run: cargo make slo-check

  mutation-testing:
    runs-on: ubuntu-latest
    # Only run on main (slow)
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation testing
        run: cargo make mutants
        continue-on-error: true

      - name: Upload mutants report
        uses: actions/upload-artifact@v4
        with:
          name: mutants-report
          path: target/mutants
```

**Deliverables**:
- ✅ Mutation testing integrated
- ✅ Comprehensive documentation
- ✅ CI/CD pipeline complete
- ✅ Coverage badges and reports

**Success Criteria**:
- ✅ Mutation testing catches > 90% of introduced bugs
- ✅ CI enforces coverage thresholds
- ✅ Documentation complete and reviewed
- ✅ All workflows passing

---

## Success Metrics

### Coverage Thresholds
- **Line Coverage**: ≥ 80%
- **Branch Coverage**: ≥ 70%
- **Capability Coverage**: ≥ 90%
- **Combination Coverage**: ≥ 75%
- **Overall Score**: ≥ 80%

### Performance Thresholds
- **SLO Compliance**: 100% (all benchmarks meet SLOs)
- **Regression Tolerance**: < 10% slowdown
- **Benchmark Stability**: Coefficient of variation < 5%

### Quality Metrics
- **Mutation Score**: ≥ 90% (mutants caught by tests)
- **Proptest Shrinking**: < 10 iterations to minimal case
- **CI Speed**: Full test suite < 15 minutes

---

## Risk Mitigation

### Risk 1: Performance Overhead
**Mitigation**: Property tests only run in dev/CI, not production

### Risk 2: Test Suite Slowdown
**Mitigation**: Parallel execution, strategic test selection

### Risk 3: Coverage Gaming
**Mitigation**: Combine structural + semantic coverage, mutation testing

### Risk 4: RDF Parsing Complexity
**Mitigation**: Use battle-tested RDF libraries (consider `oxrdf`)

---

## Long-Term Enhancements

### Future Phase 6: Advanced Features (Optional)
1. **Formal Verification**: Integrate Kani for critical paths
2. **Fuzz Testing**: Use `cargo-fuzz` with `arbitrary` trait
3. **Concurrency Model Checking**: Expand loom coverage
4. **Symbolic Execution**: Explore KLEE integration

---

## Conclusion

This 5-week roadmap transforms the existing reflexive testing foundation into a **production-grade, automated testing system** that:

1. ✅ Generates property tests from RDF ontology
2. ✅ Enforces performance SLOs automatically
3. ✅ Provides comprehensive coverage analysis
4. ✅ Validates test quality via mutation testing
5. ✅ Integrates seamlessly with CI/CD

**All work builds on proven frameworks** (proptest, criterion, tarpaulin) - no custom reinvention.

**Next Action**: Begin Phase 1 dependency audit and baseline measurements.
