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
    let full_uri = if uri.starts_with("cnv:") {
        format!("{}{}", CNV_NS, &uri[4..])
    } else if uri.starts_with("rdf:") {
        format!("{}{}", RDF_NS, &uri[4..])
    } else if uri.starts_with("rdfs:") {
        format!("{}{}", RDFS_NS, &uri[5..])
    } else if uri.contains("://") {
        uri.to_string()
    } else {
        format!("{}{}", CNV_NS, uri)
    };

    NamedNode::new(&full_uri)
        .map_err(|e| format!("Invalid URI '{}': {}", full_uri, e))
}

/// Execute SPARQL query on store
#[allow(deprecated)] // FUTURE: Migrate to SparqlEvaluator when oxigraph stabilizes API
pub fn execute_sparql(store: &Store, query: &str) -> Result<Vec<Vec<String>>, String> {
    let results = store.query(query)
        .map_err(|e| format!("SPARQL query error: {}", e))?;

    match results {
        QueryResults::Solutions(solutions) => {
            let mut rows = Vec::new();
            for solution in solutions {
                let solution = solution.map_err(|e| format!("Solution error: {}", e))?;
                let row: Vec<String> = solution.iter()
                    .map(|(_, term)| term_to_string(term))
                    .collect();
                rows.push(row);
            }
            Ok(rows)
        }
        QueryResults::Boolean(b) => Ok(vec![vec![b.to_string()]]),
        QueryResults::Graph(_) => Err("Graph results not supported".to_string()),
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
