// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for `Graph`, `Triple`, and `GraphLoadedOutput`.

use clap_noun_verb::{Graph, GraphLoadedOutput, Triple};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new();
    graph
        .add_triple(Triple::new("ex:alice", "rdf:type", "ex:Person"))
        .map_err(std::io::Error::other)?;
    graph
        .add_triple(Triple::new("ex:alice", "foaf:knows", "ex:bob"))
        .map_err(std::io::Error::other)?;
    graph
        .add_triple(Triple::new("ex:bob", "rdf:type", "ex:Person"))
        .map_err(std::io::Error::other)?;

    assert_eq!(graph.len(), 3);
    assert_eq!(graph.query_by_subject("alice").len(), 2);
    assert_eq!(graph.query_by_predicate("rdf:type").len(), 2);
    assert!(graph.validate_all().is_empty());

    let invalid = Triple::new("", "rdf:type", "ex:Person");
    assert!(graph.add_triple(invalid).is_err());
    assert_eq!(graph.len(), 3, "refused triples must not mutate the graph");

    let receipt = GraphLoadedOutput::new(graph.len(), "memory://graph-api");
    assert_eq!(receipt.status, "success");
    assert_eq!(receipt.triples_loaded, 3);
    assert_eq!(receipt.source, "memory://graph-api");

    println!("Graph admitted {} triples; invalid triple refused", graph.len());
    Ok(())
}
