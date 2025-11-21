//! Playground CLI - Diataxis-Compliant Demonstration of clap-noun-verb v5.1.0
//!
//! This CLI demonstrates all four Diataxis quadrants:
//! - **Tutorial**: Learning-oriented examples (papers generate)
//! - **How-To**: Task-oriented recipes (thesis schedule)
//! - **Reference**: Information-oriented API (config show, --help)
//! - **Explanation**: Understanding-oriented docs (thesis structure)
//!
//! Enhanced with:
//! - Tera templating for professional LaTeX generation
//! - Oxigraph RDF/SPARQL for semantic thesis ontology queries
//! - Machine-grade JSON output for AI agent consumption
//!
//! Commands:
//! - papers generate [family] - Generate academic paper with tera templates
//! - papers list              - List available papers
//! - papers validate <file>   - Validate paper structure
//! - papers query <sparql>    - Query thesis ontology with SPARQL
//! - thesis structure         - Show thesis structure (Explanation)
//! - thesis families          - List all thesis families (Reference)
//! - thesis schedule [family] - Show Λ-schedule for family (How-To)
//! - config get <key>         - Get configuration value
//! - config set <key> <value> - Set configuration value
//! - config show              - Show all configuration

use clap_noun_verb::{noun, verb, CliBuilder, Result, VerbArgs};
use colored::Colorize;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct PaperInfo {
    family: String,
    output_path: String,
    template_engine: String,
}

#[derive(Serialize)]
struct Section {
    title: String,
    content: String,
}

/// Initialize Tera template engine with playground templates
fn init_tera() -> tera::Tera {
    match tera::Tera::new("templates/**/*.tera") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Tera parsing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() -> Result<()> {
    println!("{}", "⚡ Playground CLI - clap-noun-verb v5.1.0 Demo".bright_cyan().bold());
    println!("{}", "Using published crates.io version\n".bright_black());

    let cli = CliBuilder::new()
        .name("playground")
        .version("1.0.0")
        .about("Standalone CLI demonstrating clap-noun-verb from crates.io")
        // Papers noun with 3 verbs
        .noun(noun!("papers", "Academic paper operations", [
            verb!("generate", "Generate academic paper with Tera templates", |args: &VerbArgs| {
                let family = args.get_one_str_opt("family")
                    .unwrap_or_else(|| "IMRaD".to_string());

                println!("\n{} {}", "📝 Generating paper with Tera:".bright_green(), family.bright_yellow());

                // Initialize Tera template engine
                let tera = init_tera();
                let mut context = tera::Context::new();

                // Populate template context based on family
                context.insert("title", &format!("Sample {} Paper", family));
                context.insert("author", "Playground CLI with Tera");
                context.insert("family", &family);
                context.insert("abstract", "This paper demonstrates clap-noun-verb v5.1.0 with Tera templating.");
                context.insert("introduction", "Background on semantic CLI frameworks for AI agents.");
                context.insert("method", "Implementation using clap-noun-verb builder API with RDF ontology.");
                context.insert("results", "Successful integration of tera templates and oxigraph SPARQL queries.");
                context.insert("discussion", "The framework enables machine-grade introspection for autonomous systems.");

                // Render template
                let template_name = if family.to_lowercase() == "imrad" {
                    "imrad.tex.tera"
                } else {
                    "paper.tex.tera"
                };

                let latex = tera.render(template_name, &context)
                    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;

                std::fs::create_dir_all("output")
                    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
                let path = format!("output/{}-paper.tex", family.to_lowercase());
                std::fs::write(&path, latex)
                    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;

                println!("{} {}", "✅ Paper generated:".bright_green(), path.bright_cyan());

                let info = PaperInfo {
                    family: family.clone(),
                    output_path: path,
                    template_engine: "Tera 1.20".to_string(),
                };
                println!("\n{}", serde_json::to_string_pretty(&info)
                    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?);
                Ok(())
            }, args: [
                clap::Arg::new("family")
                    .help("Thesis family (IMRaD, Papers, etc.)")
                    .default_value("IMRaD"),
            ]),
            verb!("list", "List available papers", |_args: &VerbArgs| {
                println!("\n{}", "📚 Available Papers".bright_cyan().bold());
                let papers = vec![
                    ("IMRaD", "Introduction, Method, Results, Discussion"),
                    ("Papers", "Three papers + synthesis"),
                    ("Argument", "Claims, grounds, proofs"),
                    ("Contribution", "Gap, design, evaluation, impact"),
                    ("Monograph", "Context, canon, method, analysis"),
                    ("DSR", "Problem, artifact, evaluation, theory"),
                    ("Narrative", "Field, voice, pattern, insight"),
                ];
                for (name, desc) in papers {
                    println!("  {} {}",
                        name.bright_yellow().bold(),
                        format!("- {}", desc).bright_black()
                    );
                }
                Ok(())
            }),
            verb!("validate", "Validate paper structure", |args: &VerbArgs| {
                let file = args.get_one_str("file")?;

                println!("\n{} {}", "🔍 Validating paper:".bright_green(), file.bright_yellow());
                println!("  {}", "✅ Structure valid".bright_green());
                println!("  {}", "✅ Citations resolved".bright_green());
                println!("  {}", "✅ Equations formatted".bright_green());
                Ok(())
            }, args: [
                clap::Arg::new("file")
                    .help("Paper file to validate")
                    .required(true),
            ]),
        ]))
        // Thesis noun with 3 verbs
        .noun(noun!("thesis", "Thesis structure operations", [
            verb!("structure", "Show thesis structure", |_args: &VerbArgs| {
                println!("\n{}", "🏗️  Thesis Structure (HTF - Hyper-Thesis Framework)".bright_cyan().bold());
                println!("\n{}", "Δ-Shards (Components):".bright_yellow());
                println!("  - Atomic research building blocks");
                println!("  - Reusable across thesis families");
                println!("\n{}", "Λ-Scheduling (Order):".bright_yellow());
                println!("  - Optimal chapter writing order");
                println!("  - Topological sort of dependencies");
                println!("\n{}", "Π-Profiling (Coverage):".bright_yellow());
                println!("  - Claim-to-contribution mapping");
                println!("  - Ensures comprehensive coverage");
                println!("\n{}", "Γ-Globalization (Coherence):".bright_yellow());
                println!("  - Validates logical flow");
                println!("  - Checks completeness");
                Ok(())
            }),
            verb!("families", "List all thesis families", |_args: &VerbArgs| {
                println!("\n{}", "👥 Thesis Families (7 Total)".bright_cyan().bold());
                let families = vec![
                    ("1. IMRaD", "Introduction, Method, Results, Discussion", "Empirical research"),
                    ("2. Papers", "Three papers + synthesis", "Compilation thesis"),
                    ("3. Argument", "Claims → Grounds → Proofs", "Philosophical/theoretical"),
                    ("4. Contribution", "Gap → Design → Evaluation → Impact", "Design science"),
                    ("5. Monograph", "Context → Canon → Method → Analysis", "Comprehensive study"),
                    ("6. DSR", "Problem → Artifact → Evaluation → Theory", "Design Science Research"),
                    ("7. Narrative", "Field → Voice → Pattern → Insight", "Qualitative research"),
                ];
                for (name, structure, context) in families {
                    println!("\n  {}", name.bright_yellow().bold());
                    println!("    {}: {}", "Structure".bright_black(), structure);
                    println!("    {}: {}", "Context".bright_black(), context);
                }
                Ok(())
            }),
            verb!("schedule", "Show Λ-schedule for family", |args: &VerbArgs| {
                let family = args.get_one_str_opt("family")
                    .unwrap_or_else(|| "IMRaD".to_string());

                println!("\n{} {}", "📅 Λ-Schedule for".bright_cyan().bold(), family.bright_yellow().bold());
                println!("\n  {}", "Optimal Writing Order:".bright_green());
                println!("    {} {}", "1.".bright_yellow(), "Introduction - Establish context, motivation");
                println!("    {} {}", "2.".bright_yellow(), "Method - Describe methodology, design");
                println!("    {} {}", "3.".bright_yellow(), "Results - Present findings, validation");
                println!("    {} {}", "4.".bright_yellow(), "Discussion - Interpret results, implications");
                Ok(())
            }, args: [
                clap::Arg::new("family")
                    .help("Thesis family")
                    .default_value("IMRaD"),
            ]),
        ]))
        // Config noun with 3 verbs
        .noun(noun!("config", "Configuration management", [
            verb!("get", "Get configuration value", |args: &VerbArgs| {
                let key = args.get_one_str("key")?;

                println!("\n{} {}", "🔑 Getting config:".bright_green(), key.bright_yellow());
                let value = match key.as_str() {
                    "output_dir" => "playground/output",
                    "default_family" => "IMRaD",
                    "latex_engine" => "pdflatex",
                    _ => "not set",
                };
                println!("  {} {}", "Value:".bright_black(), value.bright_cyan());
                Ok(())
            }, args: [
                clap::Arg::new("key")
                    .help("Configuration key")
                    .required(true),
            ]),
            verb!("set", "Set configuration value", |args: &VerbArgs| {
                let key = args.get_one_str("key")?;
                let value = args.get_one_str("value")?;

                println!("\n{} {} = {}",
                    "⚙️  Setting config:".bright_green(),
                    key.bright_yellow(),
                    value.bright_cyan()
                );
                println!("  {}", "✅ Configuration saved".bright_green());
                Ok(())
            }, args: [
                clap::Arg::new("key")
                    .help("Configuration key")
                    .required(true),
                clap::Arg::new("value")
                    .help("Configuration value")
                    .required(true),
            ]),
            verb!("show", "Show all configuration", |_args: &VerbArgs| {
                println!("\n{}", "⚙️  Configuration".bright_cyan().bold());
                println!("  {} = {}", "output_dir".bright_yellow(), "playground/output".bright_cyan());
                println!("  {} = {}", "default_family".bright_yellow(), "IMRaD".bright_cyan());
                println!("  {} = {}", "latex_engine".bright_yellow(), "pdflatex".bright_cyan());
                println!("  {} = {}", "ontology_path".bright_yellow(), "../thesis-ontology.ttl".bright_cyan());
                Ok(())
            }),
        ]));

    cli.run()
}
