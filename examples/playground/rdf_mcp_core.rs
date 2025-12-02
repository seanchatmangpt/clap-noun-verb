//! RDF + MCP Core Pattern (80/20 Consolidated & Abstracted)
//!
//! Single file demonstrating the essential pattern:
//! 1. Build minimal RDF ontology
//! 2. Query for agent decisions
//! 3. Coordinate via MCP memory
//!
//! Run: cargo run --example rdf_mcp_core

use clap_noun_verb::rdf::types::{RdfTriple, RdfValue};
use clap_noun_verb::rdf::CNV_NAMESPACE;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ RDF + MCP Core Pattern (Consolidated & Abstracted)\n");

    // Pattern 1: Define commands declaratively
    let commands = vec![
        cmd("services", "list", &[], Safety::Safe),
        cmd("services", "start", &["name"], Safety::Unsafe { idempotent: true }),
        cmd("services", "stop", &["name"], Safety::Unsafe { idempotent: false }),
        cmd("config", "get", &["key"], Safety::Safe),
        cmd("config", "set", &["key", "value"], Safety::Unsafe { idempotent: true }),
    ];

    // Pattern 2: Generate RDF (abstracted)
    let ontology = Ontology::from_commands(&commands);
    println!("✅ Ontology: {} triples\n", ontology.triples.len());

    // Pattern 3: Query for decisions (abstracted)
    let decisions = AgentDecisions::from_ontology(&ontology);
    decisions.display();

    // Pattern 4: Store in MCP (abstracted pattern)
    println!("\n💾 MCP Storage Pattern:");
    decisions.show_mcp_pattern();

    println!("\n✅ Core pattern complete!");
    println!("   Abstraction = Reusable, Testable, Production-Ready");

    Ok(())
}

// ============================================================================
// ABSTRACTION LAYER: Reusable patterns extracted from 3 playground files
// ============================================================================

/// Command definition (domain model)
struct Command {
    noun: &'static str,
    verb: &'static str,
    params: Vec<&'static str>,
    safety: Safety,
}

enum Safety {
    Safe,
    Unsafe { idempotent: bool },
}

/// Builder function (DRY principle)
fn cmd(noun: &'static str, verb: &'static str, params: &[&'static str], safety: Safety) -> Command {
    Command { noun, verb, params: params.to_vec(), safety }
}

/// RDF Ontology abstraction (Single Responsibility)
struct Ontology {
    triples: Vec<RdfTriple>,
}

impl Ontology {
    fn from_commands(commands: &[Command]) -> Self {
        let mut triples = Vec::new();

        for cmd in commands {
            let uri = format!("{}{}-{}", CNV_NAMESPACE, cmd.noun, cmd.verb);

            // Core identity
            triples.push(triple(&uri, "noun", cmd.noun));
            triples.push(triple(&uri, "verb", cmd.verb));

            // Parameters
            for param in &cmd.params {
                triples.push(triple(&uri, "requires", param));
            }

            // Safety metadata
            match cmd.safety {
                Safety::Safe => {
                    triples.push(triple(&uri, "effect", "read-only"));
                }
                Safety::Unsafe { idempotent } => {
                    triples.push(triple(&uri, "effect", "state-change"));
                    if idempotent {
                        triples.push(triple(&uri, "effect", "idempotent"));
                    }
                }
            }
        }

        Self { triples }
    }

    fn query_by_effect(&self, effect: &str) -> Vec<String> {
        self.triples
            .iter()
            .filter(|t| t.predicate.contains("effect") && t.object.as_str() == effect)
            .map(|t| extract_name(&t.subject))
            .collect()
    }

    fn query_params(&self) -> BTreeMap<String, Vec<String>> {
        let mut result = BTreeMap::new();
        for t in self.triples.iter().filter(|t| t.predicate.contains("requires")) {
            result
                .entry(extract_name(&t.subject))
                .or_insert_with(Vec::new)
                .push(t.object.as_str().to_string());
        }
        result
    }
}

/// Agent Decision Logic (abstracted from all 3 playgrounds)
struct AgentDecisions {
    safe: Vec<String>,
    unsafe_ops: Vec<String>,
    idempotent: Vec<String>,
    params: BTreeMap<String, Vec<String>>,
}

impl AgentDecisions {
    fn from_ontology(ontology: &Ontology) -> Self {
        Self {
            safe: ontology.query_by_effect("read-only"),
            unsafe_ops: ontology.query_by_effect("state-change"),
            idempotent: ontology.query_by_effect("idempotent"),
            params: ontology.query_params(),
        }
    }

    fn display(&self) {
        println!("🎯 Agent Decision Matrix:");
        println!("  Safe (autonomous):       {:?}", self.safe);
        println!("  Unsafe (needs approval): {:?}", self.unsafe_ops);
        println!("  Idempotent (can retry):  {:?}", self.idempotent);

        println!("\n📋 Parameter Requirements:");
        for (cmd, params) in &self.params {
            println!("  {} → {:?}", cmd, params);
        }
    }

    fn show_mcp_pattern(&self) {
        println!("  mcp_store(\"agent/safe\", {:?})", self.safe);
        println!("  mcp_store(\"agent/unsafe\", {:?})", self.unsafe_ops);
        println!("  mcp_store(\"agent/retry\", {:?})", self.idempotent);
    }
}

// Helper functions (DRY)
fn triple(subj: &str, pred: &str, obj: &str) -> RdfTriple {
    RdfTriple {
        subject: subj.to_string(),
        predicate: format!("{}{}", CNV_NAMESPACE, pred),
        object: RdfValue::literal(obj),
    }
}

fn extract_name(uri: &str) -> String {
    uri.split('#').last().unwrap_or(uri).to_string()
}
