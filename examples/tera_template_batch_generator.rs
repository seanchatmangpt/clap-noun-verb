//! Batch Template Generator for clap-noun-verb using Tera + SPARQL
//!
//! This script generates all 360 templates by:
//! 1. Querying RDF ontology for noun-verb combinations
//! 2. Loading Tera template
//! 3. Rendering template with each combination
//! 4. Saving output files to clap-360 directory
//!
//! Status: Foundation for template generation pipeline
//! Note: Requires full ggen integration for SPARQL + RDF capabilities

use oxigraph::store::Store;
use oxigraph::sparql::QueryResults;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// SPARQL Query for All Noun-Verb Combinations
// ============================================================================

const SPARQL_ALL_COMBINATIONS: &str = r#"
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

// ============================================================================
// Template Rendering Engine
// ============================================================================

pub struct TeraTemplateGenerator {
    store: Store,
    ontology_path: PathBuf,
    template_path: PathBuf,
    output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub noun: String,
    pub verb: String,
    pub operation: String,
    pub result_type: String,
    pub example_name: String,
}

impl TeraTemplateGenerator {
    pub fn new(ontology_path: PathBuf, template_path: PathBuf, output_dir: PathBuf) -> Self {
        TeraTemplateGenerator { store: Store::new().expect("Failed to create RDF store"), ontology_path, template_path, output_dir }
    }

    /// Load RDF ontology
    pub fn load_ontology(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.ontology_path)?;
        let turtle = oxigraph::io::parse(content.as_bytes(), oxigraph::io::DatasetFormat::Turtle)?;
        for quad in turtle {
            self.store.insert(&quad?);
        }
        Ok(())
    }

    /// Query all noun-verb combinations
    pub fn query_combinations(&self) -> Result<Vec<TemplateContext>, Box<dyn std::error::Error>> {
        let results = self.store.query(SPARQL_ALL_COMBINATIONS)?;
        let mut contexts = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;

                let noun = solution
                    .get("nounLabel")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let verb = solution
                    .get("verbLabel")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let operation =
                    solution.get("operation").map(|v| v.to_string()).unwrap_or_else(|| "unknown".to_string());
                let result_type = solution
                    .get("resultType")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "result".to_string());

                // Generate example name (snake_case from noun-verb)
                let example_name = format!(
                    "{}_{}",
                    noun.to_lowercase().replace(" ", "_"),
                    verb.to_lowercase().replace(" ", "_")
                );

                contexts.push(TemplateContext {
                    noun: noun.replace(" ", ""),
                    verb: verb.replace(" ", ""),
                    operation,
                    result_type,
                    example_name,
                });
            }
        }

        Ok(contexts)
    }

    /// Generate templates in batch
    pub fn generate_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Query combinations
        let contexts = self.query_combinations()?;

        println!("🚀 Generating {} templates...\n", contexts.len());

        let mut success_count = 0;
        let mut error_count = 0;

        for context in contexts {
            match self.generate_template(&context) {
                Ok(output_path) => {
                    println!("✅ Generated: {}", output_path.display());
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ Failed: {} - {}", context.example_name, e);
                    error_count += 1;
                }
            }
        }

        println!("\n📊 Generation Summary");
        println!("  ✅ Success: {}", success_count);
        println!("  ❌ Failed: {}", error_count);
        println!("  Total: {}\n", success_count + error_count);

        Ok(())
    }

    /// Generate single template
    fn generate_template(&self, context: &TemplateContext) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Load Tera template
        let template_content = fs::read_to_string(&self.template_path)?;

        // Create simple template context (would use Tera in full implementation)
        let output_content = self.render_simple_template(&template_content, context);

        // Generate output filename
        let output_filename = format!("{}_{}_.rs", context.noun.to_lowercase(), context.verb.to_lowercase());
        let output_path = self.output_dir.join(&output_filename);

        // Write to file
        fs::write(&output_path, output_content)?;

        Ok(output_path)
    }

    /// Simple template rendering (would be Tera in full implementation)
    fn render_simple_template(&self, template: &str, context: &TemplateContext) -> String {
        template
            .replace("{{noun}}", &context.noun)
            .replace("{{verb}}", &context.verb)
            .replace("{{operation}}", &context.operation)
            .replace("{{result_type}}", &context.result_type)
            .replace("{{example_name}}", &context.example_name)
            .replace("{{noun | lowercase}}", &context.noun.to_lowercase())
            .replace("{{verb | lowercase}}", &context.verb.to_lowercase())
    }
}

// ============================================================================
// Batch Generator Command
// ============================================================================

pub fn run_batch_generation(
    ontology_path: &Path,
    template_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧬 Tera + SPARQL Batch Template Generator\n");

    // Initialize generator
    let mut generator = TeraTemplateGenerator::new(ontology_path.to_path_buf(), template_path.to_path_buf(), output_dir.to_path_buf());

    // Load ontology
    println!("📚 Loading RDF ontology...");
    generator.load_ontology()?;
    println!("✅ Ontology loaded\n");

    // Generate templates
    generator.generate_batch()?;

    Ok(())
}

// ============================================================================
// Main Example
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Paths
    let ontology_path = PathBuf::from("docs/clap-capabilities.ttl");
    let template_path = PathBuf::from("examples/templates/noun_verb_command.tera");
    let output_dir = PathBuf::from("examples/templates/clap-360");

    // Verify paths exist
    if !ontology_path.exists() {
        eprintln!("❌ Ontology not found: {}", ontology_path.display());
        eprintln!("   Run this from the clap-noun-verb project root directory");
        return Ok(());
    }

    if !template_path.exists() {
        eprintln!("❌ Template not found: {}", template_path.display());
        eprintln!("   Run this from the clap-noun-verb project root directory");
        return Ok(());
    }

    // Run batch generation
    run_batch_generation(&ontology_path, &template_path, &output_dir)?;

    println!("✨ Template generation complete!\n");
    println!("📁 Output directory: {}\n", output_dir.display());
    println!("Next steps:");
    println!("  1. Verify generated templates in {}", output_dir.display());
    println!("  2. Run: cargo build --examples to compile");
    println!("  3. Test: cargo test --example '*' to validate");
    println!("  4. Integrate templates with clap-noun-verb framework");

    Ok(())
}
