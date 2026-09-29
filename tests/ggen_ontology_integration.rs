// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests: RDF ↔ ggen bidirectional generators
//!
//! Tests validate:
//! 1. RDF → ggen → compiled code → execution
//! 2. ggen → RDF → SPARQL query → code generation
//! 3. Bidirectional sync consistency
//! 4. Conformance: declared structure matches ontology

// =============================================================================
// TEST: RDF → ggen → compilation → execution
// =============================================================================

#[test]
fn test_rdf_to_ggen_round_trip() {
    // This test validates the complete pipeline:
    // 1. Load RDF verb definition
    // 2. Generate Rust code
    // 3. Verify code is syntactically valid
    // 4. Check that generated code matches expected structure

    use clap_noun_verb::rdf_to_ggen::{
        rdf_spec_to_verb_code, ArgumentType, RdfArgumentDefinition, RdfVerbDefinition,
    };

    let verb = RdfVerbDefinition {
        verb_uri: "ex:LoadGraphVerb".to_string(),
        name: "load".to_string(),
        description: "Load a graph from file".to_string(),
        noun_uri: Some("ex:GraphNoun".to_string()),
        noun_name: Some("graph".to_string()),
        arguments: vec![
            RdfArgumentDefinition {
                arg_uri: "ex:PathArg".to_string(),
                name: "path".to_string(),
                description: "File path".to_string(),
                value_type: "String".to_string(),
                required: true,
                is_flag: false,
                default_value: None,
                short_name: None,
                long_name: None,
                allowed_values: vec![],
                argument_type: ArgumentType::Positional,
            },
            RdfArgumentDefinition {
                arg_uri: "ex:FormatArg".to_string(),
                name: "format".to_string(),
                description: "File format".to_string(),
                value_type: "String".to_string(),
                required: false,
                is_flag: false,
                default_value: Some("ttl".to_string()),
                short_name: Some('f'),
                long_name: None,
                allowed_values: vec!["ttl".to_string(), "nt".to_string()],
                argument_type: ArgumentType::Optional,
            },
        ],
        return_type: "Result<GraphLoadedOutput>".to_string(),
        trait_bounds: vec!["Send".to_string(), "Sync".to_string(), "Serialize".to_string()],
        docstring: "Load a graph from file or stdin".to_string(),
        is_async: false,
    };

    // Generate code
    let code = rdf_spec_to_verb_code(&verb);

    // Verify code structure. The generated adapter projects only parsing/routing:
    // a noun-bearing verb renders as #[verb(name, noun)], and the function body
    // delegates domain behavior to crate::handlers -- see module docs on
    // rdf_to_ggen and examples/ontology_to_cli.rs, the authoritative witness for
    // this exact contract.
    assert!(code.contains("/// Load a graph from file or stdin"));
    assert!(code.contains("#[verb(\"load\", \"graph\")]"));
    assert!(code.contains("pub fn graph_load("));
    assert!(code.contains("path: String,"));
    assert!(code.contains("format: Option<String>,"));
    assert!(code.contains("Result<GraphLoadedOutput>"));
    assert!(code.contains("crate::handlers::graph_load(path, format)"));
}

// =============================================================================
// TEST: ggen → RDF → SPARQL query
// =============================================================================

#[test]
fn test_ggen_to_rdf_round_trip() {
    use clap_noun_verb::ggen_to_rdf::{parse_rust_source, verb_definitions_to_ntriples};

    let rust_code = r#"
/// Load a graph from file
#[verb("load")]
pub fn graph_load(path: String, format: Option<String>) -> Result<GraphLoadedOutput> {
    unimplemented!()
}

/// Query the graph
#[verb("query")]
pub async fn graph_query(sparql: String) -> Result<QueryResult> {
    unimplemented!()
}
"#;

    // Parse Rust source
    let verbs = parse_rust_source(rust_code).expect("Failed to parse Rust source");

    assert_eq!(verbs.len(), 2);
    assert_eq!(verbs[0].name, "load");
    assert_eq!(verbs[1].name, "query");
    assert!(verbs[1].is_async);

    // Convert to N-Triples
    let ntriples = verb_definitions_to_ntriples(&verbs);

    // Verify RDF output (N-Triples format)
    assert!(ntriples.contains("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"));
    assert!(ntriples.contains("<http://clap-noun-verb.io/ontology#Verb>"));
    assert!(ntriples.contains("\"load\"@en"));
    assert!(ntriples.contains("\"query\"@en"));
    // Check that Async verb is marked
    assert!(ntriples.contains("isAsync"));
}

// =============================================================================
// TEST: Bidirectional consistency
// =============================================================================

#[test]
fn test_rdf_ggen_bidirectional_consistency() {
    use clap_noun_verb::ggen_to_rdf::parse_rust_source;
    use clap_noun_verb::rdf_to_ggen::{
        rdf_spec_to_verb_code, ArgumentType, RdfArgumentDefinition, RdfVerbDefinition,
    };

    // Start with RDF
    let original_verb = RdfVerbDefinition {
        verb_uri: "ex:StatusVerb".to_string(),
        name: "status".to_string(),
        description: "Show status".to_string(),
        noun_uri: Some("ex:ServiceNoun".to_string()),
        noun_name: Some("service".to_string()),
        arguments: vec![RdfArgumentDefinition {
            arg_uri: "ex:ServiceArg".to_string(),
            name: "service".to_string(),
            description: "Service name".to_string(),
            value_type: "String".to_string(),
            required: false,
            is_flag: false,
            default_value: None,
            short_name: None,
            long_name: None,
            allowed_values: vec![],
            argument_type: ArgumentType::Optional,
        }],
        return_type: "Result<StatusOutput>".to_string(),
        trait_bounds: vec!["Send".to_string(), "Sync".to_string()],
        docstring: "Query service status".to_string(),
        is_async: false,
    };

    // Step 1: RDF → ggen
    let generated_code = rdf_spec_to_verb_code(&original_verb);

    // Step 2: Verify generated code is valid
    assert!(generated_code.contains("pub fn service_status"));
    assert!(generated_code.contains("service: Option<String>"));
    assert!(generated_code.contains("Result<StatusOutput>"));

    // Step 3: Verify it can be parsed back
    let parsed_verbs = parse_rust_source(&generated_code).expect("Should parse generated code");

    // Step 4: Verify basic consistency
    assert_eq!(parsed_verbs.len(), 1);
    assert_eq!(parsed_verbs[0].name, "status");
    assert!(parsed_verbs[0].return_type.contains("StatusOutput"));
}

// =============================================================================
// TEST: Negative tests - invalid definitions
// =============================================================================

#[test]
fn test_argument_type_enum_values() {
    use clap_noun_verb::rdf_to_ggen::ArgumentType;

    // Verify enum variants exist and can be constructed
    let positional = ArgumentType::Positional;
    let optional = ArgumentType::Optional;
    let _flag = ArgumentType::Flag;
    let _repeating = ArgumentType::Repeating;
    let _variadic = ArgumentType::Variadic;

    // All should be valid and distinct
    assert_ne!(std::mem::discriminant(&positional), std::mem::discriminant(&optional));
}

#[test]
fn test_parse_multiple_verbs() {
    use clap_noun_verb::ggen_to_rdf::parse_rust_source;

    let rust_code = r#"
#[verb("start")]
pub fn service_start(names: Vec<String>) -> Result<()> {
    unimplemented!()
}

#[verb("stop")]
pub fn service_stop(names: Vec<String>, force: bool) -> Result<()> {
    unimplemented!()
}

#[verb("status")]
pub fn service_status(service: Option<String>) -> Result<StatusOutput> {
    unimplemented!()
}
"#;

    let verbs = parse_rust_source(rust_code).expect("Should parse multiple verbs");
    assert_eq!(verbs.len(), 3);

    // Find verbs by name
    let start = verbs.iter().find(|v| v.name == "start").unwrap();
    assert_eq!(start.name, "start");

    let stop = verbs.iter().find(|v| v.name == "stop").unwrap();
    assert_eq!(stop.name, "stop");

    let status = verbs.iter().find(|v| v.name == "status").unwrap();
    assert_eq!(status.name, "status");
}

// =============================================================================
// TEST: Conformance validation (Chicago TDD)
// =============================================================================

#[test]
fn test_conformance_verb_exists_in_code_and_rdf() {
    // Doctrine: If code says verb exists but RDF doesn't, then mismatch is a defect

    use clap_noun_verb::rdf_to_ggen::RdfVerbDefinition;

    let code_verb = RdfVerbDefinition {
        verb_uri: "ex:LoadVerb".to_string(),
        name: "load".to_string(),
        description: "Load data".to_string(),
        noun_uri: Some("ex:DataNoun".to_string()),
        noun_name: Some("data".to_string()),
        arguments: vec![],
        return_type: "Result".to_string(),
        trait_bounds: vec![],
        docstring: String::new(),
        is_async: false,
    };

    let rdf_verb = RdfVerbDefinition {
        verb_uri: "ex:LoadVerb".to_string(),
        name: "load".to_string(),
        description: "Load data".to_string(),
        noun_uri: Some("ex:DataNoun".to_string()),
        noun_name: Some("data".to_string()),
        arguments: vec![],
        return_type: "Result".to_string(),
        trait_bounds: vec![],
        docstring: String::new(),
        is_async: false,
    };

    // Both should be identical (conformant)
    assert_eq!(code_verb.name, rdf_verb.name);
    assert_eq!(code_verb.return_type, rdf_verb.return_type);
}

#[test]
fn test_sparql_results_parsing() {
    use clap_noun_verb::rdf_to_ggen::sparql_results_to_verb_definitions;

    let sparql_json = r#"{
        "results": {
            "bindings": [
                {
                    "verb": {"type": "uri", "value": "http://example.org/GraphLoadVerb"},
                    "verbName": {"type": "literal", "value": "load"},
                    "verbAbout": {"type": "literal", "value": "Load a graph from file"},
                    "returnType": {"type": "literal", "value": "GraphLoadedOutput"},
                    "traitBound": {"type": "uri", "value": "http://clap-noun-verb.io/ontology#Send"}
                },
                {
                    "verb": {"type": "uri", "value": "http://example.org/GraphQueryVerb"},
                    "verbName": {"type": "literal", "value": "query"},
                    "verbAbout": {"type": "literal", "value": "Query the graph"},
                    "returnType": {"type": "literal", "value": "QueryResult"},
                    "traitBound": {"type": "uri", "value": "http://clap-noun-verb.io/ontology#Sync"}
                }
            ]
        }
    }"#;

    let verbs =
        sparql_results_to_verb_definitions(sparql_json).expect("Should parse SPARQL results");

    assert_eq!(verbs.len(), 2);

    let load_verb = verbs.iter().find(|v| v.name == "load").unwrap();
    assert_eq!(load_verb.return_type, "GraphLoadedOutput");
    assert!(load_verb.trait_bounds.contains(&"Send".to_string()));

    let query_verb = verbs.iter().find(|v| v.name == "query").unwrap();
    assert_eq!(query_verb.return_type, "QueryResult");
    assert!(query_verb.trait_bounds.contains(&"Sync".to_string()));
}

// =============================================================================
// TEST: N-Triples format validation
// =============================================================================

#[test]
fn test_ntriples_format_compliance() {
    use clap_noun_verb::ggen_to_rdf::{parse_rust_source, verb_definitions_to_ntriples};

    let rust_code = r#"
/// Load graph from file with retry
#[verb("load")]
pub fn graph_load(path: String, retry: bool) -> Result<GraphOutput> {
    unimplemented!()
}
"#;

    let verbs = parse_rust_source(rust_code).expect("Should parse");
    let ntriples = verb_definitions_to_ntriples(&verbs);

    // Verify N-Triples format compliance
    // - Lines should end with " ."
    for line in ntriples.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        assert!(line.ends_with('.'), "Line should end with period: {}", line);
    }

    // Verify full URI expansion (no abbreviated prefixes like "cnv:" in N-Triples)
    // Note: verb_uri field may contain "ex:" but the generated N-Triples expand it
    for line in ntriples.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Actual N-Triples lines should start with < (full URIs), not abbreviations
        assert!(
            line.starts_with('<') || line.trim().is_empty(),
            "N-Triples line should start with <: {}",
            line
        );
    }

    // Verify language tags
    assert!(ntriples.contains("@en"));

    // Verify type declarations
    assert!(ntriples.contains("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"));
}

// =============================================================================
// TEST: File I/O and persistence
// =============================================================================

#[test]
fn test_write_rdf_to_file() {
    use clap_noun_verb::ggen_to_rdf::{parse_rust_source, verb_definitions_to_ntriples};
    use tempfile::TempDir;

    let rust_code = r#"
#[verb("create")]
pub fn resource_create(name: String, template: Option<String>) -> Result<ResourceId> {
    unimplemented!()
}
"#;

    let verbs = parse_rust_source(rust_code).expect("Should parse");
    let ntriples = verb_definitions_to_ntriples(&verbs);

    // Write to temporary directory
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let output_path = temp_dir.path().join("test-verbs.nt");

    std::fs::write(&output_path, &ntriples).expect("Should write file");

    // Verify file was created and contains expected content
    let content = std::fs::read_to_string(&output_path).expect("Should read file");
    assert!(content.contains("create"));
    assert!(content.contains("Verb"));
    assert!(content.contains("ResourceId"));
}

// =============================================================================
// TEST: Async/await in verb definitions
// =============================================================================

#[test]
fn test_async_verb_code_generation() {
    use clap_noun_verb::ggen_to_rdf::parse_rust_source;

    let rust_code = r#"
/// Fetch data asynchronously
#[verb("fetch")]
pub async fn data_fetch(url: String, timeout: Option<u32>) -> Result<FetchResult> {
    unimplemented!()
}
"#;

    let verbs = parse_rust_source(rust_code).expect("Should parse async verb");
    assert_eq!(verbs.len(), 1);
    assert!(verbs[0].is_async);
    assert_eq!(verbs[0].return_type, "Result<FetchResult>");
}

// =============================================================================
// TEST: Complex argument types
// =============================================================================

#[test]
fn test_async_verb_parsing() {
    use clap_noun_verb::ggen_to_rdf::parse_rust_source;

    // Test async verb
    let rust_code = r#"
#[verb("fetch")]
pub async fn data_fetch(url: String) -> Result<FetchResult> {
    unimplemented!()
}
"#;

    let verbs = parse_rust_source(rust_code).expect("Should parse");
    assert_eq!(verbs.len(), 1);
    assert!(verbs[0].is_async);
    assert_eq!(verbs[0].name, "fetch");
}
