//! Domain Logic Layer - Pure, Testable Business Logic
//!
//! This module contains pure functions that implement the core business logic.
//! NO CLI concerns, NO I/O, NO side effects - just pure computation.
//!
//! **The Golden Rule**: Domain logic is pure Rust functions that are:
//! - Testable in isolation (no mocks needed)
//! - Reusable across different interfaces (CLI, API, GUI)
//! - Free of side effects (no file I/O, no network, no printing)
//!
//! ## v5 Feature Modules
//!
//! - `papers`: Academic paper generation
//! - `thesis`: Thesis structure and Λ-scheduling
//! - `config`: Configuration management
//! - `ontology`: RDF/Turtle ontology building
//! - `introspection`: Autonomic CLI introspection
//! - `formats`: Output format handling (JSON, YAML, Table)
//! - `middleware`: Middleware configuration
//! - `telemetry`: Metrics, tracing, and receipts
//! - `completions`: Shell completion generation

pub mod papers;
pub mod thesis;
pub mod research_thesis;
pub mod config;
pub mod ontology;
pub mod introspection;
pub mod formats;
pub mod middleware;
pub mod telemetry;
pub mod completions;

// Re-export commonly used types (only what's actively used by CLI)
pub use papers::{Paper, PaperFamily};
pub use thesis::{ThesisFamily, ThesisSchedule, ThesisStructure};
pub use research_thesis::{ResearchThesis, DefenseOutline, DefenseSlide, ThesisValidation, ValidationResult, ThesisStatus};
pub use config::Config;

// v5 feature exports - only re-export what's actually used by main.rs
pub use ontology::{EffectType, SparqlQueryType, build_playground_ontology};
pub use introspection::IntrospectionResponse;
pub use formats::{OutputFormat, format_output};
pub use middleware::{MiddlewareConfig, MiddlewareStats};
pub use telemetry::{ExecutionSpan, SpanStatus, ExecutionReceipt};
pub use completions::{ShellType, generate_completion_script};
