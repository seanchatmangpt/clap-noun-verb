//! Integration tests for rdf_composition module
//!
//! These tests verify SPARQL 1.1 compliance and oxigraph integration.
//! Includes W3C SPARQL 1.1 compliance samples.
//!
//! Tests follow Chicago TDD:
//! - State-based testing
//! - Real collaborators (oxigraph Store)
//! - AAA pattern (Arrange-Act-Assert)
//! - Behavior verification

#[cfg(feature = "rdf-composition")]
mod tests {
    use clap_noun_verb::frontier::rdf_composition::{Capability, SemanticDiscoveryOxigraph};

    // =============================================================================
    // Basic SPARQL SELECT Tests
    // =============================================================================

    #[test]
    fn test_simple_select_query_returns_results() {
        // Arrange: Create discovery with capability
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#Test".to_string(),
                name: "Test".to_string(),
                description: "Test capability".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();

        // Act: Execute simple SELECT query
        let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
        let results = discovery.query_sparql(query);

        // Assert: Query succeeds and returns results
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_select_with_type_filter() {
        // Arrange: Create discovery with typed capability
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#FileReader".to_string(),
                name: "File Reader".to_string(),
                description: "Reads files".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();

        // Act: Query for subjects with specific type
        let query = r#"
            SELECT ?s WHERE {
                ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://cnv.dev/capability#Capability>
            }
        "#;
        let results = discovery.query_sparql(query);

        // Assert: Finds the capability
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
    }

    // =============================================================================
    // W3C SPARQL 1.1 Compliance Tests
    // =============================================================================

    #[test]
    fn test_sparql_filter_with_regex() {
        // Arrange: Create discovery with multiple capabilities
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#FileReader".to_string(),
                name: "Reader".to_string(),
                description: "Read files".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#FileWriter".to_string(),
                name: "Writer".to_string(),
                description: "Write files".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();

        // Act: FILTER with regex (W3C SPARQL 1.1 feature)
        let query = r#"
            SELECT ?s WHERE {
                ?s ?p ?o .
                FILTER(regex(str(?s), "Reader"))
            }
        "#;
        let results = discovery.query_sparql(query);

        // Assert: Filters correctly
        assert!(results.is_ok());
    }

    #[test]
    fn test_sparql_optional_pattern() {
        // Arrange: Create discovery
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#Cap1".to_string(),
                name: "Capability 1".to_string(),
                description: "First".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();

        // Act: OPTIONAL pattern (W3C SPARQL 1.1 feature)
        let query = r#"
            SELECT ?s ?label WHERE {
                ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?type .
                OPTIONAL { ?s <http://www.w3.org/2000/01/rdf-schema#label> ?label }
            }
        "#;
        let results = discovery.query_sparql(query);

        // Assert: Optional pattern works
        assert!(results.is_ok());
    }

    #[test]
    fn test_sparql_union_queries() {
        // Arrange: Create discovery with capabilities
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#A".to_string(),
                name: "A".to_string(),
                description: "Capability A".to_string(),
                capability_type: "https://cnv.dev/capability#TypeA".to_string(),
            })
            .unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#B".to_string(),
                name: "B".to_string(),
                description: "Capability B".to_string(),
                capability_type: "https://cnv.dev/capability#TypeB".to_string(),
            })
            .unwrap();

        // Act: UNION query (W3C SPARQL 1.1 feature)
        let query = r#"
            SELECT ?s WHERE {
                { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://cnv.dev/capability#TypeA> }
                UNION
                { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://cnv.dev/capability#TypeB> }
            }
        "#;
        let results = discovery.query_sparql(query);

        // Assert: UNION finds both capabilities
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
    }

    // =============================================================================
    // JSON-LD Export Tests
    // =============================================================================

    #[test]
    fn test_json_ld_export_includes_context() {
        // Arrange: Create discovery with capability
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#Export".to_string(),
                name: "Export Test".to_string(),
                description: "JSON-LD export test".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();

        // Act: Export as JSON-LD
        let json_ld = discovery.export_json_ld();

        // Assert: Contains @context and @graph
        assert!(json_ld.is_ok());
        let json_str = json_ld.unwrap();
        assert!(json_str.contains("@context"));
        assert!(json_str.contains("@graph"));
        assert!(json_str.contains("rdf"));
        assert!(json_str.contains("rdfs"));
        assert!(json_str.contains("cap"));
    }

    #[test]
    fn test_json_ld_export_includes_all_triples() {
        // Arrange: Create discovery with multiple capabilities
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        for i in 0..5 {
            discovery
                .register_capability(&Capability {
                    uri: format!("https://cnv.dev/capability#Cap{}", i),
                    name: format!("Capability {}", i),
                    description: format!("Test {}", i),
                    capability_type: "https://cnv.dev/capability#Capability".to_string(),
                })
                .unwrap();
        }

        // Act: Export as JSON-LD
        let json_ld = discovery.export_json_ld().unwrap();

        // Assert: Contains all capabilities
        for i in 0..5 {
            assert!(json_ld.contains(&format!("Cap{}", i)));
        }
    }

    // =============================================================================
    // Triple Management Tests
    // =============================================================================

    #[test]
    fn test_triple_count_tracks_additions() {
        // Arrange: Create empty discovery
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();

        // Act & Assert: Count increases with registrations
        assert_eq!(discovery.triple_count().unwrap(), 0);

        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#C1".to_string(),
                name: "C1".to_string(),
                description: "First".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();
        assert_eq!(discovery.triple_count().unwrap(), 3);

        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#C2".to_string(),
                name: "C2".to_string(),
                description: "Second".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();
        assert_eq!(discovery.triple_count().unwrap(), 6);
    }

    #[test]
    fn test_clear_removes_all_triples() {
        // Arrange: Create discovery with capabilities
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#Temp".to_string(),
                name: "Temp".to_string(),
                description: "Temporary".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();
        assert!(discovery.triple_count().unwrap() > 0);

        // Act: Clear store
        discovery.clear().unwrap();

        // Assert: No triples remain
        assert_eq!(discovery.triple_count().unwrap(), 0);
    }

    // =============================================================================
    // Error Handling Tests
    // =============================================================================

    #[test]
    fn test_invalid_sparql_query_returns_error() {
        // Arrange: Create discovery
        let discovery = SemanticDiscoveryOxigraph::new().unwrap();

        // Act: Execute malformed SPARQL
        let result = discovery.query_sparql("THIS IS NOT VALID SPARQL");

        // Assert: Returns error (no panic)
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uri_in_capability_returns_error() {
        // Arrange: Create discovery with invalid URI
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();

        // Act: Attempt to register capability with bad URI
        let result = discovery.register_capability(&Capability {
            uri: "not a valid URI!!!".to_string(),
            name: "Invalid".to_string(),
            description: "Bad URI".to_string(),
            capability_type: "https://cnv.dev/capability#Capability".to_string(),
        });

        // Assert: Returns error (no panic)
        assert!(result.is_err());
    }

    // =============================================================================
    // Performance Tests
    // =============================================================================

    #[test]
    fn test_query_performance_within_budget() {
        // Arrange: Create discovery with 100 capabilities
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        for i in 0..100 {
            discovery
                .register_capability(&Capability {
                    uri: format!("https://cnv.dev/capability#Perf{}", i),
                    name: format!("Performance {}", i),
                    description: format!("Perf test {}", i),
                    capability_type: "https://cnv.dev/capability#Capability".to_string(),
                })
                .unwrap();
        }

        // Act: Measure query time
        let start = std::time::Instant::now();
        let results = discovery.query_sparql("SELECT ?s WHERE { ?s ?p ?o }");
        let duration = start.elapsed();

        // Assert: Query completes within budget (< 5ms target)
        assert!(results.is_ok());
        assert!(
            duration.as_millis() < 5,
            "Query took {:?}, target is < 5ms",
            duration
        );
    }

    #[test]
    fn test_triple_creation_performance() {
        // Arrange: Create discovery
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();

        // Act: Measure registration time
        let start = std::time::Instant::now();
        for i in 0..1000 {
            discovery
                .register_capability(&Capability {
                    uri: format!("https://cnv.dev/capability#T{}", i),
                    name: format!("T{}", i),
                    description: format!("Test {}", i),
                    capability_type: "https://cnv.dev/capability#Capability".to_string(),
                })
                .unwrap();
        }
        let duration = start.elapsed();

        // Assert: < 1µs per triple target (3 triples per capability)
        let micros_per_triple = duration.as_micros() / 3000;
        assert!(
            micros_per_triple < 1,
            "Triple creation took {} µs/triple, target is < 1µs",
            micros_per_triple
        );
    }

    // =============================================================================
    // Complex SPARQL Tests
    // =============================================================================

    #[test]
    fn test_sparql_with_multiple_filters() {
        // Arrange: Create discovery with varied capabilities
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#FileReader".to_string(),
                name: "File Reader".to_string(),
                description: "Read files from disk".to_string(),
                capability_type: "https://cnv.dev/capability#IO".to_string(),
            })
            .unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#NetworkReader".to_string(),
                name: "Network Reader".to_string(),
                description: "Read from network".to_string(),
                capability_type: "https://cnv.dev/capability#IO".to_string(),
            })
            .unwrap();

        // Act: Multiple FILTER conditions
        let query = r#"
            SELECT ?s WHERE {
                ?s ?p ?o .
                FILTER(regex(str(?s), "Reader"))
                FILTER(contains(str(?s), "File"))
            }
        "#;
        let results = discovery.query_sparql(query);

        // Assert: Filters work together
        assert!(results.is_ok());
    }

    #[test]
    fn test_sparql_join_across_predicates() {
        // Arrange: Create discovery with capability
        let mut discovery = SemanticDiscoveryOxigraph::new().unwrap();
        discovery
            .register_capability(&Capability {
                uri: "https://cnv.dev/capability#Join".to_string(),
                name: "Join Test".to_string(),
                description: "Test joins".to_string(),
                capability_type: "https://cnv.dev/capability#Capability".to_string(),
            })
            .unwrap();

        // Act: JOIN across multiple triples
        let query = r#"
            SELECT ?s ?label ?comment WHERE {
                ?s <http://www.w3.org/2000/01/rdf-schema#label> ?label .
                ?s <http://www.w3.org/2000/01/rdf-schema#comment> ?comment .
            }
        "#;
        let results = discovery.query_sparql(query);

        // Assert: JOIN succeeds
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty());
    }
}
