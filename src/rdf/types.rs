//! Core RDF types for the ontology control layer

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An RDF triple (subject, predicate, object)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: RdfValue,
}

/// RDF value types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RdfValue {
    /// URI/IRI reference
    Uri(String),
    /// Literal string value
    Literal(String),
    /// Typed literal (value, datatype)
    TypedLiteral { value: String, datatype: String },
    /// Language-tagged literal
    LangLiteral { value: String, lang: String },
    /// Blank node
    BlankNode(String),
}

impl RdfValue {
    /// Create a URI value
    pub fn uri(s: impl Into<String>) -> Self {
        Self::Uri(s.into())
    }

    /// Create a literal value
    pub fn literal(s: impl Into<String>) -> Self {
        Self::Literal(s.into())
    }

    /// Create a typed literal
    pub fn typed_literal(value: impl Into<String>, datatype: impl Into<String>) -> Self {
        Self::TypedLiteral { value: value.into(), datatype: datatype.into() }
    }

    /// Create a language-tagged literal
    pub fn lang_literal(value: impl Into<String>, lang: impl Into<String>) -> Self {
        Self::LangLiteral { value: value.into(), lang: lang.into() }
    }

    /// Create a blank node
    pub fn blank_node(id: impl Into<String>) -> Self {
        Self::BlankNode(id.into())
    }

    /// Get the string value, regardless of type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Uri(s) | Self::Literal(s) | Self::BlankNode(s) => s,
            Self::TypedLiteral { value, .. } | Self::LangLiteral { value, .. } => value,
        }
    }

    /// Check if this is a URI
    pub fn is_uri(&self) -> bool {
        matches!(self, Self::Uri(_))
    }

    /// Check if this is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_) | Self::TypedLiteral { .. } | Self::LangLiteral { .. })
    }
}

/// A structured invocation request from an agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invocation {
    /// The command URI (e.g., "cli:services-status")
    pub command: String,
    /// Arguments as key-value pairs
    pub args: BTreeMap<String, String>,
    /// Requested output format
    pub output_format: Option<String>,
    /// Invocation metadata
    pub metadata: BTreeMap<String, String>,
}

impl Invocation {
    /// Create a new invocation
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: BTreeMap::new(),
            output_format: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Add an argument
    pub fn with_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.insert(key.into(), value.into());
        self
    }

    /// Set output format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = Some(format.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get an argument value
    pub fn get_arg(&self, key: &str) -> Option<&str> {
        self.args.get(key).map(|s| s.as_str())
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
}

impl RdfTriple {
    /// Create a new RDF triple
    pub fn new(subject: impl Into<String>, predicate: impl Into<String>, object: RdfValue) -> Self {
        Self { subject: subject.into(), predicate: predicate.into(), object }
    }

    /// Convert to Turtle syntax
    pub fn to_turtle(&self) -> String {
        let obj = match &self.object {
            RdfValue::Uri(uri) => format!("<{}>", uri),
            RdfValue::Literal(lit) => format!("\"{}\"", lit.replace('\"', "\\\"")),
            RdfValue::TypedLiteral { value, datatype } => {
                format!("\"{}\"^^<{}>", value.replace('\"', "\\\""), datatype)
            }
            RdfValue::LangLiteral { value, lang } => {
                format!("\"{}\"@{}", value.replace('\"', "\\\""), lang)
            }
            RdfValue::BlankNode(id) => format!("_:{}", id),
        };
        format!("<{}> <{}> {} .", self.subject, self.predicate, obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdf_value_creation() {
        let uri = RdfValue::uri("http://example.org/foo");
        assert!(uri.is_uri());
        assert_eq!(uri.as_str(), "http://example.org/foo");

        let literal = RdfValue::literal("hello");
        assert!(literal.is_literal());
        assert_eq!(literal.as_str(), "hello");

        let typed = RdfValue::typed_literal("42", "http://www.w3.org/2001/XMLSchema#integer");
        assert!(typed.is_literal());
        assert_eq!(typed.as_str(), "42");

        let lang = RdfValue::lang_literal("bonjour", "fr");
        assert!(lang.is_literal());
        assert_eq!(lang.as_str(), "bonjour");
    }

    #[test]
    fn test_rdf_triple_to_turtle() {
        let triple = RdfTriple::new(
            "http://example.org/subject",
            "http://example.org/predicate",
            RdfValue::literal("object"),
        );
        let turtle = triple.to_turtle();
        assert!(turtle.contains("\"object\""));
        assert!(turtle.ends_with(" ."));
    }

    #[test]
    fn test_invocation_builder() {
        let inv = Invocation::new("cli:services-status")
            .with_arg("noun", "services")
            .with_arg("verb", "status")
            .with_format("json")
            .with_metadata("user", "agent-007");

        assert_eq!(inv.command, "cli:services-status");
        assert_eq!(inv.get_arg("noun"), Some("services"));
        assert_eq!(inv.get_arg("verb"), Some("status"));
        assert_eq!(inv.output_format.as_deref(), Some("json"));
        assert_eq!(inv.get_metadata("user"), Some("agent-007"));
    }

    #[test]
    fn test_invocation_missing_values() {
        let inv = Invocation::new("cli:test");
        assert_eq!(inv.get_arg("nonexistent"), None);
        assert_eq!(inv.get_metadata("missing"), None);
        assert_eq!(inv.output_format, None);
    }
}
