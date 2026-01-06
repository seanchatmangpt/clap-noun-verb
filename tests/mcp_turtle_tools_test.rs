//! Integration tests for MCP Turtle tools
//!
//! Tests the three main MCP tools for RDF Turtle CLI generation:
//! 1. GenerateCliFromTurtle
//! 2. QueryCapabilities
//! 3. ExportToTurtle
//!
//! Chicago TDD pattern: AAA (Arrange-Act-Assert), behavior verification

#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::{
    ExportToTurtle, ExportToTurtleInput, GenerateCliFromTurtle, GenerateCliInput,
    QueryCapabilities, QueryCapabilitiesInput, QueryOperation, TurtleParser,
};

/// Test GenerateCliFromTurtle tool with complete ontology
#[cfg(feature = "rdf-composition")]
#[test]
fn test_generate_cli_from_turtle_integration() {
    // Arrange - Complete ontology with nouns and verbs
    let turtle_definition = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:Services rdf:type cnv:Noun ;
    cnv:name "services" ;
    rdfs:label "Service management commands" .

cnv:Database rdf:type cnv:Noun ;
    cnv:name "database" ;
    rdfs:label "Database operations" .

cnv:StatusVerb rdf:type cnv:Verb ;
    cnv:name "status" ;
    rdfs:label "Check service status" ;
    cnv:hasNoun cnv:Services .

cnv:StartVerb rdf:type cnv:Verb ;
    cnv:name "start" ;
    rdfs:label "Start service" ;
    cnv:hasNoun cnv:Services .

cnv:MigrateVerb rdf:type cnv:Verb ;
    cnv:name "migrate" ;
    rdfs:label "Run database migrations" ;
    cnv:hasNoun cnv:Database .
"#
    .to_string();

    let input = GenerateCliInput {
        turtle_definition,
        ontology_iri: "https://cnv.dev/ontology#".to_string(),
    };

    // Act - Generate CLI code
    let result = GenerateCliFromTurtle::execute(input);

    // Assert - Verify successful code generation
    assert!(
        result.is_ok(),
        "Code generation failed: {:?}",
        result.err()
    );

    let output = result.unwrap();

    // Verify Rust code contains expected elements
    assert!(!output.rust_code.is_empty(), "Generated code is empty");
    assert!(
        output.rust_code.contains("Services"),
        "Generated code missing Services noun"
    );
    assert!(
        output.rust_code.contains("Database"),
        "Generated code missing Database noun"
    );
    assert!(
        output.rust_code.contains("status"),
        "Generated code missing status verb"
    );
    assert!(
        output.rust_code.contains("migrate"),
        "Generated code missing migrate verb"
    );

    // Verify diagnostics present
    assert!(
        !output.diagnostics.is_empty(),
        "No diagnostics generated"
    );
}

/// Test GenerateCliFromTurtle with minimal ontology
#[cfg(feature = "rdf-composition")]
#[test]
fn test_generate_cli_minimal_ontology() {
    // Arrange - Minimal ontology with single verb
    let input = GenerateCliInput {
        turtle_definition: r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:BuildVerb rdf:type cnv:Verb ;
    cnv:name "build" .
"#
        .to_string(),
        ontology_iri: "https://cnv.dev/ontology#".to_string(),
    };

    // Act
    let result = GenerateCliFromTurtle::execute(input);

    // Assert
    assert!(result.is_ok(), "Minimal ontology generation failed");
    let output = result.unwrap();
    assert!(output.rust_code.contains("build"));
}

/// Test GenerateCliFromTurtle error handling for invalid input
#[cfg(feature = "rdf-composition")]
#[test]
fn test_generate_cli_invalid_turtle() {
    // Arrange - Invalid Turtle syntax
    let input = GenerateCliInput {
        turtle_definition: "This is not valid Turtle syntax @#$%".to_string(),
        ontology_iri: "https://cnv.dev/ontology#".to_string(),
    };

    // Act
    let result = GenerateCliFromTurtle::execute(input);

    // Assert - Should return error for invalid Turtle
    assert!(result.is_err(), "Should fail on invalid Turtle");
}

/// Test GenerateCliFromTurtle with empty input
#[cfg(feature = "rdf-composition")]
#[test]
fn test_generate_cli_empty_input() {
    // Arrange
    let input = GenerateCliInput {
        turtle_definition: "".to_string(),
        ontology_iri: "https://cnv.dev/ontology#".to_string(),
    };

    // Act
    let result = GenerateCliFromTurtle::execute(input);

    // Assert - Should return InvalidInput error
    assert!(result.is_err(), "Should fail on empty input");
}

/// Test QueryCapabilities tool for listing commands
#[cfg(feature = "rdf-composition")]
#[test]
fn test_query_capabilities_list_commands() {
    // Arrange - Create ontology with commands
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:BuildVerb rdf:type cnv:Verb ;
    cnv:name "build" .

cnv:TestVerb rdf:type cnv:Verb ;
    cnv:name "test" .

cnv:DeployVerb rdf:type cnv:Verb ;
    cnv:name "deploy" .
"#;

    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse test ontology");
    let ontology = parsed.into_ontology();

    let tool = QueryCapabilities::new(ontology);

    let input = QueryCapabilitiesInput {
        sparql_query: r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            SELECT ?name WHERE {
                ?verb cnv:name ?name .
            }
        "#
        .to_string(),
        operation: QueryOperation::ListCommands,
    };

    // Act
    let result = tool.execute(input);

    // Assert
    assert!(result.is_ok(), "Query execution failed: {:?}", result.err());
    let output = result.unwrap();

    assert!(output.found, "No commands found");
    assert!(
        output.results.len() >= 3,
        "Expected at least 3 commands, found {}",
        output.results.len()
    );

    // Verify specific commands present
    let has_build = output.results.iter().any(|s| s == "build");
    let has_test = output.results.iter().any(|s| s == "test");
    let has_deploy = output.results.iter().any(|s| s == "deploy");

    assert!(has_build || has_test || has_deploy, "Missing expected commands");
}

/// Test QueryCapabilities for finding verbs
#[cfg(feature = "rdf-composition")]
#[test]
fn test_query_capabilities_find_verb() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:RunVerb rdf:type cnv:Verb ;
    cnv:name "run" .
"#;

    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let tool = QueryCapabilities::new(parsed.into_ontology());

    let input = QueryCapabilitiesInput {
        sparql_query: r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            SELECT ?verb WHERE {
                ?verb rdf:type cnv:Verb .
            }
        "#
        .to_string(),
        operation: QueryOperation::FindVerb,
    };

    // Act
    let result = tool.execute(input);

    // Assert
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.found);
    assert!(!output.results.is_empty());
}

/// Test QueryCapabilities describe operation
#[cfg(feature = "rdf-composition")]
#[test]
fn test_query_capabilities_describe() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:InfoVerb rdf:type cnv:Verb ;
    cnv:name "info" ;
    rdfs:label "Show information" .
"#;

    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let tool = QueryCapabilities::new(parsed.into_ontology());

    let input = QueryCapabilitiesInput {
        sparql_query: r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT ?name ?label WHERE {
                ?verb cnv:name ?name .
                ?verb rdfs:label ?label .
            }
        "#
        .to_string(),
        operation: QueryOperation::Describe,
    };

    // Act
    let result = tool.execute(input);

    // Assert
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.found);
    assert!(!output.results.is_empty());
    assert!(output.results[0].contains("info"));
}

/// Test ExportToTurtle stub generation
#[cfg(feature = "rdf-composition")]
#[test]
fn test_export_to_turtle_stub() {
    // Arrange & Act
    let turtle = ExportToTurtle::generate_stub("myapp");

    // Assert
    assert!(turtle.contains("@prefix cnv:"), "Missing CNV prefix");
    assert!(turtle.contains("myapp"), "Missing CLI name");
    assert!(
        turtle.contains("cnv:GeneratedCli"),
        "Missing GeneratedCli resource"
    );
    assert!(turtle.contains("rdf:type"), "Missing rdf:type property");
}

/// Test ExportToTurtle execution (should fail as not implemented)
#[cfg(feature = "rdf-composition")]
#[test]
fn test_export_to_turtle_not_implemented() {
    // Arrange
    let input = ExportToTurtleInput {
        cli_source_code: r#"
fn main() {
    println!("Test CLI");
}
"#
        .to_string(),
        cli_name: "testcli".to_string(),
    };

    // Act
    let result = ExportToTurtle::execute(input);

    // Assert - Should fail with ExportNotImplemented error
    assert!(result.is_err(), "Should fail as not implemented");
}

/// End-to-end integration test: Generate CLI, verify output
#[cfg(feature = "rdf-composition")]
#[test]
fn test_end_to_end_cli_generation() {
    // Arrange - Real-world ontology
    let input = GenerateCliInput {
        turtle_definition: r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:Project rdf:type cnv:Noun ;
    cnv:name "project" ;
    rdfs:label "Project management" .

cnv:InitVerb rdf:type cnv:Verb ;
    cnv:name "init" ;
    rdfs:label "Initialize new project" ;
    cnv:hasNoun cnv:Project .

cnv:BuildVerb rdf:type cnv:Verb ;
    cnv:name "build" ;
    rdfs:label "Build the project" ;
    cnv:hasNoun cnv:Project .

cnv:TestVerb rdf:type cnv:Verb ;
    cnv:name "test" ;
    rdfs:label "Run tests" ;
    cnv:hasNoun cnv:Project .
"#
        .to_string(),
        ontology_iri: "https://cnv.dev/ontology#".to_string(),
    };

    // Act - Generate CLI
    let result = GenerateCliFromTurtle::execute(input);

    // Assert - Comprehensive verification
    assert!(result.is_ok(), "End-to-end generation failed");

    let output = result.unwrap();
    let code = &output.rust_code;

    // Verify noun present
    assert!(code.contains("Project") || code.contains("project"));

    // Verify all verbs present
    assert!(code.contains("init"));
    assert!(code.contains("build"));
    assert!(code.contains("test"));

    // Verify Rust syntax elements
    assert!(
        code.contains("pub") || code.contains("fn"),
        "Missing Rust syntax"
    );

    // Verify diagnostics
    assert!(!output.diagnostics.is_empty());
    assert!(output.diagnostics[0].level == "info");
}
