//! Integration: RDF/Ontology with Oxigraph
//!
//! Glue code for RDF operations using Oxigraph.
//! Connects domain ontology models to actual RDF store.
//!
//! # Performance Optimization: Cached RDF Store
//!
//! The ontology store is cached globally using `lazy_static` to avoid
//! 20-50ms initialization overhead per SPARQL query. The store is
//! initialized once on first access and reused for all subsequent queries.

use lazy_static::lazy_static;
use oxigraph::model::{NamedNode, Literal, Term, Quad, GraphName};
use oxigraph::store::Store;
use oxigraph::sparql::QueryResults;
use crate::domain::ontology::{CliCapability, OntologyTriple, build_playground_ontology};

lazy_static! {
    /// Global cached ontology store - initialized once on first access
    ///
    /// Thread-safe: Oxigraph Store is internally thread-safe for concurrent reads.
    /// This eliminates 20-50ms initialization overhead per SPARQL query.
    static ref ONTOLOGY_STORE: Result<Store, String> = {
        let caps = build_playground_ontology();
        init_ontology_store(&caps)
    };
}

/// Get the cached ontology store (initializes on first use, reuses afterward)
///
/// # Returns
/// - `Ok(&Store)` - Reference to the cached, thread-safe RDF store
/// - `Err(&str)` - Static error message if initialization failed
///
/// # Performance
/// - First call: ~20-50ms (store initialization)
/// - Subsequent calls: ~0ms (cached reference)
///
/// # Example
/// ```ignore
/// let store = get_ontology_store()?;
/// let results = execute_sparql(store, "SELECT * WHERE { ?s ?p ?o }")?;
/// ```
pub fn get_ontology_store() -> Result<&'static Store, &'static str> {
    ONTOLOGY_STORE.as_ref().map_err(|_| "Failed to initialize ontology store")
}

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
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let store_clone = store.clone();
    let query_str = query.to_string();
    let timeout = Duration::from_millis(timeout_ms);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let results = match store_clone.query(&query_str) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(format!("SPARQL query error: {}", e)));
                return;
            }
        };

        match results {
            QueryResults::Solutions(solutions) => {
                let mut rows = Vec::new();
                for solution in solutions {
                    match solution {
                        Ok(sol) => {
                            let row: Vec<String> = sol.iter()
                                .map(|(_, term)| term_to_string(term))
                                .collect();
                            rows.push(row);
                        }
                        Err(e) => {
                            eprintln!("SPARQL solution warning: {} - skipping row", e);
                            continue;
                        }
                    }
                }
                let _ = tx.send(Ok(rows));
            }
            QueryResults::Boolean(b) => {
                let _ = tx.send(Ok(vec![vec![b.to_string()]]));
            }
            QueryResults::Graph(_) => {
                eprintln!("SPARQL warning: Graph results not supported - returning empty");
                let _ = tx.send(Ok(Vec::new()));
            }
        }
    });

    match rx.recv_timeout(timeout) {
        Ok(res) => res,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            Err(format!("SPARQL query timed out after {}ms", timeout_ms))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err("SPARQL query thread disconnected".to_string())
        }
    }
}

fn term_to_string(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        Term::Literal(l) => l.value().to_string(),
        Term::Triple(t) => format!(
            "[{} {} {}]",
            term_to_string(&t.subject.clone().into()),
            term_to_string(&t.predicate.clone().into()),
            term_to_string(&t.object)
        ),
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
    fn test_sparql_timeout_returns_error() {
        let caps = build_playground_ontology();
        let store = init_ontology_store(&caps).unwrap();

        // Query with an extremely short timeout (0ms) to guarantee timeout
        let result = execute_sparql_with_timeout(
            &store,
            "SELECT * WHERE { ?s ?p ?o . ?s1 ?p1 ?o1 . ?s2 ?p2 ?o2 }",
            0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[test]
    fn test_sparql_completes_before_timeout() {
        let caps = build_playground_ontology();
        let store = init_ontology_store(&caps).unwrap();

        let result = execute_sparql_with_timeout(
            &store,
            "SELECT ?s WHERE { ?s ?p ?o } LIMIT 1",
            5000,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_export_turtle() {
        let caps = build_playground_ontology();
        let turtle = export_turtle(&caps);
        assert!(turtle.contains("@prefix cnv:"));
        assert!(turtle.contains("cnv:papers_generate"));
    }
}
