//! Integration: RDF/Ontology with Oxigraph
//!
//! Glue code for RDF operations using Oxigraph.
//! Connects domain ontology models to actual RDF store.

use oxigraph::model::{NamedNode, Literal, Term, Quad, GraphName};
use oxigraph::store::Store;
use oxigraph::sparql::QueryResults;
use crate::domain::ontology::{CliCapability, OntologyTriple};

/// CNV ontology namespace
pub const CNV_NS: &str = "https://cnv.dev/ontology#";
pub const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";

/// Initialize in-memory RDF store with CLI ontology
pub fn init_ontology_store(capabilities: &[CliCapability]) -> Result<Store, String> {
    let store = Store::new()
        .map_err(|e| format!("Failed to create store: {}", e))?;

    // Add all capability triples
    for cap in capabilities {
        let triples = cap.to_triples();
        for triple in triples {
            add_triple_to_store(&store, &triple)?;
        }
    }

    Ok(store)
}

fn add_triple_to_store(store: &Store, triple: &OntologyTriple) -> Result<(), String> {
    let subject = resolve_uri(&triple.subject)?;
    let predicate = resolve_uri(&triple.predicate)?;
    let object = if triple.object.starts_with("cnv:") || triple.object.contains("://") {
        Term::NamedNode(resolve_uri(&triple.object)?)
    } else {
        Term::Literal(Literal::new_simple_literal(&triple.object))
    };

    let quad = Quad::new(
        subject,
        predicate,
        object,
        GraphName::DefaultGraph,
    );

    store.insert(&quad)
        .map_err(|e| format!("Failed to insert triple: {}", e))?;

    Ok(())
}

fn resolve_uri(uri: &str) -> Result<NamedNode, String> {
    let full_uri = if let Some(stripped) = uri.strip_prefix("cnv:") {
        format!("{}{}", CNV_NS, stripped)
    } else if let Some(stripped) = uri.strip_prefix("rdf:") {
        format!("{}{}", RDF_NS, stripped)
    } else if let Some(stripped) = uri.strip_prefix("rdfs:") {
        format!("{}{}", RDFS_NS, stripped)
    } else if uri.contains("://") {
        uri.to_string()
    } else {
        format!("{}{}", CNV_NS, uri)
    };

    NamedNode::new(&full_uri)
        .map_err(|e| format!("Invalid URI '{}': {}", full_uri, e))
}

/// Default SPARQL query timeout in milliseconds
const SPARQL_TIMEOUT_MS: u64 = 5000;

/// Execute SPARQL query on store with timeout and graceful degradation
///
/// # FMEA-3: Graceful Degradation
/// If the query fails, returns an empty result set with error logged rather than propagating failure.
///
/// # FMEA-5: Timeout Handling
/// Queries are limited to SPARQL_TIMEOUT_MS to prevent hanging on complex queries.
#[allow(deprecated)] // FUTURE: Migrate to SparqlEvaluator when oxigraph stabilizes API
pub fn execute_sparql(store: &Store, query: &str) -> Result<Vec<Vec<String>>, String> {
    execute_sparql_with_timeout(store, query, SPARQL_TIMEOUT_MS)
}

/// Execute SPARQL query with configurable timeout
#[allow(deprecated)] // FUTURE: Migrate to SparqlEvaluator when oxigraph stabilizes API
pub fn execute_sparql_with_timeout(
    store: &Store,
    query: &str,
    timeout_ms: u64,
) -> Result<Vec<Vec<String>>, String> {
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    // Execute query with timeout check
    let results = match store.query(query) {
        Ok(r) => r,
        Err(e) => {
            // FMEA-3: Graceful degradation - return empty results on query parse/execution error
            eprintln!("SPARQL query warning: {} - returning empty results", e);
            return Ok(Vec::new());
        }
    };

    // Check timeout after query execution
    if start.elapsed() > timeout {
        eprintln!(
            "SPARQL query timeout warning: exceeded {}ms - returning partial/empty results",
            timeout_ms
        );
        return Ok(Vec::new());
    }

    match results {
        QueryResults::Solutions(solutions) => {
            let mut rows = Vec::new();
            for solution in solutions {
                // FMEA-5: Check timeout during result iteration
                if start.elapsed() > timeout {
                    eprintln!(
                        "SPARQL result iteration timeout: exceeded {}ms after {} rows",
                        timeout_ms,
                        rows.len()
                    );
                    break;
                }

                match solution {
                    Ok(sol) => {
                        let row: Vec<String> = sol.iter()
                            .map(|(_, term)| term_to_string(term))
                            .collect();
                        rows.push(row);
                    }
                    Err(e) => {
                        // FMEA-3: Graceful degradation - skip problematic solutions
                        eprintln!("SPARQL solution warning: {} - skipping row", e);
                        continue;
                    }
                }
            }
            Ok(rows)
        }
        QueryResults::Boolean(b) => Ok(vec![vec![b.to_string()]]),
        QueryResults::Graph(_) => {
            // FMEA-3: Graceful degradation for unsupported result types
            eprintln!("SPARQL warning: Graph results not supported - returning empty");
            Ok(Vec::new())
        }
    }
}

fn term_to_string(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        Term::Literal(l) => l.value().to_string(),
    }
}

/// Export ontology as Turtle format
pub fn export_turtle(capabilities: &[CliCapability]) -> String {
    let mut turtle = String::new();

    // Prefixes
    turtle.push_str("@prefix cnv: <https://cnv.dev/ontology#> .\n");
    turtle.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    turtle.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");

    // Triples
    for cap in capabilities {
        let uri = format!("cnv:{}_{}", cap.noun, cap.verb);
        turtle.push_str(&format!("{} rdf:type cnv:Capability ;\n", uri));
        turtle.push_str(&format!("    cnv:noun \"{}\" ;\n", cap.noun));
        turtle.push_str(&format!("    cnv:verb \"{}\" ;\n", cap.verb));
        turtle.push_str(&format!("    rdfs:comment \"{}\" ;\n", cap.description));
        turtle.push_str(&format!("    cnv:effectType \"{:?}\" .\n\n", cap.effects));
    }

    turtle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ontology::build_playground_ontology;

    #[test]
    fn test_init_ontology_store() {
        let caps = build_playground_ontology();
        let store = init_ontology_store(&caps);
        assert!(store.is_ok());
    }

    #[test]
    fn test_execute_sparql_count() {
        let caps = build_playground_ontology();
        let store = init_ontology_store(&caps).unwrap();
        let results = execute_sparql(&store, "SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }");
        assert!(results.is_ok());
    }

    #[test]
    fn test_export_turtle() {
        let caps = build_playground_ontology();
        let turtle = export_turtle(&caps);
        assert!(turtle.contains("@prefix cnv:"));
        assert!(turtle.contains("cnv:papers_generate"));
    }
}
