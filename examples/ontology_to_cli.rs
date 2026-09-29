// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable ontology → typed adapter projection.
//!
//! Full filesystem manufacture and compilation are exercised by the pinned ggen
//! workflow. This example witnesses the in-process semantic projection boundary.

use clap_noun_verb::rdf_to_ggen::{
    rdf_spec_to_verb_code, rdf_triples_to_verb_definitions, sparql_results_to_verb_definitions,
    ArgumentType, ObjectType, RdfArgumentDefinition, RdfTriple, RdfVerbDefinition,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let verb = RdfVerbDefinition {
        verb_uri: "https://example.org/GraphLoad".to_string(),
        name: "load".to_string(),
        description: "Load an admitted graph".to_string(),
        noun_uri: Some("https://example.org/GraphNoun".to_string()),
        noun_name: Some("graph".to_string()),
        arguments: vec![RdfArgumentDefinition {
            arg_uri: "https://example.org/path".to_string(),
            name: "path".to_string(),
            description: "Admitted graph path".to_string(),
            value_type: "String".to_string(),
            required: true,
            is_flag: false,
            default_value: None,
            short_name: None,
            long_name: None,
            allowed_values: Vec::new(),
            argument_type: ArgumentType::Positional,
        }],
        return_type: "GraphLoadedOutput".to_string(),
        trait_bounds: vec!["Send".to_string(), "Sync".to_string()],
        docstring: "Load one graph through the authored domain handler".to_string(),
        is_async: false,
    };

    let adapter = rdf_spec_to_verb_code(&verb);
    assert!(adapter.contains("#[verb(\"load\", \"graph\")]"));
    assert!(adapter.contains("crate::handlers::graph_load(path)"));

    let sparql = r#"{
      "results": {
        "bindings": [
          {
            "verb": {"type": "uri", "value": "https://example.org/Zeta"},
            "verbName": {"type": "literal", "value": "zeta"}
          },
          {
            "verb": {"type": "uri", "value": "https://example.org/Alpha"},
            "verbName": {"type": "literal", "value": "alpha"}
          }
        ]
      }
    }"#;
    let from_sparql = sparql_results_to_verb_definitions(sparql)?;
    assert_eq!(
        from_sparql.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );

    let subject = "https://example.org/ValidateOntology";
    let from_triples = rdf_triples_to_verb_definitions(vec![
        RdfTriple {
            subject: subject.to_string(),
            predicate: "cnv:hasVerbName".to_string(),
            object: "validate".to_string(),
            object_type: ObjectType::Literal,
        },
        RdfTriple {
            subject: subject.to_string(),
            predicate: "cnv:belongsToNoun".to_string(),
            object: "https://example.org/OntologyNoun".to_string(),
            object_type: ObjectType::Reference,
        },
        RdfTriple {
            subject: subject.to_string(),
            predicate: "cnv:returnType".to_string(),
            object: "ValidationResultOutput".to_string(),
            object_type: ObjectType::Literal,
        },
    ])?;
    assert_eq!(from_triples[0].noun_name.as_deref(), Some("ontology"));

    println!(
        "Ontology projection admitted {} SPARQL verbs, {} RDF verb, and one typed adapter",
        from_sparql.len(),
        from_triples.len()
    );
    Ok(())
}
