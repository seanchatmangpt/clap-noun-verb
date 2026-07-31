// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable semantic coordinator built from admitted frontier primitives.

#[cfg(feature = "frontier-all")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap_noun_verb::frontier::{
        AdmissionState, Invariant, MetaFramework, RdfFragment, SemanticTriple,
    };

    let mut framework = MetaFramework::new();
    framework.register_layer("semantic").map_err(std::io::Error::other)?;
    framework.register_layer("coordination").map_err(std::io::Error::other)?;
    framework
        .admit_invariant(Invariant {
            id: "zero-unreceipted-actuation".to_string(),
            description: "Every actuation must produce a replayable receipt".to_string(),
            satisfied: true,
        })
        .map_err(std::io::Error::other)?;
    framework
        .admit_invariant(Invariant {
            id: "semantic-authority".to_string(),
            description: "RDF owns admitted semantic identity".to_string(),
            satisfied: true,
        })
        .map_err(std::io::Error::other)?;

    assert_eq!(framework.state(false), AdmissionState::Admitted);
    assert_eq!(framework.state(true), AdmissionState::Alive);

    let mut identity = RdfFragment::new();
    identity
        .insert(SemanticTriple {
            subject: "agent:coordinator".to_string(),
            predicate: "rdf:type".to_string(),
            object: "cnv:SemanticCoordinator".to_string(),
        })
        .map_err(std::io::Error::other)?;

    let mut capability = RdfFragment::new();
    capability
        .insert(SemanticTriple {
            subject: "agent:coordinator".to_string(),
            predicate: "cnv:hasCapability".to_string(),
            object: "cnv:Coordinate".to_string(),
        })
        .map_err(std::io::Error::other)?;

    let composed = identity.compose(&capability);
    assert_eq!(composed.triples().len(), 2);
    assert!(identity
        .insert(SemanticTriple {
            subject: String::new(),
            predicate: "rdf:type".to_string(),
            object: "cnv:Invalid".to_string(),
        })
        .is_err());

    println!("Semantic coordinator admitted 2 invariants and 2 canonical triples");
    Ok(())
}

#[cfg(not(feature = "frontier-all"))]
fn main() {
    println!("Enable --features frontier-all to execute the semantic coordinator witness");
}
