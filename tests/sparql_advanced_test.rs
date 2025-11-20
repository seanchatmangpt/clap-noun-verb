use clap_noun_verb::rdf::{Ontology, QueryExecutor, SparqlParser};
use std::sync::Arc;

fn create_test_ontology() -> Arc<Ontology> {
    let mut ont = Ontology::new();

    // Add verb types
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "cmd:build",
        "rdf:type",
        clap_noun_verb::rdf::RdfValue::uri("cnv:Verb"),
    ));
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "cmd:test",
        "rdf:type",
        clap_noun_verb::rdf::RdfValue::uri("cnv:Verb"),
    ));
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "cmd:deploy",
        "rdf:type",
        clap_noun_verb::rdf::RdfValue::uri("cnv:Verb"),
    ));

    // Add comments
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "cmd:build",
        "rdfs:comment",
        clap_noun_verb::rdf::RdfValue::literal("Build the project"),
    ));
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "cmd:test",
        "rdfs:comment",
        clap_noun_verb::rdf::RdfValue::literal("Run tests"),
    ));
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "cmd:deploy",
        "rdfs:comment",
        clap_noun_verb::rdf::RdfValue::literal("Check deployment status"),
    ));

    // Add property paths
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "group:dev",
        ":hasMember",
        clap_noun_verb::rdf::RdfValue::uri("user:alice"),
    ));
    ont.add_triple(clap_noun_verb::rdf::RdfTriple::new(
        "user:alice",
        ":hasMember",
        clap_noun_verb::rdf::RdfValue::uri("user:bob"),
    ));

    Arc::new(ont)
}

#[test]
fn test_simple_triple_pattern_query() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?verb WHERE { ?verb a cnv:Verb }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    assert_eq!(results.len(), 3);
    assert!(results
        .iter()
        .any(|b| b.variables.get("?verb").map_or(false, |v| v.as_str() == "cmd:build")));
}

#[test]
fn test_filter_contains() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = r#"SELECT ?verb WHERE { ?verb a cnv:Verb . ?verb rdfs:comment ?comment . FILTER(CONTAINS(?comment, "status")) }"#;
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    assert_eq!(results.len(), 1);
    assert!(results
        .iter()
        .any(|b| b.variables.get("?verb").map_or(false, |v| v.as_str() == "cmd:deploy")));
}

#[test]
fn test_property_path_query() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?x WHERE { ?x :hasMember* ?y }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    // Should return reflexive + direct + transitive paths
    assert!(!results.is_empty());
}

#[test]
fn test_union_query() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?x WHERE { { ?x a cnv:Verb } UNION { ?x a cnv:Command } }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    // Should have at least the verbs
    assert!(results.len() >= 3);
}

#[test]
fn test_optional_query() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?x ?desc WHERE { ?x a cnv:Verb . OPTIONAL { ?x rdfs:comment ?desc } }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    // All verbs should be returned, some with comments
    assert_eq!(results.len(), 3);
}

#[test]
fn test_filter_with_aggregation() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT (COUNT(?verb) AS ?count) WHERE { ?verb a cnv:Verb }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    assert_eq!(results.len(), 1);
    assert!(results[0].variables.get("?count").map_or(false, |v| v.as_str() == "3"));
}

#[test]
fn test_filter_string_functions() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query =
        r#"SELECT ?cmd WHERE { ?cmd rdfs:comment ?desc . FILTER(CONTAINS(?desc, "Build")) }"#;
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    assert_eq!(results.len(), 1);
    assert!(results
        .iter()
        .any(|b| b.variables.get("?cmd").map_or(false, |v| v.as_str() == "cmd:build")));
}

#[test]
fn test_parser_tokenization() {
    let query = "SELECT ?x WHERE { ?x a cnv:Verb }";
    let parsed = SparqlParser::parse(query).expect("parse failed");

    assert_eq!(parsed.select_vars.len(), 1);
    assert_eq!(parsed.select_vars[0], "?x");
    assert_eq!(parsed.where_patterns.len(), 1);
}

#[test]
fn test_parser_filter_syntax() {
    let query = r#"SELECT ?x WHERE { ?x rdfs:label ?label . FILTER(CONTAINS(?label, "test")) }"#;
    let parsed = SparqlParser::parse(query).expect("parse failed");

    assert_eq!(parsed.filters.len(), 1);
}

#[test]
fn test_optimizer_cardinality_estimation() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?verb WHERE { ?verb a cnv:Verb }";
    let parsed = SparqlParser::parse(query).expect("parse failed");

    // Optimizer should create an efficient plan
    let results = executor.execute(parsed).expect("execution failed");
    assert_eq!(results.len(), 3);
}

#[test]
fn test_executor_binding_join() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?verb ?comment WHERE { ?verb a cnv:Verb . ?verb rdfs:comment ?comment }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    // All verbs have comments
    assert_eq!(results.len(), 3);
    for binding in &results {
        assert!(binding.variables.contains_key("?verb"));
        assert!(binding.variables.contains_key("?comment"));
    }
}

#[test]
fn test_empty_result_handling() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = "SELECT ?x WHERE { ?x a cnv:NonExistent }";
    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    assert_eq!(results.len(), 0);
}

#[test]
fn test_complex_filter_expression() {
    let ont = create_test_ontology();
    let executor = QueryExecutor::new(ont);

    let query = r#"SELECT ?cmd WHERE {
        ?cmd a cnv:Verb .
        ?cmd rdfs:comment ?desc .
        FILTER(CONTAINS(?desc, "test"))
    }"#;

    let parsed = SparqlParser::parse(query).expect("parse failed");
    let results = executor.execute(parsed).expect("execution failed");

    assert!(results
        .iter()
        .any(|b| b.variables.get("?cmd").map_or(false, |v| v.as_str() == "cmd:test")));
}
