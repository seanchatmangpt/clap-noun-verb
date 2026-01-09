//! Integration tests for Turtle parser and SPARQL executor
//!
//! These tests verify the complete workflow:
//! 1. Parse Turtle documents into RDF ontology
//! 2. Execute SPARQL queries against the parsed ontology
//! 3. Validate query results with Chicago TDD methodology
//!
//! Test methodology: Chicago TDD
//! - State-based testing with real collaborators
//! - AAA (Arrange-Act-Assert) pattern
//! - Behavior verification through observable outputs

#![cfg(feature = "rdf-composition")]

use clap_noun_verb::rdf::sparql_executor_oxigraph::SparqlExecutor;
use clap_noun_verb::rdf::turtle_parser::TurtleParser;

/// Integration test: Parse CNV ontology and query for verbs
#[test]
fn test_parse_cnv_ontology_and_query_verbs() {
    // Arrange: Create a CNV ontology in Turtle format
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:BuildCommand rdf:type cnv:Verb ;
    cnv:name "build" ;
    rdfs:label "Build the project" ;
    cnv:description "Compiles source code and creates artifacts" .

cnv:TestCommand rdf:type cnv:Verb ;
    cnv:name "test" ;
    rdfs:label "Run tests" ;
    cnv:description "Executes test suite and reports results" .

cnv:DeployCommand rdf:type cnv:Verb ;
    cnv:name "deploy" ;
    rdfs:label "Deploy application" ;
    cnv:description "Deploys application to target environment" .
"#;

    // Act: Parse Turtle and execute SPARQL query
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse Turtle");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT ?verb ?name ?label WHERE {
            ?verb rdf:type cnv:Verb ;
                  cnv:name ?name ;
                  rdfs:label ?label .
        }
        ORDER BY ?name
    "#;

    let result = executor.execute_query(query).expect("Query execution failed");

    // Assert: Verify query results
    assert_eq!(result.len(), 3, "Expected 3 verbs");
    assert_eq!(result.variables(), &["verb", "name", "label"]);

    // Verify first verb (alphabetically sorted)
    let first = &result.bindings[0];
    assert_eq!(first.get("name"), Some("build"));
    assert_eq!(first.get("label"), Some("Build the project"));

    // Verify second verb
    let second = &result.bindings[1];
    assert_eq!(second.get("name"), Some("deploy"));

    // Verify third verb
    let third = &result.bindings[2];
    assert_eq!(third.get("name"), Some("test"));
}

/// Integration test: Parse ontology with multiple namespaces
#[test]
fn test_parse_multi_namespace_ontology() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix dc: <http://purl.org/dc/elements/1.1/> .

cnv:ServiceNoun rdf:type cnv:Noun ;
    cnv:name "service" ;
    rdfs:label "Service resource" ;
    dc:creator "CNV Framework" ;
    dc:created "2024-01-01"^^xsd:date .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");

    // Assert: Verify prefixes were extracted
    let prefixes = parsed.resolve_prefixes().expect("Failed to resolve prefixes");
    assert!(prefixes.contains_key("cnv"), "Missing cnv prefix");
    assert!(prefixes.contains_key("rdf"), "Missing rdf prefix");
    assert!(prefixes.contains_key("rdfs"), "Missing rdfs prefix");
    assert!(prefixes.contains_key("xsd"), "Missing xsd prefix");
    assert!(prefixes.contains_key("dc"), "Missing dc prefix");

    // Query the ontology
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        SELECT ?noun WHERE {
            ?noun <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> cnv:Noun .
        }
    "#;

    let result = executor.execute_query(query).expect("Query failed");
    assert_eq!(result.len(), 1, "Expected 1 noun");
}

/// Integration test: Query with FILTER clause
#[test]
fn test_sparql_filter_query() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:Cmd1 rdf:type cnv:Verb ; cnv:name "alpha" ; cnv:priority "10" .
cnv:Cmd2 rdf:type cnv:Verb ; cnv:name "beta" ; cnv:priority "20" .
cnv:Cmd3 rdf:type cnv:Verb ; cnv:name "gamma" ; cnv:priority "5" .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>

        SELECT ?cmd ?name ?priority WHERE {
            ?cmd cnv:name ?name ;
                 cnv:priority ?priority .
            FILTER(?priority = "10")
        }
    "#;

    let result = executor.execute_query(query).expect("Query failed");

    // Assert
    assert_eq!(result.len(), 1, "Expected 1 result");
    assert_eq!(result.bindings[0].get("name"), Some("alpha"));
    assert_eq!(result.bindings[0].get("priority"), Some("10"));
}

/// Integration test: List all classes in parsed ontology
#[test]
fn test_list_classes_integration() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:Verb rdf:type rdfs:Class .
cnv:Noun rdf:type rdfs:Class .
cnv:Command rdf:type rdfs:Class .

cnv:BuildVerb rdf:type cnv:Verb .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");
    let classes = executor.list_classes().expect("Failed to list classes");

    // Assert
    assert!(classes.iter().any(|c| c.contains("Verb")), "Verb class not found");
    assert!(classes.iter().any(|c| c.contains("Noun")), "Noun class not found");
    assert!(classes.iter().any(|c| c.contains("Command")), "Command class not found");
}

/// Integration test: List all properties in parsed ontology
#[test]
fn test_list_properties_integration() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:name rdf:type rdf:Property .
cnv:description rdf:type rdf:Property .

cnv:SomeVerb cnv:customProperty "value" .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");
    let properties = executor.list_properties().expect("Failed to list properties");

    // Assert
    assert!(properties.iter().any(|p| p.contains("name")), "name property not found");
    assert!(properties.iter().any(|p| p.contains("description")), "description property not found");
    assert!(properties.iter().any(|p| p.contains("customProperty")), "customProperty not found");
}

/// Integration test: Parse and validate ontology constraints
#[test]
fn test_ontology_validation_integration() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:ValidVerb rdf:type cnv:Verb ;
    cnv:name "valid" .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let validation = parsed.validate_ontology();

    // Assert
    assert!(validation.is_ok(), "Validation should pass for valid ontology");
}

/// Integration test: ASK query to check existence
#[test]
fn test_ask_query_integration() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:BuildCommand rdf:type cnv:Verb .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        ASK {
            cnv:BuildCommand rdf:type cnv:Verb .
        }
    "#;

    let result = executor.execute_query(query).expect("ASK query failed");

    // Assert
    assert_eq!(result.len(), 1);
    assert_eq!(result.bindings[0].get("result"), Some("true"));
}

/// Integration test: Complex query with OPTIONAL clause
#[test]
fn test_optional_clause_integration() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:Cmd1 rdf:type cnv:Verb ; cnv:name "cmd1" ; rdfs:comment "Has comment" .
cnv:Cmd2 rdf:type cnv:Verb ; cnv:name "cmd2" .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?cmd ?name ?comment WHERE {
            ?cmd rdf:type cnv:Verb ;
                 cnv:name ?name .
            OPTIONAL { ?cmd rdfs:comment ?comment . }
        }
        ORDER BY ?name
    "#;

    let result = executor.execute_query(query).expect("Query failed");

    // Assert
    assert_eq!(result.len(), 2, "Expected 2 results");

    // First command has comment
    assert_eq!(result.bindings[0].get("name"), Some("cmd1"));
    assert_eq!(result.bindings[0].get("comment"), Some("Has comment"));

    // Second command has no comment (optional not bound)
    assert_eq!(result.bindings[1].get("name"), Some("cmd2"));
    assert_eq!(result.bindings[1].get("comment"), None);
}

/// Integration test: Prefix resolution in parsed document
#[test]
fn test_prefix_resolution_consistency() {
    // Arrange
    let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix custom: <http://example.org/custom#> .

custom:MyVerb <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> cnv:Verb .
"#;

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(turtle).expect("Failed to parse");
    let prefixes = parsed.resolve_prefixes().expect("Failed to resolve");

    // Assert
    assert_eq!(prefixes.get("cnv"), Some(&"https://cnv.dev/ontology#".to_string()));
    assert_eq!(prefixes.get("custom"), Some(&"http://example.org/custom#".to_string()));

    // Verify ontology was populated correctly
    let ontology = parsed.ontology();
    assert!(!ontology.is_empty(), "Ontology should not be empty");
}

/// Integration test: Empty Turtle document
#[test]
fn test_empty_turtle_document() {
    // Arrange
    let empty_turtle = "";

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(empty_turtle).expect("Should parse empty document");
    let executor = SparqlExecutor::new(&parsed).expect("Should create executor");

    let query = "SELECT * WHERE { ?s ?p ?o . }";
    let result = executor.execute_query(query).expect("Query should succeed");

    // Assert
    assert!(result.is_empty(), "Empty ontology should return no results");
}

/// Integration test: Performance - parse and query large ontology
#[test]
fn test_performance_large_ontology() {
    // Arrange: Generate a larger ontology with 100 verbs
    let mut turtle = String::from(
        r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

"#,
    );

    for i in 0..100 {
        turtle.push_str(&format!(
            "cnv:Verb{} rdf:type cnv:Verb ; cnv:name \"verb{}\" ; rdfs:label \"Verb {}\" .\n",
            i, i, i
        ));
    }

    // Act
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse large ontology");
    assert_eq!(parsed.triple_count(), 300, "Expected 300 triples (3 per verb)");

    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT (COUNT(?verb) as ?count) WHERE {
            ?verb rdf:type cnv:Verb .
        }
    "#;

    let result = executor.execute_query(query).expect("Count query failed");

    // Assert
    assert_eq!(result.len(), 1);
    assert_eq!(result.bindings[0].get("count"), Some("100"));
}
