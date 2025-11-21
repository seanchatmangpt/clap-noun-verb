//! Integration Layer - Glue Code Between CLI and Domain
//!
//! This module connects the CLI layer to the domain layer.
//! Contains I/O operations, template rendering, and side effects.
//!
//! **The Golden Rule**: Integration code is thin glue that:
//! - Calls domain functions for business logic
//! - Handles I/O (file system, network, templates)
//! - Translates between CLI types and domain types
//!
//! ## v5 Integration Modules
//!
//! - `templates`: Tera template rendering
//! - `io`: File system operations
//! - `rdf`: Oxigraph RDF/SPARQL operations

pub mod templates;
pub mod io;
pub mod rdf;

// Re-export integration functions
pub use templates::{render_paper_latex, init_template_engine};
pub use io::{write_paper, ensure_output_dir};
pub use rdf::{init_ontology_store, execute_sparql, export_turtle};
