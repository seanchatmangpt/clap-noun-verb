//! Ontology builder - generates RDF ontology from registered verbs

use crate::rdf::ontology::Ontology;
use crate::rdf::types::{RdfTriple, RdfValue};
use crate::Result;

/// Builder for constructing CNV ontology from registered verbs
pub struct OntologyBuilder {
    ontology: Ontology,
}

impl OntologyBuilder {
    /// Create a new ontology builder
    pub fn new() -> Self {
        Self { ontology: Ontology::new() }
    }

    /// Add a command definition
    pub fn add_command(
        &mut self,
        name: &str,
        noun: &str,
        verb: &str,
        description: &str,
    ) -> std::result::Result<&mut Self, String> {
        let cmd_uri = format!("{}Command-{}-{}", crate::rdf::CNV_NAMESPACE, noun, verb);
        let rdf_type = format!("{}type", crate::rdf::RDF_NS);
        let command_class = format!("{}Command", crate::rdf::CNV_NAMESPACE);

        // Command is a cnv:Command
        self.ontology.add_triple(RdfTriple::new(
            &cmd_uri,
            &rdf_type,
            RdfValue::uri(&command_class),
        ));

        // Add name
        self.ontology.add_triple(RdfTriple::new(
            &cmd_uri,
            format!("{}name", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(name),
        ));

        // Add noun
        self.ontology.add_triple(RdfTriple::new(
            &cmd_uri,
            format!("{}hasNoun", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(noun),
        ));

        // Add verb
        self.ontology.add_triple(RdfTriple::new(
            &cmd_uri,
            format!("{}hasVerb", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(verb),
        ));

        // Add description
        self.ontology.add_triple(RdfTriple::new(
            &cmd_uri,
            format!("{}description", crate::rdf::RDFS_NS),
            RdfValue::literal(description),
        ));

        Ok(self)
    }

    /// Add an argument definition for a command
    pub fn add_argument(
        &mut self,
        command_name: &str,
        arg_name: &str,
        arg_type: &str,
        required: bool,
    ) -> std::result::Result<&mut Self, String> {
        let arg_uri =
            format!("{}Argument-{}-{}", crate::rdf::CNV_NAMESPACE, command_name, arg_name);
        let rdf_type = format!("{}type", crate::rdf::RDF_NS);
        let argument_class = format!("{}Argument", crate::rdf::CNV_NAMESPACE);

        // Argument is a cnv:Argument
        self.ontology.add_triple(RdfTriple::new(
            &arg_uri,
            &rdf_type,
            RdfValue::uri(&argument_class),
        ));

        // Add name
        self.ontology.add_triple(RdfTriple::new(
            &arg_uri,
            format!("{}name", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(arg_name),
        ));

        // Add type
        self.ontology.add_triple(RdfTriple::new(
            &arg_uri,
            format!("{}datatype", crate::rdf::CNV_NAMESPACE),
            RdfValue::literal(arg_type),
        ));

        // Add required flag
        self.ontology.add_triple(RdfTriple::new(
            &arg_uri,
            format!("{}required", crate::rdf::CNV_NAMESPACE),
            RdfValue::typed_literal(
                if required { "true" } else { "false" },
                format!("{}boolean", crate::rdf::XSD_NS),
            ),
        ));

        Ok(self)
    }

    /// Add a SHACL shape for validation
    pub fn add_shape(
        &mut self,
        shape_name: &str,
        target_class: &str,
    ) -> std::result::Result<&mut Self, String> {
        let shape_uri = format!("{}Shape-{}", crate::rdf::CNV_NAMESPACE, shape_name);
        let rdf_type = format!("{}type", crate::rdf::RDF_NS);
        let node_shape = format!("{}NodeShape", crate::rdf::SHACL_NS);

        // Shape is a sh:NodeShape
        self.ontology.add_triple(RdfTriple::new(&shape_uri, &rdf_type, RdfValue::uri(&node_shape)));

        // Add target class
        self.ontology.add_triple(RdfTriple::new(
            &shape_uri,
            format!("{}targetClass", crate::rdf::SHACL_NS),
            RdfValue::uri(target_class),
        ));

        Ok(self)
    }

    /// Build and return the ontology
    pub fn build(self) -> std::result::Result<Ontology, String> {
        Ok(self.ontology)
    }

    /// Get a reference to the ontology being built
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }

    /// Get a mutable reference to the ontology being built
    pub fn ontology_mut(&mut self) -> &mut Ontology {
        &mut self.ontology
    }
}

impl Default for OntologyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = OntologyBuilder::new();
        // New builder starts with prefixes but no triples
        assert!(builder.ontology().is_empty());
        assert!(builder.ontology().len() == 0);
    }

    #[test]
    fn test_add_command() {
        let mut builder = OntologyBuilder::new();
        builder
            .add_command("services-status", "services", "status", "Get service status")
            .expect("Failed to add command");

        let ontology = builder.build().expect("Failed to build ontology");
        let cmd_uri = format!("{}Command-services-status", crate::rdf::CNV_NAMESPACE);
        assert!(ontology.get_triples(&cmd_uri).is_some());
    }

    #[test]
    fn test_add_argument() {
        let mut builder = OntologyBuilder::new();
        builder
            .add_command("services-status", "services", "status", "Get service status")
            .expect("Failed to add command");
        builder
            .add_argument("services-status", "format", "string", false)
            .expect("Failed to add argument");

        let ontology = builder.build().expect("Failed to build ontology");
        let arg_uri = format!("{}Argument-services-status-format", crate::rdf::CNV_NAMESPACE);
        assert!(ontology.get_triples(&arg_uri).is_some());
    }

    #[test]
    fn test_add_shape() {
        let mut builder = OntologyBuilder::new();
        builder
            .add_shape("CommandShape", "https://cnv.dev/ontology#Command")
            .expect("Failed to add shape");

        let ontology = builder.build().expect("Failed to build ontology");
        let shape_uri = format!("{}Shape-CommandShape", crate::rdf::CNV_NAMESPACE);
        assert!(ontology.get_triples(&shape_uri).is_some());
    }

    #[test]
    fn test_build_complex_ontology() {
        let mut builder = OntologyBuilder::new();
        builder
            .add_command("test-run", "test", "run", "Run tests")
            .expect("add command")
            .add_argument("test-run", "pattern", "string", false)
            .expect("add argument")
            .add_shape("TestShape", "https://cnv.dev/ontology#Command")
            .expect("add shape");

        let ontology = builder.build().expect("build");
        assert!(ontology.len() > 0);
    }
}
