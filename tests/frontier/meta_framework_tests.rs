//! Integration tests for meta_framework module
//!
//! These tests verify the complete integration of oxrdf, typetag, and erased-serde
//! for capability introspection and RDF generation.
//!
//! Tests follow Chicago TDD:
//! - State-based testing
//! - Real collaborators (no mocks)
//! - AAA pattern (Arrange-Act-Assert)
//! - Behavior verification

#[cfg(all(feature = "meta-framework", feature = "rdf-composition"))]
mod tests {
    use clap_noun_verb::frontier::meta_framework::{
        discover_capabilities_typetag, introspect_rdf_oxrdf, serialize_erased,
        FileReaderCapability, FileWriterCapability, MetaCapability, MetaFramework,
    };
    use std::sync::Arc;

    // =============================================================================
    // Capability Introspection Tests
    // =============================================================================

    #[test]
    fn test_capability_introspection_generates_valid_rdf_triples() {
        // Arrange: Create a file reader capability
        let capability = FileReaderCapability {
            uri: "https://cnv.dev/capability#FileReader".to_string(),
            name: "File Reader".to_string(),
            description: "Reads files from the filesystem".to_string(),
        };

        // Act: Introspect capability to generate RDF triples
        let result = capability.introspect_rdf_oxrdf();

        // Assert: RDF triples generated successfully
        assert!(result.is_ok(), "Introspection should succeed");
        let triples = result.unwrap();
        assert_eq!(
            triples.len(),
            3,
            "Should generate 3 triples (type, label, comment)"
        );
    }

    #[test]
    fn test_multiple_capabilities_generate_distinct_triples() {
        // Arrange: Create two different capabilities
        let reader = FileReaderCapability {
            uri: "https://cnv.dev/capability#Reader".to_string(),
            name: "Reader".to_string(),
            description: "Read files".to_string(),
        };
        let writer = FileWriterCapability {
            uri: "https://cnv.dev/capability#Writer".to_string(),
            name: "Writer".to_string(),
            description: "Write files".to_string(),
        };

        // Act: Introspect both capabilities
        let reader_triples = reader.introspect_rdf_oxrdf().unwrap();
        let writer_triples = writer.introspect_rdf_oxrdf().unwrap();

        // Assert: Both generate triples with different subjects
        assert_eq!(reader_triples.len(), 3);
        assert_eq!(writer_triples.len(), 3);
        assert!(reader_triples[0].subject.to_string().contains("Reader"));
        assert!(writer_triples[0].subject.to_string().contains("Writer"));
    }

    // =============================================================================
    // Meta-Framework Tests
    // =============================================================================

    #[test]
    fn test_meta_framework_registers_capabilities() {
        // Arrange: Create meta-framework
        let mut framework = MetaFramework::new();
        let cap1 = Arc::new(FileReaderCapability {
            uri: "https://cnv.dev/capability#Cap1".to_string(),
            name: "Capability 1".to_string(),
            description: "First capability".to_string(),
        }) as Arc<dyn MetaCapability>;
        let cap2 = Arc::new(FileWriterCapability {
            uri: "https://cnv.dev/capability#Cap2".to_string(),
            name: "Capability 2".to_string(),
            description: "Second capability".to_string(),
        }) as Arc<dyn MetaCapability>;

        // Act: Register capabilities
        framework.register(cap1);
        framework.register(cap2);

        // Assert: Both capabilities registered
        assert_eq!(framework.capabilities().len(), 2);
        assert_eq!(
            framework.capabilities()[0].uri(),
            "https://cnv.dev/capability#Cap1"
        );
        assert_eq!(
            framework.capabilities()[1].uri(),
            "https://cnv.dev/capability#Cap2"
        );
    }

    #[test]
    fn test_framework_introspects_all_capabilities() {
        // Arrange: Create framework with multiple capabilities
        let mut framework = MetaFramework::new();
        framework.register(Arc::new(FileReaderCapability {
            uri: "https://cnv.dev/capability#A".to_string(),
            name: "A".to_string(),
            description: "Capability A".to_string(),
        }));
        framework.register(Arc::new(FileWriterCapability {
            uri: "https://cnv.dev/capability#B".to_string(),
            name: "B".to_string(),
            description: "Capability B".to_string(),
        }));
        framework.register(Arc::new(FileReaderCapability {
            uri: "https://cnv.dev/capability#C".to_string(),
            name: "C".to_string(),
            description: "Capability C".to_string(),
        }));

        // Act: Introspect all capabilities
        let result = framework.introspect_all_rdf();

        // Assert: All triples generated
        assert!(result.is_ok());
        let all_triples = result.unwrap();
        assert_eq!(all_triples.len(), 9); // 3 caps * 3 triples each
    }

    // =============================================================================
    // Type Erasure Tests
    // =============================================================================

    #[test]
    fn test_serialize_erased_produces_valid_json() {
        // Arrange: Create capability
        let capability = FileReaderCapability {
            uri: "https://cnv.dev/capability#Serializable".to_string(),
            name: "Serializable".to_string(),
            description: "Test serialization".to_string(),
        };

        // Act: Serialize using erased-serde
        let result = serialize_erased(&capability);

        // Assert: JSON is valid and contains expected data
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("FileReaderCapability"));
        assert!(json.contains("Serializable"));
        assert!(json.contains("Test serialization"));
    }

    #[test]
    fn test_serialize_different_capability_types() {
        // Arrange: Create different capability types
        let reader = FileReaderCapability {
            uri: "https://cnv.dev/capability#R".to_string(),
            name: "R".to_string(),
            description: "Reader".to_string(),
        };
        let writer = FileWriterCapability {
            uri: "https://cnv.dev/capability#W".to_string(),
            name: "W".to_string(),
            description: "Writer".to_string(),
        };

        // Act: Serialize both
        let reader_json = serialize_erased(&reader).unwrap();
        let writer_json = serialize_erased(&writer).unwrap();

        // Assert: Different type tags in JSON
        assert!(reader_json.contains("FileReaderCapability"));
        assert!(writer_json.contains("FileWriterCapability"));
    }

    // =============================================================================
    // Discovery Tests
    // =============================================================================

    #[test]
    fn test_discover_capabilities_returns_known_types() {
        // Arrange: (No setup needed - discovery uses static registry)

        // Act: Discover capabilities
        let capabilities = discover_capabilities_typetag();

        // Assert: Known capability types discovered
        assert!(!capabilities.is_empty());
        assert!(capabilities.contains(&"FileReaderCapability".to_string()));
        assert!(capabilities.contains(&"FileWriterCapability".to_string()));
    }

    // =============================================================================
    // Error Handling Tests
    // =============================================================================

    #[test]
    fn test_invalid_uri_returns_error() {
        // Arrange: Create capability with invalid URI
        let capability = FileReaderCapability {
            uri: "not a valid URI!!!".to_string(),
            name: "Invalid".to_string(),
            description: "Invalid URI test".to_string(),
        };

        // Act: Attempt introspection
        let result = capability.introspect_rdf_oxrdf();

        // Assert: Error returned (no panic)
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_capability_name_still_generates_triples() {
        // Arrange: Create capability with empty name
        let capability = FileReaderCapability {
            uri: "https://cnv.dev/capability#Empty".to_string(),
            name: "".to_string(),
            description: "Empty name test".to_string(),
        };

        // Act: Introspect capability
        let result = capability.introspect_rdf_oxrdf();

        // Assert: Still generates triples successfully
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 3);
    }

    // =============================================================================
    // Performance Characteristics Tests
    // =============================================================================

    #[test]
    fn test_introspection_completes_within_performance_budget() {
        // Arrange: Create capability
        let capability = FileReaderCapability {
            uri: "https://cnv.dev/capability#Perf".to_string(),
            name: "Performance Test".to_string(),
            description: "Performance verification".to_string(),
        };

        // Act: Measure introspection time
        let start = std::time::Instant::now();
        let result = capability.introspect_rdf_oxrdf();
        let duration = start.elapsed();

        // Assert: Completes quickly (< 1ms per triple target)
        assert!(result.is_ok());
        assert!(
            duration.as_micros() < 1000,
            "Introspection took {:?}, should be < 1ms",
            duration
        );
    }

    #[test]
    fn test_framework_scales_with_many_capabilities() {
        // Arrange: Create framework with many capabilities
        let mut framework = MetaFramework::new();
        for i in 0..100 {
            framework.register(Arc::new(FileReaderCapability {
                uri: format!("https://cnv.dev/capability#Cap{}", i),
                name: format!("Capability {}", i),
                description: format!("Test capability {}", i),
            }));
        }

        // Act: Introspect all
        let start = std::time::Instant::now();
        let result = framework.introspect_all_rdf();
        let duration = start.elapsed();

        // Assert: Scales linearly
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 300); // 100 caps * 3 triples
        assert!(
            duration.as_millis() < 10,
            "100 capabilities took {:?}, should be < 10ms",
            duration
        );
    }
}
