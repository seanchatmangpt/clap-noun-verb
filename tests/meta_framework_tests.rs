//! Chicago TDD Tests for Meta-Framework
//!
//! These tests follow Chicago TDD principles:
//! - State-based testing (verify observable outputs, not implementation)
//! - Real collaborators (use actual types, minimal mocks)
//! - Behavior verification (test what code does, not that it exists)
//! - AAA pattern (Arrange-Act-Assert)
//!
//! Test Categories:
//! - Self-introspection correctness
//! - Optimization safety
//! - Performance benchmarks

#![cfg(test)]

// Re-export supporting types from meta_framework module
use clap_noun_verb_macros::meta_aware;
pub use clap_noun_verb_macros::meta_framework::{
    Capability, CapabilityError, CapabilityProof, CapabilityType, ModificationError,
    OptimizationHint,
};

// ============================================================================
// Test Fixtures - Arrange
// ============================================================================

/// Test struct with meta-awareness for capability discovery
#[meta_aware]
struct TestAgentCapabilities {
    name: String,
    max_concurrency: usize,
    supports_async: bool,
}

/// Test struct with numeric fields for optimization testing
#[meta_aware]
struct TestPerformanceConfig {
    thread_pool_size: usize,
    batch_size: usize,
    timeout_ms: u64,
}

// ============================================================================
// Self-Introspection Tests - State-Based Verification
// ============================================================================

#[test]
fn test_introspect_capabilities_returns_valid_rdf() {
    // Arrange: Create instance with known state
    let capabilities = TestAgentCapabilities {
        name: "test-worker".to_string(),
        max_concurrency: 8,
        supports_async: true,
    };

    // Act: Generate RDF introspection
    let rdf = capabilities.introspect_capabilities();

    // Assert: Verify RDF contains expected triples (state-based)
    assert!(rdf.contains(":instance a :TestAgentCapabilities"), "RDF should declare instance type");
    assert!(rdf.contains("cnv:hasField"), "RDF should describe fields");
    assert!(rdf.contains("cnv:fieldCount \"3\""), "RDF should report correct field count");
    assert!(rdf.contains("\"name\""), "RDF should include field names");
}

#[test]
fn test_introspect_schema_is_independent_of_instance() {
    // Arrange: No instance needed (static method)

    // Act: Generate schema RDF
    let schema1 = TestAgentCapabilities::introspect_schema();
    let schema2 = TestAgentCapabilities::introspect_schema();

    // Assert: Schema is deterministic and instance-independent
    assert_eq!(schema1, schema2, "Schema should be consistent across calls");
    assert!(
        schema1.contains(":TestAgentCapabilities a rdfs:Class"),
        "Schema should declare class type"
    );
    assert!(schema1.contains("rdfs:label"), "Schema should include label");
}

#[test]
fn test_generate_similarity_query_returns_valid_sparql() {
    // Arrange: No instance needed (static method)

    // Act: Generate SPARQL query
    let query = TestAgentCapabilities::generate_similarity_query();

    // Assert: Verify query structure (state-based)
    assert!(query.contains("SELECT ?instance WHERE"), "Query should be valid SPARQL SELECT");
    assert!(query.contains("?instance a :TestAgentCapabilities"), "Query should filter by type");
    assert!(query.contains("cnv:fieldCount \"3\""), "Query should match field count");
}

// ============================================================================
// Optimization Query Tests - Safety Verification
// ============================================================================

#[test]
fn test_query_optimizations_returns_safe_suggestions() {
    // Arrange: Create config with suboptimal settings
    let config = TestPerformanceConfig {
        thread_pool_size: 2, // Low value should trigger optimization
        batch_size: 100,
        timeout_ms: 5000,
    };

    // Act: Query for optimizations
    let optimizations = config.query_optimizations();

    // Assert: Verify optimization structure and safety
    for opt in &optimizations {
        // All suggestions must have non-empty fields
        assert!(!opt.field.is_empty(), "Optimization must specify field");
        assert!(!opt.current_value.is_empty(), "Must have current value");
        assert!(!opt.suggested_value.is_empty(), "Must have suggested value");
        assert!(!opt.rationale.is_empty(), "Must explain rationale");

        // Confidence must be in valid range [0.0, 1.0]
        assert!(
            (0.0..=1.0).contains(&opt.confidence),
            "Confidence must be between 0.0 and 1.0, got: {}",
            opt.confidence
        );
    }
}

#[test]
fn test_analyze_field_optimization_suggests_increase_for_low_concurrency() {
    // Arrange: Create instance with low concurrency
    let capabilities = TestAgentCapabilities {
        name: "low-concurrency".to_string(),
        max_concurrency: 2, // Below threshold of 4
        supports_async: false,
    };

    // Act: Query optimizations
    let optimizations = capabilities.query_optimizations();

    // Assert: Should suggest increasing concurrency (behavior verification)
    let has_concurrency_opt = optimizations.iter().any(|opt| {
        opt.field.contains("max_concurrency")
            && opt.suggested_value.parse::<usize>().unwrap_or(0) > 2
    });

    assert!(has_concurrency_opt, "Should suggest increasing low concurrency");
}

#[test]
fn test_generate_optimization_query_returns_valid_sparql() {
    // Arrange: Create instance
    let config = TestPerformanceConfig { thread_pool_size: 4, batch_size: 50, timeout_ms: 3000 };

    // Act: Generate optimization query
    let query = config.generate_optimization_query();

    // Assert: Verify SPARQL syntax
    assert!(query.contains("SELECT ?instance ?config WHERE"), "Query should be valid SPARQL");
    assert!(query.contains("cnv:hasOptimization"), "Query should search for optimizations");
    assert!(query.contains("cnv:betterThan"), "Query should compare configurations");
}

// ============================================================================
// Capability Discovery Tests - Recursive Verification
// ============================================================================

#[test]
fn test_discover_capabilities_returns_complete_list() {
    // Arrange: Create instance
    let capabilities = TestAgentCapabilities {
        name: "discoverer".to_string(),
        max_concurrency: 10,
        supports_async: true,
    };

    // Act: Discover capabilities
    let discovered = capabilities.discover_capabilities();

    // Assert: Verify all capabilities are discovered (state-based)
    assert!(!discovered.is_empty(), "Should discover at least one capability");

    // Should have struct-level capability
    let has_struct_cap = discovered.iter().any(|cap| {
        cap.name == "TestAgentCapabilities"
            && matches!(
                cap.capability_type,
                clap_noun_verb_macros::meta_framework::CapabilityType::Struct
            )
    });
    assert!(has_struct_cap, "Should discover struct-level capability");

    // Should have field-level capabilities
    let field_cap_count = discovered
        .iter()
        .filter(|cap| {
            matches!(
                cap.capability_type,
                clap_noun_verb_macros::meta_framework::CapabilityType::Field
            )
        })
        .count();
    assert_eq!(field_cap_count, 3, "Should discover all 3 field capabilities");
}

#[test]
fn test_verify_capability_succeeds_for_valid_field() {
    // Arrange: Known valid capability
    let valid_capability = "name";

    // Act: Verify capability
    let result = TestAgentCapabilities::verify_capability(valid_capability);

    // Assert: Verification succeeds with proof (state-based)
    assert!(result.is_ok(), "Valid capability should verify successfully");

    let proof = result.unwrap();
    assert_eq!(proof.capability, "name");
    assert!(proof.verified, "Proof should mark as verified");
}

#[test]
fn test_verify_capability_fails_for_invalid_field() {
    // Arrange: Known invalid capability
    let invalid_capability = "nonexistent_field";

    // Act: Verify capability
    let result = TestAgentCapabilities::verify_capability(invalid_capability);

    // Assert: Verification fails with error (state-based)
    assert!(result.is_err(), "Invalid capability should fail verification");

    let err = result.unwrap_err();
    match err {
        clap_noun_verb_macros::meta_framework::CapabilityError::InvalidCapability {
            claimed,
            available,
        } => {
            assert_eq!(claimed, "nonexistent_field");
            assert!(!available.is_empty(), "Should list available capabilities");
        }
    }
}

#[test]
fn test_generate_capability_proofs_returns_all_valid_proofs() {
    // Arrange: Create instance
    let capabilities = TestAgentCapabilities {
        name: "prover".to_string(),
        max_concurrency: 5,
        supports_async: false,
    };

    // Act: Generate proofs
    let proofs = capabilities.generate_capability_proofs();

    // Assert: All valid capabilities have proofs (state-based)
    assert_eq!(proofs.len(), 4, "Should generate proof for struct + 3 fields");

    for proof in &proofs {
        assert!(proof.verified, "All proofs should be verified");
        assert!(!proof.capability.is_empty(), "Proof must name capability");
    }
}

// ============================================================================
// Type-Safe Wrapper Tests - Invariant Verification
// ============================================================================

#[test]
fn test_wrapper_new_creates_validated_instance() {
    // Arrange: Create inner value
    let capabilities = TestAgentCapabilities {
        name: "wrapped".to_string(),
        max_concurrency: 10,
        supports_async: true,
    };

    // Act: Wrap in type-safe wrapper
    let wrapper = TestAgentCapabilitiesWrapper::new(capabilities);

    // Assert: Wrapper is validated (state-based)
    assert_eq!(wrapper.inner().name, "wrapped");
    assert_eq!(wrapper.inner().max_concurrency, 10);
}

#[test]
fn test_wrapper_inner_provides_immutable_access() {
    // Arrange: Create wrapper
    let capabilities = TestAgentCapabilities {
        name: "readonly".to_string(),
        max_concurrency: 8,
        supports_async: false,
    };
    let wrapper = TestAgentCapabilitiesWrapper::new(capabilities);

    // Act: Access inner value
    let inner_ref = wrapper.inner();

    // Assert: Can read but not modify (compile-time guarantee)
    assert_eq!(inner_ref.name, "readonly");
    // Uncommenting this would fail to compile:
    // inner_ref.name = "modified".to_string();
}

#[test]
fn test_wrapper_modify_validates_invariants() {
    // Arrange: Create wrapper
    let capabilities = TestAgentCapabilities {
        name: "modifiable".to_string(),
        max_concurrency: 5,
        supports_async: false,
    };
    let wrapper = TestAgentCapabilitiesWrapper::new(capabilities);

    // Act: Safely modify
    let result = wrapper.modify(|inner| {
        inner.name = "modified".to_string();
        inner.max_concurrency = 12;
    });

    // Assert: Modification succeeds with validation
    assert!(result.is_ok(), "Valid modification should succeed");
    let modified_wrapper = result.unwrap();
    assert_eq!(modified_wrapper.inner().name, "modified");
    assert_eq!(modified_wrapper.inner().max_concurrency, 12);
}

#[test]
fn test_wrapper_into_inner_consumes_and_returns_value() {
    // Arrange: Create wrapper
    let capabilities = TestAgentCapabilities {
        name: "extractable".to_string(),
        max_concurrency: 6,
        supports_async: true,
    };
    let wrapper = TestAgentCapabilitiesWrapper::new(capabilities);

    // Act: Extract inner value
    let extracted = wrapper.into_inner();

    // Assert: Inner value is returned with correct state
    assert_eq!(extracted.name, "extractable");
    assert_eq!(extracted.max_concurrency, 6);
    assert!(extracted.supports_async);
}

// ============================================================================
// Oxigraph Integration Tests - Real Collaborator Testing
// ============================================================================

#[test]
fn test_store_in_graph_inserts_triples() {
    // Arrange: Create oxigraph store and instance
    use oxigraph::store::Store;

    let mut store = Store::new().unwrap();
    let capabilities = TestAgentCapabilities {
        name: "storable".to_string(),
        max_concurrency: 7,
        supports_async: true,
    };

    // Act: Store in graph
    let result = capabilities.store_in_graph(&mut store);

    // Assert: Storage succeeds (state-based)
    assert!(result.is_ok(), "Should store successfully: {:?}", result);

    // Verify triple count increased
    let count = store.len().unwrap();
    assert!(count > 0, "Store should contain triples");
}

#[test]
fn test_query_graph_executes_sparql() {
    // Arrange: Create store with data
    use oxigraph::store::Store;

    let mut store = Store::new().unwrap();
    let capabilities = TestAgentCapabilities {
        name: "queryable".to_string(),
        max_concurrency: 9,
        supports_async: false,
    };
    capabilities.store_in_graph(&mut store).unwrap();

    // Act: Execute SPARQL query
    let query = "SELECT * WHERE { ?s ?p ?o }";
    let results = TestAgentCapabilities::query_graph(&store, query);

    // Assert: Query succeeds and returns results
    assert!(results.is_ok(), "Query should succeed");
    let result_list = results.unwrap();
    assert!(!result_list.is_empty(), "Should return results");
}

// ============================================================================
// Performance Benchmark Tests - SLO Verification
// ============================================================================

#[test]
fn test_introspection_performance_meets_slo() {
    // Arrange: Create instance
    let capabilities = TestAgentCapabilities {
        name: "performance-test".to_string(),
        max_concurrency: 16,
        supports_async: true,
    };

    // Act: Measure introspection time
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = capabilities.introspect_capabilities();
    }
    let duration = start.elapsed();

    // Assert: Performance meets SLO (< 1ms per operation on average)
    let avg_duration = duration.as_micros() / 1000;
    assert!(
        avg_duration < 1000,
        "Introspection should take < 1ms on average, got: {}μs",
        avg_duration
    );
}

#[test]
fn test_optimization_query_performance_meets_slo() {
    // Arrange: Create instance
    let config = TestPerformanceConfig { thread_pool_size: 8, batch_size: 200, timeout_ms: 10000 };

    // Act: Measure optimization query time
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = config.query_optimizations();
    }
    let duration = start.elapsed();

    // Assert: Performance meets SLO (< 500μs per operation)
    let avg_duration = duration.as_micros() / 1000;
    assert!(
        avg_duration < 500,
        "Optimization query should take < 500μs on average, got: {}μs",
        avg_duration
    );
}

#[test]
fn test_capability_discovery_performance_meets_slo() {
    // Arrange: Create instance
    let capabilities = TestAgentCapabilities {
        name: "discovery-perf".to_string(),
        max_concurrency: 12,
        supports_async: true,
    };

    // Act: Measure discovery time
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = capabilities.discover_capabilities();
    }
    let duration = start.elapsed();

    // Assert: Performance meets SLO (< 100μs per operation)
    let avg_duration = duration.as_nanos() / 10000;
    assert!(
        avg_duration < 100_000,
        "Discovery should take < 100μs on average, got: {}ns",
        avg_duration
    );
}
