//! RDF + Oxigraph SPARQL Playground
//!
//! Demonstrates real SPARQL queries using oxigraph for semantic CLI exploration
//!
//! Run with: cargo run --example rdf_oxigraph_sparql

use clap_noun_verb::rdf::{CNV_NAMESPACE, RDFS_NS, RDF_NS};
use oxigraph::model::*;
use oxigraph::store::Store;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔬 RDF + Oxigraph SPARQL Playground\n");

    // Create in-memory RDF store
    let store = Store::new()?;

    println!("📚 Part 1: Loading CLI Ontology into Oxigraph");
    load_cli_ontology(&store)?;
    println!("✅ Loaded CLI command ontology\n");

    // SPARQL Query 1: Find all commands
    println!("🔍 Query 1: Count all commands");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT (COUNT(?cmd) AS ?count)
        WHERE {
            ?cmd rdf:type cnv:Command .
        }
    "#;
    execute_sparql(&store, query, "Count");

    // SPARQL Query 2: Find commands by effect
    println!("\n🔍 Query 2: Commands with state-change effect");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?cmd ?label ?effect
        WHERE {
            ?cmd rdfs:label ?label .
            ?cmd cnv:hasEffect ?effect .
            FILTER (?effect = "state-change")
        }
        ORDER BY ?label
    "#;
    execute_sparql(&store, query, "State-change commands");

    // SPARQL Query 3: Command parameter analysis
    println!("\n🔍 Query 3: Commands with required parameters");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?cmd ?label ?paramName
        WHERE {
            ?cmd rdfs:label ?label .
            ?cmd cnv:hasParameter ?param .
            ?param cnv:paramName ?paramName .
            ?param cnv:required "true" .
        }
        ORDER BY ?label ?paramName
    "#;
    execute_sparql(&store, query, "Required parameters");

    // SPARQL Query 4: Guard analysis
    println!("\n🔍 Query 4: Commands grouped by guards");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?guard (COUNT(?cmd) AS ?count)
        WHERE {
            ?cmd cnv:requiresGuard ?guard .
        }
        GROUP BY ?guard
        ORDER BY DESC(?count)
    "#;
    execute_sparql(&store, query, "Guard usage");

    // SPARQL Query 5: Noun-verb relationships
    println!("\n🔍 Query 5: Noun-verb command matrix");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>

        SELECT ?noun ?verb (COUNT(?cmd) AS ?count)
        WHERE {
            ?cmd cnv:noun ?noun .
            ?cmd cnv:verb ?verb .
        }
        GROUP BY ?noun ?verb
        ORDER BY ?noun ?verb
    "#;
    execute_sparql(&store, query, "Noun-verb matrix");

    // SPARQL Query 6: Complex analysis - idempotent state changes
    println!("\n🔍 Query 6: Idempotent state-changing commands");
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?label ?noun ?verb
        WHERE {
            ?cmd rdfs:label ?label .
            ?cmd cnv:noun ?noun .
            ?cmd cnv:verb ?verb .
            ?cmd cnv:hasEffect "state-change" .
            ?cmd cnv:hasEffect "idempotent" .
        }
        ORDER BY ?label
    "#;
    execute_sparql(&store, query, "Idempotent state changes");

    println!("\n🎉 Oxigraph SPARQL Playground Complete!");
    println!("\n💡 Key Insights:");
    println!("  • Oxigraph provides full SPARQL 1.1 query support");
    println!("  • RDF graphs enable semantic reasoning over CLI structure");
    println!("  • SPARQL aggregations reveal patterns (GROUP BY, COUNT)");
    println!("  • Complex queries discover command relationships");
    println!("  • MCP coordination can distribute SPARQL across swarms");

    Ok(())
}

fn load_cli_ontology(store: &Store) -> Result<(), Box<dyn Error>> {
    // Define commands as RDF triples
    let commands = vec![
        (
            "services-list",
            "services",
            "list",
            "List all available services",
            vec![("filter", "string", false), ("verbose", "boolean", false)],
            vec!["authenticated"],
            vec!["read-only"],
        ),
        (
            "services-start",
            "services",
            "start",
            "Start a service",
            vec![("name", "string", true)],
            vec!["authenticated", "authorized"],
            vec!["state-change", "idempotent"],
        ),
        (
            "services-stop",
            "services",
            "stop",
            "Stop a service",
            vec![("name", "string", true), ("force", "boolean", false)],
            vec!["authenticated", "authorized"],
            vec!["state-change"],
        ),
        (
            "config-get",
            "config",
            "get",
            "Get configuration value",
            vec![("key", "string", true)],
            vec!["authenticated"],
            vec!["read-only"],
        ),
        (
            "config-set",
            "config",
            "set",
            "Set configuration value",
            vec![("key", "string", true), ("value", "string", true)],
            vec!["authenticated", "authorized"],
            vec!["state-change", "idempotent"],
        ),
    ];

    // Load triples into store
    for (cmd_id, noun, verb, desc, params, guards, effects) in commands {
        let cmd_iri = NamedNode::new(format!("{}{}", CNV_NAMESPACE, cmd_id))?;
        let rdf_type = NamedNode::new(format!("{}type", RDF_NS))?;
        let cmd_type = NamedNode::new(format!("{}Command", CNV_NAMESPACE))?;

        // Command type
        store.insert(&Quad::new(cmd_iri.clone(), rdf_type, cmd_type, GraphName::DefaultGraph))?;

        // Label
        let label_pred = NamedNode::new(format!("{}label", RDFS_NS))?;
        store.insert(&Quad::new(
            cmd_iri.clone(),
            label_pred,
            Literal::new_simple_literal(format!("{} {}", noun, verb)),
            GraphName::DefaultGraph,
        ))?;

        // Description
        let desc_pred = NamedNode::new(format!("{}description", RDFS_NS))?;
        store.insert(&Quad::new(
            cmd_iri.clone(),
            desc_pred,
            Literal::new_simple_literal(desc),
            GraphName::DefaultGraph,
        ))?;

        // Noun
        let noun_pred = NamedNode::new(format!("{}noun", CNV_NAMESPACE))?;
        store.insert(&Quad::new(
            cmd_iri.clone(),
            noun_pred.clone(),
            Literal::new_simple_literal(noun),
            GraphName::DefaultGraph,
        ))?;

        // Verb
        let verb_pred = NamedNode::new(format!("{}verb", CNV_NAMESPACE))?;
        store.insert(&Quad::new(
            cmd_iri.clone(),
            verb_pred,
            Literal::new_simple_literal(verb),
            GraphName::DefaultGraph,
        ))?;

        // Parameters
        let has_param_pred = NamedNode::new(format!("{}hasParameter", CNV_NAMESPACE))?;
        let param_name_pred = NamedNode::new(format!("{}paramName", CNV_NAMESPACE))?;
        let param_type_pred = NamedNode::new(format!("{}paramType", CNV_NAMESPACE))?;
        let required_pred = NamedNode::new(format!("{}required", CNV_NAMESPACE))?;

        for (idx, (pname, ptype, required)) in params.iter().enumerate() {
            let param_iri = NamedNode::new(format!("{}{}_param_{}", CNV_NAMESPACE, cmd_id, idx))?;

            store.insert(&Quad::new(
                cmd_iri.clone(),
                has_param_pred.clone(),
                param_iri.clone(),
                GraphName::DefaultGraph,
            ))?;

            store.insert(&Quad::new(
                param_iri.clone(),
                param_name_pred.clone(),
                Literal::new_simple_literal(*pname),
                GraphName::DefaultGraph,
            ))?;

            store.insert(&Quad::new(
                param_iri.clone(),
                param_type_pred.clone(),
                Literal::new_simple_literal(*ptype),
                GraphName::DefaultGraph,
            ))?;

            store.insert(&Quad::new(
                param_iri,
                required_pred.clone(),
                Literal::new_simple_literal(required.to_string()),
                GraphName::DefaultGraph,
            ))?;
        }

        // Guards
        let guard_pred = NamedNode::new(format!("{}requiresGuard", CNV_NAMESPACE))?;
        for guard in guards {
            store.insert(&Quad::new(
                cmd_iri.clone(),
                guard_pred.clone(),
                Literal::new_simple_literal(guard),
                GraphName::DefaultGraph,
            ))?;
        }

        // Effects
        let effect_pred = NamedNode::new(format!("{}hasEffect", CNV_NAMESPACE))?;
        for effect in effects {
            store.insert(&Quad::new(
                cmd_iri.clone(),
                effect_pred.clone(),
                Literal::new_simple_literal(effect),
                GraphName::DefaultGraph,
            ))?;
        }
    }

    Ok(())
}

fn execute_sparql(store: &Store, query: &str, title: &str) {
    println!("📊 {}", title);

    match store.query(query) {
        Ok(results) => {
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                let mut count = 0;
                for solution in solutions {
                    match solution {
                        Ok(sol) => {
                            let vars: Vec<_> = sol.variables().into_iter().collect();
                            let values: Vec<_> = vars
                                .iter()
                                .filter_map(|v| sol.get(*v))
                                .map(|t| format!("{}", t))
                                .collect();
                            println!("  {}", values.join(" | "));
                            count += 1;
                        }
                        Err(e) => println!("  Error: {}", e),
                    }
                }
                println!("  (Total: {} results)", count);
            }
        }
        Err(e) => println!("  Query error: {}", e),
    }
}
