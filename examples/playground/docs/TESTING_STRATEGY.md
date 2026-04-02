# Comprehensive Testing Strategy for clap-noun-verb Playground

## Executive Summary

This document provides a complete testing strategy for the clap-noun-verb playground CLI, covering the test pyramid, critical untested paths, property-based testing, performance benchmarks, mutation testing, chaos engineering, and CI/CD pipeline design.

---

## 1. Current State Analysis

### 1.1 Existing Test Coverage

| Layer | Location | Tests | Coverage |
|-------|----------|-------|----------|
| Integration (CLI) | `tests/cli_v5_features.rs` | 30 | ~85% of CLI commands |
| Unit (papers) | `src/domain/papers.rs` | 6 | ~70% |
| Unit (thesis) | `src/domain/thesis.rs` | 5 | ~65% |
| Unit (config) | `src/domain/config.rs` | 5 | ~75% |
| Unit (ontology) | `src/domain/ontology.rs` | 3 | ~50% |
| Unit (introspection) | `src/domain/introspection.rs` | 2 | ~40% |
| Unit (formats) | `src/domain/formats.rs` | 3 | ~60% |
| Unit (middleware) | `src/domain/middleware.rs` | 2 | ~50% |
| Unit (telemetry) | `src/domain/telemetry.rs` | 3 | ~55% |
| Unit (completions) | `src/domain/completions.rs` | 3 | ~45% |
| Unit (io) | `src/integration/io.rs` | 2 | ~40% |
| Unit (rdf) | `src/integration/rdf.rs` | 3 | ~50% |
| Unit (templates) | `src/integration/templates.rs` | 1 | ~20% |

**Total: ~68 tests, estimated 55-60% line coverage**

### 1.2 Identified Gaps

#### Critical Untested Paths
1. **Error handling in CLI layer** - Only 2 error tests exist
2. **Template rendering with actual templates** - No template files present
3. **File I/O edge cases** - Permission errors, disk full scenarios
4. **SPARQL query edge cases** - Malformed queries, large result sets
5. **Shell completion output validation** - PowerShell and Elvish untested
6. **Format output edge cases** - Deeply nested JSON, Unicode strings
7. **Concurrent access** - No thread safety tests
8. **Resource cleanup** - File handles, temp directories

#### Modules Lacking Adequate Tests
1. `templates.rs` - Only 1 test, no actual rendering tests
2. `introspection.rs` - Missing edge cases for empty capabilities
3. `completions.rs` - Missing validation of generated script syntax

---

## 2. Test Pyramid Design

```
                    /\
                   /  \
                  / E2E \           5 tests (5%)
                 /  (10) \
                /--------\
               /          \
              / Integration \       20 tests (20%)
             /     (30+)     \
            /----------------\
           /                  \
          /      Unit Tests    \   75 tests (75%)
         /        (80+)         \
        /--------------------------\
```

### 2.1 Unit Tests (75% - Target: 80+ tests)

#### Domain Layer (Pure Functions)

**papers.rs** (Target: 15 tests)
```rust
// Current: 6 tests
// Missing tests:
- test_paper_family_serialization_roundtrip
- test_paper_section_empty_content
- test_paper_section_unicode_content
- test_paper_all_families_have_unique_names
- test_paper_all_families_have_descriptions
- test_validation_result_path_with_spaces
- test_validation_result_empty_path
- test_validation_result_path_too_long
- test_paper_new_dsr_sections_correct
```

**thesis.rs** (Target: 12 tests)
```rust
// Current: 5 tests
// Missing tests:
- test_schedule_for_all_families
- test_schedule_steps_ordered
- test_thesis_family_numbers_sequential
- test_htf_component_descriptions_nonempty
- test_thesis_structure_immutable
- test_schedule_step_descriptions_nonempty
- test_all_families_have_schedules
```

**config.rs** (Target: 12 tests)
```rust
// Current: 5 tests
// Missing tests:
- test_config_get_or_default_returns_default
- test_config_all_entries_consistent
- test_config_key_from_str_all_variants
- test_config_result_serialization
- test_config_empty_value_handling
- test_config_special_characters_in_value
- test_config_key_case_sensitivity
```

**ontology.rs** (Target: 10 tests)
```rust
// Current: 3 tests
// Missing tests:
- test_capability_with_multiple_args
- test_sparql_query_type_custom
- test_ontology_triple_serialization
- test_effect_type_equality
- test_capability_idempotent_effect
- test_sparql_select_by_effect_mutating
- test_sparql_select_by_effect_readonly
```

**introspection.rs** (Target: 8 tests)
```rust
// Current: 2 tests
// Missing tests:
- test_introspection_empty_capabilities
- test_introspection_groups_by_noun
- test_noun_metadata_verb_count
- test_execution_contract_readonly
- test_execution_contract_idempotent
- test_isolation_level_mapping
```

**formats.rs** (Target: 10 tests)
```rust
// Current: 3 tests
// Missing tests:
- test_format_yaml_nested_objects
- test_format_yaml_arrays
- test_format_table_empty_array
- test_format_table_non_object_array
- test_format_plain_nested
- test_format_json_unicode
- test_format_from_str_aliases
```

**middleware.rs** (Target: 8 tests)
```rust
// Current: 2 tests
// Missing tests:
- test_logging_config_defaults
- test_auth_config_defaults
- test_rate_limit_config_defaults
- test_cache_config_defaults
- test_profiling_config_defaults
- test_cache_hit_rate_zero_total
```

**telemetry.rs** (Target: 10 tests)
```rust
// Current: 3 tests
// Missing tests:
- test_span_duration_calculation
- test_span_multiple_attributes
- test_span_parent_span_id
- test_attribute_value_conversions
- test_cli_metrics_zero_invocations
- test_execution_receipt_args
- test_generate_id_uniqueness
```

**completions.rs** (Target: 10 tests)
```rust
// Current: 3 tests
// Missing tests:
- test_zsh_completion_contains_compdef
- test_fish_completion_structure
- test_powershell_completion_contains_register
- test_elvish_completion_structure
- test_shell_type_file_extensions
- test_completion_script_cli_name
- test_empty_capabilities_completions
```

#### Integration Layer

**io.rs** (Target: 8 tests)
```rust
// Current: 2 tests
// Missing tests:
- test_write_paper_creates_parent_dir
- test_write_paper_overwrites_existing
- test_write_config_file_creates_dir
- test_read_config_file_exists
- test_ensure_output_dir_idempotent
- test_write_paper_invalid_path
```

**rdf.rs** (Target: 10 tests)
```rust
// Current: 3 tests
// Missing tests:
- test_sparql_select_capabilities
- test_sparql_select_by_noun
- test_sparql_invalid_query_graceful
- test_sparql_timeout_handling
- test_resolve_uri_cnv_prefix
- test_resolve_uri_rdf_prefix
- test_export_turtle_prefixes
```

**templates.rs** (Target: 5 tests)
```rust
// Current: 1 test
// Missing tests:
- test_render_paper_context_sections
- test_render_paper_all_families
- test_render_paper_missing_template_error
- test_template_engine_valid_directory
```

### 2.2 Integration Tests (20% - Target: 25 tests)

**Location**: `tests/integration/`

```rust
// tests/integration/mod.rs
mod cli_commands;
mod output_formats;
mod error_handling;
mod file_operations;
mod rdf_operations;
```

**cli_commands.rs** (10 tests)
```rust
- test_full_paper_generation_workflow
- test_thesis_schedule_all_families
- test_config_set_get_roundtrip
- test_meta_introspect_json_valid
- test_meta_ontology_sparql_integration
- test_completions_all_shells
- test_quiet_flag_suppresses_banner
- test_format_flag_all_commands
- test_help_all_subcommands
- test_version_format
```

**output_formats.rs** (5 tests)
```rust
- test_json_output_parseable
- test_yaml_output_parseable
- test_table_output_alignment
- test_plain_output_human_readable
- test_format_consistency_across_commands
```

**error_handling.rs** (5 tests)
```rust
- test_invalid_noun_error_message
- test_invalid_verb_error_message
- test_missing_required_arg_error
- test_invalid_format_flag
- test_invalid_shell_type_error
```

**file_operations.rs** (3 tests)
```rust
- test_paper_generation_creates_file
- test_custom_output_path
- test_nested_output_directory
```

**rdf_operations.rs** (2 tests)
```rust
- test_sparql_custom_query
- test_ontology_export_valid_turtle
```

### 2.3 End-to-End Tests (5% - Target: 5 tests)

**Location**: `tests/e2e/`

```rust
// tests/e2e/workflows.rs
- test_complete_paper_authoring_workflow
- test_ai_agent_introspection_workflow
- test_shell_completion_installation_workflow
- test_config_persistence_workflow
- test_multi_format_export_workflow
```

---

## 3. Property-Based Testing with Proptest

### 3.1 Test Hypotheses

**Hypothesis 1: PaperFamily Parsing Roundtrip**
```rust
proptest! {
    #[test]
    fn paper_family_roundtrip(family in paper_family_strategy()) {
        let name = family.name();
        let parsed = PaperFamily::from_str(name);
        prop_assert_eq!(parsed, Some(family));
    }
}
```

**Hypothesis 2: OutputFormat Determinism**
```rust
proptest! {
    #[test]
    fn format_output_deterministic(
        data in arbitrary_json_value(),
        format in output_format_strategy()
    ) {
        let result1 = format_output(&data, format);
        let result2 = format_output(&data, format);
        prop_assert_eq!(result1, result2);
    }
}
```

**Hypothesis 3: Ontology Triple Consistency**
```rust
proptest! {
    #[test]
    fn capability_triples_count(
        noun in "[a-z]+",
        verb in "[a-z]+",
        desc in ".*"
    ) {
        let cap = CliCapability::read_only(&noun, &verb, &desc);
        let triples = cap.to_triples();
        prop_assert_eq!(triples.len(), 5); // Always 5 triples per capability
    }
}
```

**Hypothesis 4: Config Immutability**
```rust
proptest! {
    #[test]
    fn config_with_value_immutable(
        key in "output_dir|default_family|latex_engine|ontology_path",
        value in ".*"
    ) {
        let config1 = Config::default();
        let original_value = config1.get(&key).map(|s| s.to_string());
        let _config2 = config1.with_value(&key, &value);
        prop_assert_eq!(config1.get(&key).map(|s| s.to_string()), original_value);
    }
}
```

**Hypothesis 5: Shell Completion Non-Empty**
```rust
proptest! {
    #[test]
    fn completion_scripts_nonempty(
        shell in shell_type_strategy(),
        cli_name in "[a-z][a-z0-9_]{0,15}"
    ) {
        let caps = build_playground_ontology();
        let script = generate_completion_script(&cli_name, &caps, shell);
        prop_assert!(!script.script.is_empty());
        prop_assert!(script.script.contains(&cli_name));
    }
}
```

**Hypothesis 6: ExecutionSpan Duration Non-Negative**
```rust
proptest! {
    #[test]
    fn span_duration_non_negative(operation in ".*") {
        let span = ExecutionSpan::new(&operation).complete(SpanStatus::Ok);
        if let Some(duration) = span.duration_ms {
            prop_assert!(duration >= 0);
        }
    }
}
```

### 3.2 Proptest Configuration

```rust
// tests/proptest.rs
use proptest::prelude::*;

fn paper_family_strategy() -> impl Strategy<Value = PaperFamily> {
    prop_oneof![
        Just(PaperFamily::IMRaD),
        Just(PaperFamily::Papers),
        Just(PaperFamily::Argument),
        Just(PaperFamily::Contribution),
        Just(PaperFamily::Monograph),
        Just(PaperFamily::DSR),
        Just(PaperFamily::Narrative),
    ]
}

fn output_format_strategy() -> impl Strategy<Value = OutputFormat> {
    prop_oneof![
        Just(OutputFormat::Json),
        Just(OutputFormat::JsonPretty),
        Just(OutputFormat::Yaml),
        Just(OutputFormat::Table),
        Just(OutputFormat::Plain),
    ]
}

fn shell_type_strategy() -> impl Strategy<Value = ShellType> {
    prop_oneof![
        Just(ShellType::Bash),
        Just(ShellType::Zsh),
        Just(ShellType::Fish),
        Just(ShellType::PowerShell),
        Just(ShellType::Elvish),
    ]
}
```

---

## 4. Performance & Benchmark Tests

### 4.1 Target SLOs

| Operation | Target | Threshold |
|-----------|--------|-----------|
| CLI startup | <50ms | 100ms |
| Paper generation | <100ms | 200ms |
| Ontology export | <50ms | 100ms |
| SPARQL query (100 triples) | <10ms | 50ms |
| Shell completion gen | <20ms | 50ms |
| Format output (1KB) | <5ms | 20ms |

### 4.2 Benchmark Suite

```rust
// benches/cli_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn benchmark_paper_generation(c: &mut Criterion) {
    c.bench_function("paper_generation_imrad", |b| {
        b.iter(|| {
            let paper = Paper::new(PaperFamily::IMRaD, None, None);
            black_box(paper)
        })
    });
}

fn benchmark_ontology_building(c: &mut Criterion) {
    c.bench_function("build_ontology", |b| {
        b.iter(|| {
            let caps = build_playground_ontology();
            black_box(caps)
        })
    });
}

fn benchmark_format_output(c: &mut Criterion) {
    let data = serde_json::json!({
        "families": PaperFamily::all().iter().map(|f| f.name()).collect::<Vec<_>>(),
        "count": 7
    });

    c.bench_function("format_json", |b| {
        b.iter(|| format_output(&data, OutputFormat::Json))
    });

    c.bench_function("format_yaml", |b| {
        b.iter(|| format_output(&data, OutputFormat::Yaml))
    });
}

fn benchmark_sparql_query(c: &mut Criterion) {
    let caps = build_playground_ontology();
    let store = init_ontology_store(&caps).unwrap();

    c.bench_function("sparql_count", |b| {
        b.iter(|| execute_sparql(&store, "SELECT (COUNT(*) as ?n) WHERE { ?s ?p ?o }"))
    });
}

fn benchmark_completion_generation(c: &mut Criterion) {
    let caps = build_playground_ontology();

    for shell in ShellType::all() {
        c.bench_function(&format!("completion_{}", shell.name()), |b| {
            b.iter(|| generate_completion_script("playground", &caps, shell))
        });
    }
}

criterion_group!(
    benches,
    benchmark_paper_generation,
    benchmark_ontology_building,
    benchmark_format_output,
    benchmark_sparql_query,
    benchmark_completion_generation
);
criterion_main!(benches);
```

### 4.3 Memory Benchmarks

```rust
// benches/memory_benchmarks.rs
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

#[test]
fn test_ontology_memory_usage() {
    let before = ALLOCATED.load(Ordering::SeqCst);
    let caps = build_playground_ontology();
    let _store = init_ontology_store(&caps).unwrap();
    let after = ALLOCATED.load(Ordering::SeqCst);

    let usage_kb = (after - before) / 1024;
    assert!(usage_kb < 1024, "Ontology store should use less than 1MB");
}
```

---

## 5. Mutation Testing Strategy

### 5.1 Tool: cargo-mutants

```bash
# Install
cargo install cargo-mutants

# Run mutation testing
cargo mutants --package playground-cli

# Run with filtering
cargo mutants --package playground-cli --file src/domain/papers.rs
```

### 5.2 Mutation Coverage Targets

| Module | Target Mutation Score |
|--------|----------------------|
| papers.rs | >85% |
| thesis.rs | >80% |
| config.rs | >85% |
| ontology.rs | >75% |
| formats.rs | >80% |
| completions.rs | >70% |
| rdf.rs | >70% |

### 5.3 Critical Mutation Locations

```rust
// High-value mutation targets (must survive mutations)
// 1. PaperFamily::from_str - string matching
// 2. ValidationResult::validate_path - conditional logic
// 3. ThesisSchedule::for_family - match arms
// 4. OutputFormat::from_str - format parsing
// 5. execute_sparql_with_timeout - timeout logic
// 6. format_output - serialization logic
```

### 5.4 Mutation Test Suite

```rust
// tests/mutation_tests.rs
// These tests are designed to catch common mutations

#[test]
fn mutation_catcher_paper_family_boundary() {
    // Catch off-by-one in family count
    assert_eq!(PaperFamily::all().len(), 7);
    // Catch string comparison mutations
    assert!(PaperFamily::from_str("imrad").is_some());
    assert!(PaperFamily::from_str("IMRAD").is_some());
    assert!(PaperFamily::from_str("IMRaD").is_some());
}

#[test]
fn mutation_catcher_validation_logic() {
    // Catch boolean inversions
    let valid = ValidationResult::validate_path("test.tex");
    assert!(valid.is_valid);
    assert!(valid.structure_valid);

    let invalid = ValidationResult::validate_path("test.doc");
    assert!(!invalid.is_valid);
    assert!(!invalid.structure_valid);
}

#[test]
fn mutation_catcher_schedule_order() {
    // Catch ordering mutations
    let schedule = ThesisSchedule::for_family(&PaperFamily::IMRaD);
    for (i, step) in schedule.steps.iter().enumerate() {
        assert_eq!(step.order as usize, i + 1);
    }
}
```

---

## 6. Chaos & Resilience Testing

### 6.1 Fault Injection Scenarios

**Scenario 1: File System Failures**
```rust
#[test]
fn chaos_file_permission_denied() {
    // Test behavior when output directory is read-only
    // Requires temp directory with restricted permissions
}

#[test]
fn chaos_disk_full_simulation() {
    // Test behavior when disk write fails
    // Mock write operation to return ENOSPC
}
```

**Scenario 2: Template Engine Failures**
```rust
#[test]
fn chaos_template_syntax_error() {
    // Test graceful handling of malformed templates
}

#[test]
fn chaos_template_infinite_loop() {
    // Test timeout handling for recursive templates
}
```

**Scenario 3: RDF Store Failures**
```rust
#[test]
fn chaos_sparql_malformed_query() {
    let store = init_ontology_store(&[]).unwrap();
    let result = execute_sparql(&store, "INVALID SPARQL");
    // Should return empty, not panic
    assert!(result.is_ok());
}

#[test]
fn chaos_sparql_timeout() {
    // Test query timeout handling
    let store = init_ontology_store(&build_playground_ontology()).unwrap();
    let result = execute_sparql_with_timeout(&store,
        "SELECT * WHERE { ?s ?p ?o . ?s ?p2 ?o2 . ?s ?p3 ?o3 }",
        1 // 1ms timeout
    );
    assert!(result.is_ok()); // Graceful degradation
}
```

**Scenario 4: Resource Exhaustion**
```rust
#[test]
fn chaos_large_ontology() {
    // Test behavior with 10,000+ capabilities
    let mut caps = Vec::new();
    for i in 0..10000 {
        caps.push(CliCapability::read_only(
            &format!("noun{}", i),
            &format!("verb{}", i),
            "test"
        ));
    }
    let store = init_ontology_store(&caps);
    assert!(store.is_ok());
}
```

### 6.2 Recovery Testing

```rust
#[test]
fn recovery_partial_write() {
    // Simulate interrupted file write
    // Verify cleanup and error message
}

#[test]
fn recovery_corrupted_config() {
    // Simulate corrupted config file
    // Verify fallback to defaults
}
```

---

## 7. Test Data & Fixtures Strategy

### 7.1 Fixture Directory Structure

```
tests/
  fixtures/
    papers/
      imrad_sample.tex
      argument_sample.tex
      minimal.tex
      unicode_content.tex
    configs/
      default.toml
      custom.toml
      invalid.toml
    sparql/
      select_all.sparql
      select_by_noun.sparql
      complex_query.sparql
    golden/
      introspection.json
      ontology.ttl
      completions/
        bash.sh
        zsh.sh
        fish.fish
```

### 7.2 Test Data Generators

```rust
// tests/generators.rs
pub fn sample_paper(family: PaperFamily) -> Paper {
    Paper::new(family, Some("Test Paper".to_string()), Some("Test Author".to_string()))
}

pub fn sample_capabilities(count: usize) -> Vec<CliCapability> {
    (0..count).map(|i| {
        CliCapability::read_only(
            &format!("noun{}", i % 5),
            &format!("verb{}", i),
            &format!("Description {}", i)
        )
    }).collect()
}

pub fn sample_config() -> Config {
    Config::default()
        .with_value("output_dir", "/tmp/test")
        .with_value("default_family", "DSR")
}
```

### 7.3 Golden File Testing

```rust
// tests/golden.rs
use insta::assert_snapshot;

#[test]
fn golden_introspection_output() {
    let caps = build_playground_ontology();
    let response = IntrospectionResponse::from_capabilities(
        "playground", "2.0.0", "Test", &caps
    );
    let json = serde_json::to_string_pretty(&response).unwrap();
    assert_snapshot!(json);
}

#[test]
fn golden_turtle_export() {
    let caps = build_playground_ontology();
    let turtle = export_turtle(&caps);
    assert_snapshot!(turtle);
}

#[test]
fn golden_bash_completion() {
    let caps = build_playground_ontology();
    let script = generate_completion_script("playground", &caps, ShellType::Bash);
    assert_snapshot!(script.script);
}
```

---

## 8. CI/CD Test Pipeline Design

### 8.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run unit tests
        run: cargo test --lib --workspace
        timeout-minutes: 5

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage

      - uses: codecov/codecov-action@v3
        with:
          files: coverage/cobertura.xml

  integration-tests:
    runs-on: ubuntu-latest
    needs: unit-tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build binary
        run: cargo build --release

      - name: Run integration tests
        run: cargo test --test '*' --release
        timeout-minutes: 10

  property-tests:
    runs-on: ubuntu-latest
    needs: unit-tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run proptest
        run: cargo test proptest --features proptest
        env:
          PROPTEST_CASES: 1000
        timeout-minutes: 15

  benchmarks:
    runs-on: ubuntu-latest
    needs: integration-tests
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run benchmarks
        run: cargo bench -- --save-baseline main

      - name: Store benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmarks
          path: target/criterion

  mutation-tests:
    runs-on: ubuntu-latest
    needs: unit-tests
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation testing
        run: cargo mutants --timeout 60 --jobs 4
        timeout-minutes: 30
        continue-on-error: true

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: mutants.out/

  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### 8.2 Pre-commit Hooks

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --check
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-test-quick
        name: cargo test (quick)
        entry: cargo test --lib --quiet
        language: system
        types: [rust]
        pass_filenames: false
```

### 8.3 Test Stages

| Stage | Tests | Trigger | Duration |
|-------|-------|---------|----------|
| Pre-commit | fmt, clippy, unit (lib) | Every commit | <1min |
| CI Fast | Unit + Integration | PR, push | 5-10min |
| CI Full | + Property + Golden | PR merge | 15-20min |
| Nightly | + Mutation + Chaos | Scheduled | 30-60min |
| Release | + Benchmarks + Security | Tag | 20-30min |

---

## 9. Test Automation Strategy

### 9.1 Recommended Testing Libraries

| Library | Purpose | Version |
|---------|---------|---------|
| `proptest` | Property-based testing | 1.4 |
| `insta` | Snapshot testing | 1.34 |
| `criterion` | Benchmarking | 0.5 |
| `rstest` | Parameterized tests | 0.18 |
| `mockall` | Mocking (integration only) | 0.12 |
| `tempfile` | Temp directory handling | 3.9 |
| `assert_cmd` | CLI testing | 2.0 |
| `predicates` | Assertion matchers | 3.0 |
| `cargo-tarpaulin` | Coverage | 0.27 |
| `cargo-mutants` | Mutation testing | 24.2 |

### 9.2 Cargo.toml Dev Dependencies

```toml
[dev-dependencies]
proptest = "1.4"
insta = { version = "1.34", features = ["json", "yaml"] }
criterion = { version = "0.5", features = ["html_reports"] }
rstest = "0.18"
tempfile = "3.9"
assert_cmd = "2.0"
predicates = "3.0"

[[bench]]
name = "cli_benchmarks"
harness = false
```

### 9.3 Test Organization

```
playground/
  src/
    domain/
      *.rs          # Unit tests inline with #[cfg(test)]
    integration/
      *.rs          # Unit tests inline with #[cfg(test)]
  tests/
    cli_v5_features.rs    # Existing CLI integration tests
    integration/
      mod.rs
      cli_commands.rs
      output_formats.rs
      error_handling.rs
    proptest/
      mod.rs
      paper_props.rs
      format_props.rs
    golden/
      mod.rs
      introspection.rs
      completions.rs
    e2e/
      workflows.rs
    fixtures/
      ...
  benches/
    cli_benchmarks.rs
    memory_benchmarks.rs
```

---

## 10. Summary & Recommendations

### 10.1 Immediate Actions (Week 1)

1. Add missing unit tests to achieve 80% coverage
2. Set up proptest with 6 key hypotheses
3. Create golden file tests for stable outputs
4. Add benchmark suite with criterion

### 10.2 Short-term (Week 2-3)

1. Implement integration test suite
2. Configure CI/CD pipeline
3. Add mutation testing to PR workflow
4. Create test fixtures directory

### 10.3 Medium-term (Month 1)

1. Implement chaos testing scenarios
2. Add memory benchmarks
3. Create E2E workflow tests
4. Achieve 85%+ mutation score

### 10.4 Coverage Targets

| Metric | Current | Target |
|--------|---------|--------|
| Line Coverage | ~55% | 85% |
| Branch Coverage | ~45% | 75% |
| Mutation Score | N/A | 80% |
| Test Count | ~68 | 110+ |

### 10.5 Test Pyramid Summary

| Layer | Current | Target | Priority |
|-------|---------|--------|----------|
| Unit | 38 | 80 | HIGH |
| Integration | 30 | 25 | MEDIUM |
| E2E | 0 | 5 | LOW |
| Property | 0 | 10 | HIGH |
| Performance | 0 | 8 | MEDIUM |

---

## Appendix A: Test File Templates

### A.1 Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Arrange-Act-Assert pattern
    #[test]
    fn test_function_behavior_when_condition() {
        // Arrange
        let input = setup_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_output);
    }

    // Edge case tests
    #[test]
    fn test_function_empty_input() {
        let result = function_under_test("");
        assert!(result.is_none());
    }

    #[test]
    fn test_function_unicode_input() {
        let result = function_under_test("...");
        assert!(result.is_some());
    }
}
```

### A.2 Integration Test Template

```rust
// tests/integration/feature_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

fn playground() -> Command {
    Command::cargo_bin("playground").unwrap()
}

#[test]
fn test_feature_happy_path() {
    playground()
        .args(&["noun", "verb", "arg"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}

#[test]
fn test_feature_error_path() {
    playground()
        .args(&["invalid", "args"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

### A.3 Property Test Template

```rust
// tests/proptest/module_props.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_name(
        input in strategy()
    ) {
        let result = function_under_test(input);
        prop_assert!(invariant_holds(result));
    }
}
```

---

*Document generated for clap-noun-verb playground testing strategy*
*Version: 1.0.0*
*Last updated: 2024*
