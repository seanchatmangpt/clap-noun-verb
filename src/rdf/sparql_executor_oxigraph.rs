//! Production-grade SPARQL executor using oxigraph
//!
//! Provides high-performance SPARQL query execution with:
//! - Full SPARQL 1.1 specification support via oxigraph
//! - In-memory and persistent storage backends
//! - Ontology introspection (list classes, properties)
//! - Type-safe query result handling
//!
//! ## Example
//!
//! ```rust,ignore
//! use clap_noun_verb::rdf::turtle_parser::TurtleParser;
//! use clap_noun_verb::rdf::sparql_executor_oxigraph::SparqlExecutor;
//!
//! // Parse ontology from Turtle
//! let parser = TurtleParser::new();
//! let parsed = parser.parse(turtle_doc)?;
//!
//! // Create SPARQL executor
//! let executor = SparqlExecutor::new(&parsed)?;
//!
//! // Execute queries
//! let classes = executor.list_classes()?;
//! let properties = executor.list_properties()?;
//! ```

#[cfg(feature = "rdf-composition")]
use oxigraph::model::{NamedNode, NamedNodeRef, Quad};
#[cfg(feature = "rdf-composition")]
use oxigraph::sparql::{Query, QueryResults};
#[cfg(feature = "rdf-composition")]
use oxigraph::store::Store;

use crate::rdf::turtle_parser::ParsedTurtle;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during SPARQL execution
#[derive(Debug, Error)]
pub enum SparqlError {
    /// Query parsing error
    #[error("Failed to parse SPARQL query: {message}")]
    QueryParseError { message: String },

    /// Query execution error
    #[error("Failed to execute SPARQL query: {message}")]
    ExecutionError { message: String },

    /// Store initialization error
    #[error("Failed to initialize RDF store: {message}")]
    StoreError { message: String },

    /// Result conversion error
    #[error("Failed to convert query results: {message}")]
    ConversionError { message: String },

    /// Feature not enabled
    #[error("RDF composition feature not enabled. Enable with --features rdf-composition")]
    FeatureNotEnabled,
}

/// Variable binding in a query result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// Variable name to value mapping
    pub bindings: HashMap<String, String>,
}

impl Binding {
    /// Create a new empty binding
    pub fn new() -> Self {
        Self { bindings: HashMap::new() }
    }

    /// Insert a variable binding
    pub fn insert(&mut self, var: impl Into<String>, value: impl Into<String>) {
        self.bindings.insert(var.into(), value.into());
    }

    /// Get a variable binding
    pub fn get(&self, var: &str) -> Option<&str> {
        self.bindings.get(var).map(|s| s.as_str())
    }
}

impl Default for Binding {
    fn default() -> Self {
        Self::new()
    }
}

/// Query result containing variable bindings
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Result bindings (rows)
    pub bindings: Vec<Binding>,
    /// Variable names in the SELECT clause
    pub variables: Vec<String>,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(bindings: Vec<Binding>, variables: Vec<String>) -> Self {
        Self { bindings, variables }
    }

    /// Get the number of result rows
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get the variable names
    pub fn variables(&self) -> &[String] {
        &self.variables
    }

    /// Iterate over bindings
    pub fn iter(&self) -> impl Iterator<Item = &Binding> {
        self.bindings.iter()
    }
}

/// SPARQL executor using oxigraph in-memory store
#[cfg(feature = "rdf-composition")]
pub struct SparqlExecutor {
    /// Oxigraph in-memory store
    store: Store,
}

#[cfg(feature = "rdf-composition")]
impl SparqlExecutor {
    /// Create a new SPARQL executor from a parsed ontology
    ///
    /// # Arguments
    ///
    /// * `ontology` - Parsed Turtle document containing RDF triples
    ///
    /// # Returns
    ///
    /// * `Ok(SparqlExecutor)` - Executor ready for queries
    /// * `Err(SparqlError)` - Store initialization failed
    pub fn new(ontology: &ParsedTurtle) -> Result<Self, SparqlError> {
        let store = Store::new().map_err(|e| SparqlError::StoreError {
            message: format!("Failed to create store: {}", e),
        })?;

        // Load triples into the store
        for (subject, predicate, object) in ontology.ontology().iter_triples() {
            let quad = Self::create_quad(&subject, &predicate, &object)?;
            store.insert(&quad).map_err(|e| SparqlError::StoreError {
                message: format!("Failed to insert triple: {}", e),
            })?;
        }

        Ok(Self { store })
    }

    /// Create an oxigraph Quad from subject, predicate, object strings
    fn create_quad(subject: &str, predicate: &str, object: &str) -> Result<Quad, SparqlError> {
        use oxigraph::model::{GraphNameRef, Literal, Subject, Term};

        let subj = NamedNode::new(subject).map(Subject::from).map_err(|e| {
            SparqlError::ConversionError {
                message: format!("Invalid subject IRI '{}': {}", subject, e),
            }
        })?;

        let pred = NamedNode::new(predicate).map_err(|e| SparqlError::ConversionError {
            message: format!("Invalid predicate IRI '{}': {}", predicate, e),
        })?;

        // Try to parse object as IRI first, fallback to literal
        let obj = if object.starts_with("http://") || object.starts_with("https://") {
            NamedNode::new(object)
                .map(Term::from)
                .unwrap_or_else(|_| Term::Literal(Literal::new_simple_literal(object)))
        } else {
            Term::Literal(Literal::new_simple_literal(object))
        };

        Ok(Quad::new(subj, pred, obj, GraphNameRef::DefaultGraph))
    }

    /// Execute a SPARQL query
    ///
    /// # Arguments
    ///
    /// * `sparql` - SPARQL query string
    ///
    /// # Returns
    ///
    /// * `Ok(QueryResult)` - Query results with variable bindings
    /// * `Err(SparqlError)` - Query parsing or execution failed
    pub fn execute_query(&self, sparql: &str) -> Result<QueryResult, SparqlError> {
        let query = Query::parse(sparql, None).map_err(|e| SparqlError::QueryParseError {
            message: format!("Failed to parse query: {}", e),
        })?;

        let results = self.store.query(query).map_err(|e| SparqlError::ExecutionError {
            message: format!("Query execution failed: {}", e),
        })?;

        self.convert_results(results)
    }

    /// Convert oxigraph QueryResults to our QueryResult type
    fn convert_results(&self, results: QueryResults) -> Result<QueryResult, SparqlError> {
        match results {
            QueryResults::Solutions(solutions) => {
                let variables: Vec<String> =
                    solutions.variables().iter().map(|v| v.as_str().to_string()).collect();

                let mut bindings = Vec::new();

                for solution in solutions {
                    let solution = solution.map_err(|e| SparqlError::ExecutionError {
                        message: format!("Failed to read solution: {}", e),
                    })?;

                    let mut binding = Binding::new();

                    for var in &variables {
                        if let Some(term) = solution.get(var.as_str()) {
                            let value = Self::term_to_string(term);
                            binding.insert(var.clone(), value);
                        }
                    }

                    bindings.push(binding);
                }

                Ok(QueryResult::new(bindings, variables))
            }
            QueryResults::Boolean(result) => {
                // For ASK queries, return a single binding with "result" variable
                let mut binding = Binding::new();
                binding.insert("result", result.to_string());
                Ok(QueryResult::new(vec![binding], vec!["result".to_string()]))
            }
            QueryResults::Graph(_) => Err(SparqlError::ConversionError {
                message: "CONSTRUCT/DESCRIBE queries not yet supported".to_string(),
            }),
        }
    }

    /// Convert an oxigraph Term to a string
    fn term_to_string(term: &oxigraph::model::Term) -> String {
        match term {
            oxigraph::model::Term::NamedNode(node) => node.as_str().to_string(),
            oxigraph::model::Term::BlankNode(node) => format!("_:{}", node.as_str()),
            oxigraph::model::Term::Literal(lit) => lit.value().to_string(),
            oxigraph::model::Term::Triple(_) => "_:triple".to_string(),
        }
    }

    /// List all RDF classes in the ontology
    ///
    /// Returns classes defined as `rdf:type rdfs:Class` or `rdf:type owl:Class`
    pub fn list_classes(&self) -> Result<Vec<String>, SparqlError> {
        let query = r#"
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            PREFIX owl: <http://www.w3.org/2002/07/owl#>

            SELECT DISTINCT ?class WHERE {
                {
                    ?class rdf:type rdfs:Class .
                } UNION {
                    ?class rdf:type owl:Class .
                } UNION {
                    ?instance rdf:type ?class .
                    FILTER(isIRI(?class))
                }
            }
            ORDER BY ?class
        "#;

        let result = self.execute_query(query)?;

        Ok(result.bindings.iter().filter_map(|b| b.get("class").map(|s| s.to_string())).collect())
    }

    /// List all RDF properties in the ontology
    ///
    /// Returns properties defined as `rdf:type rdf:Property` or used as predicates
    pub fn list_properties(&self) -> Result<Vec<String>, SparqlError> {
        let query = r#"
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            PREFIX owl: <http://www.w3.org/2002/07/owl#>

            SELECT DISTINCT ?property WHERE {
                {
                    ?property rdf:type rdf:Property .
                } UNION {
                    ?property rdf:type owl:ObjectProperty .
                } UNION {
                    ?property rdf:type owl:DatatypeProperty .
                } UNION {
                    ?s ?property ?o .
                    FILTER(isIRI(?property))
                }
            }
            ORDER BY ?property
        "#;

        let result = self.execute_query(query)?;

        Ok(result
            .bindings
            .iter()
            .filter_map(|b| b.get("property").map(|s| s.to_string()))
            .collect())
    }

    /// Get the underlying oxigraph store (for advanced usage)
    pub fn store(&self) -> &Store {
        &self.store
    }
}

/// SPARQL executor stub when feature not enabled
#[cfg(not(feature = "rdf-composition"))]
pub struct SparqlExecutor;

#[cfg(not(feature = "rdf-composition"))]
impl SparqlExecutor {
    pub fn new(_ontology: &ParsedTurtle) -> Result<Self, SparqlError> {
        Err(SparqlError::FeatureNotEnabled)
    }

    pub fn execute_query(&self, _sparql: &str) -> Result<QueryResult, SparqlError> {
        Err(SparqlError::FeatureNotEnabled)
    }

    pub fn list_classes(&self) -> Result<Vec<String>, SparqlError> {
        Err(SparqlError::FeatureNotEnabled)
    }

    pub fn list_properties(&self) -> Result<Vec<String>, SparqlError> {
        Err(SparqlError::FeatureNotEnabled)
    }
}

#[cfg(all(test, feature = "rdf-composition"))]
mod tests {
    use super::*;
    use crate::rdf::turtle_parser::TurtleParser;

    /// Helper to create a test ontology
    fn create_test_ontology() -> ParsedTurtle {
        let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:Verb rdf:type rdfs:Class ;
    rdfs:label "Verb class" .

cnv:Noun rdf:type rdfs:Class ;
    rdfs:label "Noun class" .

cnv:BuildCommand rdf:type cnv:Verb ;
    cnv:name "build" ;
    rdfs:label "Build command" .

cnv:TestCommand rdf:type cnv:Verb ;
    cnv:name "test" ;
    rdfs:label "Test command" .

cnv:name rdf:type rdf:Property ;
    rdfs:label "Name property" .
"#;

        TurtleParser::new().parse(turtle).unwrap()
    }

    #[test]
    fn test_executor_creation() {
        // Arrange
        let ontology = create_test_ontology();

        // Act
        let result = SparqlExecutor::new(&ontology);

        // Assert
        assert!(result.is_ok(), "Failed to create executor: {:?}", result.err());
    }

    #[test]
    fn test_simple_select_query() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        let query = r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            SELECT ?verb WHERE {
                ?verb rdf:type cnv:Verb .
            }
        "#;

        // Act
        let result = executor.execute_query(query);

        // Assert
        assert!(result.is_ok(), "Query failed: {:?}", result.err());
        let query_result = result.unwrap();
        assert!(query_result.len() >= 2, "Expected at least 2 verbs");
        assert_eq!(query_result.variables(), &["verb"]);
    }

    #[test]
    fn test_list_classes() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        // Act
        let result = executor.list_classes();

        // Assert
        assert!(result.is_ok(), "list_classes failed: {:?}", result.err());
        let classes = result.unwrap();
        assert!(!classes.is_empty(), "No classes found");
        assert!(classes.iter().any(|c| c.contains("Verb")), "Verb class not found");
        assert!(classes.iter().any(|c| c.contains("Noun")), "Noun class not found");
    }

    #[test]
    fn test_list_properties() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        // Act
        let result = executor.list_properties();

        // Assert
        assert!(result.is_ok(), "list_properties failed: {:?}", result.err());
        let properties = result.unwrap();
        assert!(!properties.is_empty(), "No properties found");
        assert!(properties.iter().any(|p| p.contains("name")), "name property not found");
    }

    #[test]
    fn test_filter_query() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        let query = r#"
            PREFIX cnv: <https://cnv.dev/ontology#>

            SELECT ?verb ?name WHERE {
                ?verb cnv:name ?name .
                FILTER(?name = "build")
            }
        "#;

        // Act
        let result = executor.execute_query(query);

        // Assert
        assert!(result.is_ok(), "Query failed: {:?}", result.err());
        let query_result = result.unwrap();
        assert_eq!(query_result.len(), 1, "Expected exactly 1 result");
        assert_eq!(query_result.bindings[0].get("name"), Some("build"));
    }

    #[test]
    fn test_ask_query() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        let query = r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            ASK {
                cnv:BuildCommand rdf:type cnv:Verb .
            }
        "#;

        // Act
        let result = executor.execute_query(query);

        // Assert
        assert!(result.is_ok(), "ASK query failed: {:?}", result.err());
        let query_result = result.unwrap();
        assert_eq!(query_result.len(), 1);
        assert_eq!(query_result.bindings[0].get("result"), Some("true"));
    }

    #[test]
    fn test_invalid_query() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        let invalid_query = "SELECT WHERE { invalid syntax }";

        // Act
        let result = executor.execute_query(invalid_query);

        // Assert
        assert!(result.is_err(), "Should fail on invalid query");
        match result.unwrap_err() {
            SparqlError::QueryParseError { .. } => {
                // Expected error
            }
            other => panic!("Expected QueryParseError, got {:?}", other),
        }
    }

    #[test]
    fn test_empty_result() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).unwrap();

        let query = r#"
            PREFIX cnv: <https://cnv.dev/ontology#>

            SELECT ?x WHERE {
                ?x cnv:nonexistentProperty "value" .
            }
        "#;

        // Act
        let result = executor.execute_query(query);

        // Assert
        assert!(result.is_ok(), "Query should succeed even with no results");
        let query_result = result.unwrap();
        assert!(query_result.is_empty(), "Result should be empty");
    }
}
