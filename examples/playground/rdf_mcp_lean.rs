//! RDF + MCP Lean Playground (80/20)
//!
//! Focuses on the 20% of features that deliver 80% of value:
//! - Critical SPARQL queries for agent decision-making
//! - Essential MCP memory coordination
//! - Minimal code, maximum insight
//!
//! Run: cargo run --example rdf_mcp_lean

use clap_noun_verb::rdf::CNV_NAMESPACE;
use clap_noun_verb::rdf::types::{RdfTriple, RdfValue};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ RDF + MCP Lean Playground (80/20)\n");

    // PART 1: Build minimal ontology (only what matters)
    let triples = build_minimal_ontology();
    println!("✅ {} triples loaded\n", triples.len());

    // PART 2: Critical queries only
    println!("🎯 Critical Query 1: Safe vs Unsafe Commands");
    query_command_safety(&triples);

    println!("\n🎯 Critical Query 2: Command Parameter Requirements");
    query_required_params(&triples);

    println!("\n🎯 Critical Query 3: Idempotent Operations");
    query_idempotent_commands(&triples);

    // PART 3: MCP coordination pattern
    println!("\n💾 MCP Memory Pattern:");
    demonstrate_mcp_pattern();

    println!("\n✅ Done! 80/20 = Maximum value, minimal code");
    Ok(())
}

// 80/20: Build only essential command metadata
fn build_minimal_ontology() -> Vec<RdfTriple> {
    let commands = vec![
        // Safe read-only commands
        ("services-list", "services", "list", vec![], vec!["read-only"]),
        ("config-get", "config", "get", vec!["key"], vec!["read-only"]),

        // Unsafe state-changing commands
        ("services-start", "services", "start", vec!["name"], vec!["state-change", "idempotent"]),
        ("services-stop", "services", "stop", vec!["name"], vec!["state-change"]),
        ("config-set", "config", "set", vec!["key", "value"], vec!["state-change", "idempotent"]),
    ];

    let mut triples = Vec::new();

    for (id, noun, verb, params, effects) in commands {
        let uri = format!("{}{}", CNV_NAMESPACE, id);

        // Essential metadata only
        triples.push(triple(&uri, "type", RdfValue::uri(format!("{}Command", CNV_NAMESPACE))));
        triples.push(triple(&uri, "noun", RdfValue::literal(noun)));
        triples.push(triple(&uri, "verb", RdfValue::literal(verb)));

        // Critical: required parameters
        for param in params {
            triples.push(triple(&uri, "requires", RdfValue::literal(param)));
        }

        // Critical: effects for safety analysis
        for effect in effects {
            triples.push(triple(&uri, "effect", RdfValue::literal(effect)));
        }
    }

    triples
}

fn triple(subj: &str, pred: &str, obj: RdfValue) -> RdfTriple {
    RdfTriple {
        subject: subj.to_string(),
        predicate: format!("{}{}", CNV_NAMESPACE, pred),
        object: obj,
    }
}

// 80/20 Query 1: What's safe for autonomous agents?
fn query_command_safety(triples: &[RdfTriple]) {
    let read_only: Vec<_> = triples.iter()
        .filter(|t| t.predicate.contains("effect") && t.object.as_str() == "read-only")
        .map(|t| extract_command_name(&t.subject))
        .collect();

    let state_change: Vec<_> = triples.iter()
        .filter(|t| t.predicate.contains("effect") && t.object.as_str() == "state-change")
        .map(|t| extract_command_name(&t.subject))
        .collect();

    println!("  ✅ Safe (autonomous): {:?}", read_only);
    println!("  ⚠️  Unsafe (needs approval): {:?}", state_change);
}

// 80/20 Query 2: What inputs are required?
fn query_required_params(triples: &[RdfTriple]) {
    let mut cmd_params: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for t in triples.iter().filter(|t| t.predicate.contains("requires")) {
        cmd_params.entry(extract_command_name(&t.subject))
            .or_default()
            .push(t.object.as_str().to_string());
    }

    for (cmd, params) in cmd_params {
        if params.is_empty() {
            println!("  {} → no params", cmd);
        } else {
            println!("  {} → {:?}", cmd, params);
        }
    }
}

// 80/20 Query 3: Which operations are safe to retry?
fn query_idempotent_commands(triples: &[RdfTriple]) {
    let idempotent: Vec<_> = triples.iter()
        .filter(|t| t.predicate.contains("effect") && t.object.as_str() == "idempotent")
        .map(|t| extract_command_name(&t.subject))
        .collect();

    println!("  Safe to retry: {:?}", idempotent);
    println!("  → Agents can auto-retry these on failure");
}

fn extract_command_name(uri: &str) -> String {
    uri.split('#').last().unwrap_or(uri).to_string()
}

// 80/20 MCP Pattern: Store only critical decision data
fn demonstrate_mcp_pattern() {
    println!("  Store:");
    println!("    key: agent/decisions/safe-commands");
    println!("    value: [services-list, config-get]");
    println!("    → Autonomous agents query this for safe operations");

    println!("\n  Store:");
    println!("    key: agent/decisions/requires-approval");
    println!("    value: [services-start, services-stop, config-set]");
    println!("    → Agents request human approval for these");

    println!("\n  Store:");
    println!("    key: agent/retry/idempotent");
    println!("    value: [services-start, config-set]");
    println!("    → Agents can auto-retry these on transient failures");
}
