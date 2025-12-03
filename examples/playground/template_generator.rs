//! Template-based semantic CLI command generator using ggen patterns
//!
//! This example demonstrates how to use ggen's template system to generate
//! semantic CLI commands from a declarative ontology definition. Instead of
//! manually writing commands for each venue, we define the venue metadata
//! in a YAML file and use Handlebars templates to generate code.
//!
//! # Example Usage
//!
//! ```bash
//! cargo run --example template_generator -- generate icse
//! cargo run --example template_generator -- generate ecsa --format json
//! cargo run --example template_generator -- list
//! ```

use clap::{Parser, Subcommand};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Template-based semantic CLI command generator
#[derive(Debug, Parser)]
#[command(name = "template-generator")]
#[command(about = "Generate semantic CLI commands from ggen templates")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate a command for a specific venue
    Generate {
        /// Venue identifier (icse, ecsa, pldi_oopsla, ase_workshop, arxiv)
        venue: String,

        /// Output format (rust, markdown, json)
        #[arg(long, default_value = "rust")]
        format: String,

        /// Output file (default: stdout)
        #[arg(long, short)]
        output: Option<String>,
    },

    /// List all available venues in ontology
    List {
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Show ontology metadata for a venue
    Show {
        /// Venue identifier
        venue: String,

        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Validate ontology YAML file
    Validate {
        /// Path to ontology YAML file
        #[arg(default_value = "examples/templates/semantic_ontology.yaml")]
        ontology_file: String,
    },
}

/// Ontology definition loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ontology {
    namespace: String,
    version: String,
    description: String,
    entities: HashMap<String, Entity>,
    venues: HashMap<String, VenueMetadata>,
    properties: HashMap<String, Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entity {
    uri: String,
    description: String,
    properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VenueMetadata {
    name: String,
    venue_key: String,
    pascal_case: String,
    track: String,
    page_limit: u32,
    acceptance_probability: u32,
    emphasis: Vec<String>,
    required_sections: Vec<String>,
    optional_sections: Vec<String>,
    constraints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Property {
    #[serde(rename = "type")]
    property_type: String,
    cardinality: String,
    description: String,
}

/// Load ontology from YAML file
fn load_ontology(path: &str) -> anyhow::Result<Ontology> {
    let content = fs::read_to_string(path)?;
    let ontology: Ontology = serde_yaml::from_str(&content)?;
    Ok(ontology)
}

/// Generate Rust code for a venue using templates
fn generate_command(
    ontology: &Ontology,
    venue_key: &str,
    template: &str,
) -> anyhow::Result<String> {
    let venue = ontology
        .venues
        .get(venue_key)
        .ok_or_else(|| anyhow::anyhow!("Venue not found: {}", venue_key))?;

    let mut handlebars = Handlebars::new();

    // Build template context - pre-compute pascal_case version
    let mut context = serde_json::json!(venue);

    // Add venue_name alias for template (template uses venue_name, struct has name)
    context["venue_name"] = serde_json::json!(&venue.name);

    // Add pascal_case venue key for template
    let pascal_venue_key = venue
        .venue_key
        .split('_')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();
    context["pascal_venue_key"] = serde_json::json!(pascal_venue_key);

    // Add derived values
    context["namespace"] = serde_json::json!(ontology.namespace);
    context["ontology_version"] = serde_json::json!(ontology.version);

    // Render template
    let rendered = handlebars.render_template(template, &context)?;
    Ok(rendered)
}

/// Load template from file
fn load_template(path: &str) -> anyhow::Result<String> {
    fs::read_to_string(path).map_err(|e| anyhow::anyhow!("Failed to load template: {}", e))
}

/// Main command handler
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Generate { venue, format: _, output } => {
            // Load ontology
            let ontology = load_ontology("examples/templates/semantic_ontology.yaml")?;

            // Load template
            let template = load_template("examples/templates/semantic_projection_command.rs.hbs")?;

            // Generate code
            let generated = generate_command(&ontology, &venue, &template)?;

            // Output
            if let Some(path) = output {
                fs::write(&path, &generated)?;
                println!("✅ Generated {} to {}", venue, path);
            } else {
                println!("{}", generated);
            }
        }

        Commands::List { format: format_type } => {
            let ontology = load_ontology("examples/templates/semantic_ontology.yaml")?;

            match format_type.as_str() {
                "text" => {
                    println!("🗂️  Available Venues:");
                    println!();
                    for (key, venue) in &ontology.venues {
                        println!(
                            "  {} ({}%) - {}",
                            venue.name, venue.acceptance_probability, venue.track
                        );
                        println!("    Key: {}", key);
                        println!("    Pages: {}", venue.page_limit);
                        println!();
                    }
                }
                "json" => {
                    let venues: Vec<_> = ontology.venues.iter().collect();
                    println!("{}", serde_json::to_string_pretty(&venues)?);
                }
                _ => anyhow::bail!("Unknown format: {}", format_type),
            }
        }

        Commands::Show { venue, format: format_type } => {
            let ontology = load_ontology("examples/templates/semantic_ontology.yaml")?;

            let venue_meta = ontology
                .venues
                .get(&venue)
                .ok_or_else(|| anyhow::anyhow!("Venue not found: {}", venue))?;

            match format_type.as_str() {
                "text" => {
                    println!("📋 {} Metadata", venue_meta.name);
                    println!();
                    println!("Track: {}", venue_meta.track);
                    println!("Page Limit: {}", venue_meta.page_limit);
                    println!("Acceptance Probability: {}%", venue_meta.acceptance_probability);
                    println!();
                    println!("Key Emphasis:");
                    for emphasis in &venue_meta.emphasis {
                        println!("  • {}", emphasis);
                    }
                    println!();
                    println!("Required Sections: {}", venue_meta.required_sections.len());
                    for section in &venue_meta.required_sections {
                        println!("  • {}", section);
                    }
                    if !venue_meta.optional_sections.is_empty() {
                        println!();
                        println!("Optional Sections: {}", venue_meta.optional_sections.len());
                        for section in &venue_meta.optional_sections {
                            println!("  • {}", section);
                        }
                    }
                    println!();
                    println!("Constraints:");
                    for (key, value) in &venue_meta.constraints {
                        println!("  {}: {}", key, value);
                    }
                }
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&venue_meta)?);
                }
                _ => anyhow::bail!("Unknown format: {}", format_type),
            }
        }

        Commands::Validate { ontology_file } => match load_ontology(&ontology_file) {
            Ok(ontology) => {
                println!("✅ Ontology is valid");
                println!();
                println!("Namespace: {}", ontology.namespace);
                println!("Version: {}", ontology.version);
                println!("Description: {}", ontology.description);
                println!();
                println!("Entities: {}", ontology.entities.len());
                println!("Venues: {}", ontology.venues.len());
                println!("Properties: {}", ontology.properties.len());
            }
            Err(e) => {
                println!("❌ Ontology validation failed:");
                println!("{}", e);
                std::process::exit(1);
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_ontology() {
        let ontology = load_ontology("examples/templates/semantic_ontology.yaml");
        assert!(ontology.is_ok());

        let ontology = ontology.unwrap();
        assert!(!ontology.venues.is_empty());
        assert!(ontology.venues.contains_key("icse"));
    }

    #[test]
    fn test_venue_metadata() {
        let ontology = load_ontology("examples/templates/semantic_ontology.yaml").unwrap();
        let icse = &ontology.venues["icse"];

        assert_eq!(icse.name, "ICSE 2026");
        assert_eq!(icse.page_limit, 12);
        assert!(icse.acceptance_probability >= 60);
    }

    #[test]
    fn test_pascal_case_conversion() {
        let input = "hello_world_test";
        let expected = "HelloWorldTest";

        // Manual implementation for test
        let result = input
            .split('_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        assert_eq!(result, expected);
    }
}
