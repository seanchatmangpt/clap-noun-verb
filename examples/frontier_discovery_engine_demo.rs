// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for the bounded discovery engine.

#[cfg(feature = "discovery-engine")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap_noun_verb::frontier::{DiscoveryEngine, DiscoveryRecord};
    use std::collections::BTreeSet;

    fn record(name: &str, tags: &[&str], route: &str) -> DiscoveryRecord {
        DiscoveryRecord {
            name: name.to_string(),
            tags: tags.iter().map(|tag| (*tag).to_string()).collect::<BTreeSet<_>>(),
            route: route.to_string(),
        }
    }

    let mut engine = DiscoveryEngine::default();
    engine
        .register(record("semantic-analysis", &["semantic", "analysis"], "semantic analyze"))
        .map_err(std::io::Error::other)?;
    engine
        .register(record("code-generation", &["code", "generation"], "code generate"))
        .map_err(std::io::Error::other)?;
    engine
        .register(record("learning-optimization", &["learning"], "learning optimize"))
        .map_err(std::io::Error::other)?;

    let semantic = engine.search("semantic");
    assert_eq!(semantic.len(), 1);
    assert_eq!(semantic[0].name, "semantic-analysis");
    assert_eq!(engine.search("code-generation")[0].route, "code generate");

    let duplicate = engine.register(record(
        "semantic-analysis",
        &["duplicate"],
        "semantic duplicate",
    ));
    assert!(duplicate.is_err(), "duplicate capability names must refuse");
    assert!(engine.search("missing").is_empty());

    println!("Discovery engine admitted 3 records; duplicate and missing routes refused");
    Ok(())
}

#[cfg(not(feature = "discovery-engine"))]
fn main() {
    println!("Enable --features discovery-engine to execute this bounded witness");
}
