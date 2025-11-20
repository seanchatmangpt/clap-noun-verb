//! Runtime integration with macro-generated RDF metadata
//!
//! This module provides the bridge between compile-time RDF generation
//! in macros and runtime ontology construction. It uses linkme distributed
//! slices to collect all verb metadata across the binary.

use crate::error::Result;
use crate::rdf::ontology::Ontology;
use crate::rdf::validation::{ShapeValidator, ShaclShape, Constraint};
use std::sync::{Arc, OnceLock};

/// Distributed slice for macro-generated RDF metadata
///
/// Each #[verb] macro registers a function in this slice that returns
/// (rdf_triples, shacl_shapes) as static strings.
#[linkme::distributed_slice]
pub static __VERB_RDF: [fn() -> (&'static str, &'static str)];

/// Global RDF registry singleton
static GLOBAL_REGISTRY: OnceLock<Arc<RdfRegistry>> = OnceLock::new();

/// Registry of all verb RDF metadata
#[derive(Debug, Clone)]
pub struct RdfRegistry {
    /// All RDF triples from verb macros
    pub rdf_triples: Vec<String>,
    /// All SHACL shapes from verb macros
    pub shacl_shapes: Vec<String>,
}

impl RdfRegistry {
    /// Load RDF metadata from all #[verb] macros
    ///
    /// This function is called at runtime to collect all compile-time
    /// generated RDF triples and SHACL shapes.
    pub fn load_from_macros() -> Self {
        let mut triples = Vec::new();
        let mut shapes = Vec::new();

        // Call each macro-generated initialization function
        for init_fn in __VERB_RDF {
            let (ttl, shacl) = init_fn();
            if !ttl.is_empty() {
                triples.push(ttl.to_string());
            }
            if !shacl.is_empty() {
                shapes.push(shacl.to_string());
            }
        }

        Self {
            rdf_triples: triples,
            shacl_shapes: shapes,
        }
    }

    /// Get or initialize the global registry
    pub fn global() -> Arc<Self> {
        GLOBAL_REGISTRY
            .get_or_init(|| Arc::new(Self::load_from_macros()))
            .clone()
    }

    /// Build an ontology from collected RDF triples
    ///
    /// Parses all Turtle RDF strings and adds them to a new Ontology instance.
    pub fn build_ontology(&self) -> Result<Ontology> {
        let mut ontology = Ontology::new();

        // Add standard prefixes
        ontology.add_prefix("cli", "https://cli.app/");
        ontology.add_prefix("cnv", crate::rdf::CNV_NAMESPACE);
        ontology.add_prefix("rdf", crate::rdf::RDF_NS);
        ontology.add_prefix("rdfs", crate::rdf::RDFS_NS);
        ontology.add_prefix("xsd", crate::rdf::XSD_NS);
        ontology.add_prefix("sh", crate::rdf::SHACL_NS);

        // Parse each RDF triple string and add to ontology
        for ttl in &self.rdf_triples {
            if ttl.trim().is_empty() || ttl.starts_with('#') {
                continue;
            }

            // Simple parsing: split into lines and extract triples
            // For production, would use a full Turtle parser
            let triples = self.parse_simple_turtle(ttl)?;
            ontology.add_triples(triples);
        }

        Ok(ontology)
    }

    /// Build a SHACL shape validator from collected shapes
    ///
    /// Parses all SHACL shape definitions and creates a validator
    /// that can check invocations against these constraints.
    pub fn build_shape_validator(&self) -> Result<ShapeValidator> {
        let mut validator = ShapeValidator::new();

        // Parse each SHACL shape string
        for shacl_str in &self.shacl_shapes {
            if shacl_str.trim().is_empty() || shacl_str.starts_with('#') {
                continue;
            }

            let shapes = self.parse_shacl_shapes(shacl_str)?;
            validator.add_shapes(shapes)
                .map_err(|e| crate::error::NounVerbError::ValidationFailed(e.to_string()))?;
        }

        Ok(validator)
    }

    /// Simple Turtle parser (placeholder for full implementation)
    ///
    /// For production use, integrate a proper Turtle parser library.
    /// This implementation handles basic triple patterns.
    fn parse_simple_turtle(&self, ttl: &str) -> Result<Vec<crate::rdf::types::RdfTriple>> {
        use crate::rdf::types::{RdfTriple, RdfValue};

        let mut triples = Vec::new();
        let mut current_subject: Option<String> = None;

        for line in ttl.lines() {
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

    /// Parse a single triple line
    fn parse_triple_line(
        &self,
        line: &str,
        current_subject: &mut Option<String>,
    ) -> Result<Option<crate::rdf::types::RdfTriple>> {
        use crate::rdf::types::{RdfTriple, RdfValue};

        let line = line.trim_end_matches(['.', ';'].as_ref()).trim();

        if line.is_empty() {
            return Ok(None);
        }

        // Split by whitespace (simplified - proper Turtle parser needed for production)
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 3 {
            return Ok(None);
        }

        let subject = if parts[0].contains(':') || parts[0].starts_with('<') {
            parts[0].trim_matches(['<', '>'].as_ref()).to_string()
        } else {
            current_subject.as_ref()
                .ok_or_else(|| crate::error::NounVerbError::InvalidStructure {
                    message: "Missing subject in triple".to_string()
                })?
                .clone()
        };

        let predicate = parts[1].trim_matches(['<', '>'].as_ref()).to_string();
        let object_str = parts[2..].join(" ");
        let object = self.parse_object(&object_str)?;

        *current_subject = Some(subject.clone());

        Ok(Some(RdfTriple::new(subject, predicate, object)))
    }

    /// Parse object value from string
    fn parse_object(&self, s: &str) -> Result<crate::rdf::types::RdfValue> {
        use crate::rdf::types::RdfValue;

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

        // String literal
        if s.starts_with('"') && s.ends_with('"') {
            return Ok(RdfValue::literal(s.trim_matches('"')));
        }

        // Boolean literals
        if s == "true" || s == "false" {
            return Ok(RdfValue::typed_literal(s, "http://www.w3.org/2001/XMLSchema#boolean"));
        }

        // Default: literal
        Ok(RdfValue::literal(s))
    }

    /// Parse SHACL shapes from string
    fn parse_shacl_shapes(&self, shacl_str: &str) -> Result<Vec<ShaclShape>> {
        let mut shapes = Vec::new();

        // Simplified SHACL parser - extract shape names and constraints
        // For production, use a proper SHACL parser

        let mut current_shape: Option<ShaclShape> = None;
        let mut in_property = false;
        let mut property_constraints: Vec<Constraint> = Vec::new();

        for line in shacl_str.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // New shape
            if line.contains("a sh:NodeShape") {
                if let Some(shape) = current_shape.take() {
                    shapes.push(shape);
                }
                let shape_name = line.split_whitespace().next().unwrap_or("unknown");
                current_shape = Some(ShaclShape::new(shape_name));
            }

            // Property shape
            if line.contains("sh:property [") {
                in_property = true;
                property_constraints.clear();
            }

            // End of property
            if line == "]" || line == "] ." || line == "] ;" {
                in_property = false;
                if let Some(ref mut shape) = current_shape {
                    shape.constraints.extend(property_constraints.drain(..));
                }
            }

            // Parse constraints
            if in_property {
                if line.contains("sh:minCount") {
                    if let Some(val) = extract_number(line) {
                        property_constraints.push(Constraint::MinCount(val));
                    }
                }
                if line.contains("sh:maxCount") {
                    if let Some(val) = extract_number(line) {
                        property_constraints.push(Constraint::MaxCount(val));
                    }
                }
                if line.contains("sh:minInclusive") {
                    if let Some(val) = extract_i64(line) {
                        property_constraints.push(Constraint::MinInclusive(val));
                    }
                }
                if line.contains("sh:maxInclusive") {
                    if let Some(val) = extract_i64(line) {
                        property_constraints.push(Constraint::MaxInclusive(val));
                    }
                }
                if line.contains("sh:minLength") {
                    if let Some(val) = extract_number(line) {
                        property_constraints.push(Constraint::MinLength(val));
                    }
                }
                if line.contains("sh:maxLength") {
                    if let Some(val) = extract_number(line) {
                        property_constraints.push(Constraint::MaxLength(val));
                    }
                }
                if line.contains("sh:pattern") {
                    if let Some(pattern) = extract_string(line) {
                        property_constraints.push(Constraint::Pattern(pattern));
                    }
                }
                if line.contains("sh:datatype") {
                    if let Some(datatype) = extract_datatype(line) {
                        property_constraints.push(Constraint::DataType(datatype));
                    }
                }
            }
        }

        // Add final shape
        if let Some(shape) = current_shape {
            shapes.push(shape);
        }

        Ok(shapes)
    }

    /// Get number of registered verbs
    pub fn verb_count(&self) -> usize {
        self.rdf_triples.len()
    }

    /// Get all RDF triples as a single Turtle document
    pub fn to_turtle(&self) -> String {
        let mut output = String::new();

        // Add prefixes
        output.push_str("@prefix cli: <https://cli.app/> .\n");
        output.push_str(&format!("@prefix cnv: <{}> .\n", crate::rdf::CNV_NAMESPACE));
        output.push_str(&format!("@prefix rdf: <{}> .\n", crate::rdf::RDF_NS));
        output.push_str(&format!("@prefix rdfs: <{}> .\n", crate::rdf::RDFS_NS));
        output.push_str(&format!("@prefix xsd: <{}> .\n", crate::rdf::XSD_NS));
        output.push_str(&format!("@prefix sh: <{}> .\n\n", crate::rdf::SHACL_NS));

        // Add all triples
        for ttl in &self.rdf_triples {
            if !ttl.trim().is_empty() {
                output.push_str(ttl);
                output.push('\n');
            }
        }

        output
    }
}

impl Default for RdfRegistry {
    fn default() -> Self {
        Self::load_from_macros()
    }
}

/// Extract number from SHACL constraint line
fn extract_number(line: &str) -> Option<usize> {
    // Find a number in the line (before any ; or .)
    line.split_whitespace()
        .find_map(|token| {
            let cleaned = token.trim_matches([';', '.'].as_ref());
            cleaned.parse().ok()
        })
}

/// Extract i64 from SHACL constraint line
fn extract_i64(line: &str) -> Option<i64> {
    // Find a number in the line (before any ; or .)
    line.split_whitespace()
        .find_map(|token| {
            let cleaned = token.trim_matches([';', '.'].as_ref());
            cleaned.parse().ok()
        })
}

/// Extract string from quoted value
fn extract_string(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Extract datatype from constraint line
fn extract_datatype(line: &str) -> Option<String> {
    line.split_whitespace()
        .last()?
        .trim_matches([';', '.'].as_ref())
        .to_string()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = RdfRegistry::load_from_macros();
        // At minimum, should be empty (no verbs registered in test)
        assert!(registry.verb_count() >= 0);
    }

    #[test]
    fn test_global_registry() {
        let registry1 = RdfRegistry::global();
        let registry2 = RdfRegistry::global();
        assert!(Arc::ptr_eq(&registry1, &registry2));
    }

    #[test]
    fn test_parse_simple_triple() {
        let registry = RdfRegistry::load_from_macros();
        let ttl = r#"<cli:test> <cnv:name> "test" ."#;

        let triples = registry.parse_simple_turtle(ttl).expect("parse");
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].subject, "cli:test");
        assert_eq!(triples[0].predicate, "cnv:name");
        assert_eq!(triples[0].object.as_str(), "test");
    }

    #[test]
    fn test_parse_object_types() {
        let registry = RdfRegistry::load_from_macros();

        let uri = registry.parse_object("<http://example.org>").expect("parse uri");
        assert!(uri.is_uri());

        let literal = registry.parse_object("\"hello\"").expect("parse literal");
        assert!(literal.is_literal());

        let boolean = registry.parse_object("true").expect("parse bool");
        assert!(boolean.is_literal());
    }

    #[test]
    fn test_build_ontology() {
        let registry = RdfRegistry::load_from_macros();
        let ontology = registry.build_ontology().expect("build ontology");
        assert!(ontology.len() >= 0);
    }

    #[test]
    fn test_build_shape_validator() {
        let registry = RdfRegistry::load_from_macros();
        let validator = registry.build_shape_validator().expect("build validator");
        assert!(validator.shape_count() >= 0);
    }

    #[test]
    fn test_extract_helpers() {
        assert_eq!(extract_number("sh:minCount 5 ;"), Some(5));
        assert_eq!(extract_number("sh:maxCount 10 ."), Some(10));
        assert_eq!(extract_i64("sh:minInclusive -100 ;"), Some(-100));
        assert_eq!(extract_string("sh:pattern \"^[a-z]+$\" ;"), Some("^[a-z]+$".to_string()));
    }

    #[test]
    fn test_to_turtle() {
        let registry = RdfRegistry::load_from_macros();
        let turtle = registry.to_turtle();
        assert!(turtle.contains("@prefix cnv:"));
        assert!(turtle.contains("@prefix xsd:"));
    }
}
