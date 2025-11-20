//! RDF invocation parser - converts agent RDF requests to structured invocations

use crate::rdf::ontology::Ontology;
use crate::rdf::types::{Invocation, RdfTriple, RdfValue};
use crate::Result;
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;

/// Parser for RDF invocations
pub struct InvocationParser {
    ontology: Option<Arc<Ontology>>,
}

/// A parsed invocation ready for validation and execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInvocation {
    pub command: String,
    pub args: BTreeMap<String, String>,
    pub output_format: Option<String>,
}

/// Invocation parsing errors
#[derive(Debug, Error)]
pub enum InvocationError {
    #[error("Failed to parse Turtle: {0}")]
    ParseError(String),
    #[error("Unknown property: {0}")]
    UnknownProperty(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid command format: {0}")]
    InvalidCommand(String),
}

impl InvocationParser {
    /// Create a new invocation parser
    pub fn new() -> Self {
        Self { ontology: None }
    }

    /// Create a parser with ontology reference
    pub fn with_ontology(ontology: Arc<Ontology>) -> Self {
        Self { ontology: Some(ontology) }
    }

    /// Parse Turtle RDF into a ParsedInvocation
    pub fn parse_turtle(
        &self,
        ttl: &str,
    ) -> std::result::Result<ParsedInvocation, InvocationError> {
        let triples = self.parse_turtle_to_triples(ttl)?;
        self.parse_triples(&triples)
    }

    /// Parse RDF triples into a ParsedInvocation
    pub fn parse_triples(
        &self,
        triples: &[RdfTriple],
    ) -> std::result::Result<ParsedInvocation, InvocationError> {
        let mut command = None;
        let mut args = BTreeMap::new();
        let mut output_format = None;

        for triple in triples {
            let pred_local = self.extract_local_name(&triple.predicate);

            match pred_local.as_str() {
                "invokesCommand" => {
                    command = Some(triple.object.as_str().to_string());
                }
                "hasArgument" | "argument" => {
                    // Extract argument name and value
                    if let Some((name, value)) = self.parse_argument(&triple.object) {
                        args.insert(name, value);
                    }
                }
                "outputFormat" | "format" => {
                    output_format = Some(triple.object.as_str().to_string());
                }
                "hasNoun" | "noun" => {
                    args.insert("noun".to_string(), triple.object.as_str().to_string());
                }
                "hasVerb" | "verb" => {
                    args.insert("verb".to_string(), triple.object.as_str().to_string());
                }
                _ => {
                    // Store unknown properties as arguments
                    if !pred_local.starts_with("type") && !pred_local.is_empty() {
                        args.insert(pred_local, triple.object.as_str().to_string());
                    }
                }
            }
        }

        let command =
            command.ok_or_else(|| InvocationError::MissingField("command".to_string()))?;

        Ok(ParsedInvocation { command, args, output_format })
    }

    /// Parse Turtle syntax to RDF triples (simple parser)
    fn parse_turtle_to_triples(
        &self,
        ttl: &str,
    ) -> std::result::Result<Vec<RdfTriple>, InvocationError> {
        let mut triples = Vec::new();
        let lines: Vec<&str> = ttl.lines().collect();

        let mut current_subject: Option<String> = None;

        for line in lines {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Skip prefix declarations
            if line.starts_with("@prefix") || line.starts_with("PREFIX") {
                continue;
            }

            // Parse triple
            if let Some(triple) = self.parse_triple_line(line, &mut current_subject)? {
                triples.push(triple);
            }
        }

        Ok(triples)
    }

    /// Parse a single line into an RDF triple
    fn parse_triple_line(
        &self,
        line: &str,
        current_subject: &mut Option<String>,
    ) -> std::result::Result<Option<RdfTriple>, InvocationError> {
        let line = line.trim_end_matches(['.', ';'].as_ref()).trim();

        if line.is_empty() {
            return Ok(None);
        }

        // Split by whitespace
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 3 {
            return Ok(None);
        }

        let subject = if parts[0].starts_with('_') || parts[0].contains(':') {
            parts[0].trim_matches(['<', '>'].as_ref()).to_string()
        } else {
            current_subject
                .as_ref()
                .ok_or_else(|| InvocationError::ParseError("Missing subject".to_string()))?
                .clone()
        };

        let predicate = parts[1].trim_matches(['<', '>'].as_ref()).to_string();
        let object_str = parts[2..].join(" ");
        let object = self.parse_object(&object_str)?;

        *current_subject = Some(subject.clone());

        Ok(Some(RdfTriple::new(subject, predicate, object)))
    }

    /// Parse object value from string
    fn parse_object(&self, s: &str) -> std::result::Result<RdfValue, InvocationError> {
        let s = s.trim();

        // URI
        if s.starts_with('<') && s.ends_with('>') {
            return Ok(RdfValue::uri(s.trim_matches(['<', '>'].as_ref())));
        }

        // Typed literal
        if s.contains("^^") {
            let parts: Vec<&str> = s.split("^^").collect();
            if parts.len() == 2 {
                let value = parts[0].trim_matches('"');
                let datatype = parts[1].trim_matches(['<', '>'].as_ref());
                return Ok(RdfValue::typed_literal(value, datatype));
            }
        }

        // Language literal
        if s.contains('@') {
            let parts: Vec<&str> = s.rsplitn(2, '@').collect();
            if parts.len() == 2 {
                let lang = parts[0];
                let value = parts[1].trim_matches('"');
                return Ok(RdfValue::lang_literal(value, lang));
            }
        }

        // Blank node
        if s.starts_with("_:") {
            return Ok(RdfValue::blank_node(s.trim_start_matches("_:")));
        }

        // String literal
        if s.starts_with('"') && s.ends_with('"') {
            return Ok(RdfValue::literal(s.trim_matches('"')));
        }

        // Default: literal
        Ok(RdfValue::literal(s))
    }

    /// Extract local name from URI
    fn extract_local_name(&self, uri: &str) -> String {
        if let Some(pos) = uri.rfind('#') {
            uri[pos + 1..].to_string()
        } else if let Some(pos) = uri.rfind('/') {
            uri[pos + 1..].to_string()
        } else if let Some(pos) = uri.rfind(':') {
            uri[pos + 1..].to_string()
        } else {
            uri.to_string()
        }
    }

    /// Parse argument value (simple key=value)
    fn parse_argument(&self, value: &RdfValue) -> Option<(String, String)> {
        let s = value.as_str();
        if let Some(pos) = s.find('=') {
            let name = s[..pos].to_string();
            let val = s[pos + 1..].to_string();
            Some((name, val))
        } else {
            None
        }
    }
}

impl Default for InvocationParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsedInvocation {
    /// Extract noun from command
    pub fn noun(&self) -> std::result::Result<&str, InvocationError> {
        self.args
            .get("noun")
            .map(|s| s.as_str())
            .or_else(|| self.command.split('-').next())
            .ok_or_else(|| InvocationError::InvalidCommand("Cannot extract noun".to_string()))
    }

    /// Extract verb from command
    pub fn verb(&self) -> std::result::Result<&str, InvocationError> {
        self.args
            .get("verb")
            .map(|s| s.as_str())
            .or_else(|| self.command.split('-').nth(1))
            .ok_or_else(|| InvocationError::InvalidCommand("Cannot extract verb".to_string()))
    }

    /// Get argument value
    pub fn get_arg(&self, name: &str) -> Option<&str> {
        self.args.get(name).map(|s| s.as_str())
    }

    /// Convert to Invocation
    pub fn to_invocation(&self) -> Invocation {
        let mut inv = Invocation::new(&self.command);
        for (k, v) in &self.args {
            inv = inv.with_arg(k, v);
        }
        if let Some(fmt) = &self.output_format {
            inv = inv.with_format(fmt);
        }
        inv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = InvocationParser::new();
        assert!(parser.ontology.is_none());
    }

    #[test]
    fn test_parse_simple_turtle() {
        let parser = InvocationParser::new();
        let ttl = r#"
            <cli:inv1> <cnv:invokesCommand> "services-status" .
            <cli:inv1> <cnv:hasNoun> "services" .
            <cli:inv1> <cnv:hasVerb> "status" .
        "#;

        let result = parser.parse_turtle(ttl);
        assert!(result.is_ok());
        let inv = result.expect("parse");
        assert_eq!(inv.command, "services-status");
        assert_eq!(inv.get_arg("noun"), Some("services"));
        assert_eq!(inv.get_arg("verb"), Some("status"));
    }

    #[test]
    fn test_parse_with_format() {
        let parser = InvocationParser::new();
        let ttl = r#"
            <cli:inv1> <cnv:invokesCommand> "test-run" .
            <cli:inv1> <cnv:outputFormat> "json" .
        "#;

        let result = parser.parse_turtle(ttl);
        assert!(result.is_ok());
        let inv = result.expect("parse");
        assert_eq!(inv.output_format.as_deref(), Some("json"));
    }

    #[test]
    fn test_parse_missing_command() {
        let parser = InvocationParser::new();
        let ttl = r#"
            <cli:inv1> <cnv:hasNoun> "services" .
        "#;

        let result = parser.parse_turtle(ttl);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_noun_verb() {
        let inv = ParsedInvocation {
            command: "services-status".to_string(),
            args: BTreeMap::new(),
            output_format: None,
        };

        assert_eq!(inv.noun().ok(), Some("services"));
        assert_eq!(inv.verb().ok(), Some("status"));
    }

    #[test]
    fn test_extract_noun_verb_from_args() {
        let inv = ParsedInvocation {
            command: "test".to_string(),
            args: BTreeMap::from([
                ("noun".to_string(), "myapp".to_string()),
                ("verb".to_string(), "build".to_string()),
            ]),
            output_format: None,
        };

        assert_eq!(inv.noun().ok(), Some("myapp"));
        assert_eq!(inv.verb().ok(), Some("build"));
    }

    #[test]
    fn test_parse_object_types() {
        let parser = InvocationParser::new();

        let uri = parser.parse_object("<http://example.org/test>").expect("parse uri");
        assert!(uri.is_uri());

        let literal = parser.parse_object("\"hello\"").expect("parse literal");
        assert!(literal.is_literal());

        let typed = parser
            .parse_object("\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>")
            .expect("parse typed");
        assert!(typed.is_literal());
    }

    #[test]
    fn test_to_invocation() {
        let parsed = ParsedInvocation {
            command: "test-run".to_string(),
            args: BTreeMap::from([("pattern".to_string(), "*.rs".to_string())]),
            output_format: Some("json".to_string()),
        };

        let inv = parsed.to_invocation();
        assert_eq!(inv.command, "test-run");
        assert_eq!(inv.get_arg("pattern"), Some("*.rs"));
        assert_eq!(inv.output_format.as_deref(), Some("json"));
    }
}
