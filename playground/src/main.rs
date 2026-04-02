//! Playground CLI - clap-noun-verb v5.6.0 Documentation-Style Demo
//!
//! Architecture: CLI validates, domain computes, integration connects
//!
//! ```text
//! ┌─────────────┐
//! │   CLI Layer │  ← commands/ (thin, #[verb] with doc comment tags)
//! └──────┬──────┘
//!        │
//! ┌──────▼──────────┐
//! │ Integration     │  ← integration/ (Tera, Oxigraph, file I/O)
//! └──────┬──────────┘
//!        │
//! ┌──────▼──────────┐
//! │  Domain Logic   │  ← domain/ (pure, testable business logic)
//! └─────────────────┘
//! ```
//!
//! ## v5 Features Demonstrated
//!
//! - **Documentation-Style**: `#[verb]` with `# Arguments` doc comments and tags
//! - **Argument Tags**: `[default:]`, `[env:]`, `[value_hint:]`, `[group:]`, `[requires:]`, `[conflicts:]`
//! - **RDF/Ontology**: SPARQL queries and Turtle export
//! - **Output Formats**: JSON, YAML, Table output modes
//! - **Shell Completions**: Bash, Zsh, Fish, PowerShell
//! - **Middleware**: Logging, profiling, rate-limiting
//! - **Telemetry**: Execution receipts and metrics

// Domain imports - pure business logic (NO I/O, NO CLI)
mod domain;

// Integration imports - glue code with side effects
mod integration;

// CLI commands - thin validation layer
mod commands;

// Output types for JSON serialization
mod outputs;

use clap_noun_verb::Result;

fn main() -> Result<()> {
    // Auto-discover all #[verb] commands and run
    clap_noun_verb::run()
}
