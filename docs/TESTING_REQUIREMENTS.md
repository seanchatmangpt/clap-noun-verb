# Testing Requirements Matrix for clap-noun-verb

**Version**: 26.9.1  
**Last Updated**: 2026-08-20  
**Framework**: Rust CLI framework with noun-verb patterns, proc-macros, and frontier features

This document defines comprehensive testing requirements across 10 dimensions, with explicit pass/fail criteria and automation hooks.

---

## Table of Contents

1. [Unit Test Requirements](#1-unit-test-requirements)
2. [Integration Test Requirements](#2-integration-test-requirements)
3. [Edge Case Testing](#3-edge-case-testing)
4. [Performance Testing](#4-performance-testing)
5. [Feature Interaction Matrix](#5-feature-interaction-matrix)
6. [Platform Testing](#6-platform-testing)
7. [Regression Testing](#7-regression-testing)
8. [Property Testing](#8-property-testing)
9. [Stress Testing](#9-stress-testing)
10. [Documentation Testing](#10-documentation-testing)

---

## 1. Unit Test Requirements

### 1.1 Scope and Philosophy

**What Must Be Tested:**
- Pure functions with deterministic outputs (registry lookup, argument parsing, tree building)
- Trait implementations (NounCommand, VerbCommand, NounVerbError)
- Type system guarantees (TypeMap, VerbContext, AppContext)
- Validation logic (compile-time and runtime)
- Format conversion (JSON serialization, RDF generation)

**When to Skip:**
- ❌ I/O operations that require temp files (use integration tests with assert_fs)
- ❌ Async operations (use tokio-test or integration tests)
- ❌ Terminal interactions (use CLI integration tests)
- ❌ Network calls (use integration tests with mocking)
- ❌ Tests that only check `assert!(result.is_ok())` without verifying behavior

**Required Coverage Minimums:**

| Category | Minimum Coverage | Tools | Automation |
|----------|-----------------|-------|-----------|
| `src/verb.rs` | 85% | `cargo tarpaulin` | Pre-commit hook |
| `src/noun.rs` | 85% | `cargo tarpaulin` | Pre-commit hook |
| `src/registry.rs` | 90% | `cargo tarpaulin` | CI gate |
| `src/tree.rs` | 80% | `cargo tarpaulin` | CI gate |
| `src/error.rs` | 95% | `cargo tarpaulin` | CI gate |
| `src/format.rs` | 75% | `cargo tarpaulin` | CI gate |
| Macro validation | 85% | Unit tests in `tests/` | CI gate |

### 1.2 AAA Pattern Enforcement

**Requirements:**
```rust
#[test]
fn test_verb_command_executes_successfully_with_required_args() {
    // Arrange: Set up state
    let verb = create_test_verb("status");
    let args = VerbArgs::new("status");
    
    // Act: Execute behavior
    let result = verb.execute(&args);
    
    // Assert: Verify observable behavior
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, "running");
}
```

**FAIL Criteria:**
- ❌ Test lacks one or more sections (Arrange, Act, Assert)
- ❌ Test name doesn't describe the scenario (e.g., `test_foo()` instead of `test_foo_returns_error_when_invalid()`)
- ❌ Only checks `is_ok()` without verifying actual result values
- ❌ Setup code appears in Act section (side effects during assertion setup)

### 1.3 Test Categories

#### A. Verb Trait Tests
```
tests/unit/verb_trait.rs:
  ✓ test_verb_execution_with_valid_args
  ✓ test_verb_error_handling_with_missing_args
  ✓ test_verb_context_propagation
  ✓ test_verb_args_extraction_string
  ✓ test_verb_args_extraction_number
  ✓ test_verb_args_extraction_pathbuf
  ✓ test_verb_args_extraction_url
  ✓ test_verb_global_flag_access
  ✓ test_verb_dependency_injection_via_context
```

#### B. Noun Trait Tests
```
tests/unit/noun_trait.rs:
  ✓ test_noun_registration_with_single_verb
  ✓ test_noun_registration_with_multiple_verbs
  ✓ test_noun_validation_rejects_empty_verb_list
  ✓ test_noun_context_creation
  ✓ test_compound_noun_nesting
  ✓ test_noun_duplicate_verb_detection
```

#### C. Registry Tests
```
tests/unit/registry.rs:
  ✓ test_registry_lookup_first_verb
  ✓ test_registry_lookup_middle_verb
  ✓ test_registry_lookup_last_verb
  ✓ test_registry_lookup_nonexistent_verb_returns_none
  ✓ test_registry_command_structure_completeness
  ✓ test_registry_noun_hierarchy
  ✓ test_registry_auto_validate_enabled
  ✓ test_registry_auto_validate_disabled
```

#### D. Tree Building Tests
```
tests/unit/tree_builder.rs:
  ✓ test_flat_command_tree
  ✓ test_nested_command_tree
  ✓ test_deep_nesting_3_levels
  ✓ test_tree_with_root_verb
  ✓ test_tree_pattern_noun_verb
  ✓ test_tree_pattern_service_action
  ✓ test_tree_branch_merging
```

#### E. Error Handling Tests
```
tests/unit/error_handling.rs:
  ✓ test_error_serialization_to_json
  ✓ test_error_severity_classification
  ✓ test_error_action_template_substitution
  ✓ test_structured_error_from_string
  ✓ test_error_context_preservation
  ✓ test_error_chain_formatting
```

#### F. Type System Tests
```
tests/unit/type_system.rs:
  ✓ test_typemap_insert_and_retrieve
  ✓ test_typemap_multiple_types
  ✓ test_typemap_type_isolation
  ✓ test_verb_context_data_access
  ✓ test_verb_context_extension_access
  ✓ test_app_context_concurrent_safe
```

#### G. Macro Validation Tests
```
tests/unit/macro_validation.rs:
  ✓ test_verb_macro_accepts_simple_function
  ✓ test_verb_macro_serializable_return_type
  ✓ test_verb_macro_rejects_non_serializable
  ✓ test_verb_macro_duplicate_verb_detection
  ✓ test_arg_macro_attribute_parsing
  ✓ test_arg_macro_env_variable_detection
  ✓ test_arg_macro_default_value_parsing
```

### 1.4 Automation

**Command:**
```bash
cargo make test-lib-deterministic
```

**CI Gate:**
```bash
cargo make ci  # Runs test-lib-deterministic
```

**Acceptance Criteria:**
- All tests pass in <500ms (parallel execution)
- Coverage report generated: `target/coverage/index.html`
- No test panics outside `#[should_panic]`

---

## 2. Integration Test Requirements

### 2.1 Scope

**What Must Be Tested:**
- End-to-end command dispatch: noun/verb lookup → argument parsing → handler execution
- CLI builder API: registering commands, constructing clap Command tree
- Nested noun hierarchies: services > database > migrate
- Global flag propagation: `--verbose`, `--config` across subcommands
- I/O integration: file reading/writing with clio
- Fixture composition: multiple verbs, multiple nouns

**Testing Tools:**
- `assert_cmd` - CLI integration testing
- `assert_fs` - Temporary file system setup
- `predicates` - Output matching (stdout, stderr, exit codes)
- `tempfile` - Temp directories for I/O tests

### 2.2 Feature Combination Matrix

| Feature Set | Command | Duration | CI |
|------------|---------|----------|-----|
| No features | `cargo test --no-default-features` | <2s | ✓ |
| Default | `cargo test` | <3s | ✓ |
| All features | `cargo test --all-features` | <5s | ✓ |
| Repl only | `cargo test --features repl` | <2s | ✓ |
| Federated only | `cargo test --features federated-network` | <3s | ✓ |
| Otel only | `cargo test --features otel` | <3s | ✓ |

### 2.3 End-to-End Flow Tests

#### A. Basic CLI Construction
```
tests/integration/cli_builder.rs:
  ✓ test_cli_with_single_noun_single_verb
  ✓ test_cli_with_multiple_nouns
  ✓ test_cli_with_nested_nouns_2_levels
  ✓ test_cli_with_nested_nouns_3_levels
  ✓ test_cli_with_global_flags
  ✓ test_cli_with_required_global_args
```

#### B. Command Dispatch
```
tests/integration/command_dispatch.rs:
  ✓ test_dispatch_to_correct_verb_handler
  ✓ test_dispatch_with_arguments_propagation
  ✓ test_dispatch_with_global_flags_inherited
  ✓ test_dispatch_error_propagates_to_output
  ✓ test_dispatch_multiple_calls_deterministic
```

#### C. CLI Execution
```
tests/integration/cli_execution.rs:
  ✓ test_execute_simple_command_returns_success
  ✓ test_execute_command_with_json_output
  ✓ test_execute_command_with_error_json_format
  ✓ test_execute_with_stdin_redirection
  ✓ test_execute_with_stdout_capture
```

#### D. Nested Command Hierarchies
```
tests/integration/nested_commands.rs:
  ✓ test_services_database_migrate_flow
  ✓ test_services_database_backup_flow
  ✓ test_deep_nesting_app_cluster_node_status
  ✓ test_cross_noun_argument_isolation
  ✓ test_nested_global_flag_propagation
```

#### E. I/O Integration
```
tests/integration/io_integration.rs:
  ✓ test_read_input_from_stdin
  ✓ test_read_input_from_file_via_clio
  ✓ test_write_output_to_file
  ✓ test_write_output_to_stdout
  ✓ test_io_error_handling_missing_file
```

#### F. Global Arguments
```
tests/integration/global_args.rs:
  ✓ test_verbose_flag_count_propagation
  ✓ test_config_file_path_propagation
  ✓ test_multiple_global_flags
  ✓ test_global_args_with_nested_nouns
  ✓ test_global_arg_default_values
```

#### G. JSON Output Format
```
tests/integration/json_output.rs:
  ✓ test_success_response_json_structure
  ✓ test_error_response_json_structure
  ✓ test_nested_json_output
  ✓ test_json_output_with_special_characters
  ✓ test_json_array_output
```

### 2.4 Automation

**Command:**
```bash
cargo make test-integration-isolated
```

**CI Gate:**
```bash
cargo make ci  # Includes test-integration-isolated
```

**Acceptance Criteria:**
- All tests pass in <2s single-threaded
- No flaky tests (3 consecutive runs all pass)
- Error messages are JSON-formatted and valid
- No hardcoded paths (use `tempfile::tempdir()`)

---

## 3. Edge Case Testing

### 3.1 Boundary Conditions

#### A. Argument Boundary Tests
```
tests/edge_cases/argument_boundaries.rs:
  ✓ test_empty_string_argument
  ✓ test_very_long_string_argument (10KB)
  ✓ test_unicode_argument_chinese_characters
  ✓ test_unicode_argument_emoji
  ✓ test_null_byte_in_argument
  ✓ test_whitespace_only_argument
  ✓ test_special_shell_chars_in_argument
  ✓ test_argument_count_zero
  ✓ test_argument_count_extreme (1000 args)
```

#### B. Command Name Boundary Tests
```
tests/edge_cases/command_boundaries.rs:
  ✓ test_single_char_noun
  ✓ test_single_char_verb
  ✓ test_max_length_noun (255 chars)
  ✓ test_max_length_verb (255 chars)
  ✓ test_hyphenated_command_names
  ✓ test_uppercase_command_names
  ✓ test_numeric_command_names
```

#### C. Nesting Depth Boundaries
```
tests/edge_cases/nesting_boundaries.rs:
  ✓ test_flat_hierarchy_0_nesting
  ✓ test_single_level_nesting_noun_verb
  ✓ test_deep_nesting_5_levels
  ✓ test_pathological_deep_nesting_10_levels
  ✓ test_wide_noun_registry_100_verbs
```

### 3.2 Error Path Testing

#### A. Input Validation Errors
```
tests/edge_cases/input_errors.rs:
  ✓ test_missing_required_argument_error
  ✓ test_invalid_argument_type_conversion
  ✓ test_argument_out_of_range_error
  ✓ test_url_parsing_invalid_format
  ✓ test_pathbuf_nonexistent_file
  ✓ test_enum_variant_not_found
```

#### B. Command Not Found Errors
```
tests/edge_cases/command_not_found.rs:
  ✓ test_typo_in_noun_name
  ✓ test_typo_in_verb_name
  ✓ test_completely_invalid_command
  ✓ test_case_sensitivity_enforcement
  ✓ test_suggestion_provided_on_typo
```

#### C. Registration Errors
```
tests/edge_cases/registration_errors.rs:
  ✓ test_duplicate_noun_registration_rejected
  ✓ test_duplicate_verb_in_same_noun_rejected
  ✓ test_noun_with_no_verbs_rejected
  ✓ test_empty_command_name_rejected
  ✓ test_whitespace_command_name_rejected
```

#### D. Serialization Errors
```
tests/edge_cases/serialization_errors.rs:
  ✓ test_non_serializable_return_type_compile_error
  ✓ test_json_serialization_failure_handling
  ✓ test_json_output_with_circular_reference
  ✓ test_output_encoding_utf8_validation
```

### 3.3 Recovery Scenarios

#### A. Graceful Degradation
```
tests/edge_cases/recovery.rs:
  ✓ test_continue_after_single_verb_error
  ✓ test_continue_after_missing_global_arg
  ✓ test_continue_after_io_error_on_one_verb
  ✓ test_fallback_when_handler_panics
```

### 3.4 Automation

**Command:**
```bash
cargo test edge_cases::
```

**CI Integration:**
```bash
cargo make test-all  # Includes edge cases
```

**Acceptance Criteria:**
- All tests pass
- Error messages are user-friendly and non-leaking
- No panics in production code paths
- Resource cleanup verified (no leaks in temp files)

---

## 4. Performance Testing

### 4.1 Benchmark Targets and Thresholds

| Operation | Threshold | Tool | Baseline |
|-----------|-----------|------|----------|
| Registry lookup (10 verbs) | <50µs | Criterion | Saved in `target/criterion/` |
| Registry lookup (100 verbs) | <200µs | Criterion | Tracked per commit |
| Argument parsing (simple) | <500µs | Criterion | Tracked per commit |
| Argument parsing (complex, 10 args) | <1ms | Criterion | Tracked per commit |
| Command dispatch | <50µs | Criterion | Tracked per commit |
| Tree building (100 commands) | <5ms | Criterion | Tracked per commit |
| JSON serialization (1KB output) | <100µs | Criterion | Tracked per commit |
| Total CLI startup | <5ms | assert_cmd | Manual sampling |

### 4.2 Benchmark Structure

```
benches/dispatch.rs:
  ✓ bench_registry_lookup_linear_first
  ✓ bench_registry_lookup_linear_middle
  ✓ bench_registry_lookup_linear_last
  ✓ bench_registry_lookup_miss
  ✓ bench_argument_parsing_simple
  ✓ bench_argument_parsing_with_flags
  ✓ bench_command_dispatch_verb_lookup
  ✓ bench_command_dispatch_with_validation
```

### 4.3 SLO Validation

**Incremental Compilation Target:** ≤2s
```bash
# Fresh build from scratch
cargo clean && time cargo build
# Expect: ~0.66s typical (from CLAUDE.md)
```

**Binary Size Target:** ≤10MB
```bash
ls -lh target/release/clap-noun-verb-gen
# Expect: ~2.2MB typical (from CLAUDE.md)
```

### 4.4 Regression Detection

**Baseline Setup:**
```bash
cargo make bench-baseline  # Save as 'main'
```

**Comparison:**
```bash
cargo make bench-compare  # Compare against 'main'
```

**Acceptance Criteria:**
- No regression >5% for core operations
- Regressions >10% trigger investigation issue
- Baseline updated on major refactors

### 4.5 Automation

**Command:**
```bash
cargo make bench
```

**CI Integration:**
```bash
# Benchmarks run in CI but don't gate (informational only)
cargo bench --all-features > bench_report.txt
```

**Continuous Tracking:**
- Criterion generates HTML reports: `target/criterion/report/index.html`
- Baseline comparisons tracked per PR

---

## 5. Feature Interaction Matrix

### 5.1 Feature Combinations

**Core Feature Set:**
- `default` (no features)
- `repl` - Interactive REPL shell
- `process-data` - Data processing operators
- `autonomic` - CI/CD agent policies (implies `process-data`)
- `contrib` - Community verbs (implies `process-data`)
- `federated-network` - Federated capability network
- `otel` - OpenTelemetry instrumentation

**Frontier Features:**
- `meta-framework` - Self-introspection
- `rdf-composition` - RDF ontology integration
- `executable-specs` - Spec-driven development
- `fractal-patterns` - Recursive pattern composition
- `discovery-engine` - Capability discovery
- `learning-trajectories` - Learning path generation
- `reflexive-testing` - Self-testing capabilities
- `economic-sim` - Economic simulation engine
- `quantum-ready` - Quantum algorithm stubs

**Meta-Features:**
- `frontier-semantic` = `meta-framework` + `rdf-composition` + `executable-specs`
- `frontier-intelligence` = `discovery-engine` + `learning-trajectories` + `reflexive-testing`
- `frontier-quality` = `reflexive-testing` + `executable-specs`
- `frontier-all` = All frontier features

### 5.2 Compatibility Matrix

**Mandatory Tests:**

| Feature Combo | Command | Status | Notes |
|---------------|---------|--------|-------|
| No features | `cargo test --no-default-features` | MUST PASS | Minimal compile |
| Default | `cargo test` | MUST PASS | Daily use |
| All features | `cargo test --all-features` | MUST PASS | CI gate |
| repl | `cargo test --features repl` | MUST PASS | Interactive mode |
| federated-network | `cargo test --features federated-network` | MUST PASS | Network code |
| otel | `cargo test --features otel` | MUST PASS | Telemetry |
| frontier-all | `cargo test --features frontier-all` | MUST PASS | Frontier features |
| frontier-semantic | `cargo test --features frontier-semantic` | MUST PASS | Semantic subset |
| frontier-intelligence | `cargo test --features frontier-intelligence` | MUST PASS | Intelligence subset |
| frontier-quality | `cargo test --features frontier-quality` | MUST PASS | Quality subset |

### 5.3 Interaction Tests

```
tests/feature_interactions/compatibility.rs:
  ✓ test_repl_works_with_default_features
  ✓ test_repl_works_with_process_data
  ✓ test_autonomic_requires_process_data
  ✓ test_federated_network_independent
  ✓ test_otel_independent
  ✓ test_frontier_all_includes_all_features
  ✓ test_frontier_semantic_coherent
  ✓ test_frontier_intelligence_coherent
  ✓ test_frontier_quality_coherent
```

```
tests/feature_interactions/mutual_exclusivity.rs:
  ✓ test_no_conflicting_feature_combinations
  ✓ test_macro_expansion_consistent_across_features
```

### 5.4 Frontier Feature Validation

```
tests/frontier/semantic_composition.rs:
  ✓ test_rdf_generation_from_verbs
  ✓ test_ontology_consistency
  ✓ test_semantic_search_capability
  ✓ test_executable_spec_generation

tests/frontier/intelligence_capabilities.rs:
  ✓ test_discovery_engine_verb_enumeration
  ✓ test_learning_path_generation
  ✓ test_reflexive_test_generation

tests/frontier/quality_assurance.rs:
  ✓ test_reflexive_testing_self_validation
  ✓ test_spec_compliance_verification
```

### 5.5 Automation

**Command:**
```bash
cargo make test-feature-combinations
```

**CI Gate:**
```bash
cargo make ci  # Runs all feature combinations
```

**Acceptance Criteria:**
- All feature combinations compile
- All feature combination tests pass
- No feature enables unsafe code outside `linkme`
- Frontier features build independently

---

## 6. Platform Testing

### 6.1 Supported Platforms

| Platform | Architecture | CI | Status |
|----------|--------------|-----|--------|
| Linux | x86_64 | GitHub Actions | Primary target |
| Linux | aarch64 | GitHub Actions (if available) | Secondary target |
| macOS | x86_64 | GitHub Actions | Supported |
| macOS | aarch64 (Apple Silicon) | GitHub Actions | Supported |
| Windows | x86_64 | GitHub Actions (MSVC) | Supported |
| Windows | x86_64 | GitHub Actions (GNU) | Best effort |

### 6.2 Platform-Specific Tests

#### A. Windows-Specific
```
tests/platform/windows.rs (cfg(windows)):
  ✓ test_pathbuf_with_backslashes
  ✓ test_command_line_arg_escaping
  ✓ test_crlf_line_ending_handling
  ✓ test_file_locking_behavior
  ✓ test_env_var_case_insensitivity
```

#### B. Unix-Specific (Linux/macOS)
```
tests/platform/unix.rs (cfg(unix)):
  ✓ test_pathbuf_with_forward_slashes
  ✓ test_signal_handling
  ✓ test_socket_path_support
  ✓ test_posix_permissions
  ✓ test_shell_integration
```

#### C. Endianness-Safe Code
```
tests/platform/endianness.rs:
  ✓ test_binary_serialization_portable
  ✓ test_network_byte_order_handling
  ✓ test_json_number_precision
```

#### D. Bitness-Independent Code
```
tests/platform/bitness.rs:
  ✓ test_usize_conversion_no_overflow
  ✓ test_pointer_arithmetic_bounds
  ✓ test_large_number_handling
```

### 6.3 CI Configuration

**GitHub Actions Matrix:**
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, nightly]
    exclude:
      - os: windows-latest
        rust: nightly  # Optional: skip to reduce CI time

jobs:
  test:
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo make ci
```

### 6.4 Automation

**Local Simulation:**
```bash
# Linux (primary)
cargo test --release

# macOS (if available)
cargo test --release

# Windows (if available)
cargo test --release
```

**CI Gate:**
```bash
# Runs on all platforms automatically
```

**Acceptance Criteria:**
- All tests pass on Linux
- All tests pass on macOS
- All tests pass on Windows
- No platform-specific panics
- Path handling works cross-platform

---

## 7. Regression Testing

### 7.1 Critical Paths (Must Never Regress)

#### A. Core Dispatch Loop
```rust
// Path: verb_definition -> registry -> dispatch -> handler
#[test]
fn test_regression_basic_verb_dispatch_flow() {
    // This is the most critical path - never allowed to break
}
```

**Test Cases:**
```
tests/regression/core_dispatch.rs:
  ✓ test_verb_discovery_via_linkme
  ✓ test_registry_accumulates_all_verbs
  ✓ test_command_tree_hierarchy_construction
  ✓ test_argument_parsing_clap_integration
  ✓ test_handler_execution_correct_noun_verb
  ✓ test_handler_receives_correct_arguments
  ✓ test_handler_receives_correct_global_args
  ✓ test_output_serialized_to_json
  ✓ test_error_serialized_to_json
```

#### B. Macro Compilation
```
tests/regression/macro_compilation.rs:
  ✓ test_verb_macro_compiles
  ✓ test_noun_macro_compiles
  ✓ test_arg_attribute_parsing
  ✓ test_multiple_verbs_per_noun
  ✓ test_nested_nouns_structure
  ✓ test_macro_expansion_idempotent
```

#### C. Type Safety
```
tests/regression/type_safety.rs:
  ✓ test_verb_context_typemap_invariance
  ✓ test_app_context_thread_safety
  ✓ test_serializability_enforcement
  ✓ test_error_type_chain_integrity
```

#### D. JSON Output Format
```
tests/regression/json_format.rs:
  ✓ test_success_response_schema
  ✓ test_error_response_schema
  ✓ test_json_valid_utf8
  ✓ test_json_deserializable
  ✓ test_json_preserves_structure
```

### 7.2 Past Bug Prevention Tests

**Bug Template:** When fixing a bug, add a test with this comment:
```rust
#[test]
fn test_regression_issue_123_verb_dispatch_deadlock() {
    // Regression test for https://github.com/seanchatmangpt/clap-noun-verb/issues/123
    // Bug: Concurrent verb access caused deadlock in CommandRegistry
    // Fix: Changed RwLock to parking_lot::RwLock with timeout
    // This test ensures the bug doesn't resurface
    
    // Arrange, Act, Assert...
}
```

### 7.3 Stability Checks

**Determinism Verification:**
```bash
# Run same test suite multiple times
for i in {1..5}; do
  cargo test --lib --quiet
  if [ $? -ne 0 ]; then
    echo "Test flaked on iteration $i"
    exit 1
  fi
done
echo "All 5 iterations passed - no flakiness detected"
```

**Command:**
```bash
cargo make test-lib-deterministic  # Single-threaded execution
```

### 7.4 Automation

**Command:**
```bash
cargo make test-all
```

**Pre-Release Checklist:**
```bash
cargo make release-check
```

**Acceptance Criteria:**
- All regression tests pass
- No new failures on clean checkout
- Same test results on 3 consecutive runs
- No performance regressions >5%

---

## 8. Property Testing

### 8.1 Invariants to Verify

#### A. Registry Invariants
```
Invariant 1: Every registered verb appears in lookup results
Invariant 2: Registry lookup is deterministic (same input = same output)
Invariant 3: Duplicate registration is rejected or deduplicated
Invariant 4: Noun with no verbs is invalid
Invariant 5: Verb names are unique within a noun
```

#### B. Argument Parsing Invariants
```
Invariant 1: Parsed args match provided values
Invariant 2: Type conversion is reversible (parse → serialize → parse)
Invariant 3: Default values are used when arg not provided
Invariant 4: Required args cause error when missing
Invariant 5: Invalid types cause parse error
```

#### C. JSON Output Invariants
```
Invariant 1: Output is valid JSON
Invariant 2: Output deserializes to original type
Invariant 3: Special characters are properly escaped
Invariant 4: Numbers don't lose precision
Invariant 5: Nested structures maintain hierarchy
```

#### D. Error Handling Invariants
```
Invariant 1: All errors serialize to JSON
Invariant 2: Error JSON contains required fields
Invariant 3: Error chain preserved in output
Invariant 4: No error causes panic
Invariant 5: Recoverable errors don't crash CLI
```

### 8.2 Property-Based Tests (proptest)

```rust
// tests/property/registry_properties.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_registry_lookup_idempotent(verb_name in "\\PC{1,50}") {
        // Arrange: Create registry
        let registry = create_test_registry();
        
        // Act: Look up same verb twice
        let result1 = registry.find_verb(&verb_name);
        let result2 = registry.find_verb(&verb_name);
        
        // Assert: Results are identical
        assert_eq!(result1, result2);
    }
    
    #[test]
    fn prop_argument_parse_then_serialize(value in any::<i32>()) {
        // Invariant: round-trip conversion preserves value
        let parsed = VerbArgs::parse_i32(&value.to_string()).unwrap();
        assert_eq!(parsed, value);
    }
    
    #[test]
    fn prop_json_output_always_valid(result in any::<VerbResult>()) {
        // Invariant: JSON serialization never fails
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
        
        // Invariant: Serialized JSON is deserializable
        let deserialized: Result<VerbResult, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
```

**Test Suite:**
```
tests/property/argument_properties.rs
tests/property/registry_properties.rs
tests/property/json_properties.rs
tests/property/error_properties.rs
tests/property/command_tree_properties.rs
```

### 8.3 Concurrency Properties

```rust
// tests/property/concurrency_properties.rs
use loom::thread;
use proptest::prelude::*;

#[test]
fn loom_typemap_concurrent_access() {
    loom::model(|| {
        let type_map = Arc::new(Mutex::new(TypeMap::new()));
        
        let t1 = {
            let map = type_map.clone();
            thread::spawn(move || {
                let mut m = map.lock().unwrap();
                m.insert(42i32);
            })
        };
        
        let t2 = {
            let map = type_map.clone();
            thread::spawn(move || {
                let m = map.lock().unwrap();
                assert_eq!(m.get::<i32>(), Some(&42));
            })
        };
        
        t1.join().unwrap();
        t2.join().unwrap();
    });
}
```

### 8.4 Automation

**Command:**
```bash
cargo test property_
```

**Runs automatically in:**
```bash
cargo make test-all
```

**Configuration (Cargo.toml):**
```toml
proptest { cases = 1000, max_shrink_iters = 10000 }
loom { max_iterations = 10000 }
```

**Acceptance Criteria:**
- All invariants hold over 1000+ cases
- No counterexamples found
- Loom model checking completes successfully

---

## 9. Stress Testing

### 9.1 Concurrent Access Tests

#### A. Concurrent Registry Lookups
```
tests/stress/concurrent_registry.rs:
  ✓ test_100_concurrent_registry_lookups
  ✓ test_1000_concurrent_registry_lookups
  ✓ test_mixed_register_and_lookup
  ✓ test_concurrent_read_no_deadlock
```

**Implementation:**
```rust
#[test]
fn test_concurrent_registry_lookups() {
    let registry = Arc::new(create_test_registry());
    let barrier = Arc::new(Barrier::new(100));
    
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let reg = registry.clone();
            let barrier_clone = barrier.clone();
            thread::spawn(move || {
                barrier_clone.wait();  // Synchronize start
                for _ in 0..1000 {
                    let _ = reg.find_verb(&format!("verb_{}", i % 10));
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

#### B. Concurrent Context Modifications
```
tests/stress/concurrent_context.rs:
  ✓ test_100_threads_concurrent_typemap_insert
  ✓ test_100_threads_concurrent_typemap_read
  ✓ test_mixed_read_write_typemap
  ✓ test_app_context_concurrent_handlers
```

#### C. Concurrent CLI Execution
```
tests/stress/concurrent_cli.rs:
  ✓ test_10_concurrent_cli_invocations
  ✓ test_100_concurrent_simple_commands
  ✓ test_concurrent_different_verbs
  ✓ test_concurrent_same_verb_different_args
```

### 9.2 Resource Limit Tests

#### A. Memory Pressure
```
tests/stress/memory_limits.rs:
  ✓ test_registry_with_10k_verbs
  ✓ test_command_tree_deep_nesting_100_levels
  ✓ test_large_argument_1mb
  ✓ test_large_json_output_10mb
```

**Implementation:**
```rust
#[test]
fn test_registry_with_10k_verbs() {
    let mut registry = CommandRegistry::new();
    
    // Create 10,000 verbs
    for i in 0..10_000 {
        let verb_name = format!("verb_{}", i);
        registry.register_verb(&verb_name, |_| Ok(()));
    }
    
    // Verify lookup performance degrades gracefully
    let start = Instant::now();
    for i in 0..1000 {
        let _ = registry.find_verb(&format!("verb_{}", i));
    }
    let duration = start.elapsed();
    
    // Lookup should still be <100ms for 1000 lookups
    assert!(duration.as_millis() < 100);
}
```

#### B. Argument Count Limits
```
tests/stress/argument_limits.rs:
  ✓ test_parse_1000_arguments
  ✓ test_parse_10k_arguments
  ✓ test_combined_size_100mb_args
```

#### C. Command Tree Scaling
```
tests/stress/command_tree_scaling.rs:
  ✓ test_build_tree_with_100_nouns
  ✓ test_build_tree_with_1000_verbs
  ✓ test_deep_nesting_50_levels
```

### 9.3 Performance Under Load

#### A. Dispatch Latency Under Contention
```
tests/stress/dispatch_latency.rs:
  ✓ test_dispatch_latency_p99_baseline
  ✓ test_dispatch_latency_p99_under_load
  ✓ test_dispatch_throughput_per_second
```

**Acceptance Criteria:**
- P99 latency <5ms under 100 concurrent threads
- Throughput remains >100 dispatches/sec
- No deadlocks or timeout panics

#### B. JSON Serialization at Scale
```
tests/stress/json_serialization.rs:
  ✓ test_serialize_large_json_10mb
  ✓ test_deserialize_large_json_10mb
  ✓ test_concurrent_serialization
```

### 9.4 Automation

**Command:**
```bash
cargo test stress_
```

**Duration:** ~30 seconds

**CI Integration:**
```bash
# Stress tests run in separate CI job (slow, informational)
cargo test stress_ --release --test-threads=1
```

**Acceptance Criteria:**
- All stress tests pass
- No panics under load
- Memory usage doesn't exceed 500MB
- No file descriptor leaks
- Deterministic results

---

## 10. Documentation Testing

### 10.1 Doctest Requirements

#### A. Module-Level Doctests
```rust
//! # CLI Framework
//!
//! ```rust
//! use clap_noun_verb::{CommandRegistry, VerbCommand};
//!
//! fn example() -> Result<()> {
//!     let registry = CommandRegistry::new();
//!     Ok(())
//! }
//! ```
```

**All public items must have:**
- Module docs with example
- At least one doctest per public function
- Tested example that compiles and runs

#### B. Example Programs

```
examples/tutorial/basic.rs         - "Hello world" CLI
examples/tutorial/arguments.rs     - Argument handling
examples/tutorial/positional.rs    - Positional args
examples/tutorial/services.rs      - Multi-noun example
examples/howto/arg_groups.rs       - Argument groups
examples/howto/validation.rs       - Input validation
examples/howto/env_vars.rs         - Environment variables
examples/howto/arg_actions.rs      - Argument actions
examples/howto/deprecation.rs      - Deprecation warnings
examples/reference/attribute_macro.rs      - Macro usage
examples/reference/framework.rs            - Framework API
examples/reference/nested.rs               - Nested commands
examples/reference/collector.rs            - Verb collector
examples/reference/format.rs               - Output formats
examples/reference/context.rs              - Context usage
examples/reference/root_verb.rs            - Root-level verbs
```

### 10.2 Doctest Validation

#### A. Doctest Compilation
```bash
cargo test --doc --all-features
```

**Acceptance Criteria:**
- All doctests compile
- All doctests run without error
- No `ignore` or `no_run` without justification
- Examples are copy-paste ready

#### B. Example Program Execution
```bash
cargo make build-examples
```

**Test:**
```
tests/integration/example_execution.rs:
  ✓ test_example_tutorial_basic_runs
  ✓ test_example_tutorial_arguments_runs
  ✓ test_example_tutorial_services_runs
  ✓ test_example_howto_validation_runs
  ✓ test_example_reference_framework_runs
```

### 10.3 Code Comment Quality

#### A. Comment Density
- Public items: 100% must have doc comments
- Private items: Complex logic (>20 lines) should have explanatory comments
- Invariants: Document pre/post conditions for complex functions

#### B. Example Accuracy
```rust
/// Finds a verb by name in the registry.
///
/// # Arguments
/// * `name` - The verb name to search for
///
/// # Returns
/// `Some(verb)` if found, `None` otherwise
///
/// # Example
/// ```
/// use clap_noun_verb::CommandRegistry;
/// let registry = CommandRegistry::new();
/// assert_eq!(registry.find_verb("nonexistent"), None);
/// ```
pub fn find_verb(&self, name: &str) -> Option<&VerbCommand> {
    // Implementation...
}
```

### 10.4 Docstring Formatting

**Required Format (rustdoc):**
```rust
/// Short one-line summary.
///
/// Longer description if needed.
///
/// # Arguments
/// * `arg1` - Description
/// * `arg2` - Description
///
/// # Returns
/// Description of return value
///
/// # Errors
/// * `Error::InvalidArg` - When arg is invalid
/// * `Error::Io` - When I/O fails
///
/// # Panics
/// Never panics (or specify when)
///
/// # Example
/// ```
/// // Code example
/// ```
```

### 10.5 Tutorial Accuracy

**Tutorial Checklist:**
```
docs/tutorial/:
  ✓ Gets started with basic example
  ✓ Explains noun-verb concepts
  ✓ Shows argument handling
  ✓ Demonstrates error handling
  ✓ Covers nesting patterns
  ✓ Explains JSON output
  ✓ All code examples run successfully
```

### 10.6 API Reference Completeness

**Reference Checklist:**
```
docs/reference/:
  ✓ All public items documented
  ✓ All macros explained
  ✓ All traits with examples
  ✓ Performance characteristics noted
  ✓ Feature gates documented
```

### 10.7 Automation

**Command:**
```bash
cargo make doc
```

**CI Gate:**
```bash
cargo test --doc --all-features
cargo make build-examples
```

**Acceptance Criteria:**
- All doctests pass
- All examples build and run
- No `unknown_docs` warnings
- No orphaned documentation
- README examples work

---

## Testing Automation Summary

### Quick Check
```bash
# Local pre-commit (fast, <10s)
cargo make format-check
cargo make clippy
cargo test --lib
```

### Full CI Pipeline
```bash
# Full verification (<2m)
cargo make ci
```

**CI Tasks:**
1. Format check
2. Clippy linting
3. Feature combination tests
4. Unfailable test architecture
5. Example build verification
6. Full feature compilation check

### Release Checklist
```bash
# Pre-release verification (<5m)
cargo make release-check
```

**Release Tasks:**
1. Format check
2. Clippy
3. All feature tests
4. Release build
5. Example build
6. Documentation build with warnings-as-errors

---

## Success Metrics

### Coverage Metrics
- **Minimum overall coverage:** 80%
- **Critical modules (verb, noun, registry):** 90%
- **Error paths:** 85%

### Test Performance
- **Unit tests:** <500ms
- **Integration tests:** <2s single-threaded
- **All tests (parallel):** <5s with `test-all`
- **Stress tests:** <30s (optional)

### Reliability
- **Flakiness:** 0% (deterministic execution)
- **Platform coverage:** 3+ (Linux, macOS, Windows)
- **Feature combinations:** All valid combinations test
- **Regression tests:** 100% pass before release

### Documentation
- **Doctest coverage:** 100% of public API
- **Example programs:** All build and run
- **Tutorial accuracy:** 100% (reviewed per release)
- **Comments on complex code:** 100%

---

## Appendix A: Test Command Reference

| Command | Duration | Purpose |
|---------|----------|---------|
| `cargo make test` | <3s | Quick test (default features) |
| `cargo make test-lib-deterministic` | <500ms | Unit tests (single-threaded) |
| `cargo make test-integration-isolated` | <2s | Integration tests (single-threaded) |
| `cargo make test-all` | <10s | All tests, all features |
| `cargo make test-feature-combinations` | <10s | Feature matrix tests |
| `cargo make test-frontier-all` | <5s | Frontier features test |
| `cargo make benchmark` | <30s | Performance benchmarks |
| `cargo make lint` | <30s | Format + Clippy |
| `cargo make ci` | <2m | Full CI pipeline |
| `cargo make release-check` | <5m | Release pre-flight |

---

## Appendix B: Continuous Integration Configuration

**GitHub Actions Workflow (.github/workflows/test.yml):**
```yaml
name: Test Matrix

on: [push, pull_request]

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2
      - run: cargo make ci

  benchmarks:
    name: Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo bench --all-features

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo tarpaulin --all-features --timeout 300
```

---

## Appendix C: Local Test Workflow

### For Contributors

```bash
# 1. Before commit (pre-commit hook)
./scripts/pre-commit.sh

# 2. Before push
cargo make ci

# 3. Optional: stress test
cargo test stress_ --release

# 4. Optional: property tests
cargo test property_ -- --nocapture
```

### For Maintainers

```bash
# 1. PR acceptance
cargo make ci

# 2. Before release
cargo make release-check

# 3. Publish (macros first)
cargo make publish-all
```

---

**Document Version:** 1.0  
**Last Reviewed:** 2026-08-20  
**Maintained By:** Core Team  
**Review Frequency:** Quarterly or after major refactoring
