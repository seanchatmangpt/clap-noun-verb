//! RDF Interactive Playground - MCP-Coordinated Semantic CLI Exploration
//!
//! This playground demonstrates:
//! - Building RDF graphs for CLI commands using clap-noun-verb's RDF layer
//! - SPARQL queries for discovering command relationships
//! - SHACL validation for command structure
//! - MCP memory integration for persistent semantic knowledge
//!
//! Run with: cargo run --example rdf_interactive_playground

use clap_noun_verb::rdf::prelude::*;
use clap_noun_verb::rdf::{CNV_NAMESPACE, RDF_NS, RDFS_NS};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 RDF Interactive Playground - Semantic CLI Exploration\n");

    // PART 1: Build CLI Command Ontology
    println!("📊 Part 1: Building CLI Command Ontology");
    let ontology = build_cli_ontology()?;
    println!("✅ Created ontology with {} commands\n", ontology.commands.len());

    // PART 2: Create RDF Graph
    println!("🌐 Part 2: Converting to RDF Triples");
    let triples = ontology_to_rdf_triples(&ontology);
    println!("✅ Generated {} RDF triples\n", triples.len());

    // Display sample triples
    println!("Sample triples:");
    for triple in triples.iter().take(5) {
        println!("  {} → {} → {:?}", triple.subject, triple.predicate, triple.object);
    }
    println!();

    // PART 3: Demonstrate SPARQL Queries (conceptual)
    println!("🔍 Part 3: SPARQL Query Examples");
    demonstrate_sparql_queries(&triples);

    // PART 4: Command Relationship Analysis
    println!("\n📈 Part 4: Command Relationship Analysis");
    analyze_command_relationships(&ontology);

    // PART 5: Validate with SHACL Shapes (conceptual)
    println!("\n✅ Part 5: SHACL Validation");
    demonstrate_shacl_validation(&ontology);

    // PART 6: MCP Memory Integration
    println!("\n💾 Part 6: MCP Memory Integration (Conceptual)");
    demonstrate_mcp_integration(&ontology);

    println!("\n🎉 Playground Complete!");
    println!("\n💡 Next Steps:");
    println!("  1. Modify build_cli_ontology() to add your own commands");
    println!("  2. Create custom SPARQL queries to explore relationships");
    println!("  3. Add SHACL shapes for custom validation rules");
    println!("  4. Integrate with MCP swarm for distributed semantic reasoning");

    Ok(())
}

/// CLI Command Ontology
#[derive(Debug, Clone)]
struct CliOntology {
    commands: Vec<CommandDefinition>,
}

#[derive(Debug, Clone)]
struct CommandDefinition {
    uri: String,
    noun: String,
    verb: String,
    description: String,
    parameters: Vec<Parameter>,
    guards: Vec<String>,
    effects: Vec<String>,
}

#[derive(Debug, Clone)]
struct Parameter {
    name: String,
    param_type: String,
    required: bool,
}

/// Build sample CLI ontology
fn build_cli_ontology() -> Result<CliOntology, Box<dyn std::error::Error>> {
    let commands = vec![
        CommandDefinition {
            uri: format!("{}services-list", CNV_NAMESPACE),
            noun: "services".to_string(),
            verb: "list".to_string(),
            description: "List all available services".to_string(),
            parameters: vec![
                Parameter {
                    name: "filter".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                },
                Parameter {
                    name: "verbose".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                },
            ],
            guards: vec!["authenticated".to_string()],
            effects: vec!["read-only".to_string()],
        },
        CommandDefinition {
            uri: format!("{}services-start", CNV_NAMESPACE),
            noun: "services".to_string(),
            verb: "start".to_string(),
            description: "Start a service".to_string(),
            parameters: vec![
                Parameter {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
            ],
            guards: vec!["authenticated".to_string(), "authorized".to_string()],
            effects: vec!["state-change".to_string(), "idempotent".to_string()],
        },
        CommandDefinition {
            uri: format!("{}services-stop", CNV_NAMESPACE),
            noun: "services".to_string(),
            verb: "stop".to_string(),
            description: "Stop a service".to_string(),
            parameters: vec![
                Parameter {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                Parameter {
                    name: "force".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                },
            ],
            guards: vec!["authenticated".to_string(), "authorized".to_string()],
            effects: vec!["state-change".to_string()],
        },
        CommandDefinition {
            uri: format!("{}config-get", CNV_NAMESPACE),
            noun: "config".to_string(),
            verb: "get".to_string(),
            description: "Get configuration value".to_string(),
            parameters: vec![
                Parameter {
                    name: "key".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
            ],
            guards: vec!["authenticated".to_string()],
            effects: vec!["read-only".to_string()],
        },
        CommandDefinition {
            uri: format!("{}config-set", CNV_NAMESPACE),
            noun: "config".to_string(),
            verb: "set".to_string(),
            description: "Set configuration value".to_string(),
            parameters: vec![
                Parameter {
                    name: "key".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                Parameter {
                    name: "value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
            ],
            guards: vec!["authenticated".to_string(), "authorized".to_string()],
            effects: vec!["state-change".to_string(), "idempotent".to_string()],
        },
    ];

    Ok(CliOntology { commands })
}

/// Convert ontology to RDF triples
fn ontology_to_rdf_triples(ontology: &CliOntology) -> Vec<RdfTriple> {
    let mut triples = Vec::new();

    for cmd in &ontology.commands {
        // Command type
        triples.push(RdfTriple {
            subject: cmd.uri.clone(),
            predicate: format!("{}type", RDF_NS),
            object: RdfValue::uri(format!("{}Command", CNV_NAMESPACE)),
        });

        // Command label
        triples.push(RdfTriple {
            subject: cmd.uri.clone(),
            predicate: format!("{}label", RDFS_NS),
            object: RdfValue::literal(format!("{} {}", cmd.noun, cmd.verb)),
        });

        // Command description
        triples.push(RdfTriple {
            subject: cmd.uri.clone(),
            predicate: format!("{}description", RDFS_NS),
            object: RdfValue::literal(cmd.description.clone()),
        });

        // Noun
        triples.push(RdfTriple {
            subject: cmd.uri.clone(),
            predicate: format!("{}noun", CNV_NAMESPACE),
            object: RdfValue::literal(cmd.noun.clone()),
        });

        // Verb
        triples.push(RdfTriple {
            subject: cmd.uri.clone(),
            predicate: format!("{}verb", CNV_NAMESPACE),
            object: RdfValue::literal(cmd.verb.clone()),
        });

        // Parameters
        for (idx, param) in cmd.parameters.iter().enumerate() {
            let param_uri = format!("{}_param_{}", cmd.uri, idx);

            triples.push(RdfTriple {
                subject: cmd.uri.clone(),
                predicate: format!("{}hasParameter", CNV_NAMESPACE),
                object: RdfValue::uri(param_uri.clone()),
            });

            triples.push(RdfTriple {
                subject: param_uri.clone(),
                predicate: format!("{}paramName", CNV_NAMESPACE),
                object: RdfValue::literal(param.name.clone()),
            });

            triples.push(RdfTriple {
                subject: param_uri.clone(),
                predicate: format!("{}paramType", CNV_NAMESPACE),
                object: RdfValue::literal(param.param_type.clone()),
            });

            triples.push(RdfTriple {
                subject: param_uri,
                predicate: format!("{}required", CNV_NAMESPACE),
                object: RdfValue::literal(param.required.to_string()),
            });
        }

        // Guards
        for guard in &cmd.guards {
            triples.push(RdfTriple {
                subject: cmd.uri.clone(),
                predicate: format!("{}requiresGuard", CNV_NAMESPACE),
                object: RdfValue::literal(guard.clone()),
            });
        }

        // Effects
        for effect in &cmd.effects {
            triples.push(RdfTriple {
                subject: cmd.uri.clone(),
                predicate: format!("{}hasEffect", CNV_NAMESPACE),
                object: RdfValue::literal(effect.clone()),
            });
        }
    }

    triples
}

/// Demonstrate SPARQL query patterns
fn demonstrate_sparql_queries(triples: &[RdfTriple]) {
    println!("Query 1: Find all commands with 'state-change' effect");
    let state_change_cmds = find_commands_by_effect(triples, "state-change");
    for cmd in state_change_cmds {
        println!("  - {}", cmd);
    }

    println!("\nQuery 2: Find all 'services' noun commands");
    let services_cmds = find_commands_by_noun(triples, "services");
    for cmd in services_cmds {
        println!("  - {}", cmd);
    }

    println!("\nQuery 3: Find commands requiring 'authorized' guard");
    let auth_cmds = find_commands_by_guard(triples, "authorized");
    for cmd in auth_cmds {
        println!("  - {}", cmd);
    }
}

fn find_commands_by_effect(triples: &[RdfTriple], effect: &str) -> Vec<String> {
    triples
        .iter()
        .filter(|t| {
            t.predicate.contains("hasEffect") && t.object.as_str() == effect
        })
        .map(|t| t.subject.clone())
        .collect()
}

fn find_commands_by_noun(triples: &[RdfTriple], noun: &str) -> Vec<String> {
    triples
        .iter()
        .filter(|t| {
            t.predicate.contains("noun") && t.object.as_str() == noun
        })
        .map(|t| t.subject.clone())
        .collect()
}

fn find_commands_by_guard(triples: &[RdfTriple], guard: &str) -> Vec<String> {
    triples
        .iter()
        .filter(|t| {
            t.predicate.contains("requiresGuard") && t.object.as_str() == guard
        })
        .map(|t| t.subject.clone())
        .collect()
}

/// Analyze command relationships
fn analyze_command_relationships(ontology: &CliOntology) {
    // Group by noun
    let mut noun_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for cmd in &ontology.commands {
        noun_groups
            .entry(cmd.noun.clone())
            .or_default()
            .push(cmd.verb.clone());
    }

    println!("Command groups by noun:");
    for (noun, verbs) in noun_groups {
        println!("  {}: {:?}", noun, verbs);
    }

    // Analyze guard patterns
    let mut guard_usage: BTreeMap<String, usize> = BTreeMap::new();
    for cmd in &ontology.commands {
        for guard in &cmd.guards {
            *guard_usage.entry(guard.clone()).or_insert(0) += 1;
        }
    }

    println!("\nGuard usage frequency:");
    for (guard, count) in guard_usage {
        println!("  {}: {} commands", guard, count);
    }
}

/// Demonstrate SHACL validation
fn demonstrate_shacl_validation(ontology: &CliOntology) {
    println!("SHACL Shape: Command must have:");
    println!("  - rdf:type = cnv:Command");
    println!("  - cnv:noun (required, string)");
    println!("  - cnv:verb (required, string)");
    println!("  - rdfs:description (required, string)");

    let mut valid_count = 0;
    for cmd in &ontology.commands {
        let is_valid = !cmd.noun.is_empty()
            && !cmd.verb.is_empty()
            && !cmd.description.is_empty();

        if is_valid {
            valid_count += 1;
        }
    }

    println!("\n✅ Validation: {}/{} commands conform to shape",
             valid_count, ontology.commands.len());
}

/// Demonstrate MCP integration
fn demonstrate_mcp_integration(ontology: &CliOntology) {
    println!("MCP Memory Keys (conceptual):");
    println!("  - swarm/rdf/ontology/commands → {} commands", ontology.commands.len());
    println!("  - swarm/rdf/ontology/nouns → [services, config]");
    println!("  - swarm/rdf/ontology/verbs → [list, start, stop, get, set]");
    println!("  - swarm/rdf/ontology/guards → [authenticated, authorized]");
    println!("  - swarm/rdf/ontology/effects → [read-only, state-change, idempotent]");

    println!("\n💡 Use mcp__claude-flow__memory_usage to store/retrieve:");
    println!("  action: store");
    println!("  key: swarm/rdf/ontology/commands");
    println!("  value: <serialized ontology>");
    println!("  namespace: clap-noun-verb-playground");
}
