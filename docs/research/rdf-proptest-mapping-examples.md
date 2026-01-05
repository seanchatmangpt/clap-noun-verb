# RDF-to-Proptest Mapping Examples
## Concrete Implementation Patterns for Reflexive Testing

**Date**: 2026-01-05
**Purpose**: Practical code examples for mapping RDF ontology to proptest strategies

---

## 1. Basic RDF-to-Strategy Mapping

### 1.1 Single Capability

**RDF Ontology**:
```turtle
@prefix cli: <http://example.org/cli#> .
@prefix cnv: <http://example.org/cnv#> .

cli:cmd-read a cnv:Command ;
    cnv:hasCapability cli:cap-read ;
    cnv:inputType "String" ;
    cnv:outputType "String" ;
    cnv:expectedDuration "50ms" .
```

**Generated Proptest**:
```rust
use proptest::prelude::*;

// Strategy for input type
fn input_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_/.-]{1,100}"
}

// Property test
proptest! {
    #[test]
    fn cmd_read_succeeds_with_valid_input(input in input_strategy()) {
        let result = execute_command("read", &input);
        prop_assert!(result.is_ok());

        // Verify output type
        let output: String = result.unwrap();
        prop_assert!(!output.is_empty());
    }

    #[test]
    fn cmd_read_performance_slo(input in input_strategy()) {
        let start = std::time::Instant::now();
        let _ = execute_command("read", &input);
        let duration = start.elapsed();

        // From RDF: cnv:expectedDuration "50ms"
        prop_assert!(
            duration.as_millis() <= 50,
            "Performance SLO violated: {}ms > 50ms",
            duration.as_millis()
        );
    }
}
```

---

### 1.2 Multiple Capabilities (Combination)

**RDF Ontology**:
```turtle
cli:cmd-transform a cnv:Command ;
    cnv:hasCapability cli:cap-parse ;
    cnv:hasCapability cli:cap-validate ;
    cnv:hasCapability cli:cap-transform ;
    cnv:inputType "JsonValue" ;
    cnv:outputType "JsonValue" .
```

**Generated Proptest**:
```rust
use proptest::prelude::*;
use serde_json::Value as JsonValue;

// Strategy for JSON values
fn json_value_strategy() -> impl Strategy<Value = JsonValue> {
    prop_oneof![
        Just(JsonValue::Null),
        any::<bool>().prop_map(JsonValue::Bool),
        any::<i64>().prop_map(|n| JsonValue::Number(n.into())),
        ".*".prop_map(JsonValue::String),
        prop::collection::vec(any::<i64>().prop_map(|n| JsonValue::Number(n.into())), 0..10)
            .prop_map(JsonValue::Array),
    ]
}

proptest! {
    #[test]
    fn cmd_transform_pipeline_succeeds(input in json_value_strategy()) {
        // Test full pipeline: parse → validate → transform
        let parsed = parse_capability(&input);
        prop_assert!(parsed.is_ok());

        let validated = validate_capability(parsed.unwrap());
        prop_assert!(validated.is_ok());

        let transformed = transform_capability(validated.unwrap());
        prop_assert!(transformed.is_ok());
    }

    #[test]
    fn cmd_transform_idempotent(input in json_value_strategy()) {
        // Property: transform(transform(x)) == transform(x)
        let once = execute_command("transform", &input).ok();
        if let Some(result1) = once {
            let twice = execute_command("transform", &result1).ok();
            if let Some(result2) = twice {
                prop_assert_eq!(result1, result2, "Transform must be idempotent");
            }
        }
    }
}
```

---

### 1.3 Capability Constraints (Exclusions)

**RDF Ontology**:
```turtle
cli:cap-read a cnv:Capability ;
    cnv:excludes cli:cap-write ;
    cnv:effectLevel "ReadOnly" .

cli:cap-write a cnv:Capability ;
    cnv:excludes cli:cap-read ;
    cnv:effectLevel "Mutate" .

cli:cap-execute a cnv:Capability ;
    cnv:effectLevel "Execute" .
```

**Generated Proptest**:
```rust
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq)]
enum Capability {
    Read,
    Write,
    Execute,
}

// Strategy that respects exclusion rules
fn capability_combination_strategy() -> impl Strategy<Value = Vec<Capability>> {
    prop::collection::vec(
        prop_oneof![
            Just(Capability::Read),
            Just(Capability::Write),
            Just(Capability::Execute),
        ],
        1..=3
    ).prop_filter("Exclusion rules", |caps| {
        // From RDF: Read excludes Write
        !(caps.contains(&Capability::Read) && caps.contains(&Capability::Write))
    })
}

proptest! {
    #[test]
    fn capability_combinations_respect_exclusions(
        caps in capability_combination_strategy()
    ) {
        // Property: No excluded combinations
        prop_assert!(
            !(caps.contains(&Capability::Read) && caps.contains(&Capability::Write)),
            "Read and Write are mutually exclusive"
        );

        // Property: Execute can combine with either
        if caps.contains(&Capability::Execute) {
            prop_assert!(
                caps.contains(&Capability::Read) || caps.contains(&Capability::Write) ||
                caps.len() == 1
            );
        }
    }
}
```

---

## 2. Advanced Patterns

### 2.1 State Machine Testing

**RDF Ontology**:
```turtle
cli:cert-state-machine a cnv:StateMachine ;
    cnv:initialState cli:state-unchecked ;
    cnv:transitions [
        cnv:from cli:state-unchecked ;
        cnv:to cli:state-policy-checked ;
        cnv:trigger "policy_check"
    ] , [
        cnv:from cli:state-policy-checked ;
        cnv:to cli:state-verified ;
        cnv:trigger "capability_check"
    ] .

# Monotonicity property: Never goes backward
cli:cert-state-machine cnv:property "monotonic" .
```

**Generated Proptest**:
```rust
use proptest::prelude::*;
use proptest::state_machine::{ReferenceStateMachine, StateMachineTest};

#[derive(Debug, Clone, PartialEq)]
enum CertState {
    Unchecked,
    PolicyChecked,
    Verified,
}

#[derive(Debug, Clone)]
enum CertTransition {
    PolicyCheck,
    CapabilityCheck,
}

// State machine model from RDF
struct CertStateMachine {
    state: CertState,
}

impl ReferenceStateMachine for CertStateMachine {
    type State = CertState;
    type Transition = CertTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        Just(CertState::Unchecked).boxed()
    }

    fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> {
        match state {
            CertState::Unchecked => Just(CertTransition::PolicyCheck).boxed(),
            CertState::PolicyChecked => Just(CertTransition::CapabilityCheck).boxed(),
            CertState::Verified => prop::strategy::LazyJust::new(|| {
                panic!("No transitions from Verified (terminal state)")
            }).boxed(),
        }
    }

    fn apply(state: Self::State, transition: &Self::Transition) -> Self::State {
        match (state, transition) {
            (CertState::Unchecked, CertTransition::PolicyCheck) => CertState::PolicyChecked,
            (CertState::PolicyChecked, CertTransition::CapabilityCheck) => CertState::Verified,
            _ => panic!("Invalid transition"),
        }
    }

    fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
        match (state, transition) {
            (CertState::Unchecked, CertTransition::PolicyCheck) => true,
            (CertState::PolicyChecked, CertTransition::CapabilityCheck) => true,
            _ => false,
        }
    }
}

proptest! {
    #[test]
    fn certificate_state_machine_is_monotonic(
        transitions in prop::collection::vec(any::<CertTransition>(), 0..10)
    ) {
        let mut state = CertState::Unchecked;
        let mut state_order = vec![0]; // Unchecked = 0

        for transition in transitions {
            if CertStateMachine::preconditions(&state, &transition) {
                state = CertStateMachine::apply(state, &transition);
                let order = match state {
                    CertState::Unchecked => 0,
                    CertState::PolicyChecked => 1,
                    CertState::Verified => 2,
                };
                state_order.push(order);
            }
        }

        // Property from RDF: monotonic (never decreases)
        for window in state_order.windows(2) {
            prop_assert!(
                window[1] >= window[0],
                "State machine must be monotonic: {} → {}",
                window[0], window[1]
            );
        }
    }
}
```

---

### 2.2 Temporal Constraints

**RDF Ontology**:
```turtle
cli:delegation-token a cnv:Certificate ;
    cnv:hasConstraint [
        a cnv:TemporalConstraint ;
        cnv:minDuration "PT1M" ;  # ISO 8601: 1 minute
        cnv:maxDuration "PT1H" ;  # 1 hour
        cnv:mustBeValid "true"^^xsd:boolean
    ] .
```

**Generated Proptest**:
```rust
use proptest::prelude::*;
use std::time::{Duration, SystemTime};

// Strategy for valid durations (1 min to 1 hour)
fn valid_duration_strategy() -> impl Strategy<Value = Duration> {
    (60..=3600u64).prop_map(Duration::from_secs)
}

proptest! {
    #[test]
    fn delegation_token_respects_temporal_constraints(
        duration in valid_duration_strategy()
    ) {
        let now = SystemTime::now();
        let expiry = now + duration;

        let token = DelegationToken::new(
            Principal::default(),
            Principal::default(),
            CapabilityConstraint::unrestricted(),
            TemporalConstraint::valid_until(expiry),
        );

        // Property: Token is valid within duration
        prop_assert!(token.is_valid_at(now));
        prop_assert!(token.is_valid_at(now + Duration::from_secs(30)));

        // Property: Token expires after duration
        prop_assert!(!token.is_valid_at(expiry + Duration::from_secs(1)));
    }

    #[test]
    fn delegation_token_duration_bounds(
        duration_secs in 60..=3600u64
    ) {
        let duration = Duration::from_secs(duration_secs);

        // From RDF: minDuration = 1 minute, maxDuration = 1 hour
        prop_assert!(
            duration >= Duration::from_secs(60),
            "Duration must be ≥ 1 minute"
        );
        prop_assert!(
            duration <= Duration::from_secs(3600),
            "Duration must be ≤ 1 hour"
        );
    }
}
```

---

### 2.3 Resource Constraints

**RDF Ontology**:
```turtle
cli:execution-contract a cnv:Contract ;
    cnv:resourceLimits [
        cnv:maxMemoryBytes "10485760"^^xsd:long ;  # 10MB
        cnv:maxCpuTimeMs "100"^^xsd:long ;
        cnv:maxFileDescriptors "10"^^xsd:int ;
        cnv:maxNetworkConnections "5"^^xsd:int
    ] .
```

**Generated Proptest**:
```rust
use proptest::prelude::*;

// Strategy for resource limits within bounds
fn resource_limits_strategy() -> impl Strategy<Value = ResourceLimits> {
    (
        1_000_000..=10_485_760usize,  // Memory: 1MB to 10MB
        10..=100u64,                   // CPU time: 10ms to 100ms
        1..=10usize,                   // File descriptors
        0..=5usize,                    // Network connections
    ).prop_map(|(mem, cpu, fds, net)| {
        ResourceLimits {
            max_memory_bytes: Some(mem),
            max_cpu_time: Some(Duration::from_millis(cpu)),
            max_file_descriptors: Some(fds),
            max_network_connections: Some(net),
        }
    })
}

proptest! {
    #[test]
    fn execution_contract_respects_resource_limits(
        limits in resource_limits_strategy()
    ) {
        let contract = ExecutionContract::builder()
            .duration_class(DurationClass::Interactive)
            .resource_limits(limits.clone())
            .build();

        // Property: Limits are enforced
        if let Some(max_mem) = limits.max_memory_bytes {
            prop_assert!(max_mem <= 10_485_760, "Memory limit exceeds max");
        }

        if let Some(max_cpu) = limits.max_cpu_time {
            prop_assert!(
                max_cpu <= Duration::from_millis(100),
                "CPU time exceeds max"
            );
        }

        // Property: Resource estimation is conservative
        let estimate = contract.estimate_resources();
        prop_assert!(
            estimate.memory_bytes <= limits.max_memory_bytes.unwrap_or(usize::MAX),
            "Estimated memory exceeds limit"
        );
    }
}
```

---

## 3. Shrinking Strategies

### 3.1 Custom Shrinking for Domain Types

**Problem**: Default shrinking may produce invalid values.

**Solution**: Custom `Arbitrary` implementation with domain-aware shrinking.

```rust
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityId {
    noun: String,
    verb: String,
}

impl CapabilityId {
    pub fn from_path(path: &str) -> Self {
        let parts: Vec<_> = path.split('.').collect();
        Self {
            noun: parts[0].to_string(),
            verb: parts.get(1).unwrap_or(&"").to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.noun.is_empty() && !self.verb.is_empty()
    }
}

impl<'a> Arbitrary<'a> for CapabilityId {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let noun_len = u.int_in_range(3..=10)?;
        let verb_len = u.int_in_range(3..=10)?;

        let noun: String = (0..noun_len)
            .map(|_| u.choose(&b"abcdefghijklmnopqrstuvwxyz"[..]))
            .collect::<arbitrary::Result<Vec<_>>>()?
            .into_iter()
            .map(|&b| b as char)
            .collect();

        let verb: String = (0..verb_len)
            .map(|_| u.choose(&b"abcdefghijklmnopqrstuvwxyz"[..]))
            .collect::<arbitrary::Result<Vec<_>>>()?
            .into_iter()
            .map(|&b| b as char)
            .collect();

        Ok(Self { noun, verb })
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        // Custom shrinking: reduce string lengths while maintaining validity
        let nouns = shrink_string(&self.noun, 3);
        let verbs = shrink_string(&self.verb, 3);

        Box::new(
            nouns
                .flat_map(move |n| {
                    verbs.clone().map(move |v| CapabilityId {
                        noun: n.clone(),
                        verb: v.clone(),
                    })
                })
                .filter(|cap| cap.is_valid())
        )
    }
}

fn shrink_string(s: &str, min_len: usize) -> Box<dyn Iterator<Item = String>> {
    let s = s.to_string();
    Box::new((min_len..s.len()).rev().map(move |len| s[..len].to_string()))
}
```

---

### 3.2 Regression Test Persistence

**Proptest Feature**: Automatically saves failing test cases for regression testing.

```rust
// proptest.regressions file (auto-generated)
// test_capability_parsing failures:
xs 0 1 2 3 4
  [
    ("user.create", Ok(())),
    ("admin.delete", Err("Unauthorized")),
    ("...", Err("Invalid")),  // Shrunk minimal failing case
  ]
```

**Usage**:
```rust
proptest! {
    // This test will automatically check regression cases first
    #[test]
    fn test_capability_parsing(cap_str in "[a-z]{1,20}\\.[a-z]{1,20}") {
        let result = CapabilityId::from_path(&cap_str);
        prop_assert!(result.is_valid());
    }
}
```

---

## 4. Performance Testing with Criterion

### 4.1 RDF-Derived Benchmarks

**RDF Ontology**:
```turtle
cli:cmd-parse a cnv:Command ;
    cnv:expectedDuration "100ms"^^xsd:long ;
    cnv:expectedThroughput "10000"^^xsd:long .  # ops/sec

cli:cmd-execute a cnv:Command ;
    cnv:expectedDuration "50ms"^^xsd:long ;
    cnv:expectedThroughput "20000"^^xsd:long .
```

**Generated Criterion Benchmark**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

fn bench_cmd_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_command");

    // From RDF: expectedThroughput = 10000 ops/sec
    group.throughput(Throughput::Elements(10000));

    group.bench_function("parse", |b| {
        b.iter(|| {
            black_box(execute_command("parse", "input.txt"))
        })
    });

    // SLO assertion: must complete within 100ms
    let mean_time = group.measurement_time();
    assert!(
        mean_time <= Duration::from_millis(100),
        "Performance SLO violated: {}ms > 100ms",
        mean_time.as_millis()
    );

    group.finish();
}

fn bench_cmd_execute(c: &mut Criterion) {
    let mut group = c.benchmark_group("execute_command");

    // From RDF: expectedThroughput = 20000 ops/sec
    group.throughput(Throughput::Elements(20000));

    group.bench_function("execute", |b| {
        b.iter(|| {
            black_box(execute_command("execute", "script.sh"))
        })
    });

    // SLO assertion: must complete within 50ms
    let mean_time = group.measurement_time();
    assert!(
        mean_time <= Duration::from_millis(50),
        "Performance SLO violated: {}ms > 50ms",
        mean_time.as_millis()
    );

    group.finish();
}

criterion_group!(benches, bench_cmd_parse, bench_cmd_execute);
criterion_main!(benches);
```

---

### 4.2 Baseline Comparison

```bash
# Save baseline (main branch)
git checkout main
cargo bench --all-features -- --save-baseline main

# Compare feature branch
git checkout feature/new-impl
cargo bench --all-features -- --baseline main

# Criterion output:
# parse_command/parse    time:   [98.2 ms 100.1 ms 102.3 ms]
#                        change: [+2.1% +3.4% +4.8%] (p = 0.00 < 0.05)
#                        Performance has regressed.
```

---

## 5. Coverage Integration

### 5.1 Combined Coverage Report

```rust
use std::collections::HashMap;

pub struct CombinedCoverageReport {
    // From tarpaulin
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,

    // From RDF semantic analysis
    pub capability_coverage: f64,
    pub combination_coverage: f64,

    // Gap analysis
    pub untested_capabilities: Vec<String>,
    pub untested_combinations: Vec<Vec<String>>,
    pub uncovered_critical_paths: Vec<String>,
}

impl CombinedCoverageReport {
    pub fn from_reports(
        tarpaulin: TarpaulinReport,
        rdf_coverage: RdfCoverageReport,
    ) -> Self {
        let untested_capabilities = rdf_coverage.all_capabilities
            .iter()
            .filter(|cap| !rdf_coverage.tested_capabilities.contains(*cap))
            .cloned()
            .collect();

        let untested_combinations = rdf_coverage.all_combinations
            .iter()
            .filter(|combo| !rdf_coverage.tested_combinations.contains(*combo))
            .cloned()
            .collect();

        Self {
            line_coverage: tarpaulin.line_coverage,
            branch_coverage: tarpaulin.branch_coverage,
            function_coverage: tarpaulin.function_coverage,
            capability_coverage: rdf_coverage.capability_coverage,
            combination_coverage: rdf_coverage.combination_coverage,
            untested_capabilities,
            untested_combinations,
            uncovered_critical_paths: tarpaulin.uncovered_critical_paths,
        }
    }

    pub fn overall_score(&self) -> f64 {
        // Weighted combination
        0.25 * self.line_coverage
            + 0.20 * self.branch_coverage
            + 0.15 * self.function_coverage
            + 0.20 * self.capability_coverage
            + 0.20 * self.combination_coverage
    }

    pub fn is_acceptable(&self) -> bool {
        // Thresholds from CLAUDE.md
        self.line_coverage >= 80.0
            && self.capability_coverage >= 90.0
            && self.combination_coverage >= 75.0
    }
}
```

---

## 6. Macro Code Generation

### 6.1 Generate Tests from RDF

```rust
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_reflexive_tests(ontology: &str) -> TokenStream {
    let generator = SemanticTestGenerator::new(ontology.to_string());
    let combinations = generator.extract_combinations();

    let test_functions: Vec<_> = combinations
        .iter()
        .enumerate()
        .map(|(idx, combo)| {
            let test_name = format_ident!("test_combination_{}", idx);
            let caps_str = combo.join(", ");

            // Generate strategy
            let strategy = generate_strategy_for_combo(combo);

            quote! {
                proptest! {
                    #[test]
                    fn #test_name(input in #strategy) {
                        // Test capability combination: #caps_str
                        let result = execute_capability_combination(&[#(#combo),*], input);
                        prop_assert!(result.is_ok(), "Combination failed: {}", #caps_str);
                    }
                }
            }
        })
        .collect();

    quote! {
        #[cfg(test)]
        mod reflexive_tests {
            use super::*;
            use proptest::prelude::*;

            #(#test_functions)*
        }
    }
}

fn generate_strategy_for_combo(combo: &[String]) -> TokenStream {
    // Map capability names to strategies
    let strategies: Vec<_> = combo
        .iter()
        .map(|cap| {
            match cap.as_str() {
                "parse" => quote! { "[a-zA-Z0-9_]{1,50}" },
                "validate" => quote! { any::<bool>() },
                "execute" => quote! { "[a-z]{3,10}" },
                _ => quote! { ".*" },
            }
        })
        .collect();

    if strategies.len() == 1 {
        strategies[0].clone()
    } else {
        quote! { (#(#strategies),*) }
    }
}
```

---

## 7. Complete Example: End-to-End

### 7.1 RDF Ontology

```turtle
@prefix cli: <http://example.org/cli#> .
@prefix cnv: <http://example.org/cnv#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

cli:user-service a cnv:Service ;
    cnv:hasCommand cli:cmd-create-user , cli:cmd-delete-user .

cli:cmd-create-user a cnv:Command ;
    cnv:hasCapability cli:cap-validate ;
    cnv:hasCapability cli:cap-create ;
    cnv:inputType "UserData" ;
    cnv:outputType "UserId" ;
    cnv:expectedDuration "100"^^xsd:long ;
    cnv:sideEffect "Mutate" .

cli:cmd-delete-user a cnv:Command ;
    cnv:hasCapability cli:cap-lookup ;
    cnv:hasCapability cli:cap-delete ;
    cnv:inputType "UserId" ;
    cnv:outputType "boolean" ;
    cnv:expectedDuration "50"^^xsd:long ;
    cnv:sideEffect "Mutate" .

# Exclusion: Cannot create and delete in same transaction
cli:cap-create cnv:excludes cli:cap-delete .
```

### 7.2 Generated Test Code

```rust
// Auto-generated from RDF ontology
#[cfg(test)]
mod user_service_tests {
    use super::*;
    use proptest::prelude::*;

    // ========================================================================
    // Property Tests
    // ========================================================================

    proptest! {
        #[test]
        fn cmd_create_user_succeeds(
            username in "[a-z]{3,20}",
            email in "[a-z]{3,10}@[a-z]{3,10}\\.[a-z]{2,3}"
        ) {
            let user_data = UserData { username, email };

            // Test capability pipeline: validate → create
            let validated = validate_user_data(&user_data);
            prop_assert!(validated.is_ok());

            let created = create_user(validated.unwrap());
            prop_assert!(created.is_ok());

            // Output type check (from RDF: outputType = "UserId")
            let user_id: UserId = created.unwrap();
            prop_assert!(user_id.0 > 0);
        }

        #[test]
        fn cmd_delete_user_succeeds(user_id in 1..=1000u64) {
            let user_id = UserId(user_id);

            // Test capability pipeline: lookup → delete
            let found = lookup_user(&user_id);
            if found.is_ok() {
                let deleted = delete_user(&user_id);
                prop_assert!(deleted.is_ok());

                // Output type check (from RDF: outputType = "boolean")
                let success: bool = deleted.unwrap();
                prop_assert!(success);
            }
        }

        #[test]
        fn exclusion_rule_enforced(
            create_input in any::<UserData>(),
            delete_input in any::<UserId>()
        ) {
            // From RDF: cap-create excludes cap-delete
            let result = execute_both_capabilities(&create_input, &delete_input);

            // Should fail due to exclusion
            prop_assert!(
                result.is_err(),
                "Create and Delete cannot execute in same transaction"
            );
        }
    }

    // ========================================================================
    // Performance Tests (Criterion Benchmarks)
    // ========================================================================

    use criterion::{black_box, Criterion};

    pub fn bench_create_user(c: &mut Criterion) {
        c.bench_function("cmd_create_user", |b| {
            let user_data = UserData {
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
            };

            b.iter(|| {
                black_box(execute_command("create_user", &user_data))
            })
        });

        // SLO assertion (from RDF: expectedDuration = 100ms)
        let mean = c.benchmark_group("cmd_create_user").mean_estimate();
        assert!(
            mean.as_millis() <= 100,
            "Performance SLO violated: {}ms > 100ms",
            mean.as_millis()
        );
    }

    // ========================================================================
    // Snapshot Tests (Insta)
    // ========================================================================

    #[test]
    fn test_rdf_generation_for_user_service() {
        use insta::assert_yaml_snapshot;

        let service = UserService::new();
        let rdf = service.generate_rdf_ontology();

        assert_yaml_snapshot!(rdf);
    }
}
```

---

## 8. Summary

This document provides concrete implementation patterns for mapping RDF ontology to proptest strategies, criterion benchmarks, and comprehensive test coverage. Key takeaways:

1. **RDF → Proptest**: Map capabilities to strategies, exclusions to filters
2. **RDF → Criterion**: Extract performance SLOs, generate benchmarks
3. **RDF → Coverage**: Combine structural (tarpaulin) + semantic (RDF) coverage
4. **Shrinking**: Custom domain-aware shrinking for valid test cases
5. **Code Generation**: Macros generate test code from RDF at compile time

All examples follow:
- ✅ Chicago TDD (state-based, AAA pattern)
- ✅ Type-first thinking (compile-time guarantees)
- ✅ Zero-cost abstractions (proptest strategies)
- ✅ DfLSS principles (prevent defects from start)

**Next Steps**: Implement macro code generation in `clap-noun-verb-macros` to auto-generate these tests from RDF ontology.
