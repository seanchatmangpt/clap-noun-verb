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
//!
//! ## ggen v26.4.2 Integration Modules
//!
//! - `workspace`: Workspace detection for finding project roots
//! - `lockfile`: Lockfile persistence
//! - `receipt_store`: Receipt storage
//! - `registry_client`: Registry protocol client

pub mod templates;
pub mod io;
pub mod rdf;

// ggen v26.4.2 integration
pub mod workspace;
pub mod lockfile;
pub mod receipt_store;
pub mod registry_client;

// Re-export integration functions
pub use templates::{render_paper_latex, get_template_engine};
pub use io::{write_paper, ensure_output_dir};
pub use rdf::{get_ontology_store, execute_sparql, export_turtle};

// ggen v26.4.2 exports
pub use workspace::WorkspaceDetector;
pub use lockfile::LockfileStore;
pub use receipt_store::ReceiptStore;
pub use registry_client::{RegistryClient, RegistrySearchResult, RegistryInfo, RegistrySource};
