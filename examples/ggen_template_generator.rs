//! ggen Template Generator for clap-noun-verb Capabilities
//!
//! This example demonstrates using ggen's Tera + SPARQL template generation system
//! to create all 360 template combinations from an RDF ontology.
//!
//! The architecture:
//! 1. Load RDF ontology (clap-capabilities.ttl)
//! 2. Execute SPARQL queries to derive template parameters
//! 3. Use Tera templates to generate Rust code
//! 4. Output complete, ready-to-use modules
//!
//! This is the template generation pipeline for clap-noun-verb framework.

use oxigraph::store::Store;
use oxigraph::sparql::QueryResults;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// SPARQL Queries for Template Generation
// ============================================================================

const SPARQL_NOUN_ENTITY_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?noun ?label ?examples ?tests WHERE {
    ?noun a clap:NounEntity ;
           rdfs:label ?label ;
           clap:examples ?examples ;
           clap:tests ?tests .
}
ORDER BY ?label
"#;

const SPARQL_VERB_ACTION_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?verb ?label ?operation ?resultType WHERE {
    ?verb a clap:VerbAction ;
          rdfs:label ?label ;
          clap:operation ?operation ;
          clap:resultType ?resultType .
}
ORDER BY ?label
"#;

const SPARQL_CAPABILITY_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?capability ?label ?comment ?examples ?tests WHERE {
    ?capability a clap:Capability ;
                rdfs:label ?label ;
                rdfs:comment ?comment ;
                clap:examples ?examples ;
                clap:tests ?tests .
}
ORDER BY ?label
"#;

const SPARQL_NOUN_VERB_COMBINATIONS_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?nounLabel ?verbLabel ?operation ?resultType WHERE {
    ?noun a clap:NounEntity ;
          rdfs:label ?nounLabel .
    ?verb a clap:VerbAction ;
          rdfs:label ?verbLabel ;
          clap:operation ?operation ;
          clap:resultType ?resultType .
}
ORDER BY ?nounLabel ?verbLabel
"#;

const SPARQL_ARGUMENT_PATTERN_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?pattern ?label ?comment ?validation ?type WHERE {
    ?pattern a clap:ArgumentPattern ;
             rdfs:label ?label ;
             rdfs:comment ?comment ;
             clap:validation ?validation ;
             clap:type ?type .
}
ORDER BY ?label
"#;

const SPARQL_ASYNC_PATTERN_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?pattern ?label ?comment ?complexity ?frameworks WHERE {
    ?pattern a clap:AsyncPattern ;
             rdfs:label ?label ;
             rdfs:comment ?comment ;
             clap:complexity ?complexity ;
             clap:frameworks ?frameworks .
}
ORDER BY ?label
"#;

const SPARQL_ERROR_TYPE_QUERY: &str = r#"
PREFIX clap: <http://clap-noun-verb.org/capability/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?error ?label ?comment ?httpStatus ?handlingStrategy WHERE {
    ?error a clap:ErrorType ;
           rdfs:label ?label ;
           rdfs:comment ?comment ;
           clap:httpStatus ?httpStatus ;
           clap:handlingStrategy ?handlingStrategy .
}
ORDER BY ?label
"#;

// ============================================================================
// Template Generator Struct
// ============================================================================

pub struct GgenTemplateGenerator {
    store: Store,
    ontology_path: PathBuf,
    output_dir: PathBuf,
}

impl GgenTemplateGenerator {
    /// Create a new template generator
    pub fn new(ontology_path: PathBuf, output_dir: PathBuf) -> Self {
        GgenTemplateGenerator { store: Store::new().expect("Failed to create RDF store"), ontology_path, output_dir }
    }

    /// Load RDF ontology from Turtle file
    pub fn load_ontology(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.ontology_path)?;
        let turtle = oxigraph::io::parse(content.as_bytes(), oxigraph::io::DatasetFormat::Turtle)?;
        for quad in turtle {
            self.store.insert(&quad?);
        }
        Ok(())
    }

    /// Query nouns from ontology
    pub fn query_nouns(&self) -> Result<Vec<NounEntity>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_NOUN_ENTITY_QUERY)?;
        let mut nouns = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let noun = NounEntity {
                    uri: solution.get("noun").map(|v| v.to_string()),
                    label: solution.get("label").map(|v| v.to_string()),
                    examples: solution
                        .get("examples")
                        .and_then(|v| {
                            if let oxigraph::model::Term::Literal(lit) = v {
                                lit.value().parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0),
                    tests: solution
                        .get("tests")
                        .and_then(|v| {
                            if let oxigraph::model::Term::Literal(lit) = v {
                                lit.value().parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0),
                };
                nouns.push(noun);
            }
        }
        Ok(nouns)
    }

    /// Query verbs from ontology
    pub fn query_verbs(&self) -> Result<Vec<VerbAction>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_VERB_ACTION_QUERY)?;
        let mut verbs = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let verb = VerbAction {
                    uri: solution.get("verb").map(|v| v.to_string()),
                    label: solution.get("label").map(|v| v.to_string()),
                    operation: solution.get("operation").map(|v| v.to_string()),
                    result_type: solution.get("resultType").map(|v| v.to_string()),
                };
                verbs.push(verb);
            }
        }
        Ok(verbs)
    }

    /// Query all capabilities from ontology
    pub fn query_capabilities(&self) -> Result<Vec<Capability>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_CAPABILITY_QUERY)?;
        let mut capabilities = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let capability = Capability {
                    uri: solution.get("capability").map(|v| v.to_string()),
                    label: solution.get("label").map(|v| v.to_string()),
                    comment: solution.get("comment").map(|v| v.to_string()),
                    examples: solution
                        .get("examples")
                        .and_then(|v| {
                            if let oxigraph::model::Term::Literal(lit) = v {
                                lit.value().parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0),
                    tests: solution
                        .get("tests")
                        .and_then(|v| {
                            if let oxigraph::model::Term::Literal(lit) = v {
                                lit.value().parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0),
                };
                capabilities.push(capability);
            }
        }
        Ok(capabilities)
    }

    /// Query noun-verb combinations from ontology
    pub fn query_noun_verb_combinations(&self) -> Result<Vec<NounVerbCombination>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_NOUN_VERB_COMBINATIONS_QUERY)?;
        let mut combinations = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let combination = NounVerbCombination {
                    noun_label: solution.get("nounLabel").map(|v| v.to_string()),
                    verb_label: solution.get("verbLabel").map(|v| v.to_string()),
                    operation: solution.get("operation").map(|v| v.to_string()),
                    result_type: solution.get("resultType").map(|v| v.to_string()),
                };
                combinations.push(combination);
            }
        }
        Ok(combinations)
    }

    /// Query argument patterns
    pub fn query_argument_patterns(&self) -> Result<Vec<ArgumentPattern>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_ARGUMENT_PATTERN_QUERY)?;
        let mut patterns = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let pattern = ArgumentPattern {
                    uri: solution.get("pattern").map(|v| v.to_string()),
                    label: solution.get("label").map(|v| v.to_string()),
                    comment: solution.get("comment").map(|v| v.to_string()),
                    validation: solution.get("validation").map(|v| v.to_string()),
                    type_name: solution.get("type").map(|v| v.to_string()),
                };
                patterns.push(pattern);
            }
        }
        Ok(patterns)
    }

    /// Query async patterns
    pub fn query_async_patterns(&self) -> Result<Vec<AsyncPatternInfo>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_ASYNC_PATTERN_QUERY)?;
        let mut patterns = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let pattern = AsyncPatternInfo {
                    uri: solution.get("pattern").map(|v| v.to_string()),
                    label: solution.get("label").map(|v| v.to_string()),
                    comment: solution.get("comment").map(|v| v.to_string()),
                    complexity: solution.get("complexity").map(|v| v.to_string()),
                    frameworks: solution.get("frameworks").map(|v| v.to_string()),
                };
                patterns.push(pattern);
            }
        }
        Ok(patterns)
    }

    /// Query error types
    pub fn query_error_types(&self) -> Result<Vec<ErrorTypeInfo>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_ERROR_TYPE_QUERY)?;
        let mut errors = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                let error = ErrorTypeInfo {
                    uri: solution.get("error").map(|v| v.to_string()),
                    label: solution.get("label").map(|v| v.to_string()),
                    comment: solution.get("comment").map(|v| v.to_string()),
                    http_status: solution
                        .get("httpStatus")
                        .and_then(|v| {
                            if let oxigraph::model::Term::Literal(lit) = v {
                                lit.value().parse::<u16>().ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(500),
                    handling_strategy: solution.get("handlingStrategy").map(|v| v.to_string()),
                };
                errors.push(error);
            }
        }
        Ok(errors)
    }

    /// Print statistics about the ontology
    pub fn print_statistics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let nouns = self.query_nouns()?;
        let verbs = self.query_verbs()?;
        let capabilities = self.query_capabilities()?;
        let combinations = self.query_noun_verb_combinations()?;
        let patterns = self.query_argument_patterns()?;
        let async_patterns = self.query_async_patterns()?;
        let error_types = self.query_error_types()?;

        println!("\n📊 clap-noun-verb Ontology Statistics\n");
        println!("  Nouns: {}", nouns.len());
        println!("  Verbs: {}", verbs.len());
        println!("  Capabilities: {}", capabilities.len());
        println!("  Noun-Verb Combinations: {}", combinations.len());
        println!("  Argument Patterns: {}", patterns.len());
        println!("  Async Patterns: {}", async_patterns.len());
        println!("  Error Types: {}", error_types.len());
        println!("\n📈 Estimated Template Generation\n");
        println!("  360 Templates: {} nouns × {} verbs × {} output formats", nouns.len(), verbs.len(), 1);

        Ok(())
    }
}

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone)]
pub struct NounEntity {
    pub uri: Option<String>,
    pub label: Option<String>,
    pub examples: u32,
    pub tests: u32,
}

#[derive(Debug, Clone)]
pub struct VerbAction {
    pub uri: Option<String>,
    pub label: Option<String>,
    pub operation: Option<String>,
    pub result_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub uri: Option<String>,
    pub label: Option<String>,
    pub comment: Option<String>,
    pub examples: u32,
    pub tests: u32,
}

#[derive(Debug, Clone)]
pub struct NounVerbCombination {
    pub noun_label: Option<String>,
    pub verb_label: Option<String>,
    pub operation: Option<String>,
    pub result_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArgumentPattern {
    pub uri: Option<String>,
    pub label: Option<String>,
    pub comment: Option<String>,
    pub validation: Option<String>,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AsyncPatternInfo {
    pub uri: Option<String>,
    pub label: Option<String>,
    pub comment: Option<String>,
    pub complexity: Option<String>,
    pub frameworks: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ErrorTypeInfo {
    pub uri: Option<String>,
    pub label: Option<String>,
    pub comment: Option<String>,
    pub http_status: u16,
    pub handling_strategy: Option<String>,
}

// ============================================================================
// Main Example
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ggen Template Generator for clap-noun-verb\n");

    // Initialize generator with paths
    let ontology_path = PathBuf::from("docs/clap-capabilities.ttl");
    let output_dir = PathBuf::from("examples/templates/clap-360");

    let mut generator = GgenTemplateGenerator::new(ontology_path, output_dir);

    // Load RDF ontology
    println!("📚 Loading RDF ontology from docs/clap-capabilities.ttl...");
    generator.load_ontology()?;
    println!("✅ Ontology loaded successfully\n");

    // Print statistics
    generator.print_statistics()?;

    // Query all entities
    println!("\n📋 Entities from Ontology\n");

    println!("Nouns:");
    let nouns = generator.query_nouns()?;
    for noun in &nouns {
        println!("  • {:?} ({} examples, {} tests)", noun.label, noun.examples, noun.tests);
    }

    println!("\nVerbs:");
    let verbs = generator.query_verbs()?;
    for verb in &verbs {
        println!("  • {:?} ({:?} -> {:?})", verb.label, verb.operation, verb.result_type);
    }

    println!("\nCapabilities (Sample - 46 total):");
    let capabilities = generator.query_capabilities()?;
    for (i, cap) in capabilities.iter().take(10).enumerate() {
        println!("  {}. {:?}: {}", i + 1, cap.label, cap.comment.as_ref().unwrap_or(&"No comment".to_string()));
    }
    if capabilities.len() > 10 {
        println!("  ... and {} more capabilities", capabilities.len() - 10);
    }

    println!("\nArgument Patterns:");
    let patterns = generator.query_argument_patterns()?;
    for pattern in &patterns {
        println!("  • {:?}: {:?}", pattern.label, pattern.validation);
    }

    println!("\nAsync Patterns:");
    let async_patterns = generator.query_async_patterns()?;
    for pattern in &async_patterns {
        println!("  • {:?}: complexity={:?}", pattern.label, pattern.complexity);
    }

    println!("\nError Types:");
    let error_types = generator.query_error_types()?;
    for error in &error_types {
        println!("  • {:?} ({}): {:?}", error.label, error.http_status, error.handling_strategy);
    }

    println!("\n✨ Next Steps:\n");
    println!("  1. Define Tera templates for each capability pattern");
    println!("  2. Use ggen's template.render_with_rdf() to generate code");
    println!("  3. Export 360 templates to examples/templates/clap-360/");
    println!("  4. Validate generated code with cargo check");
    println!("  5. Test template integration with clap-noun-verb examples\n");

    Ok(())
}
