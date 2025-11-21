//! Domain Logic: RDF/Ontology Operations
//!
//! Pure functions for ontology building and SPARQL query planning.
//! NO I/O - just data structures and transformations.

use serde::{Deserialize, Serialize};

/// Ontology triple representation (domain model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl OntologyTriple {
    pub fn new(subject: impl Into<String>, predicate: impl Into<String>, object: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }
    }
}

/// CLI capability metadata for ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCapability {
    pub noun: String,
    pub verb: String,
    pub description: String,
    pub args: Vec<ArgMetadata>,
    pub effects: EffectType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgMetadata {
    pub name: String,
    pub arg_type: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    ReadOnly,
    Mutating,
    Idempotent,
}

impl CliCapability {
    pub fn read_only(noun: &str, verb: &str, description: &str) -> Self {
        Self {
            noun: noun.to_string(),
            verb: verb.to_string(),
            description: description.to_string(),
            args: Vec::new(),
            effects: EffectType::ReadOnly,
        }
    }

    pub fn mutating(noun: &str, verb: &str, description: &str) -> Self {
        Self {
            noun: noun.to_string(),
            verb: verb.to_string(),
            description: description.to_string(),
            args: Vec::new(),
            effects: EffectType::Mutating,
        }
    }

    pub fn with_arg(mut self, name: &str, arg_type: &str, required: bool) -> Self {
        self.args.push(ArgMetadata {
            name: name.to_string(),
            arg_type: arg_type.to_string(),
            required,
            default: None,
        });
        self
    }

    /// Convert to RDF triples
    pub fn to_triples(&self) -> Vec<OntologyTriple> {
        let capability_uri = format!("cnv:{}_{}", self.noun, self.verb);
        vec![
            OntologyTriple::new(&capability_uri, "rdf:type", "cnv:Capability"),
            OntologyTriple::new(&capability_uri, "cnv:noun", &self.noun),
            OntologyTriple::new(&capability_uri, "cnv:verb", &self.verb),
            OntologyTriple::new(&capability_uri, "rdfs:comment", &self.description),
            OntologyTriple::new(&capability_uri, "cnv:effectType", format!("{:?}", self.effects)),
        ]
    }
}

/// Build playground ontology (pure function)
pub fn build_playground_ontology() -> Vec<CliCapability> {
    vec![
        // Papers noun
        CliCapability::mutating("papers", "generate", "Generate academic paper with Tera templates")
            .with_arg("family", "string", false),
        CliCapability::read_only("papers", "list", "List available paper families"),
        CliCapability::read_only("papers", "validate", "Validate paper structure")
            .with_arg("file", "path", true),
        // Thesis noun
        CliCapability::read_only("thesis", "structure", "Show HTF thesis structure"),
        CliCapability::read_only("thesis", "families", "List all thesis families"),
        CliCapability::read_only("thesis", "schedule", "Show Λ-schedule for family")
            .with_arg("family", "string", false),
        // Config noun
        CliCapability::read_only("config", "get", "Get configuration value")
            .with_arg("key", "string", true),
        CliCapability::mutating("config", "set", "Set configuration value")
            .with_arg("key", "string", true)
            .with_arg("value", "string", true),
        CliCapability::read_only("config", "show", "Show all configuration"),
        // Meta noun (v5 features)
        CliCapability::read_only("meta", "introspect", "Machine-grade CLI introspection"),
        CliCapability::read_only("meta", "ontology", "Export CLI as RDF/Turtle ontology"),
        CliCapability::read_only("meta", "completions", "Generate shell completions")
            .with_arg("shell", "string", true),
    ]
}

/// SPARQL query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SparqlQueryType {
    SelectCapabilities,
    SelectByNoun(String),
    SelectByEffect(EffectType),
    Custom(String),
}

impl SparqlQueryType {
    pub fn to_sparql(&self) -> String {
        match self {
            Self::SelectCapabilities => r#"
                SELECT ?noun ?verb ?description
                WHERE {
                    ?cap rdf:type cnv:Capability .
                    ?cap cnv:noun ?noun .
                    ?cap cnv:verb ?verb .
                    ?cap rdfs:comment ?description .
                }
            "#.to_string(),
            Self::SelectByNoun(noun) => format!(r#"
                SELECT ?verb ?description
                WHERE {{
                    ?cap rdf:type cnv:Capability .
                    ?cap cnv:noun "{}" .
                    ?cap cnv:verb ?verb .
                    ?cap rdfs:comment ?description .
                }}
            "#, noun),
            Self::SelectByEffect(effect) => format!(r#"
                SELECT ?noun ?verb
                WHERE {{
                    ?cap rdf:type cnv:Capability .
                    ?cap cnv:effectType "{:?}" .
                    ?cap cnv:noun ?noun .
                    ?cap cnv:verb ?verb .
                }}
            "#, effect),
            Self::Custom(query) => query.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_playground_ontology() {
        let capabilities = build_playground_ontology();
        assert!(capabilities.len() >= 9);
    }

    #[test]
    fn test_capability_to_triples() {
        let cap = CliCapability::read_only("papers", "list", "List papers");
        let triples = cap.to_triples();
        assert_eq!(triples.len(), 5);
        assert!(triples[0].predicate.contains("type"));
    }

    #[test]
    fn test_sparql_query_generation() {
        let query = SparqlQueryType::SelectByNoun("papers".to_string());
        let sparql = query.to_sparql();
        assert!(sparql.contains("papers"));
    }
}
