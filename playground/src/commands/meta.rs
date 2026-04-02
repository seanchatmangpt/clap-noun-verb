//! v5 autonomic features
//!
//! Introspection, ontology, SPARQL, completions, middleware, telemetry.
//!
//! Following the golden rule: CLI validates, domain computes, integration connects.

use clap_noun_verb_macros::verb;
use clap_noun_verb::{NounVerbError, Result};

use crate::domain::{
    build_playground_ontology, ExecutionReceipt, ExecutionSpan, IntrospectionResponse,
    MiddlewareConfig, MiddlewareStats, ShellType, SpanStatus, SparqlQueryType,
};
use crate::integration::{execute_sparql, export_turtle, get_ontology_store};
use crate::outputs::{
    CompletionScriptOutput, FormatInfoOutput, HealthOutput, MiddlewareOutput,
    OntologyOutput, SparqlResultOutput, TelemetryOutput,
};

/// Emit deprecation warning for migrated commands
fn emit_deprecation(message: &str) {
    eprintln!("⚠️  DEPRECATED: {}", message);
    eprintln!("   This command will be removed in v27.0.0");
}

/// Health check endpoint returning JSON status
///
/// Returns basic health information for monitoring.
///
/// # Arguments
/// * (none)
#[verb("health")]
fn health_check() -> Result<HealthOutput> {
    emit_deprecation("Health check is now: ggen doctor env");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(HealthOutput {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
    })
}

/// Machine-grade CLI introspection for AI agents
///
/// Returns structured capability metadata for autonomous agent discovery.
///
/// # Arguments
/// * (none)
#[verb("introspect")]
fn introspect_cli() -> Result<IntrospectionResponse> {
    emit_deprecation("Use MCP/A2A introspection");

    // Delegate to domain logic
    let capabilities = build_playground_ontology();
    Ok(IntrospectionResponse::from_capabilities(
        "playground",
        env!("CARGO_PKG_VERSION"),
        "Comprehensive v5 feature showcase",
        &capabilities,
    ))
}

/// Export CLI as RDF/Turtle ontology
///
/// Generates semantic ontology for SPARQL queries.
///
/// # Arguments
/// * (none)
#[verb("ontology")]
fn export_ontology() -> Result<OntologyOutput> {
    emit_deprecation("Ontology is now pack metadata");

    // Delegate to domain + integration
    let capabilities = build_playground_ontology();
    let turtle = export_turtle(&capabilities);

    Ok(OntologyOutput {
        format: "text/turtle".to_string(),
        triples: capabilities.len() * 5,
        content: turtle,
    })
}

/// Execute SPARQL query on CLI ontology
///
/// Runs SPARQL queries against the CLI's RDF ontology.
///
/// # Arguments
/// * `query` - Query type or custom SPARQL [default: capabilities]
#[verb("sparql")]
fn run_sparql(query: Option<String>) -> Result<SparqlResultOutput> {
    emit_deprecation("SPARQL moved to MCP surface");

    // 1. Validate inputs (CLI validates)
    let query_type = query.unwrap_or_else(|| "capabilities".to_string());

    // 2. Get store (integration layer)
    let store = get_ontology_store()
        .map_err(|e| NounVerbError::execution_error(e))?;

    // 3. Build SPARQL query (domain logic)
    let sparql = match query_type.as_str() {
        "capabilities" => SparqlQueryType::SelectCapabilities.to_sparql(),
        "papers" => SparqlQueryType::SelectByNoun("papers".to_string()).to_sparql(),
        "mutating" => {
            SparqlQueryType::SelectByEffect(crate::domain::EffectType::Mutating).to_sparql()
        }
        custom => custom.to_string(),
    };

    // 4. Execute query (integration layer)
    let results = execute_sparql(&store, &sparql)
        .map_err(|e| NounVerbError::execution_error(e))?;

    Ok(SparqlResultOutput {
        query: query_type,
        rows: results.len(),
        results,
    })
}

/// Generate shell completion scripts
///
/// Creates completion scripts for bash, zsh, fish, powershell, or elvish.
///
/// # Arguments
/// * `shell` - Shell type
#[verb("completions")]
fn generate_completions(shell: String) -> Result<CompletionScriptOutput> {
    // 1. Validate inputs (CLI validates)
    let shell_type = ShellType::from_str(&shell)
        .ok_or_else(|| {
            NounVerbError::validation_error(
                "shell".to_string(),
                shell.clone(),
                Some("Use: bash, zsh, fish, powershell, elvish")
            )
        })?;

    // 2. Delegate to domain logic
    let capabilities = build_playground_ontology();
    let script = crate::domain::generate_completion_script("playground", &capabilities, shell_type);

    Ok(CompletionScriptOutput {
        shell: shell_type.name().to_string(),
        cli: script.cli_name,
        script: script.script,
    })
}

/// Show middleware configuration and stats
///
/// Displays logging, rate limiting, and caching settings.
///
/// # Arguments
/// * (none)
#[verb("middleware")]
fn show_middleware() -> Result<MiddlewareOutput> {
    // Delegate to domain logic
    let config = MiddlewareConfig::default();
    let stats = MiddlewareStats::default();

    Ok(MiddlewareOutput { config, stats })
}

/// Show telemetry data and execution receipts
///
/// Displays execution spans and receipts for observability.
///
/// # Arguments
/// * (none)
#[verb("telemetry")]
fn show_telemetry() -> Result<TelemetryOutput> {
    // Delegate to domain logic
    let span = ExecutionSpan::new("meta.telemetry")
        .with_attribute("format", "json-pretty")
        .complete(SpanStatus::Ok);

    let receipt = ExecutionReceipt::new("meta telemetry", &[], span.duration_ms.unwrap_or(0), true)
        .with_agent("playground-cli");

    Ok(TelemetryOutput {
        trace_id: span.trace_id.clone(),
        span,
        receipt,
    })
}

/// List available output formats
///
/// Shows all supported output formats (json, yaml, table, plain).
///
/// # Arguments
/// * (none)
#[verb("formats")]
fn list_formats() -> Result<Vec<FormatInfoOutput>> {
    // Use framework's built-in output formats
    use clap_noun_verb::OutputFormat;
    let formats: Vec<_> = OutputFormat::available_formats()
        .iter()
        .map(|&name| {
            let fmt = name.parse::<OutputFormat>().unwrap_or(OutputFormat::JsonPretty);
            FormatInfoOutput {
                name: name.to_string(),
                description: fmt.description().to_string(),
            }
        })
        .collect();

    Ok(formats)
}

/// Output basic man page format
///
/// Generates UNIX man page documentation for the playground CLI.
///
/// # Arguments
/// * (none)
#[verb("manpage")]
fn generate_manpage() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        ".TH PLAYGROUND 1 \"2024\" \"v{}\" \"Playground CLI Manual\"",
        version
    );
    println!(
        r#"
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
v5 autonomic features (introspect, ontology, sparql, completions, middleware, telemetry, formats, manpage, health)

.SH EXAMPLES
.TP
Generate an IMRaD paper:
.B playground papers generate IMRaD
.TP
List paper families:
.B playground papers list
.TP
Get CLI introspection:
.B playground meta introspect
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
"#
    );
    Ok(())
}
