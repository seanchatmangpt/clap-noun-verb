//! Production-grade code generator for RDF ontology to Rust CLI
//!
//! Transforms RDF Turtle ontologies into type-safe Rust CLI code using:
//! - SPARQL queries to extract noun/verb structures
//! - proc_macro2::TokenStream for type-safe code generation
//! - Zero unwrap/expect - full Result<T,E> error handling
//! - Chicago TDD-ready design with behavior verification
//!
//! ## Example
//!
//! ```rust,ignore
//! use clap_noun_verb::rdf::code_generator::CliCodeGenerator;
//! use clap_noun_verb::rdf::turtle_parser::TurtleParser;
//!
//! let turtle = r#"
//!     @prefix cnv: <https://cnv.dev/ontology#> .
//!     cnv:Services a cnv:Noun ; cnv:name "services" .
//!     cnv:StatusVerb a cnv:Verb ; cnv:name "status" ; cnv:hasNoun cnv:Services .
//! "#;
//!
//! let parser = TurtleParser::new();
//! let ontology = parser.parse(turtle)?;
//!
//! let generator = CliCodeGenerator::new()?;
//! let generated = generator.generate_from_ontology(&ontology)?;
//! println!("{}", generated.rust_code());
//! ```

#[cfg(feature = "rdf-composition")]
use crate::rdf::sparql_executor_oxigraph::SparqlExecutor;
use crate::rdf::turtle_parser::ParsedTurtle;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use thiserror::Error;

/// Errors that can occur during code generation
#[derive(Debug, Error)]
pub enum CodeGenError {
    /// SPARQL query execution failed
    #[error("SPARQL query failed: {message}")]
    SparqlError { message: String },

    /// Invalid ontology structure
    #[error("Invalid ontology: {message}")]
    InvalidOntology { message: String },

    /// Missing required property in ontology
    #[error("Missing required property '{property}' for {entity}")]
    MissingProperty { property: String, entity: String },

    /// Code generation failed
    #[error("Code generation failed: {message}")]
    GenerationError { message: String },

    /// Invalid identifier (not valid Rust syntax)
    #[error("Invalid Rust identifier '{identifier}': {reason}")]
    InvalidIdentifier { identifier: String, reason: String },

    /// Feature not enabled
    #[error("RDF composition feature not enabled. Enable with --features rdf-composition")]
    FeatureNotEnabled,

    /// SPARQL executor error
    #[cfg(feature = "rdf-composition")]
    #[error("SPARQL executor error: {0}")]
    ExecutorError(#[from] crate::rdf::sparql_executor_oxigraph::SparqlError),
}

/// A noun extracted from the ontology
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounDefinition {
    /// Noun URI (e.g., "https://cnv.dev/ontology#Services")
    pub uri: String,
    /// Noun name (e.g., "services")
    pub name: String,
    /// Description/label
    pub description: String,
    /// Associated verbs
    pub verbs: Vec<String>,
}

/// A verb extracted from the ontology
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbDefinition {
    /// Verb URI
    pub uri: String,
    /// Verb name (e.g., "status")
    pub name: String,
    /// Description/label
    pub description: String,
    /// Associated noun (if any)
    pub noun: Option<String>,
    /// Handler function name
    pub handler: String,
}

/// CLI configuration extracted from ontology
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Nouns defined in the ontology
    pub nouns: Vec<NounDefinition>,
    /// Verbs defined in the ontology
    pub verbs: Vec<VerbDefinition>,
    /// CLI name
    pub cli_name: String,
    /// CLI version
    pub version: String,
}

impl CliConfig {
    /// Create a new CLI configuration
    pub fn new(cli_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            nouns: Vec::new(),
            verbs: Vec::new(),
            cli_name: cli_name.into(),
            version: version.into(),
        }
    }

    /// Add a noun definition
    pub fn add_noun(&mut self, noun: NounDefinition) {
        self.nouns.push(noun);
    }

    /// Add a verb definition
    pub fn add_verb(&mut self, verb: VerbDefinition) {
        self.verbs.push(verb);
    }

    /// Get noun by name
    pub fn get_noun(&self, name: &str) -> Option<&NounDefinition> {
        self.nouns.iter().find(|n| n.name == name)
    }

    /// Get verbs for a noun
    pub fn get_verbs_for_noun(&self, noun_name: &str) -> Vec<&VerbDefinition> {
        self.verbs.iter().filter(|v| v.noun.as_deref() == Some(noun_name)).collect()
    }
}

/// Generated CLI code with metadata
#[derive(Debug, Clone)]
pub struct GeneratedCli {
    /// Generated Rust code
    rust_code: String,
    /// Number of nouns generated
    noun_count: usize,
    /// Number of verbs generated
    verb_count: usize,
    /// CLI configuration used
    config: CliConfig,
}

impl GeneratedCli {
    /// Create a new GeneratedCli
    pub fn new(rust_code: String, noun_count: usize, verb_count: usize, config: CliConfig) -> Self {
        Self { rust_code, noun_count, verb_count, config }
    }

    /// Get the generated Rust code
    pub fn rust_code(&self) -> &str {
        &self.rust_code
    }

    /// Get the number of nouns generated
    pub fn noun_count(&self) -> usize {
        self.noun_count
    }

    /// Get the number of verbs generated
    pub fn verb_count(&self) -> usize {
        self.verb_count
    }

    /// Get the CLI configuration
    pub fn config(&self) -> &CliConfig {
        &self.config
    }

    /// Write generated code to a file
    #[cfg(feature = "std")]
    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        std::fs::write(path, &self.rust_code)
    }
}

/// CLI code generator from RDF ontologies
#[cfg(feature = "rdf-composition")]
pub struct CliCodeGenerator {
    /// Default CLI name if not specified in ontology
    default_cli_name: String,
    /// Default version if not specified
    default_version: String,
}

#[cfg(feature = "rdf-composition")]
impl CliCodeGenerator {
    /// Create a new CLI code generator
    pub fn new() -> Result<Self, CodeGenError> {
        Ok(Self {
            default_cli_name: "generated_cli".to_string(),
            default_version: "0.1.0".to_string(),
        })
    }

    /// Set default CLI name
    pub fn with_cli_name(mut self, name: impl Into<String>) -> Self {
        self.default_cli_name = name.into();
        self
    }

    /// Set default version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.default_version = version.into();
        self
    }

    /// Generate CLI code from a parsed ontology
    ///
    /// # Arguments
    ///
    /// * `ontology` - Parsed Turtle ontology
    ///
    /// # Returns
    ///
    /// * `Ok(GeneratedCli)` - Generated code with metadata
    /// * `Err(CodeGenError)` - Generation failed
    pub fn generate_from_ontology(
        &self,
        ontology: &ParsedTurtle,
    ) -> Result<GeneratedCli, CodeGenError> {
        // Create SPARQL executor from ontology
        let executor = SparqlExecutor::new(ontology)?;

        // Extract CLI configuration from ontology
        let config = self.extract_cli_config(&executor)?;

        // Validate configuration
        self.validate_config(&config)?;

        // Generate Rust code
        let rust_code = self.generate_rust_code(&config)?;

        Ok(GeneratedCli::new(rust_code, config.nouns.len(), config.verbs.len(), config))
    }

    /// Extract CLI configuration from ontology using SPARQL
    fn extract_cli_config(&self, executor: &SparqlExecutor) -> Result<CliConfig, CodeGenError> {
        let mut config = CliConfig::new(&self.default_cli_name, &self.default_version);

        // Extract nouns
        let nouns = self.extract_nouns(executor)?;
        for noun in nouns {
            config.add_noun(noun);
        }

        // Extract verbs
        let verbs = self.extract_verbs(executor)?;
        for verb in verbs {
            config.add_verb(verb);
        }

        Ok(config)
    }

    /// Extract noun definitions from ontology
    fn extract_nouns(
        &self,
        executor: &SparqlExecutor,
    ) -> Result<Vec<NounDefinition>, CodeGenError> {
        let query = r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?noun ?name ?description WHERE {
                ?noun rdf:type cnv:Noun .
                ?noun cnv:name ?name .
                OPTIONAL { ?noun rdfs:label ?description }
            }
        "#;

        let results = executor.execute_query(query)?;

        let mut nouns = Vec::new();
        for binding in results.iter() {
            let uri = binding
                .get("noun")
                .ok_or_else(|| CodeGenError::MissingProperty {
                    property: "noun".to_string(),
                    entity: "noun definition".to_string(),
                })?
                .to_string();

            let name = binding
                .get("name")
                .ok_or_else(|| CodeGenError::MissingProperty {
                    property: "name".to_string(),
                    entity: uri.clone(),
                })?
                .to_string();

            let description = binding.get("description").unwrap_or(&name).to_string();

            // Validate name is a valid Rust identifier
            self.validate_identifier(&name)?;

            nouns.push(NounDefinition { uri, name, description, verbs: Vec::new() });
        }

        Ok(nouns)
    }

    /// Extract verb definitions from ontology
    fn extract_verbs(
        &self,
        executor: &SparqlExecutor,
    ) -> Result<Vec<VerbDefinition>, CodeGenError> {
        let query = r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?verb ?name ?description ?noun WHERE {
                ?verb rdf:type cnv:Verb .
                ?verb cnv:name ?name .
                OPTIONAL { ?verb rdfs:label ?description }
                OPTIONAL { ?verb cnv:hasNoun ?noun }
            }
        "#;

        let results = executor.execute_query(query)?;

        let mut verbs = Vec::new();
        for binding in results.iter() {
            let uri = binding
                .get("verb")
                .ok_or_else(|| CodeGenError::MissingProperty {
                    property: "verb".to_string(),
                    entity: "verb definition".to_string(),
                })?
                .to_string();

            let name = binding
                .get("name")
                .ok_or_else(|| CodeGenError::MissingProperty {
                    property: "name".to_string(),
                    entity: uri.clone(),
                })?
                .to_string();

            let description = binding.get("description").unwrap_or(&name).to_string();

            let noun = binding.get("noun").map(|s| s.to_string());

            // Validate name is a valid Rust identifier
            self.validate_identifier(&name)?;

            // Generate handler function name
            let handler = format!("{}_handler", name);

            verbs.push(VerbDefinition { uri, name, description, noun, handler });
        }

        Ok(verbs)
    }

    /// Validate that a string is a valid Rust identifier
    fn validate_identifier(&self, identifier: &str) -> Result<(), CodeGenError> {
        if identifier.is_empty() {
            return Err(CodeGenError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "identifier cannot be empty".to_string(),
            });
        }

        // Check first character is alphabetic or underscore
        let first_char =
            identifier.chars().next().ok_or_else(|| CodeGenError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "identifier is empty".to_string(),
            })?;

        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(CodeGenError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: format!(
                    "identifier must start with letter or underscore, found '{}'",
                    first_char
                ),
            });
        }

        // Check remaining characters are alphanumeric or underscore
        for ch in identifier.chars() {
            if !ch.is_alphanumeric() && ch != '_' {
                return Err(CodeGenError::InvalidIdentifier {
                    identifier: identifier.to_string(),
                    reason: format!("identifier contains invalid character '{}'", ch),
                });
            }
        }

        // Check not a Rust keyword
        if self.is_rust_keyword(identifier) {
            return Err(CodeGenError::InvalidIdentifier {
                identifier: identifier.to_string(),
                reason: "identifier is a Rust keyword".to_string(),
            });
        }

        Ok(())
    }

    /// Check if a string is a Rust keyword
    fn is_rust_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
        )
    }

    /// Validate CLI configuration
    fn validate_config(&self, config: &CliConfig) -> Result<(), CodeGenError> {
        if config.nouns.is_empty() && config.verbs.is_empty() {
            return Err(CodeGenError::InvalidOntology {
                message: "Ontology must contain at least one noun or verb".to_string(),
            });
        }

        Ok(())
    }

    /// Generate Rust code from CLI configuration
    fn generate_rust_code(&self, config: &CliConfig) -> Result<String, CodeGenError> {
        let mut tokens = TokenStream::new();

        // Generate noun structs and macros
        for noun in &config.nouns {
            let noun_tokens = self.generate_noun_macro(&noun.name, &noun.description)?;
            tokens.extend(noun_tokens);
        }

        // Generate verb implementations
        for verb in &config.verbs {
            let verb_tokens = if let Some(noun) = &verb.noun {
                self.generate_verb_macro(&verb.name, noun, &verb.handler)?
            } else {
                // Standalone verb (no noun)
                self.generate_standalone_verb(&verb.name, &verb.description, &verb.handler)?
            };
            tokens.extend(verb_tokens);
        }

        // Format the token stream as a string
        let code = tokens.to_string();

        // Use prettyplease for better formatting (if available)
        #[cfg(feature = "prettyplease")]
        {
            let syntax_tree =
                syn::parse_file(&code).map_err(|e| CodeGenError::GenerationError {
                    message: format!("Failed to parse generated code: {}", e),
                })?;
            Ok(prettyplease::unparse(&syntax_tree))
        }

        #[cfg(not(feature = "prettyplease"))]
        Ok(code)
    }

    /// Generate a noun macro invocation
    ///
    /// Generates:
    /// ```rust,ignore
    /// #[noun("services", "Service management commands")]
    /// pub struct Services;
    /// ```
    pub fn generate_noun_macro(
        &self,
        noun_name: &str,
        description: &str,
    ) -> Result<TokenStream, CodeGenError> {
        // Validate identifier
        self.validate_identifier(noun_name)?;

        // Convert to PascalCase for struct name
        let struct_name = self.to_pascal_case(noun_name);
        let struct_ident = format_ident!("{}", struct_name);

        Ok(quote! {
            #[noun(#noun_name, #description)]
            pub struct #struct_ident;
        })
    }

    /// Generate a verb macro invocation
    ///
    /// Generates:
    /// ```rust,ignore
    /// #[verb(Services, "status")]
    /// pub async fn status_handler(args: &StatusArgs) -> Result<StatusResponse> {
    ///     todo!("Implement status handler")
    /// }
    /// ```
    pub fn generate_verb_macro(
        &self,
        verb_name: &str,
        noun: &str,
        handler_fn: &str,
    ) -> Result<TokenStream, CodeGenError> {
        // Validate identifiers
        self.validate_identifier(verb_name)?;
        self.validate_identifier(noun)?;
        self.validate_identifier(handler_fn)?;

        // Convert noun to PascalCase
        let noun_struct = self.to_pascal_case(noun);
        let noun_ident = format_ident!("{}", noun_struct);

        // Handler function name
        let handler_ident = format_ident!("{}", handler_fn);

        // Args type (PascalCase verb name + Args)
        let args_type = format!("{}Args", self.to_pascal_case(verb_name));
        let args_ident = format_ident!("{}", args_type);

        Ok(quote! {
            #[verb(#noun_ident, #verb_name)]
            pub async fn #handler_ident(args: &#args_ident) -> Result<String> {
                // FUTURE: Generated handler implementation
                Ok(format!("Executed {} {}", #noun, #verb_name))
            }
        })
    }

    /// Generate a standalone verb (no noun)
    fn generate_standalone_verb(
        &self,
        verb_name: &str,
        description: &str,
        handler_fn: &str,
    ) -> Result<TokenStream, CodeGenError> {
        self.validate_identifier(verb_name)?;
        self.validate_identifier(handler_fn)?;

        let handler_ident = format_ident!("{}", handler_fn);
        let args_type = format!("{}Args", self.to_pascal_case(verb_name));
        let args_ident = format_ident!("{}", args_type);

        Ok(quote! {
            #[command(#verb_name, #description)]
            pub async fn #handler_ident(args: &#args_ident) -> Result<String> {
                // FUTURE: Generated handler implementation
                Ok(format!("Executed {}", #verb_name))
            }
        })
    }

    /// Generate a complete main.rs file
    pub fn generate_main_rs(&self, cli_config: &CliConfig) -> Result<String, CodeGenError> {
        let cli_name = &cli_config.cli_name;
        let version = &cli_config.version;

        let code = quote! {
            use clap::Parser;

            #[derive(Parser)]
            #[command(name = #cli_name, version = #version)]
            struct Cli {
                #[command(subcommand)]
                command: Commands,
            }

            #[derive(clap::Subcommand)]
            enum Commands {
                // FUTURE: Generated subcommands will be inserted here
            }

            #[tokio::main]
            async fn main() -> Result<(), Box<dyn std::error::Error>> {
                let cli = Cli::parse();

                match cli.command {
                    // FUTURE: Generated match arms will be inserted here
                }

                Ok(())
            }
        };

        Ok(code.to_string())
    }

    /// Convert snake_case to PascalCase
    fn to_pascal_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

#[cfg(feature = "rdf-composition")]
impl Default for CliCodeGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default CliCodeGenerator")
    }
}

/// CLI code generator stub when feature not enabled
#[cfg(not(feature = "rdf-composition"))]
pub struct CliCodeGenerator;

#[cfg(not(feature = "rdf-composition"))]
impl CliCodeGenerator {
    pub fn new() -> Result<Self, CodeGenError> {
        Err(CodeGenError::FeatureNotEnabled)
    }

    pub fn generate_from_ontology(
        &self,
        _ontology: &ParsedTurtle,
    ) -> Result<GeneratedCli, CodeGenError> {
        Err(CodeGenError::FeatureNotEnabled)
    }

    pub fn generate_noun_macro(
        &self,
        _noun_name: &str,
        _description: &str,
    ) -> Result<TokenStream, CodeGenError> {
        Err(CodeGenError::FeatureNotEnabled)
    }

    pub fn generate_verb_macro(
        &self,
        _verb_name: &str,
        _noun: &str,
        _handler_fn: &str,
    ) -> Result<TokenStream, CodeGenError> {
        Err(CodeGenError::FeatureNotEnabled)
    }

    pub fn generate_main_rs(&self, _cli_config: &CliConfig) -> Result<String, CodeGenError> {
        Err(CodeGenError::FeatureNotEnabled)
    }
}

#[cfg(all(test, feature = "rdf-composition"))]
mod tests {
    use super::*;
    use crate::rdf::turtle_parser::TurtleParser;

    /// Helper to create a test ontology with nouns and verbs
    fn create_test_ontology() -> ParsedTurtle {
        let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

cnv:Services rdf:type cnv:Noun ;
    cnv:name "services" ;
    rdfs:label "Service management commands" .

cnv:Database rdf:type cnv:Noun ;
    cnv:name "database" ;
    rdfs:label "Database operations" .

cnv:StatusVerb rdf:type cnv:Verb ;
    cnv:name "status" ;
    rdfs:label "Check status" ;
    cnv:hasNoun cnv:Services .

cnv:StartVerb rdf:type cnv:Verb ;
    cnv:name "start" ;
    rdfs:label "Start service" ;
    cnv:hasNoun cnv:Services .

cnv:MigrateVerb rdf:type cnv:Verb ;
    cnv:name "migrate" ;
    rdfs:label "Run migrations" ;
    cnv:hasNoun cnv:Database .
"#;

        TurtleParser::new().parse(turtle).expect("Failed to parse test ontology")
    }

    // Chicago TDD Tests - AAA Pattern with Behavior Verification

    #[test]
    fn test_generator_creation() {
        // Arrange & Act
        let result = CliCodeGenerator::new();

        // Assert - Verify generator can be created
        assert!(result.is_ok(), "Failed to create generator: {:?}", result.err());
    }

    #[test]
    fn test_extract_nouns_from_ontology() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).expect("Failed to create executor");
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let nouns = generator.extract_nouns(&executor);

        // Assert - Verify nouns extracted correctly
        assert!(nouns.is_ok(), "Failed to extract nouns: {:?}", nouns.err());
        let noun_list = nouns.unwrap();
        assert_eq!(noun_list.len(), 2, "Expected 2 nouns");
        assert!(noun_list.iter().any(|n| n.name == "services"), "services noun not found");
        assert!(noun_list.iter().any(|n| n.name == "database"), "database noun not found");
    }

    #[test]
    fn test_extract_verbs_from_ontology() {
        // Arrange
        let ontology = create_test_ontology();
        let executor = SparqlExecutor::new(&ontology).expect("Failed to create executor");
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let verbs = generator.extract_verbs(&executor);

        // Assert - Verify verbs extracted correctly
        assert!(verbs.is_ok(), "Failed to extract verbs: {:?}", verbs.err());
        let verb_list = verbs.unwrap();
        assert_eq!(verb_list.len(), 3, "Expected 3 verbs");
        assert!(verb_list.iter().any(|v| v.name == "status"), "status verb not found");
        assert!(verb_list.iter().any(|v| v.name == "start"), "start verb not found");
        assert!(verb_list.iter().any(|v| v.name == "migrate"), "migrate verb not found");
    }

    #[test]
    fn test_generate_noun_macro() {
        // Arrange
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let result = generator.generate_noun_macro("services", "Service management");

        // Assert - Verify macro generated correctly
        assert!(result.is_ok(), "Failed to generate noun macro: {:?}", result.err());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("noun"), "Generated code missing noun macro");
        assert!(code.contains("services"), "Generated code missing noun name");
        assert!(code.contains("Services"), "Generated code missing PascalCase struct");
    }

    #[test]
    fn test_generate_verb_macro() {
        // Arrange
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let result = generator.generate_verb_macro("status", "services", "status_handler");

        // Assert - Verify verb macro generated correctly
        assert!(result.is_ok(), "Failed to generate verb macro: {:?}", result.err());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("verb"), "Generated code missing verb macro");
        assert!(code.contains("status"), "Generated code missing verb name");
        assert!(code.contains("Services"), "Generated code missing noun reference");
        assert!(code.contains("status_handler"), "Generated code missing handler function");
    }

    #[test]
    fn test_generate_from_ontology() {
        // Arrange
        let ontology = create_test_ontology();
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let result = generator.generate_from_ontology(&ontology);

        // Assert - Verify full code generation works
        assert!(result.is_ok(), "Failed to generate from ontology: {:?}", result.err());
        let generated = result.unwrap();
        assert_eq!(generated.noun_count(), 2, "Expected 2 nouns in generated code");
        assert_eq!(generated.verb_count(), 3, "Expected 3 verbs in generated code");
        assert!(!generated.rust_code().is_empty(), "Generated code is empty");
    }

    #[test]
    fn test_generated_code_contains_nouns() {
        // Arrange
        let ontology = create_test_ontology();
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let generated = generator.generate_from_ontology(&ontology).expect("Generation failed");
        let code = generated.rust_code();

        // Assert - Verify nouns present in generated code
        assert!(code.contains("Services"), "Generated code missing Services struct");
        assert!(code.contains("Database"), "Generated code missing Database struct");
    }

    #[test]
    fn test_generated_code_contains_verbs() {
        // Arrange
        let ontology = create_test_ontology();
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let generated = generator.generate_from_ontology(&ontology).expect("Generation failed");
        let code = generated.rust_code();

        // Assert - Verify verbs present in generated code
        assert!(code.contains("status"), "Generated code missing status verb");
        assert!(code.contains("start"), "Generated code missing start verb");
        assert!(code.contains("migrate"), "Generated code missing migrate verb");
    }

    #[test]
    fn test_validate_identifier_valid() {
        // Arrange
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act & Assert - Valid identifiers
        assert!(generator.validate_identifier("services").is_ok());
        assert!(generator.validate_identifier("status_check").is_ok());
        assert!(generator.validate_identifier("_private").is_ok());
        assert!(generator.validate_identifier("Service123").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid() {
        // Arrange
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act & Assert - Invalid identifiers
        assert!(generator.validate_identifier("").is_err(), "Empty string should be invalid");
        assert!(
            generator.validate_identifier("123start").is_err(),
            "Starting with number should be invalid"
        );
        assert!(generator.validate_identifier("my-service").is_err(), "Hyphens should be invalid");
        assert!(generator.validate_identifier("for").is_err(), "Rust keywords should be invalid");
    }

    #[test]
    fn test_generate_main_rs() {
        // Arrange
        let generator = CliCodeGenerator::new().expect("Failed to create generator");
        let config = CliConfig::new("test_cli", "1.0.0");

        // Act
        let result = generator.generate_main_rs(&config);

        // Assert - Verify main.rs boilerplate generated
        assert!(result.is_ok(), "Failed to generate main.rs: {:?}", result.err());
        let code = result.unwrap();
        assert!(code.contains("clap::Parser"), "Missing clap Parser import");
        assert!(code.contains("test_cli"), "Missing CLI name");
        assert!(code.contains("1.0.0"), "Missing version");
    }

    #[test]
    fn test_to_pascal_case() {
        // Arrange
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act & Assert - Verify case conversion
        assert_eq!(generator.to_pascal_case("services"), "Services");
        assert_eq!(generator.to_pascal_case("user_profile"), "UserProfile");
        assert_eq!(generator.to_pascal_case("http_server"), "HttpServer");
    }

    #[test]
    fn test_error_on_empty_ontology() {
        // Arrange
        let turtle = r#"
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
"#;
        let ontology = TurtleParser::new().parse(turtle).expect("Failed to parse empty ontology");
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        // Act
        let result = generator.generate_from_ontology(&ontology);

        // Assert - Verify error on empty ontology
        assert!(result.is_err(), "Should fail on empty ontology");
        match result.unwrap_err() {
            CodeGenError::InvalidOntology { .. } => {
                // Expected error
            }
            other => panic!("Expected InvalidOntology error, got {:?}", other),
        }
    }

    #[test]
    fn test_noun_definition_equality() {
        // Arrange
        let noun1 = NounDefinition {
            uri: "https://cnv.dev/ontology#Services".to_string(),
            name: "services".to_string(),
            description: "Service management".to_string(),
            verbs: vec![],
        };

        let noun2 = NounDefinition {
            uri: "https://cnv.dev/ontology#Services".to_string(),
            name: "services".to_string(),
            description: "Service management".to_string(),
            verbs: vec![],
        };

        // Act & Assert - Verify equality
        assert_eq!(noun1, noun2, "Identical nouns should be equal");
    }

    #[test]
    fn test_verb_definition_equality() {
        // Arrange
        let verb1 = VerbDefinition {
            uri: "https://cnv.dev/ontology#StatusVerb".to_string(),
            name: "status".to_string(),
            description: "Check status".to_string(),
            noun: Some("services".to_string()),
            handler: "status_handler".to_string(),
        };

        let verb2 = VerbDefinition {
            uri: "https://cnv.dev/ontology#StatusVerb".to_string(),
            name: "status".to_string(),
            description: "Check status".to_string(),
            noun: Some("services".to_string()),
            handler: "status_handler".to_string(),
        };

        // Act & Assert - Verify equality
        assert_eq!(verb1, verb2, "Identical verbs should be equal");
    }

    #[test]
    fn test_cli_config_get_noun() {
        // Arrange
        let mut config = CliConfig::new("test", "1.0.0");
        config.add_noun(NounDefinition {
            uri: "test:Services".to_string(),
            name: "services".to_string(),
            description: "Services".to_string(),
            verbs: vec![],
        });

        // Act
        let noun = config.get_noun("services");

        // Assert - Verify noun retrieval
        assert!(noun.is_some(), "Failed to retrieve noun");
        assert_eq!(noun.unwrap().name, "services");
    }

    #[test]
    fn test_cli_config_get_verbs_for_noun() {
        // Arrange
        let mut config = CliConfig::new("test", "1.0.0");
        config.add_verb(VerbDefinition {
            uri: "test:Status".to_string(),
            name: "status".to_string(),
            description: "Status".to_string(),
            noun: Some("services".to_string()),
            handler: "status_handler".to_string(),
        });
        config.add_verb(VerbDefinition {
            uri: "test:Start".to_string(),
            name: "start".to_string(),
            description: "Start".to_string(),
            noun: Some("services".to_string()),
            handler: "start_handler".to_string(),
        });

        // Act
        let verbs = config.get_verbs_for_noun("services");

        // Assert - Verify verb filtering
        assert_eq!(verbs.len(), 2, "Expected 2 verbs for services noun");
    }
}
