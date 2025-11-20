//! SPARQL query planner for agent introspection
//!
//! NOTE: Full SPARQL parser and executor implementations are placeholders for future development.
//! For Phase 3, we use simple string-based query execution.

use crate::rdf::ontology::Ontology;
use crate::Result;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

// FUTURE: Import full SPARQL modules when implemented
// use crate::rdf::sparql_parser::{SparqlParser, ParsedQuery};
// use crate::rdf::sparql_executor::{QueryExecutor, Binding};

// Placeholder types until full implementation
#[derive(Debug, Error, Clone)]
#[error("Query error: {0}")]
pub struct QueryError(String);

#[derive(Debug, Clone)]
pub struct SparqlParser;

impl SparqlParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, _query: &str) -> std::result::Result<ParsedQuery, QueryError> {
        Ok(ParsedQuery)
    }
}

#[derive(Debug, Clone)]
pub struct ParsedQuery;

#[derive(Debug, Clone)]
pub struct QueryExecutor;

impl QueryExecutor {
    pub fn new(_ontology: Arc<Ontology>) -> Self {
        Self
    }

    pub fn execute(&self, _query: &ParsedQuery) -> std::result::Result<Vec<Binding>, QueryError> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub variables: std::collections::HashMap<String, String>,
}

impl Binding {
    pub fn new() -> Self {
        Self { variables: std::collections::HashMap::new() }
    }
}

/// LRU cache implementation
struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: Vec<K>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self { capacity, map: HashMap::new(), order: Vec::new() }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to front
            self.order.retain(|k| k != key);
            self.order.push(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }

    fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            // Remove oldest
            if let Some(oldest) = self.order.first().cloned() {
                self.order.remove(0);
                self.map.remove(&oldest);
            }
        }

        self.map.insert(key.clone(), value);
        self.order.retain(|k| k != &key);
        self.order.push(key);
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

/// SPARQL query planner
pub struct SparqlPlanner {
    ontology: Arc<Ontology>,
    cache: Arc<Mutex<LruCache<String, Vec<String>>>>,
    executor: QueryExecutor,
}

/// SPARQL query patterns
pub enum QueryPattern {
    /// List all nouns
    ListNouns,
    /// List verbs for a noun
    ListVerbs { noun: String },
    /// Find command for noun-verb pair
    FindCommand { noun: String, verb: String },
    /// Find all read-only commands
    FindReadOnlyCommands,
    /// Custom SPARQL query
    Custom(String),
}

/// SPARQL query structure
pub struct SparqlQuery {
    pub pattern: String,
    pub variables: Vec<String>,
}

/// SPARQL errors
#[derive(Debug, Error)]
pub enum SparqlError {
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
    #[error("Invalid query pattern: {0}")]
    InvalidPattern(String),
    #[error("No results found")]
    NoResults,
}

impl SparqlPlanner {
    /// Create a new SPARQL planner
    pub fn new(ontology: Arc<Ontology>) -> Self {
        let executor = QueryExecutor::new(ontology.clone());
        Self { ontology, cache: Arc::new(Mutex::new(LruCache::new(1000))), executor }
    }

    /// Execute raw SPARQL 1.1 query with full parser/optimizer/executor
    pub fn execute_raw(&self, sparql: &str) -> std::result::Result<Vec<Binding>, SparqlError> {
        let parser = SparqlParser::new();
        let parsed =
            parser.parse(sparql).map_err(|e| SparqlError::InvalidPattern(e.to_string()))?;

        self.executor.execute(&parsed).map_err(|e| SparqlError::ExecutionError(e.to_string()))
    }

    /// Discover commands by intent using SPARQL filters
    pub fn discover_by_intent(
        &self,
        intent: &str,
    ) -> std::result::Result<Vec<String>, SparqlError> {
        let query = format!(
            r#"SELECT ?cmd WHERE {{ ?cmd rdfs:comment ?desc . FILTER(CONTAINS(?desc, "{}")) }}"#,
            intent
        );

        let bindings = self.execute_raw(&query)?;
        Ok(bindings
            .into_iter()
            .filter_map(|b| b.variables.get("?cmd").map(|v| v.as_str().to_string()))
            .collect())
    }

    /// Find commands by argument names
    pub fn find_commands_by_args(
        &self,
        arg_names: &[&str],
    ) -> std::result::Result<Vec<String>, SparqlError> {
        let mut commands = Vec::new();

        for arg in arg_names {
            let query = format!(
                r#"SELECT ?cmd WHERE {{ ?cmd cnv:hasArgument ?arg . ?arg cnv:name "{}" }}"#,
                arg
            );

            let bindings = self.execute_raw(&query)?;
            commands.extend(
                bindings
                    .into_iter()
                    .filter_map(|b| b.variables.get("?cmd").map(|v| v.as_str().to_string())),
            );
        }

        commands.sort();
        commands.dedup();
        Ok(commands)
    }

    /// Get related commands using property paths
    pub fn get_related_commands(
        &self,
        command: &str,
    ) -> std::result::Result<Vec<String>, SparqlError> {
        // Find commands with same noun or verb using property paths
        let query = format!(
            r#"SELECT ?related WHERE {{
                {{ <{}> cnv:hasNoun ?noun . ?related cnv:hasNoun ?noun }}
                UNION
                {{ <{}> cnv:hasVerb ?verb . ?related cnv:hasVerb ?verb }}
            }}"#,
            command, command
        );

        let bindings = self.execute_raw(&query)?;
        Ok(bindings
            .into_iter()
            .filter_map(|b| b.variables.get("?related").map(|v| v.as_str().to_string()))
            .collect())
    }

    /// Execute a query pattern
    pub fn query(&self, pattern: QueryPattern) -> std::result::Result<Vec<String>, SparqlError> {
        let query = match pattern {
            QueryPattern::ListNouns => SparqlQuery::list_nouns(),
            QueryPattern::ListVerbs { noun } => SparqlQuery::list_verbs(&noun),
            QueryPattern::FindCommand { noun, verb } => SparqlQuery::find_command(&noun, &verb),
            QueryPattern::FindReadOnlyCommands => SparqlQuery::find_read_only(),
            QueryPattern::Custom(sparql) => {
                SparqlQuery { pattern: sparql, variables: vec!["?result".to_string()] }
            }
        };

        self.execute(&query.pattern)
    }

    /// Execute raw SPARQL query
    pub fn execute(&self, sparql: &str) -> std::result::Result<Vec<String>, SparqlError> {
        // Check cache first
        {
            let mut cache = self.cache.lock();
            if let Some(results) = cache.get(&sparql.to_string()) {
                return Ok(results.clone());
            }
        }

        // Execute query
        let results = self.execute_query(sparql)?;

        // Cache results
        {
            let mut cache = self.cache.lock();
            cache.insert(sparql.to_string(), results.clone());
        }

        Ok(results)
    }

    /// Execute SPARQL query against ontology
    fn execute_query(&self, sparql: &str) -> std::result::Result<Vec<String>, SparqlError> {
        let mut results = Vec::new();

        // Parse query type
        if sparql.contains("LIST NOUNS") {
            results = self.list_nouns_impl();
        } else if sparql.contains("LIST VERBS") {
            if let Some(noun) = self.extract_noun(sparql) {
                results = self.list_verbs_impl(&noun);
            }
        } else if sparql.contains("FIND COMMAND") {
            if let Some((noun, verb)) = self.extract_noun_verb(sparql) {
                if let Some(cmd) = self.find_command_impl(&noun, &verb) {
                    results.push(cmd);
                }
            }
        } else if sparql.contains("READ-ONLY") {
            results = self.find_read_only_impl();
        } else {
            // Generic SELECT query
            results = self.generic_select(sparql)?;
        }

        Ok(results)
    }

    /// List all nouns
    fn list_nouns_impl(&self) -> Vec<String> {
        let has_noun_pred = format!("{}hasNoun", crate::rdf::CNV_NAMESPACE);
        let mut nouns = Vec::new();

        if let Some(subjects) = self.ontology.find_by_predicate(&has_noun_pred) {
            for subject in subjects {
                if let Some(noun_value) = self.ontology.get_object(subject, &has_noun_pred) {
                    let noun = noun_value.as_str().to_string();
                    if !nouns.contains(&noun) {
                        nouns.push(noun);
                    }
                }
            }
        }

        nouns.sort();
        nouns
    }

    /// List verbs for a noun
    fn list_verbs_impl(&self, noun: &str) -> Vec<String> {
        let has_noun_pred = format!("{}hasNoun", crate::rdf::CNV_NAMESPACE);
        let has_verb_pred = format!("{}hasVerb", crate::rdf::CNV_NAMESPACE);
        let mut verbs = Vec::new();

        if let Some(subjects) = self.ontology.find_by_predicate(&has_noun_pred) {
            for subject in subjects {
                if let Some(noun_value) = self.ontology.get_object(subject, &has_noun_pred) {
                    if noun_value.as_str() == noun {
                        if let Some(verb_value) = self.ontology.get_object(subject, &has_verb_pred)
                        {
                            let verb = verb_value.as_str().to_string();
                            if !verbs.contains(&verb) {
                                verbs.push(verb);
                            }
                        }
                    }
                }
            }
        }

        verbs.sort();
        verbs
    }

    /// Find command for noun-verb pair
    fn find_command_impl(&self, noun: &str, verb: &str) -> Option<String> {
        let has_noun_pred = format!("{}hasNoun", crate::rdf::CNV_NAMESPACE);
        let has_verb_pred = format!("{}hasVerb", crate::rdf::CNV_NAMESPACE);
        let name_pred = format!("{}name", crate::rdf::CNV_NAMESPACE);

        if let Some(subjects) = self.ontology.find_by_predicate(&has_noun_pred) {
            for subject in subjects {
                let matches_noun = self
                    .ontology
                    .get_object(subject, &has_noun_pred)
                    .map_or(false, |v| v.as_str() == noun);
                let matches_verb = self
                    .ontology
                    .get_object(subject, &has_verb_pred)
                    .map_or(false, |v| v.as_str() == verb);

                if matches_noun && matches_verb {
                    if let Some(name_value) = self.ontology.get_object(subject, &name_pred) {
                        return Some(name_value.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    /// Find read-only commands
    fn find_read_only_impl(&self) -> Vec<String> {
        // For now, return commands with specific verbs
        let read_verbs = ["status", "list", "get", "show", "describe"];
        let has_verb_pred = format!("{}hasVerb", crate::rdf::CNV_NAMESPACE);
        let name_pred = format!("{}name", crate::rdf::CNV_NAMESPACE);
        let mut commands = Vec::new();

        if let Some(subjects) = self.ontology.find_by_predicate(&has_verb_pred) {
            for subject in subjects {
                if let Some(verb_value) = self.ontology.get_object(subject, &has_verb_pred) {
                    if read_verbs.contains(&verb_value.as_str()) {
                        if let Some(name_value) = self.ontology.get_object(subject, &name_pred) {
                            commands.push(name_value.as_str().to_string());
                        }
                    }
                }
            }
        }

        commands.sort();
        commands
    }

    /// Generic SELECT query
    fn generic_select(&self, _sparql: &str) -> std::result::Result<Vec<String>, SparqlError> {
        // Simplified SPARQL parsing - extract predicates
        let results = Vec::new();
        // This would be a full SPARQL implementation
        // For now, return empty results for unhandled queries
        Ok(results)
    }

    /// Extract noun from query
    fn extract_noun(&self, query: &str) -> Option<String> {
        // Simple extraction: NOUN="value"
        if let Some(start) = query.find("NOUN=\"") {
            let s = &query[start + 6..];
            if let Some(end) = s.find('"') {
                return Some(s[..end].to_string());
            }
        }
        None
    }

    /// Extract noun and verb from query
    fn extract_noun_verb(&self, query: &str) -> Option<(String, String)> {
        let noun = self.extract_noun(query)?;

        // Extract verb: VERB="value"
        if let Some(start) = query.find("VERB=\"") {
            let s = &query[start + 6..];
            if let Some(end) = s.find('"') {
                let verb = s[..end].to_string();
                return Some((noun, verb));
            }
        }
        None
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.lock().len()
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.lock().clear();
    }
}

impl SparqlQuery {
    /// Create query to list all nouns
    pub fn list_nouns() -> Self {
        Self { pattern: "LIST NOUNS".to_string(), variables: vec!["?noun".to_string()] }
    }

    /// Create query to list verbs for a noun
    pub fn list_verbs(noun: &str) -> Self {
        Self {
            pattern: format!("LIST VERBS NOUN=\"{}\"", noun),
            variables: vec!["?verb".to_string()],
        }
    }

    /// Create query to find command for noun-verb
    pub fn find_command(noun: &str, verb: &str) -> Self {
        Self {
            pattern: format!("FIND COMMAND NOUN=\"{}\" VERB=\"{}\"", noun, verb),
            variables: vec!["?command".to_string()],
        }
    }

    /// Create query to find read-only commands
    pub fn find_read_only() -> Self {
        Self {
            pattern: "FIND READ-ONLY COMMANDS".to_string(),
            variables: vec!["?command".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdf::builder::OntologyBuilder;

    fn create_test_ontology() -> Arc<Ontology> {
        let mut builder = OntologyBuilder::new();
        builder
            .add_command("services-status", "services", "status", "Get status")
            .expect("add command");
        builder
            .add_command("services-list", "services", "list", "List services")
            .expect("add command");
        builder.add_command("config-set", "config", "set", "Set config").expect("add command");

        Arc::new(builder.build().expect("build ontology"))
    }

    #[test]
    fn test_sparql_planner_creation() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);
        assert_eq!(planner.cache_size(), 0);
    }

    #[test]
    fn test_list_nouns() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);

        let results = planner.query(QueryPattern::ListNouns).expect("list nouns");
        assert!(results.contains(&"services".to_string()));
        assert!(results.contains(&"config".to_string()));
    }

    #[test]
    fn test_list_verbs() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);

        let results = planner
            .query(QueryPattern::ListVerbs { noun: "services".to_string() })
            .expect("list verbs");
        assert!(results.contains(&"status".to_string()));
        assert!(results.contains(&"list".to_string()));
    }

    #[test]
    fn test_find_command() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);

        let results = planner
            .query(QueryPattern::FindCommand {
                noun: "services".to_string(),
                verb: "status".to_string(),
            })
            .expect("find command");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "services-status");
    }

    #[test]
    fn test_find_read_only() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);

        let results = planner.query(QueryPattern::FindReadOnlyCommands).expect("find read-only");
        assert!(results.contains(&"services-status".to_string()));
        assert!(results.contains(&"services-list".to_string()));
        assert!(!results.contains(&"config-set".to_string()));
    }

    #[test]
    fn test_cache() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);

        // First query - cache miss
        let results1 = planner.query(QueryPattern::ListNouns).expect("query");
        assert_eq!(planner.cache_size(), 1);

        // Second query - cache hit
        let results2 = planner.query(QueryPattern::ListNouns).expect("query");
        assert_eq!(results1, results2);
        assert_eq!(planner.cache_size(), 1);
    }

    #[test]
    fn test_clear_cache() {
        let ontology = create_test_ontology();
        let planner = SparqlPlanner::new(ontology);

        planner.query(QueryPattern::ListNouns).expect("query");
        assert_eq!(planner.cache_size(), 1);

        planner.clear_cache();
        assert_eq!(planner.cache_size(), 0);
    }

    #[test]
    fn test_sparql_query_builders() {
        let query = SparqlQuery::list_nouns();
        assert_eq!(query.variables[0], "?noun");

        let query = SparqlQuery::list_verbs("services");
        assert!(query.pattern.contains("services"));

        let query = SparqlQuery::find_command("services", "status");
        assert!(query.pattern.contains("services"));
        assert!(query.pattern.contains("status"));

        let query = SparqlQuery::find_read_only();
        assert!(query.pattern.contains("READ-ONLY"));
    }
}
