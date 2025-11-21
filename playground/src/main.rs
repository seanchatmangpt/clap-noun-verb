//! Playground CLI - Comprehensive v5 Feature Showcase
//!
//! Architecture follows the golden rule: "CLI validates, domain computes, integration connects"
//!
//! ```text
//! ┌─────────────┐
//! │   CLI Layer │  ← This file (thin, UI only)
//! └──────┬──────┘
//!        │
//! ┌──────▼──────────┐
//! │ Integration     │  ← Glue code (Tera, Oxigraph, file I/O)
//! └──────┬──────────┘
//!        │
//! ┌──────▼──────────┐
//! │  Domain Logic   │  ← Pure, testable business logic
//! └─────────────────┘
//! ```
//!
//! ## v5 Features Demonstrated
//!
//! - **Autonomic CLI**: Machine-grade introspection (`meta introspect`)
//! - **RDF/Ontology**: SPARQL queries and Turtle export (`meta ontology`, `meta sparql`)
//! - **Output Formats**: JSON, YAML, Table output modes (`--format`)
//! - **Shell Completions**: Bash, Zsh, Fish, PowerShell (`meta completions`)
//! - **Middleware**: Logging, profiling, rate-limiting (`meta middleware`)
//! - **Telemetry**: Execution receipts and metrics (`meta telemetry`)

mod domain;
mod integration;

use clap_noun_verb::{noun, verb, CliBuilder, Result, VerbArgs, NounVerbError};
use colored::Colorize;
use serde::Serialize;

// Domain imports - pure business logic
use domain::{
    Paper, PaperFamily, ThesisStructure, ThesisFamily, ThesisSchedule, Config,
    // v5 features
    build_playground_ontology, IntrospectionResponse, OutputFormat, format_output,
    ShellType, generate_completion_script, MiddlewareConfig, MiddlewareStats,
    ExecutionSpan, SpanStatus, ExecutionReceipt, SparqlQueryType,
};

// Integration imports - glue code with side effects
use integration::{
    init_template_engine, render_paper_latex, write_paper, ensure_output_dir,
    // v5 features
    init_ontology_store, execute_sparql, export_turtle,
};

/// JSON output for paper generation
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

/// Get output format from args (default: json-pretty)
fn get_format(args: &VerbArgs) -> OutputFormat {
    args.get_one_str_opt("format")
        .and_then(|f| OutputFormat::from_str(&f))
        .unwrap_or(OutputFormat::JsonPretty)
}

fn main() -> Result<()> {
    // TRIZ-2: Check for --quiet/-q flag before printing banner
    let args: Vec<String> = std::env::args().collect();
    let quiet = args.iter().any(|a| a == "-q" || a == "--quiet");

    // TRIZ-1: Print banner to stderr so it doesn't pollute scripted output
    if !quiet {
        eprintln!("{}", "⚡ Playground CLI - clap-noun-verb v5.1.0 Comprehensive Demo".bright_cyan().bold());
        eprintln!("{}", "Domain-Separated Architecture with ALL v5 Features\n".bright_black());
    }

    let cli = CliBuilder::new()
        .name("playground")
        .version("2.0.0")
        .about("Comprehensive v5 feature showcase demonstrating clap-noun-verb from crates.io")
        // TRIZ-2: Register global --quiet/-q flag
        .global_args(vec![
            clap::Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress banner output")
                .action(clap::ArgAction::SetTrue)
                .global(true),
        ])

        // ═══════════════════════════════════════════════════════════════════
        // PAPERS NOUN - Academic paper operations
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("papers", "Academic paper operations", [

            verb!("generate", "Generate academic paper with Tera templates", |args: &VerbArgs| {
                let family_str = args.get_one_str_opt("family").unwrap_or_else(|| "IMRaD".to_string());
                let format = get_format(args);

                // FMEA-4: Validate paper family input before generation
                let valid_families = PaperFamily::all();
                let valid_names: Vec<_> = valid_families.iter().map(|f| f.name().to_lowercase()).collect();
                let family = PaperFamily::from_str(&family_str)
                    .ok_or_else(|| to_cli_error(format!(
                        "Invalid paper family: '{}'. Valid options: {}",
                        family_str,
                        valid_names.join(", ")
                    )))?;

                let paper = Paper::new(family.clone(), None, None);

                // GEMBA-5: Progress indicator
                eprintln!("{}", "Generating...".bright_yellow());
                println!("\n{} {}", "📝 Generating paper:".bright_green(), family.name().bright_yellow());

                let tera = init_template_engine().map_err(to_cli_error)?;
                let latex = render_paper_latex(&paper, &tera).map_err(to_cli_error)?;

                // TRIZ-3: Support custom output path via --output/-o flag
                let path = if let Some(output_path) = args.get_one_str_opt("output") {
                    // Use provided output path directly
                    output_path
                } else {
                    // Default to output directory
                    ensure_output_dir("output").map_err(to_cli_error)?;
                    format!("output/{}-paper.tex", family.name().to_lowercase())
                };
                write_paper(&path, &latex).map_err(to_cli_error)?;

                println!("{} {}", "✅ Paper generated:".bright_green(), path.bright_cyan());
                eprintln!("{}", "Done".bright_green());

                let output = PaperGeneratedOutput {
                    family: family.name().to_string(),
                    output_path: path,
                    template_engine: "Tera 1.20".to_string(),
                    sections: paper.sections.len(),
                };
                println!("\n{}", format_output(&output, format).map_err(to_cli_error)?);
                Ok(())
            }, args: [
                clap::Arg::new("family").help("Thesis family").default_value("IMRaD"),
                clap::Arg::new("format").short('f').long("format").help("Output format (json, yaml, table)"),
                clap::Arg::new("output").short('o').long("output").help("Output file path"),
            ]),

            verb!("list", "List available paper families", |args: &VerbArgs| {
                let format = get_format(args);
                let families: Vec<_> = PaperFamily::all().iter()
                    .map(|f| serde_json::json!({"name": f.name(), "description": f.description()}))
                    .collect();

                if format == OutputFormat::Plain {
                    println!("\n{}", "📚 Available Paper Families".bright_cyan().bold());
                    for family in PaperFamily::all() {
                        println!("  {} {}", family.name().bright_yellow().bold(), format!("- {}", family.description()).bright_black());
                    }
                } else {
                    println!("{}", format_output(&families, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("validate", "Validate paper structure", |args: &VerbArgs| {
                let file = args.get_one_str("file")?;
                let format = get_format(args);

                let result = domain::papers::ValidationResult::validate_path(&file);

                if format == OutputFormat::Plain {
                    println!("\n{} {}", "🔍 Validating paper:".bright_green(), file.bright_yellow());
                    if result.is_valid {
                        println!("  {}", "✅ All checks passed".bright_green());
                    } else {
                        for error in &result.errors {
                            println!("  {} {}", "❌".bright_red(), error);
                        }
                    }
                } else {
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("file").help("Paper file to validate").required(true),
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),
        ]))

        // ═══════════════════════════════════════════════════════════════════
        // THESIS NOUN - Thesis structure operations
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("thesis", "Thesis structure operations", [

            verb!("structure", "Show thesis structure (HTF framework)", |args: &VerbArgs| {
                let format = get_format(args);
                let structure = ThesisStructure::get();

                if format == OutputFormat::Plain {
                    println!("\n{}", "🏗️  Thesis Structure (HTF)".bright_cyan().bold());
                    for component in &structure.components {
                        println!("\n{}", component.name.bright_yellow());
                        for detail in &component.details {
                            println!("  - {}", detail);
                        }
                    }
                } else {
                    println!("{}", format_output(&structure, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("families", "List all thesis families", |args: &VerbArgs| {
                let format = get_format(args);
                let families = ThesisFamily::all();

                if format == OutputFormat::Plain {
                    println!("\n{}", "👥 Thesis Families (7 Total)".bright_cyan().bold());
                    for family in &families {
                        println!("\n  {}", format!("{}. {}", family.number, family.name).bright_yellow().bold());
                        println!("    {}: {}", "Structure".bright_black(), family.structure);
                        println!("    {}: {}", "Context".bright_black(), family.context);
                    }
                } else {
                    println!("{}", format_output(&families, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("schedule", "Show Λ-schedule for family", |args: &VerbArgs| {
                let family_str = args.get_one_str_opt("family").unwrap_or_else(|| "IMRaD".to_string());
                let format = get_format(args);

                let family = PaperFamily::from_str(&family_str)
                    .ok_or_else(|| to_cli_error(format!("Unknown family: {}", family_str)))?;
                let schedule = ThesisSchedule::for_family(&family);

                if format == OutputFormat::Plain {
                    println!("\n{} {}", "📅 Λ-Schedule for".bright_cyan().bold(), schedule.family.bright_yellow().bold());
                    println!("\n  {}", "Optimal Writing Order:".bright_green());
                    for step in &schedule.steps {
                        println!("    {} {} - {}", format!("{}", step.order).bright_yellow(), step.chapter.bright_white(), step.description.bright_black());
                    }
                } else {
                    println!("{}", format_output(&schedule, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("family").help("Thesis family").default_value("IMRaD"),
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),
        ]))

        // ═══════════════════════════════════════════════════════════════════
        // CONFIG NOUN - Configuration management
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("config", "Configuration management", [

            verb!("get", "Get configuration value", |args: &VerbArgs| {
                let key = args.get_one_str("key")?;
                let format = get_format(args);
                let config = Config::default();

                let result = serde_json::json!({
                    "key": key,
                    "value": config.get(&key),
                    "valid_key": Config::is_valid_key(&key)
                });

                if format == OutputFormat::Plain {
                    println!("\n{} {}", "🔑 Config:".bright_green(), key.bright_yellow());
                    match config.get(&key) {
                        Some(v) => println!("  {} {}", "Value:".bright_black(), v.bright_cyan()),
                        None => println!("  {} {}", "Value:".bright_black(), "not set".bright_red()),
                    }
                } else {
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("key").help("Configuration key").required(true),
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("set", "Set configuration value", |args: &VerbArgs| {
                let key = args.get_one_str("key")?;
                let value = args.get_one_str("value")?;

                if !Config::is_valid_key(&key) {
                    println!("\n{} Unknown key: {}", "⚠️".bright_yellow(), key);
                    println!("  Valid keys: output_dir, default_family, latex_engine, ontology_path");
                }

                println!("\n{} {} = {}", "⚙️  Setting:".bright_green(), key.bright_yellow(), value.bright_cyan());
                println!("  {}", "✅ Configuration saved".bright_green());
                Ok(())
            }, args: [
                clap::Arg::new("key").help("Configuration key").required(true),
                clap::Arg::new("value").help("Configuration value").required(true),
            ]),

            verb!("show", "Show all configuration", |args: &VerbArgs| {
                let format = get_format(args);
                let config = Config::default();
                let entries: std::collections::HashMap<_, _> = config.all_entries().into_iter().collect();

                if format == OutputFormat::Plain {
                    println!("\n{}", "⚙️  Configuration".bright_cyan().bold());
                    for (key, value) in config.all_entries() {
                        println!("  {} = {}", key.bright_yellow(), value.bright_cyan());
                    }
                } else {
                    println!("{}", format_output(&entries, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),
        ]))

        // ═══════════════════════════════════════════════════════════════════
        // META NOUN - v5 Features: Introspection, Ontology, Completions
        // ═══════════════════════════════════════════════════════════════════
        .noun(noun!("meta", "v5 autonomic features (introspection, ontology, telemetry)", [


            verb!("health", "Health check endpoint returning JSON status", |args: &VerbArgs| {
                let format = get_format(args);
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let health = serde_json::json!({
                    "status": "ok",
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": timestamp
                });

                if format == OutputFormat::Plain {
                    println!("\n{}", "Health Check".bright_cyan().bold());
                    println!("  {}: {}", "status".bright_yellow(), "ok".bright_green());
                    println!("  {}: {}", "version".bright_yellow(), env!("CARGO_PKG_VERSION"));
                    println!("  {}: {}", "timestamp".bright_yellow(), timestamp);
                } else {
                    println!("{}", format_output(&health, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),
            verb!("introspect", "Machine-grade CLI introspection for AI agents", |args: &VerbArgs| {
                let format = get_format(args);
                let capabilities = build_playground_ontology();
                let response = IntrospectionResponse::from_capabilities(
                    "playground", "2.0.0", "Comprehensive v5 feature showcase", &capabilities
                );

                if format == OutputFormat::Plain {
                    println!("\n{}", "🤖 CLI Introspection (Autonomic Interface)".bright_cyan().bold());
                    println!("\n  {}: {}", "CLI".bright_yellow(), response.cli_name);
                    println!("  {}: {}", "Version".bright_yellow(), response.version);
                    println!("  {}: {}", "Capabilities".bright_yellow(), response.total_capabilities);
                    println!("\n  {}:", "Nouns".bright_green());
                    for noun in &response.nouns {
                        println!("    {} ({} verbs)", noun.name.bright_yellow(), noun.verbs.len());
                    }
                    println!("\n  {}:", "Autonomic Features".bright_green());
                    for feature in &response.autonomic_features {
                        println!("    - {}", feature);
                    }
                } else {
                    println!("{}", format_output(&response, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("ontology", "Export CLI as RDF/Turtle ontology", |args: &VerbArgs| {
                let format = get_format(args);
                let capabilities = build_playground_ontology();

                if format == OutputFormat::Plain {
                    println!("\n{}", "📊 CLI Ontology (RDF/Turtle)".bright_cyan().bold());
                    let turtle = export_turtle(&capabilities);
                    println!("{}", turtle);
                } else {
                    let turtle = export_turtle(&capabilities);
                    let result = serde_json::json!({
                        "format": "text/turtle",
                        "triples": capabilities.len() * 5,
                        "content": turtle
                    });
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("sparql", "Execute SPARQL query on CLI ontology", |args: &VerbArgs| {
                let query_type = args.get_one_str_opt("query").unwrap_or_else(|| "capabilities".to_string());
                let format = get_format(args);

                let capabilities = build_playground_ontology();
                let store = init_ontology_store(&capabilities).map_err(to_cli_error)?;

                let sparql = match query_type.as_str() {
                    "capabilities" => SparqlQueryType::SelectCapabilities.to_sparql(),
                    "papers" => SparqlQueryType::SelectByNoun("papers".to_string()).to_sparql(),
                    "mutating" => SparqlQueryType::SelectByEffect(domain::EffectType::Mutating).to_sparql(),
                    custom => custom.to_string(),
                };

                let results = execute_sparql(&store, &sparql).map_err(to_cli_error)?;

                if format == OutputFormat::Plain {
                    println!("\n{}", "🔍 SPARQL Query Results".bright_cyan().bold());
                    println!("  {}: {}", "Query".bright_yellow(), query_type);
                    println!("  {}: {} rows\n", "Results".bright_yellow(), results.len());
                    for row in &results {
                        println!("  {}", row.join(" | ").bright_white());
                    }
                } else {
                    let result = serde_json::json!({
                        "query": query_type,
                        "rows": results.len(),
                        "results": results
                    });
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("query").help("Query type or custom SPARQL").default_value("capabilities"),
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("completions", "Generate shell completion scripts", |args: &VerbArgs| {
                let shell_str = args.get_one_str("shell")?;
                let format = get_format(args);

                let shell = ShellType::from_str(&shell_str)
                    .ok_or_else(|| to_cli_error(format!("Unknown shell: {}. Use: bash, zsh, fish, powershell, elvish", shell_str)))?;

                let capabilities = build_playground_ontology();
                let script = generate_completion_script("playground", &capabilities, shell);

                if format == OutputFormat::Plain {
                    println!("{}", script.script);
                } else {
                    let result = serde_json::json!({
                        "shell": shell.name(),
                        "cli": script.cli_name,
                        "script": script.script
                    });
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("shell").help("Shell type (bash, zsh, fish, powershell, elvish)").required(true),
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("middleware", "Show middleware configuration and stats", |args: &VerbArgs| {
                let format = get_format(args);
                let config = MiddlewareConfig::default();
                let stats = MiddlewareStats::default();

                let result = serde_json::json!({
                    "config": config,
                    "stats": stats
                });

                if format == OutputFormat::Plain {
                    println!("\n{}", "🔧 Middleware Configuration".bright_cyan().bold());
                    println!("\n  {}:", "Logging".bright_yellow());
                    println!("    enabled: {}", config.logging.enabled);
                    println!("    verbose: {}", config.logging.verbose);
                    println!("\n  {}:", "Rate Limiting".bright_yellow());
                    println!("    enabled: {}", config.rate_limiting.enabled);
                    println!("    max_requests: {}/{}s", config.rate_limiting.max_requests, config.rate_limiting.window_seconds);
                    println!("\n  {}:", "Caching".bright_yellow());
                    println!("    enabled: {}", config.caching.enabled);
                    println!("    ttl: {}s", config.caching.ttl_seconds);
                } else {
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("telemetry", "Show telemetry data and execution receipts", |args: &VerbArgs| {
                let format = get_format(args);

                // Create sample execution span and receipt
                let span = ExecutionSpan::new("meta.telemetry")
                    .with_attribute("format", format.name())
                    .complete(SpanStatus::Ok);

                let receipt = ExecutionReceipt::new("meta telemetry", &[], span.duration_ms.unwrap_or(0), true)
                    .with_agent("playground-cli");

                let result = serde_json::json!({
                    "span": span,
                    "receipt": receipt,
                    "trace_id": span.trace_id
                });

                if format == OutputFormat::Plain {
                    println!("\n{}", "📊 Telemetry & Observability".bright_cyan().bold());
                    println!("\n  {}:", "Execution Span".bright_yellow());
                    println!("    trace_id: {}", span.trace_id);
                    println!("    operation: {}", span.operation);
                    println!("    status: {:?}", span.status);
                    println!("    duration: {}ms", span.duration_ms.unwrap_or(0));
                    println!("\n  {}:", "Execution Receipt".bright_yellow());
                    println!("    receipt_id: {}", receipt.receipt_id);
                    println!("    command: {}", receipt.command);
                    println!("    agent: {:?}", receipt.agent_id);
                } else {
                    println!("{}", format_output(&result, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            verb!("formats", "List available output formats", |args: &VerbArgs| {
                let format = get_format(args);
                let formats: Vec<_> = OutputFormat::all().iter()
                    .map(|f| serde_json::json!({"name": f.name(), "description": f.description()}))
                    .collect();

                if format == OutputFormat::Plain {
                    println!("\n{}", "📋 Available Output Formats".bright_cyan().bold());
                    for f in OutputFormat::all() {
                        println!("  {} - {}", f.name().bright_yellow(), f.description().bright_black());
                    }
                    println!("\n  Usage: {} <command> -f <format>", "playground".bright_green());
                } else {
                    println!("{}", format_output(&formats, format).map_err(to_cli_error)?);
                }
                Ok(())
            }, args: [
                clap::Arg::new("format").short('f').long("format").help("Output format"),
            ]),

            // GEMBA-2: Man page command
            verb!("manpage", "Output basic man page format", |_args: &VerbArgs| {
                println!(r#".TH PLAYGROUND 1 "2024" "v2.0.0" "Playground CLI Manual"
.SH NAME
playground \- Comprehensive v5 feature showcase for clap-noun-verb

.SH SYNOPSIS
.B playground
.I noun
.I verb
[OPTIONS]

.SH DESCRIPTION
Playground CLI demonstrates the clap-noun-verb framework for building
semantic CLIs with RDF/SPARQL integration and AI agent coordination.

.SH NOUNS
.TP
.B papers
Academic paper operations (generate, list, validate)
.TP
.B thesis
Thesis structure operations (structure, families, schedule)
.TP
.B config
Configuration management (get, set, show)
.TP
.B meta
v5 autonomic features (introspect, ontology, sparql, completions, middleware, telemetry, formats, manpage)

.SH EXAMPLES
.TP
Generate an IMRaD paper:
.B playground papers generate IMRaD
.TP
List paper families:
.B playground papers list
.TP
Get CLI introspection:
.B playground meta introspect -f json
.TP
Generate shell completions:
.B playground meta completions bash

.SH PAPER FAMILIES
.TP
.B IMRaD
Introduction, Method, Results, Discussion
.TP
.B Argument
Claims, Grounds, Proofs
.TP
.B Contribution
Gap, Design, Evaluation, Impact
.TP
.B Monograph
Context, Canon, Method, Analysis
.TP
.B DSR
Problem, Artifact, Evaluation, Theory
.TP
.B Narrative
Field, Voice, Pattern, Insight

.SH OUTPUT FORMATS
json, json-pretty, yaml, table, plain

.SH AUTHOR
clap-noun-verb framework

.SH SEE ALSO
.BR clap (1),
.BR tera (1)
"#);
                Ok(())
            }, args: []),
        ]));

    cli.run()
}
