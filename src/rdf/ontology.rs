//! RDF ontology storage and querying

use crate::rdf::types::{RdfTriple, RdfValue};
use std::collections::BTreeMap;
use std::sync::Arc;

/// In-memory RDF graph representing the CNV ontology
#[derive(Debug, Clone)]
pub struct Ontology {
    /// All triples indexed by subject
    triples: BTreeMap<String, Vec<RdfTriple>>,
    /// Reverse index: predicate -> subjects
    predicate_index: BTreeMap<String, Vec<String>>,
    /// Namespace prefixes
    prefixes: BTreeMap<String, String>,
}

impl Ontology {
    /// Create an empty ontology
    pub fn new() -> Self {
        let mut prefixes = BTreeMap::new();
        prefixes.insert("cnv".to_string(), crate::rdf::CNV_NAMESPACE.to_string());
        prefixes.insert("rdf".to_string(), crate::rdf::RDF_NS.to_string());
        prefixes.insert("rdfs".to_string(), crate::rdf::RDFS_NS.to_string());
        prefixes.insert("xsd".to_string(), crate::rdf::XSD_NS.to_string());
        prefixes.insert("shacl".to_string(), crate::rdf::SHACL_NS.to_string());

        Self {
            triples: BTreeMap::new(),
            predicate_index: BTreeMap::new(),
            prefixes,
        }
    }

    /// Add a triple to the ontology
    pub fn add_triple(&mut self, triple: RdfTriple) {
        // Add to predicate index
        self.predicate_index
            .entry(triple.predicate.clone())
            .or_default()
            .push(triple.subject.clone());

        // Add to subject index
        self.triples
            .entry(triple.subject.clone())
            .or_default()
            .push(triple);
    }

    /// Add multiple triples
    pub fn add_triples<I>(&mut self, triples: I)
    where
        I: IntoIterator<Item = RdfTriple>,
    {
        for triple in triples {
            self.add_triple(triple);
        }
    }

    /// Get all triples for a subject
    pub fn get_triples(&self, subject: &str) -> Option<&[RdfTriple]> {
        self.triples.get(subject).map(|v| v.as_slice())
    }

    /// Find subjects with a specific predicate
    pub fn find_by_predicate(&self, predicate: &str) -> Option<&[String]> {
        self.predicate_index.get(predicate).map(|v| v.as_slice())
    }

    /// Get object value for subject-predicate pair
    pub fn get_object(&self, subject: &str, predicate: &str) -> Option<&RdfValue> {
        self.triples.get(subject).and_then(|triples| {
            triples
                .iter()
                .find(|t| t.predicate == predicate)
                .map(|t| &t.object)
        })
    }

    /// Get all triples with a specific predicate and object pattern
    pub fn find_triples(&self, predicate: Option<&str>, object_filter: Option<&str>) -> Vec<&RdfTriple> {
        let mut results = Vec::new();

        for triples in self.triples.values() {
            for triple in triples {
                let predicate_match = predicate.map_or(true, |p| triple.predicate == p);
                let object_match = object_filter.map_or(true, |o| triple.object.as_str() == o);

                if predicate_match && object_match {
                    results.push(triple);
                }
            }
        }

        results
    }

    /// Add a namespace prefix
    pub fn add_prefix(&mut self, prefix: impl Into<String>, uri: impl Into<String>) {
        self.prefixes.insert(prefix.into(), uri.into());
    }

    /// Get URI for a prefixed name (e.g., "cnv:Command" -> "https://cnv.dev/ontology#Command")
    pub fn expand_prefix(&self, prefixed: &str) -> Option<String> {
        let parts: Vec<&str> = prefixed.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        self.prefixes
            .get(parts[0])
            .map(|uri| format!("{}{}", uri, parts[1]))
    }

    /// Compact a full URI to prefixed form
    pub fn compact_uri(&self, uri: &str) -> Option<String> {
        for (prefix, ns_uri) in &self.prefixes {
            if let Some(local) = uri.strip_prefix(ns_uri) {
                return Some(format!("{}:{}", prefix, local));
            }
        }
        None
    }

    /// Convert entire ontology to Turtle format
    pub fn to_turtle(&self) -> String {
        let mut output = String::new();

        // Prefixes
        for (prefix, uri) in &self.prefixes {
            output.push_str(&format!("@prefix {}: <{}> .\n", prefix, uri));
        }
        output.push('\n');

        // Triples
        for triples in self.triples.values() {
            for triple in triples {
                output.push_str(&triple.to_turtle());
                output.push('\n');
            }
        }

        output
    }

    /// Get total number of triples
    pub fn len(&self) -> usize {
        self.triples.values().map(|v| v.len()).sum()
    }

    /// Check if ontology is empty
    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    /// Get all subjects
    pub fn subjects(&self) -> impl Iterator<Item = &str> {
        self.triples.keys().map(|s| s.as_str())
    }

    /// Get all predicates
    pub fn predicates(&self) -> impl Iterator<Item = &str> {
        self.predicate_index.keys().map(|s| s.as_str())
    }

    /// Create an Arc-wrapped ontology for sharing
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Iterate over all triples (returns owned strings to avoid lifetime issues)
    pub fn iter_triples(&self) -> Vec<(String, String, String)> {
        self.triples
            .values()
            .flatten()
            .map(|t| {
                (
                    t.subject.clone(),
                    t.predicate.clone(),
                    t.object.as_str().to_string(),
                )
            })
            .collect()
    }
}

impl Default for Ontology {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_creation() {
        let ont = Ontology::new();
        assert!(ont.is_empty());
        assert_eq!(ont.len(), 0);
        assert_eq!(ont.prefixes.len(), 5); // cnv, rdf, rdfs, xsd, shacl
    }

    #[test]
    fn test_add_triple() {
        let mut ont = Ontology::new();
        let triple = RdfTriple::new(
            "http://example.org/s1",
            "http://example.org/p1",
            RdfValue::literal("object1"),
        );
        ont.add_triple(triple.clone());

        assert_eq!(ont.len(), 1);
        let triples = ont.get_triples("http://example.org/s1");
        assert!(triples.is_some());
        assert_eq!(triples.map(|t| t.len()), Some(1));
    }

    #[test]
    fn test_predicate_index() {
        let mut ont = Ontology::new();
        ont.add_triple(RdfTriple::new(
            "http://example.org/s1",
            "http://example.org/type",
            RdfValue::uri("http://example.org/Class1"),
        ));
        ont.add_triple(RdfTriple::new(
            "http://example.org/s2",
            "http://example.org/type",
            RdfValue::uri("http://example.org/Class2"),
        ));

        let subjects = ont.find_by_predicate("http://example.org/type");
        assert_eq!(subjects.map(|s| s.len()), Some(2));
    }

    #[test]
    fn test_get_object() {
        let mut ont = Ontology::new();
        ont.add_triple(RdfTriple::new(
            "http://example.org/s1",
            "http://example.org/label",
            RdfValue::literal("Test Label"),
        ));

        let obj = ont.get_object("http://example.org/s1", "http://example.org/label");
        assert!(obj.is_some());
        assert_eq!(obj.map(|o| o.as_str()), Some("Test Label"));
    }

    #[test]
    fn test_prefix_expansion() {
        let ont = Ontology::new();
        let expanded = ont.expand_prefix("cnv:Command");
        assert_eq!(
            expanded.as_deref(),
            Some("https://cnv.dev/ontology#Command")
        );
    }

    #[test]
    fn test_uri_compaction() {
        let ont = Ontology::new();
        let compacted = ont.compact_uri("https://cnv.dev/ontology#Command");
        assert_eq!(compacted.as_deref(), Some("cnv:Command"));
    }

    #[test]
    fn test_to_turtle() {
        let mut ont = Ontology::new();
        ont.add_triple(RdfTriple::new(
            "http://example.org/s1",
            "http://example.org/p1",
            RdfValue::literal("test"),
        ));

        let turtle = ont.to_turtle();
        assert!(turtle.contains("@prefix cnv:"));
        assert!(turtle.contains("\"test\""));
    }

    #[test]
    fn test_find_triples() {
        let mut ont = Ontology::new();
        ont.add_triple(RdfTriple::new(
            "http://example.org/s1",
            "http://example.org/type",
            RdfValue::uri("http://example.org/Class"),
        ));
        ont.add_triple(RdfTriple::new(
            "http://example.org/s2",
            "http://example.org/name",
            RdfValue::literal("test"),
        ));

        let results = ont.find_triples(Some("http://example.org/type"), None);
        assert_eq!(results.len(), 1);

        let all_results = ont.find_triples(None, None);
        assert_eq!(all_results.len(), 2);
    }
}

/// CNV Ontology constants for class and property URIs
pub struct ClnvOntology;

impl ClnvOntology {
    /// Noun class
    pub fn noun() -> &'static str {
        "https://cnv.dev/ontology#Noun"
    }

    /// Verb class
    pub fn verb() -> &'static str {
        "https://cnv.dev/ontology#Verb"
    }

    /// Command class
    pub fn command() -> &'static str {
        "https://cnv.dev/ontology#Command"
    }

    /// Argument class
    pub fn argument() -> &'static str {
        "https://cnv.dev/ontology#Argument"
    }

    /// Result class
    pub fn result() -> &'static str {
        "https://cnv.dev/ontology#Result"
    }

    /// Receipt class
    pub fn receipt() -> &'static str {
        "https://cnv.dev/ontology#Receipt"
    }

    /// Guard class
    pub fn guard() -> &'static str {
        "https://cnv.dev/ontology#Guard"
    }

    /// EffectModel class
    pub fn effect_model() -> &'static str {
        "https://cnv.dev/ontology#EffectModel"
    }

    /// name property
    pub fn name() -> &'static str {
        "https://cnv.dev/ontology#name"
    }

    /// nounName property
    pub fn noun_name() -> &'static str {
        "https://cnv.dev/ontology#nounName"
    }

    /// verbName property
    pub fn verb_name() -> &'static str {
        "https://cnv.dev/ontology#verbName"
    }

    /// hasVerb property
    pub fn has_verb() -> &'static str {
        "https://cnv.dev/ontology#hasVerb"
    }

    /// hasArgument property
    pub fn has_argument() -> &'static str {
        "https://cnv.dev/ontology#hasArgument"
    }

    /// argumentType property
    pub fn argument_type() -> &'static str {
        "https://cnv.dev/ontology#argumentType"
    }

    /// isOptional property
    pub fn is_optional() -> &'static str {
        "https://cnv.dev/ontology#isOptional"
    }

    /// invokesCommand property
    pub fn invokes_command() -> &'static str {
        "https://cnv.dev/ontology#invokesCommand"
    }

    /// exitCode property
    pub fn exit_code() -> &'static str {
        "https://cnv.dev/ontology#exitCode"
    }

    /// resultHash property
    pub fn result_hash() -> &'static str {
        "https://cnv.dev/ontology#resultHash"
    }

    /// timestamp property
    pub fn timestamp() -> &'static str {
        "https://cnv.dev/ontology#timestamp"
    }
}
