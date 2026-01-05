//! Semantic discovery via RDF and SPARQL
//!
//! This module provides semantic agent discovery using RDF triples and SPARQL queries.
//! Agents register capabilities as RDF triples, and can be discovered via semantic queries.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::agents::semantic::*;
//!
//! let mut discovery = SemanticDiscovery::new();
//!
//! // Register agent capabilities
//! discovery.register_agent(
//!     "agent-001",
//!     vec![
//!         Capability::new("nlp", "natural language processing"),
//!         Capability::new("vision", "image recognition"),
//!     ],
//! );
//!
//! // Discover agents by capability
//! let query = SparqlQueryBuilder::new()
//!     .select_agents_with_capability("nlp")
//!     .build();
//!
//! let results = discovery.query(&query)?;
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{NounVerbError, Result};

// =============================================================================
// Capability - Semantic capability descriptor
// =============================================================================

/// Semantic capability descriptor
///
/// Represents a single capability with semantic metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    /// Capability identifier (e.g., "nlp", "vision")
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Semantic tags for matching
    pub tags: Vec<String>,
}

impl Capability {
    /// Create new capability
    ///
    /// # Arguments
    ///
    /// * `id` - Capability identifier
    /// * `description` - Human-readable description
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            tags: Vec::new(),
        }
    }

    /// Add semantic tag
    ///
    /// # Arguments
    ///
    /// * `tag` - Semantic tag to add
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

// =============================================================================
// RDF Triple - Subject-Predicate-Object
// =============================================================================

/// RDF triple for semantic representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RdfTriple {
    /// Subject (e.g., "agent-001")
    pub subject: String,

    /// Predicate (e.g., "hasCapability")
    pub predicate: String,

    /// Object (e.g., "nlp")
    pub object: String,
}

impl RdfTriple {
    /// Create new RDF triple
    pub fn new(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }
    }
}

// =============================================================================
// SPARQL Query Builder
// =============================================================================

/// SPARQL query builder for semantic discovery
#[derive(Debug, Clone)]
pub struct SparqlQueryBuilder {
    /// Query patterns
    patterns: Vec<String>,

    /// Variable bindings
    variables: Vec<String>,
}

impl SparqlQueryBuilder {
    /// Create new query builder
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            variables: Vec::new(),
        }
    }

    /// Select agents with specific capability
    ///
    /// # Arguments
    ///
    /// * `capability` - Capability to match
    pub fn select_agents_with_capability(mut self, capability: &str) -> Self {
        self.variables.push("agent".to_string());
        self.patterns.push(format!(
            "?agent <hasCapability> \"{}\"",
            capability
        ));
        self
    }

    /// Select agents with tag
    ///
    /// # Arguments
    ///
    /// * `tag` - Semantic tag to match
    pub fn select_agents_with_tag(mut self, tag: &str) -> Self {
        self.variables.push("agent".to_string());
        self.patterns.push(format!(
            "?agent <hasTag> \"{}\"",
            tag
        ));
        self
    }

    /// Build SPARQL query string
    pub fn build(self) -> String {
        let vars = if self.variables.is_empty() {
            "*".to_string()
        } else {
            self.variables.join(" ")
        };

        let patterns = self.patterns.join(" . ");

        format!("SELECT {} WHERE {{ {} }}", vars, patterns)
    }
}

impl Default for SparqlQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Semantic Discovery Engine
// =============================================================================

/// Semantic discovery engine for agent capabilities
///
/// Maintains RDF triple store and executes SPARQL-like queries
#[derive(Debug, Clone)]
pub struct SemanticDiscovery {
    /// RDF triple store
    triples: Vec<RdfTriple>,

    /// Agent capability index
    agent_capabilities: HashMap<String, Vec<Capability>>,
}

impl SemanticDiscovery {
    /// Create new semantic discovery engine
    pub fn new() -> Self {
        Self {
            triples: Vec::new(),
            agent_capabilities: HashMap::new(),
        }
    }

    /// Register agent with capabilities
    ///
    /// Generates RDF triples from capabilities and adds to store
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `capabilities` - List of capabilities
    pub fn register_agent(&mut self, agent_id: &str, capabilities: Vec<Capability>) {
        // Store capabilities
        self.agent_capabilities.insert(agent_id.to_string(), capabilities.clone());

        // Generate RDF triples
        for capability in &capabilities {
            // Agent hasCapability capability_id
            self.triples.push(RdfTriple::new(
                agent_id,
                "hasCapability",
                &capability.id,
            ));

            // Agent hasDescription description
            self.triples.push(RdfTriple::new(
                agent_id,
                "hasDescription",
                &capability.description,
            ));

            // Agent hasTag tag (for each tag)
            for tag in &capability.tags {
                self.triples.push(RdfTriple::new(
                    agent_id,
                    "hasTag",
                    tag,
                ));
            }
        }
    }

    /// Query for agents matching SPARQL-like query
    ///
    /// Simplified SPARQL implementation for capability matching
    ///
    /// # Arguments
    ///
    /// * `query` - SPARQL query string
    ///
    /// # Returns
    ///
    /// List of matching agent IDs
    pub fn query(&self, query: &str) -> Result<Vec<String>> {
        // Simple pattern matching for demo
        // In production, use full SPARQL engine like oxigraph

        let mut results = Vec::new();

        // Extract capability from query (looking for pattern: <hasCapability> "value")
        if let Some(cap_start) = query.find("<hasCapability>") {
            // Find the opening quote after hasCapability
            if let Some(quote_start) = query[cap_start + 15..].find('"') {
                let value_start = cap_start + 15 + quote_start + 1;
                // Find closing quote
                if let Some(quote_len) = query[value_start..].find('"') {
                    let capability = &query[value_start..value_start + quote_len];

                    // Find agents with this capability
                    for triple in &self.triples {
                        if triple.predicate == "hasCapability" && triple.object == capability {
                            if !results.contains(&triple.subject) {
                                results.push(triple.subject.clone());
                            }
                        }
                    }
                }
            }
        }

        // Extract tag from query (looking for pattern: <hasTag> "value")
        if let Some(tag_start) = query.find("<hasTag>") {
            // Find the opening quote after hasTag
            if let Some(quote_start) = query[tag_start + 8..].find('"') {
                let value_start = tag_start + 8 + quote_start + 1;
                // Find closing quote
                if let Some(quote_len) = query[value_start..].find('"') {
                    let tag = &query[value_start..value_start + quote_len];

                    // Find agents with this tag
                    for triple in &self.triples {
                        if triple.predicate == "hasTag" && triple.object == tag {
                            if !results.contains(&triple.subject) {
                                results.push(triple.subject.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get all triples for agent
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    pub fn get_agent_triples(&self, agent_id: &str) -> Vec<RdfTriple> {
        self.triples
            .iter()
            .filter(|t| t.subject == agent_id)
            .cloned()
            .collect()
    }

    /// Get agent capabilities
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    pub fn get_agent_capabilities(&self, agent_id: &str) -> Option<&Vec<Capability>> {
        self.agent_capabilities.get(agent_id)
    }

    /// Semantic matching score between two capability sets
    ///
    /// Uses Jaccard similarity coefficient
    ///
    /// # Arguments
    ///
    /// * `caps1` - First capability set
    /// * `caps2` - Second capability set
    ///
    /// # Returns
    ///
    /// Similarity score (0.0 - 1.0)
    pub fn semantic_match_score(caps1: &[Capability], caps2: &[Capability]) -> f64 {
        let tags1: Vec<String> = caps1
            .iter()
            .flat_map(|c| c.tags.clone())
            .collect();

        let tags2: Vec<String> = caps2
            .iter()
            .flat_map(|c| c.tags.clone())
            .collect();

        if tags1.is_empty() && tags2.is_empty() {
            return 1.0;
        }

        let intersection: Vec<_> = tags1
            .iter()
            .filter(|t| tags2.contains(t))
            .collect();

        let mut union = tags1.clone();
        for tag in tags2 {
            if !union.contains(&tag) {
                union.push(tag);
            }
        }

        if union.is_empty() {
            0.0
        } else {
            intersection.len() as f64 / union.len() as f64
        }
    }
}

impl Default for SemanticDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        // Arrange & Act
        let cap = Capability::new("nlp", "Natural Language Processing")
            .with_tag("text")
            .with_tag("language");

        // Assert
        assert_eq!(cap.id, "nlp");
        assert_eq!(cap.description, "Natural Language Processing");
        assert_eq!(cap.tags.len(), 2);
        assert!(cap.tags.contains(&"text".to_string()));
    }

    #[test]
    fn test_rdf_triple_creation() {
        // Arrange & Act
        let triple = RdfTriple::new("agent-001", "hasCapability", "nlp");

        // Assert
        assert_eq!(triple.subject, "agent-001");
        assert_eq!(triple.predicate, "hasCapability");
        assert_eq!(triple.object, "nlp");
    }

    #[test]
    fn test_sparql_query_builder() {
        // Arrange
        let builder = SparqlQueryBuilder::new();

        // Act
        let query = builder.select_agents_with_capability("nlp").build();

        // Assert
        assert!(query.contains("SELECT"));
        assert!(query.contains("agent"));
        assert!(query.contains("hasCapability"));
        assert!(query.contains("nlp"));
    }

    #[test]
    fn test_agent_registration() {
        // Arrange
        let mut discovery = SemanticDiscovery::new();
        let capabilities = vec![
            Capability::new("nlp", "Natural Language Processing"),
            Capability::new("vision", "Computer Vision"),
        ];

        // Act
        discovery.register_agent("agent-001", capabilities);

        // Assert
        let triples = discovery.get_agent_triples("agent-001");
        assert_eq!(triples.len(), 4); // 2 capabilities + 2 descriptions

        let caps = discovery.get_agent_capabilities("agent-001");
        assert!(caps.is_some());
        assert_eq!(caps.unwrap().len(), 2);
    }

    #[test]
    fn test_semantic_query() {
        // Arrange
        let mut discovery = SemanticDiscovery::new();
        discovery.register_agent(
            "agent-001",
            vec![Capability::new("nlp", "NLP")],
        );
        discovery.register_agent(
            "agent-002",
            vec![Capability::new("vision", "Vision")],
        );

        let query = SparqlQueryBuilder::new()
            .select_agents_with_capability("nlp")
            .build();

        // Act
        let results = discovery.query(&query);

        // Assert
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "agent-001");
    }

    #[test]
    fn test_semantic_match_score() {
        // Arrange
        let caps1 = vec![
            Capability::new("nlp", "NLP").with_tag("text").with_tag("language"),
        ];
        let caps2 = vec![
            Capability::new("vision", "Vision").with_tag("text").with_tag("image"),
        ];

        // Act
        let score = SemanticDiscovery::semantic_match_score(&caps1, &caps2);

        // Assert
        // Intersection: {text}, Union: {text, language, image}
        assert_eq!(score, 1.0 / 3.0);
    }

    #[test]
    fn test_semantic_match_score_identical() {
        // Arrange
        let caps1 = vec![
            Capability::new("nlp", "NLP").with_tag("text"),
        ];
        let caps2 = vec![
            Capability::new("nlp", "NLP").with_tag("text"),
        ];

        // Act
        let score = SemanticDiscovery::semantic_match_score(&caps1, &caps2);

        // Assert
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_tag_query() {
        // Arrange
        let mut discovery = SemanticDiscovery::new();
        discovery.register_agent(
            "agent-001",
            vec![Capability::new("nlp", "NLP").with_tag("language")],
        );

        let query = SparqlQueryBuilder::new()
            .select_agents_with_tag("language")
            .build();

        // Act
        let results = discovery.query(&query);

        // Assert
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "agent-001");
    }
}
