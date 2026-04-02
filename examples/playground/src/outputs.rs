//! Output types for CLI commands
//!
//! All CLI commands return these types for JSON serialization.
//! Following domain separation: CLI validates, domain computes, integration connects.

use serde::Serialize;

/// Output from paper generation
#[derive(Serialize)]
pub struct PaperGeneratedOutput {
    pub family: String,
    pub output_path: String,
    pub template_engine: String,
    pub sections: usize,
}

/// Output from paper family listing
#[derive(Serialize)]
pub struct PaperFamilyOutput {
    pub name: String,
    pub description: String,
}

/// Output from paper validation
#[derive(Serialize)]
pub struct ValidationResultOutput {
    pub is_valid: bool,
    pub structure_valid: bool,
    pub citations_valid: bool,
    pub formatting_valid: bool,
    pub errors: Vec<String>,
}

/// Output from config get operation
#[derive(Serialize)]
pub struct ConfigValueOutput {
    pub key: String,
    pub value: Option<String>,
    pub valid_key: bool,
}

/// Output from config set operation
#[derive(Serialize)]
pub struct ConfigSetOutput {
    pub key: String,
    pub value: String,
    pub valid_key: bool,
    pub saved: bool,
}

/// Output from config show operation
#[derive(Serialize)]
pub struct ConfigAllOutput {
    pub entries: std::collections::HashMap<String, String>,
}

/// Output from health check
#[derive(Serialize)]
pub struct HealthOutput {
    pub status: String,
    pub version: String,
    pub timestamp: u64,
}

/// Output from ontology export
#[derive(Serialize)]
pub struct OntologyOutput {
    pub format: String,
    pub triples: usize,
    pub content: String,
}

/// Output from SPARQL query
#[derive(Serialize)]
pub struct SparqlResultOutput {
    pub query: String,
    pub rows: usize,
    pub results: Vec<Vec<String>>,
}

/// Output from shell completion generation
#[derive(Serialize)]
pub struct CompletionScriptOutput {
    pub shell: String,
    pub cli: String,
    pub script: String,
}

/// Output from middleware display
#[derive(Serialize)]
pub struct MiddlewareOutput {
    pub config: crate::domain::MiddlewareConfig,
    pub stats: crate::domain::MiddlewareStats,
}

/// Output from telemetry display
#[derive(Serialize)]
pub struct TelemetryOutput {
    pub span: crate::domain::ExecutionSpan,
    pub receipt: crate::domain::ExecutionReceipt,
    pub trace_id: String,
}

/// Output from format listing
#[derive(Serialize)]
pub struct FormatInfoOutput {
    pub name: String,
    pub description: String,
}
