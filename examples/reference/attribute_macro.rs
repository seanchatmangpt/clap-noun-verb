//! Example: Attribute Macro API
//!
//! This example demonstrates the attribute macro API with automatic
//! type inference, JSON output, and docstring help generation.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
// These can be used by CLI, API, Web, etc.

#[derive(Serialize, Debug)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

#[derive(Serialize, Debug)]
struct Logs {
    service: String,
    lines: usize,
    entries: Vec<String>,
}

fn get_service_status() -> Status {
    Status { services: vec!["api".to_string(), "worker".to_string()], healthy: true }
}

fn get_service_logs(service: String, lines: usize) -> Logs {
    Logs { service, lines, entries: vec!["log1".to_string(), "log2".to_string()] }
}

// CLI Layer (Input Validation + Output Shaping Only)
// These functions delegate to business logic and auto-serialize output to JSON

/// Show service status
#[verb] // Verb name "status" auto-inferred, noun "attribute_macro" auto-inferred from filename
fn show_status() -> Result<Status> {
    // 1. Validate inputs (none here)
    // 2. Delegate to business logic
    Ok(get_service_status())
    // 3. Output shaping (auto-serializes to JSON)
}

/// Show service logs
///
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50)
#[verb] // Verb name "logs" auto-inferred, noun "attribute_macro" auto-inferred from filename
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    // 1. Validate inputs (auto-inferred: service required, lines optional)
    // Verb name and noun name are auto-inferred/detected - no need to specify!
    let lines = lines.unwrap_or(50);
    // 2. Delegate to business logic
    Ok(get_service_logs(service, lines))
    // 3. Output shaping (auto-serializes to JSON)
    // 4. Help text automatically extracted from # Arguments section above!
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
