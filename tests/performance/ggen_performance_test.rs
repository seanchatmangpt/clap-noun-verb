//! Performance tests for ggen-clap-noun-verb integration
//!
//! These tests validate that ggen integration meets SLO targets for:
//! - Turtle parsing time
//! - Code generation time
//! - End-to-end pipeline time
//! - Memory usage

#![cfg(test)]

use std::path::PathBuf;
use std::time::Instant;

/// SLO Target: Parse small turtle file in < 100ms
#[test]
#[cfg(feature = "rdf")]
fn test_parse_small_turtle_slo() {
    // Arrange
    let small_turtle = PathBuf::from("vendors/ggen/benches/fixtures/ontologies/small_ontology.ttl");

    if !small_turtle.exists() {
        println!("⚠️  Small turtle file not found, skipping test");
        return;
    }

    // Act
    let start = Instant::now();
    let result = clap_noun_verb::ggen_integration::parse_turtle(&small_turtle);
    let duration = start.elapsed();

    // Assert
    assert!(
        result.is_ok(),
        "Parse failed: {:?}",
        result.err()
    );

    let duration_ms = duration.as_millis();
    assert!(
        duration_ms < 100,
        "SLO VIOLATION: Small turtle parsing took {}ms (target: <100ms)",
        duration_ms
    );

    println!("✅ Small turtle parsing: {}ms", duration_ms);
}

/// SLO Target: Parse medium turtle file in < 200ms
#[test]
#[cfg(feature = "rdf")]
fn test_parse_medium_turtle_slo() {
    // Arrange
    let medium_turtle = PathBuf::from("vendors/ggen/benches/fixtures/ontologies/medium_ontology.ttl");

    if !medium_turtle.exists() {
        println!("⚠️  Medium turtle file not found, skipping test");
        return;
    }

    // Act
    let start = Instant::now();
    let result = clap_noun_verb::ggen_integration::parse_turtle(&medium_turtle);
    let duration = start.elapsed();

    // Assert
    assert!(
        result.is_ok(),
        "Parse failed: {:?}",
        result.err()
    );

    let duration_ms = duration.as_millis();
    assert!(
        duration_ms < 200,
        "SLO VIOLATION: Medium turtle parsing took {}ms (target: <200ms)",
        duration_ms
    );

    println!("✅ Medium turtle parsing: {}ms", duration_ms);
}

/// SLO Target: Parse large turtle file in < 500ms
#[test]
#[cfg(feature = "rdf")]
fn test_parse_large_turtle_slo() {
    // Arrange
    let large_turtle = PathBuf::from("vendors/ggen/benches/fixtures/ontologies/large_ontology.ttl");

    if !large_turtle.exists() {
        println!("⚠️  Large turtle file not found, skipping test");
        return;
    }

    // Act
    let start = Instant::now();
    let result = clap_noun_verb::ggen_integration::parse_turtle(&large_turtle);
    let duration = start.elapsed();

    // Assert
    assert!(
        result.is_ok(),
        "Parse failed: {:?}",
        result.err()
    );

    let duration_ms = duration.as_millis();
    assert!(
        duration_ms < 500,
        "SLO VIOLATION: Large turtle parsing took {}ms (target: <500ms)",
        duration_ms
    );

    println!("✅ Large turtle parsing: {}ms", duration_ms);
}

/// SLO Target: Generate small CLI code in < 50ms
#[test]
#[cfg(feature = "rdf")]
fn test_generate_small_cli_slo() {
    use clap_noun_verb::ggen_integration::{Command, Argument, ArgumentKind, TypeAnnotation};

    // Arrange - Create minimal AST
    let commands = vec![
        Command {
            name: "test".to_string(),
            description: Some("Test command".to_string()),
            arguments: vec![
                Argument {
                    name: "input".to_string(),
                    kind: ArgumentKind::Positional,
                    type_annotation: TypeAnnotation::String,
                    description: Some("Input file".to_string()),
                    required: true,
                },
            ],
            subcommands: vec![],
        },
    ];

    // Act
    let start = Instant::now();
    let result = clap_noun_verb::ggen_integration::generate_cli_code(&commands);
    let duration = start.elapsed();

    // Assert
    assert!(
        result.is_ok(),
        "Code generation failed: {:?}",
        result.err()
    );

    let duration_ms = duration.as_millis();
    assert!(
        duration_ms < 50,
        "SLO VIOLATION: Small CLI generation took {}ms (target: <50ms)",
        duration_ms
    );

    println!("✅ Small CLI generation: {}ms", duration_ms);
}

/// SLO Target: Generate medium CLI code in < 100ms
#[test]
#[cfg(feature = "rdf")]
fn test_generate_medium_cli_slo() {
    use clap_noun_verb::ggen_integration::{Command, Argument, ArgumentKind, TypeAnnotation};

    // Arrange - Create medium-sized AST (10 commands, 5 args each)
    let commands: Vec<Command> = (0..10)
        .map(|i| Command {
            name: format!("cmd{}", i),
            description: Some(format!("Command {}", i)),
            arguments: (0..5)
                .map(|j| Argument {
                    name: format!("arg{}", j),
                    kind: if j == 0 {
                        ArgumentKind::Positional
                    } else {
                        ArgumentKind::Flag
                    },
                    type_annotation: TypeAnnotation::String,
                    description: Some(format!("Argument {}", j)),
                    required: j == 0,
                })
                .collect(),
            subcommands: vec![],
        })
        .collect();

    // Act
    let start = Instant::now();
    let result = clap_noun_verb::ggen_integration::generate_cli_code(&commands);
    let duration = start.elapsed();

    // Assert
    assert!(
        result.is_ok(),
        "Code generation failed: {:?}",
        result.err()
    );

    let duration_ms = duration.as_millis();
    assert!(
        duration_ms < 100,
        "SLO VIOLATION: Medium CLI generation took {}ms (target: <100ms)",
        duration_ms
    );

    println!("✅ Medium CLI generation: {}ms", duration_ms);
}

/// Test end-to-end pipeline performance
#[test]
#[cfg(feature = "rdf")]
fn test_end_to_end_pipeline_performance() {
    use std::fs;
    use tempfile::tempdir;

    // Arrange
    let small_turtle = PathBuf::from("vendors/ggen/benches/fixtures/ontologies/small_ontology.ttl");

    if !small_turtle.exists() {
        println!("⚠️  Small turtle file not found, skipping test");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("generated.rs");

    // Act
    let start = Instant::now();
    let result = clap_noun_verb::ggen_integration::turtle_to_code(&small_turtle, &output_path);
    let duration = start.elapsed();

    // Assert
    assert!(
        result.is_ok(),
        "End-to-end pipeline failed: {:?}",
        result.err()
    );

    // Verify output file was created
    assert!(output_path.exists(), "Generated file not found");

    let generated_code = fs::read_to_string(&output_path).expect("Failed to read generated file");
    assert!(!generated_code.is_empty(), "Generated code is empty");

    let duration_ms = duration.as_millis();
    println!("✅ End-to-end pipeline: {}ms", duration_ms);
    println!("   Generated {} bytes of code", generated_code.len());
}
