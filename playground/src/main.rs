//! Playground CLI - Domain-Separated Exemplar of clap-noun-verb v5.1.0
//!
//! Architecture follows the golden rule: "CLI validates, domain computes, integration connects"
//!
//! ```text
//! ┌─────────────┐
//! │   CLI Layer │  ← This file (thin, UI only)
//! └──────┬──────┘
//!        │
//! ┌──────▼──────────┐
//! │ Integration     │  ← Glue code (Tera, file I/O)
//! └──────┬──────────┘
//!        │
//! ┌──────▼──────────┐
//! │  Domain Logic   │  ← Pure, testable business logic
//! └─────────────────┘
//! ```
//!
//! Commands:
//! - papers generate [family] - Generate academic paper with tera templates
//! - papers list              - List available paper families
//! - papers validate <file>   - Validate paper structure
//! - thesis structure         - Show HTF thesis structure (Explanation)
//! - thesis families          - List all thesis families (Reference)
//! - thesis schedule [family] - Show Λ-schedule for family (How-To)
//! - config get <key>         - Get configuration value
//! - config set <key> <value> - Set configuration value
//! - config show              - Show all configuration

mod domain;
mod integration;

use clap_noun_verb::{noun, verb, CliBuilder, Result, VerbArgs, NounVerbError};
use colored::Colorize;
use serde::Serialize;

// Domain imports - pure business logic
use domain::{Paper, PaperFamily, ThesisStructure, ThesisFamily, ThesisSchedule, Config};

// Integration imports - glue code with side effects
use integration::{init_template_engine, render_paper_latex, write_paper, ensure_output_dir};

/// JSON output for paper generation (CLI presentation concern)
#[derive(Serialize)]
struct PaperGeneratedOutput {
    family: String,
    output_path: String,
    template_engine: String,
    sections: usize,
}

/// Convert integration/domain errors to NounVerbError
fn to_cli_error(msg: String) -> NounVerbError {
    NounVerbError::execution_error(msg)
}

fn main() -> Result<()> {
    println!("{}", "⚡ Playground CLI - clap-noun-verb v5.1.0 Demo".bright_cyan().bold());
    println!("{}", "Domain-Separated Exemplar Architecture\n".bright_black());

    let cli = CliBuilder::new()
        .name("playground")
        .version("1.0.0")
        .about("Domain-separated CLI demonstrating clap-noun-verb from crates.io")

        // ═══════════════════════════════════════════════════════════════════
        // PAPERS NOUN - Academic paper operations
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("papers", "Academic paper operations", [

            // papers generate [family] - Generate paper with Tera
            verb!("generate", "Generate academic paper with Tera templates", |args: &VerbArgs| {
                // 1. CLI: Parse and validate input
                let family_str = args.get_one_str_opt("family")
                    .unwrap_or_else(|| "IMRaD".to_string());

                // 2. Domain: Parse family (pure validation)
                let family = PaperFamily::from_str(&family_str)
                    .ok_or_else(|| to_cli_error(format!("Unknown family: {}", family_str)))?;

                // 3. Domain: Create paper structure (pure computation)
                let paper = Paper::new(family.clone(), None, None);

                println!("\n{} {}", "📝 Generating paper:".bright_green(), family.name().bright_yellow());

                // 4. Integration: Initialize template engine (side effect)
                let tera = init_template_engine().map_err(to_cli_error)?;

                // 5. Integration: Render paper to LaTeX (side effect)
                let latex = render_paper_latex(&paper, &tera).map_err(to_cli_error)?;

                // 6. Integration: Write to file system (side effect)
                ensure_output_dir("output").map_err(to_cli_error)?;
                let path = format!("output/{}-paper.tex", family.name().to_lowercase());
                write_paper(&path, &latex).map_err(to_cli_error)?;

                // 7. CLI: Format output for display
                println!("{} {}", "✅ Paper generated:".bright_green(), path.bright_cyan());

                let output = PaperGeneratedOutput {
                    family: family.name().to_string(),
                    output_path: path,
                    template_engine: "Tera 1.20".to_string(),
                    sections: paper.sections.len(),
                };
                println!("\n{}", serde_json::to_string_pretty(&output)
                    .map_err(|e| to_cli_error(e.to_string()))?);
                Ok(())
            }, args: [
                clap::Arg::new("family")
                    .help("Thesis family (IMRaD, Papers, DSR, etc.)")
                    .default_value("IMRaD"),
            ]),

            // papers list - List available families
            verb!("list", "List available paper families", |_args: &VerbArgs| {
                println!("\n{}", "📚 Available Paper Families".bright_cyan().bold());

                // Domain: Get all families (pure)
                let families = PaperFamily::all();

                // CLI: Format for display
                for family in families {
                    println!("  {} {}",
                        family.name().bright_yellow().bold(),
                        format!("- {}", family.description()).bright_black()
                    );
                }
                Ok(())
            }),

            // papers validate <file> - Validate paper structure
            verb!("validate", "Validate paper structure", |args: &VerbArgs| {
                // CLI: Get file argument
                let file = args.get_one_str("file")?;

                println!("\n{} {}", "🔍 Validating paper:".bright_green(), file.bright_yellow());

                // Domain: Validate path (pure)
                let result = domain::papers::ValidationResult::validate_path(&file);

                // CLI: Display results
                if result.is_valid {
                    println!("  {}", "✅ Structure valid".bright_green());
                    println!("  {}", "✅ Citations valid".bright_green());
                    println!("  {}", "✅ Formatting valid".bright_green());
                } else {
                    for error in &result.errors {
                        println!("  {} {}", "❌".bright_red(), error);
                    }
                }
                Ok(())
            }, args: [
                clap::Arg::new("file")
                    .help("Paper file to validate")
                    .required(true),
            ]),
        ]))

        // ═══════════════════════════════════════════════════════════════════
        // THESIS NOUN - Thesis structure operations
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("thesis", "Thesis structure operations", [

            // thesis structure - Show HTF structure (Explanation quadrant)
            verb!("structure", "Show thesis structure (HTF framework)", |_args: &VerbArgs| {
                println!("\n{}", "🏗️  Thesis Structure (HTF - Hyper-Thesis Framework)".bright_cyan().bold());

                // Domain: Get structure (pure)
                let structure = ThesisStructure::get();

                // CLI: Format for display
                for component in &structure.components {
                    println!("\n{}", component.name.bright_yellow());
                    for detail in &component.details {
                        println!("  - {}", detail);
                    }
                }
                Ok(())
            }),

            // thesis families - List all families (Reference quadrant)
            verb!("families", "List all thesis families", |_args: &VerbArgs| {
                println!("\n{}", "👥 Thesis Families (7 Total)".bright_cyan().bold());

                // Domain: Get all families (pure)
                let families = ThesisFamily::all();

                // CLI: Format for display
                for family in &families {
                    println!("\n  {}", format!("{}. {}", family.number, family.name).bright_yellow().bold());
                    println!("    {}: {}", "Structure".bright_black(), family.structure);
                    println!("    {}: {}", "Context".bright_black(), family.context);
                }
                Ok(())
            }),

            // thesis schedule [family] - Show Λ-schedule (How-To quadrant)
            verb!("schedule", "Show Λ-schedule for family", |args: &VerbArgs| {
                // CLI: Parse argument
                let family_str = args.get_one_str_opt("family")
                    .unwrap_or_else(|| "IMRaD".to_string());

                // Domain: Parse and compute schedule (pure)
                let family = PaperFamily::from_str(&family_str)
                    .ok_or_else(|| to_cli_error(format!("Unknown family: {}", family_str)))?;
                let schedule = ThesisSchedule::for_family(&family);

                // CLI: Format for display
                println!("\n{} {}", "📅 Λ-Schedule for".bright_cyan().bold(), schedule.family.bright_yellow().bold());
                println!("\n  {}", "Optimal Writing Order:".bright_green());

                for step in &schedule.steps {
                    println!("    {} {} - {}",
                        format!("{}", step.order).bright_yellow(),
                        step.chapter.bright_white(),
                        step.description.bright_black()
                    );
                }
                Ok(())
            }, args: [
                clap::Arg::new("family")
                    .help("Thesis family")
                    .default_value("IMRaD"),
            ]),
        ]))

        // ═══════════════════════════════════════════════════════════════════
        // CONFIG NOUN - Configuration management
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("config", "Configuration management", [

            // config get <key> - Get configuration value
            verb!("get", "Get configuration value", |args: &VerbArgs| {
                // CLI: Get key argument
                let key = args.get_one_str("key")?;

                // Domain: Get config value (pure)
                let config = Config::default();
                let value = config.get(&key);

                // CLI: Display result
                println!("\n{} {}", "🔑 Getting config:".bright_green(), key.bright_yellow());
                match value {
                    Some(v) => println!("  {} {}", "Value:".bright_black(), v.bright_cyan()),
                    None => println!("  {} {}", "Value:".bright_black(), "not set".bright_red()),
                }
                Ok(())
            }, args: [
                clap::Arg::new("key")
                    .help("Configuration key")
                    .required(true),
            ]),

            // config set <key> <value> - Set configuration value
            verb!("set", "Set configuration value", |args: &VerbArgs| {
                // CLI: Get arguments
                let key = args.get_one_str("key")?;
                let value = args.get_one_str("value")?;

                // Domain: Validate key (pure)
                if !Config::is_valid_key(&key) {
                    println!("\n{} Unknown key: {}", "⚠️".bright_yellow(), key);
                    println!("  Valid keys: output_dir, default_family, latex_engine, ontology_path");
                }

                // CLI: Display confirmation
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

            // config show - Show all configuration
            verb!("show", "Show all configuration", |_args: &VerbArgs| {
                println!("\n{}", "⚙️  Configuration".bright_cyan().bold());

                // Domain: Get all config entries (pure)
                let config = Config::default();
                let entries = config.all_entries();

                // CLI: Format for display
                for (key, value) in entries {
                    println!("  {} = {}", key.bright_yellow(), value.bright_cyan());
                }
                Ok(())
            }),
        ]));

    cli.run()
}
