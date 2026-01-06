//! MCP tools for RDF Turtle CLI generation
//!
//! Provides three main MCP tools for agent-driven CLI generation:
//! 1. GenerateCliFromTurtle - Generate Rust CLI code from Turtle ontology
//! 2. QueryCapabilities - Query ontology using SPARQL
//! 3. ExportToTurtle - Export Rust CLI code back to Turtle ontology
//!
//! These tools enable agents to introspect, generate, and export CLI definitions.

use crate::rdf::{CliCodeGenerator, CodeGenError, Ontology, ParsedTurtle, SparqlExecutor, TurtleParser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur in MCP turtle tools
#[derive(Debug, Error)]
pub enum TurtleToolError {
    /// Code generation error
    #[error("Code generation failed: {0}")]
    CodeGenError(#[from] CodeGenError),

    /// Turtle parsing error
    #[error("Turtle parsing failed: {0}")]
    TurtleParseError(String),

    /// SPARQL query error
    #[error("SPARQL query failed: {0}")]
    SparqlError(String),

    /// Invalid input
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Feature not enabled
    #[error("RDF composition feature not enabled. Enable with --features rdf-composition")]
    FeatureNotEnabled,

    /// Export not implemented
    #[error("Export to Turtle from source code not yet implemented")]
    ExportNotImplemented,
}

/// Input for GenerateCliFromTurtle tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCliInput {
    /// Turtle ontology definition
    pub turtle_definition: String,
    /// Ontology IRI (namespace)
    pub ontology_iri: String,
}

/// Output from GenerateCliFromTurtle tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCliOutput {
    /// Generated Rust code
    pub rust_code: String,
    /// Diagnostic messages
    pub diagnostics: Vec<DiagnosticMessage>,
}

/// Diagnostic message for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticMessage {
    /// Severity level (info, warning, error)
    pub level: String,
    /// Message content
    pub message: String,
    /// Related IRI (optional)
    pub iri: Option<String>,
}

/// Input for QueryCapabilities tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCapabilitiesInput {
    /// SPARQL query to execute
    pub sparql_query: String,
    /// Operation type
    pub operation: QueryOperation,
}

/// Query operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOperation {
    ListCommands,
    FindVerb,
    Describe,
}

/// Output from QueryCapabilities tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCapabilitiesOutput {
    /// Query results as strings
    pub results: Vec<String>,
    /// Whether results were found
    pub found: bool,
}

/// Input for ExportToTurtle tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportToTurtleInput {
    /// Rust CLI source code
    pub cli_source_code: String,
    /// CLI name
    pub cli_name: String,
}

/// Output from ExportToTurtle tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportToTurtleOutput {
    /// Generated Turtle ontology
    pub turtle_ontology: String,
}

/// MCP Tool: Generate CLI from Turtle definition
pub struct GenerateCliFromTurtle;

impl GenerateCliFromTurtle {
    /// Execute the GenerateCliFromTurtle tool
    #[cfg(feature = "rdf-composition")]
    pub fn execute(input: GenerateCliInput) -> Result<GenerateCliOutput, TurtleToolError> {
        // Validate input
        if input.turtle_definition.trim().is_empty() {
            return Err(TurtleToolError::InvalidInput {
                message: "Turtle definition cannot be empty".to_string(),
            });
        }

        // Parse Turtle document
        let parser = TurtleParser::new().with_base_uri(input.ontology_iri);
        let parsed = parser
            .parse(&input.turtle_definition)
            .map_err(|e| TurtleToolError::TurtleParseError(e.to_string()))?;

        // Generate CLI code
        let generator = CliCodeGenerator::new()?;
        let result = generator.generate_from_ontology(&parsed)?;

        // Create diagnostic message
        let diagnostics = vec![DiagnosticMessage {
            level: "info".to_string(),
            message: format!(
                "Generated CLI with {} nouns and {} verbs",
                result.noun_count(),
                result.verb_count()
            ),
            iri: None,
        }];

        Ok(GenerateCliOutput {
            rust_code: result.rust_code().to_string(),
            diagnostics,
        })
    }

    /// Execute (fallback when feature not enabled)
    #[cfg(not(feature = "rdf-composition"))]
    pub fn execute(_input: GenerateCliInput) -> Result<GenerateCliOutput, TurtleToolError> {
        Err(TurtleToolError::FeatureNotEnabled)
    }
}

/// MCP Tool: Query ontology capabilities
pub struct QueryCapabilities {
    ontology: Ontology,
}

impl QueryCapabilities {
    /// Create a new QueryCapabilities tool with an ontology
    pub fn new(ontology: Ontology) -> Self {
        Self { ontology }
    }

    /// Execute the QueryCapabilities tool
    #[cfg(feature = "rdf-composition")]
    pub fn execute(&self, input: QueryCapabilitiesInput) -> Result<QueryCapabilitiesOutput, TurtleToolError> {
        // Create ParsedTurtle from ontology
        let parsed = ParsedTurtle::new(self.ontology.clone(), HashMap::new(), self.ontology.len(), 0);

        // Create SPARQL executor
        let executor = SparqlExecutor::new(&parsed)
            .map_err(|e| TurtleToolError::SparqlError(e.to_string()))?;

        // Execute query
        let query_result = executor
            .execute_query(&input.sparql_query)
            .map_err(|e| TurtleToolError::SparqlError(e.to_string()))?;

        // Extract results based on operation type
        let results: Vec<String> = match input.operation {
            QueryOperation::ListCommands => {
                // Extract command names from results
                query_result
                    .iter()
                    .filter_map(|binding| binding.get("name").map(|s| s.to_string()))
                    .collect()
            }
            QueryOperation::FindVerb => {
                // Extract verb IRIs
                query_result
                    .iter()
                    .filter_map(|binding| binding.get("verb").map(|s| s.to_string()))
                    .collect()
            }
            QueryOperation::Describe => {
                // Extract all variable bindings as JSON-like strings
                query_result
                    .iter()
                    .map(|binding| {
                        let pairs: Vec<String> = binding
                            .bindings
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect();
                        pairs.join(", ")
                    })
                    .collect()
            }
        };

        let found = !results.is_empty();

        Ok(QueryCapabilitiesOutput { results, found })
    }

    /// Execute (fallback when feature not enabled)
    #[cfg(not(feature = "rdf-composition"))]
    pub fn execute(&self, _input: QueryCapabilitiesInput) -> Result<QueryCapabilitiesOutput, TurtleToolError> {
        Err(TurtleToolError::FeatureNotEnabled)
    }
}

/// MCP Tool: Export Rust CLI to Turtle
pub struct ExportToTurtle;

impl ExportToTurtle {
    /// Execute the ExportToTurtle tool
    ///
    /// Note: This is a placeholder implementation. Full implementation would require
    /// parsing Rust source code and extracting command definitions.
    pub fn execute(_input: ExportToTurtleInput) -> Result<ExportToTurtleOutput, TurtleToolError> {
        // FUTURE: Implement reverse engineering of Rust CLI code to Turtle
        // This would involve:
        // 1. Parsing Rust source with syn crate
        // 2. Extracting Clap command structures
        // 3. Generating Turtle triples for verbs, nouns, arguments
        // 4. Serializing to Turtle format
        Err(TurtleToolError::ExportNotImplemented)
    }

    /// Generate minimal Turtle stub (for testing)
    pub fn generate_stub(cli_name: &str) -> String {
        format!(
            r#"@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Generated from CLI: {}
# FUTURE: Full reverse engineering implementation

cnv:GeneratedCli rdf:type cnv:Command ;
    rdfs:label "{} CLI" ;
    cnv:name "{}" .
"#,
            cli_name, cli_name, cli_name
        )
    }
}

#[cfg(all(test, feature = "rdf-composition"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cli_from_turtle() {
        // Arrange
        let input = GenerateCliInput {
            turtle_definition: r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:BuildVerb rdf:type cnv:Verb ;
    cnv:name "build" ;
    rdfs:label "Build the project" .
"#
            .to_string(),
            ontology_iri: "https://cnv.dev/ontology#".to_string(),
        };

        // Act
        let result = GenerateCliFromTurtle::execute(input);

        // Assert
        assert!(result.is_ok(), "Tool execution failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(!output.rust_code.is_empty());
        assert!(output.rust_code.contains("Commands"));
        assert!(!output.diagnostics.is_empty());
    }

    #[test]
    fn test_generate_cli_empty_input() {
        // Arrange
        let input = GenerateCliInput {
            turtle_definition: "".to_string(),
            ontology_iri: "https://cnv.dev/ontology#".to_string(),
        };

        // Act
        let result = GenerateCliFromTurtle::execute(input);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            TurtleToolError::InvalidInput { .. } => {
                // Expected
            }
            other => panic!("Expected InvalidInput, got {:?}", other),
        }
    }

    #[test]
    fn test_query_capabilities_list_commands() {
        // Arrange
        let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:TestVerb rdf:type cnv:Verb ;
    cnv:name "test" .

cnv:BuildVerb rdf:type cnv:Verb ;
    cnv:name "build" .
"#;

        let parser = TurtleParser::new();
        let parsed = parser.parse(turtle).unwrap();
        let tool = QueryCapabilities::new(parsed.into_ontology());

        let input = QueryCapabilitiesInput {
            sparql_query: r#"
                PREFIX cnv: <https://cnv.dev/ontology#>
                SELECT ?name WHERE {
                    ?verb cnv:name ?name .
                }
            "#
            .to_string(),
            operation: QueryOperation::ListCommands,
        };

        // Act
        let result = tool.execute(input);

        // Assert
        assert!(result.is_ok(), "Query failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.found);
        assert!(!output.results.is_empty());
        assert!(output.results.contains(&"test".to_string()) || output.results.contains(&"build".to_string()));
    }

    #[test]
    fn test_query_capabilities_find_verb() {
        // Arrange
        let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:DeployVerb rdf:type cnv:Verb ;
    cnv:name "deploy" .
"#;

        let parser = TurtleParser::new();
        let parsed = parser.parse(turtle).unwrap();
        let tool = QueryCapabilities::new(parsed.into_ontology());

        let input = QueryCapabilitiesInput {
            sparql_query: r#"
                PREFIX cnv: <https://cnv.dev/ontology#>
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                SELECT ?verb WHERE {
                    ?verb rdf:type cnv:Verb .
                }
            "#
            .to_string(),
            operation: QueryOperation::FindVerb,
        };

        // Act
        let result = tool.execute(input);

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.found);
        assert!(!output.results.is_empty());
    }

    #[test]
    fn test_export_to_turtle_stub() {
        // Arrange & Act
        let turtle = ExportToTurtle::generate_stub("myapp");

        // Assert
        assert!(turtle.contains("@prefix cnv:"));
        assert!(turtle.contains("myapp"));
        assert!(turtle.contains("cnv:GeneratedCli"));
    }

    #[test]
    fn test_export_to_turtle_not_implemented() {
        // Arrange
        let input = ExportToTurtleInput {
            cli_source_code: "fn main() {}".to_string(),
            cli_name: "test".to_string(),
        };

        // Act
        let result = ExportToTurtle::execute(input);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            TurtleToolError::ExportNotImplemented => {
                // Expected
            }
            other => panic!("Expected ExportNotImplemented, got {:?}", other),
        }
    }
}
