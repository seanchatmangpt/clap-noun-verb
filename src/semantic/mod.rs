//! Semantic CLI Composition - Runtime Support
//!
//! This module provides runtime infrastructure for semantic capability discovery,
//! composition validation, and dynamic CLI generation. It complements the
//! `#[semantic_composable]` proc macro from `clap-noun-verb-macros`.
//!
//! # Architecture
//!
//! The semantic composition system enables:
//! - **Discovery**: SPARQL queries over RDF capability metadata
//! - **Validation**: Type-safe composition constraint checking
//! - **Generation**: Dynamic CLI construction from capabilities
//! - **Communication**: MCP protocol for agent-to-agent sharing
//!
//! # Type-First Design
//!
//! All types encode invariants at compile time:
//! - `CapabilityMetadata`: Immutable capability descriptor
//! - `CapabilityRegistry`: Thread-safe RDF store with HNSW indexing
//! - `CompositionValidator`: Proof-carrying composition validator
//! - `RuntimeBuilder`: Zero-cost CLI builder
//!
//! # Usage
//!
//! ```rust,ignore
//! use clap_noun_verb::semantic::{CapabilityRegistry, discover_capabilities};
//!
//! // Discover all capabilities matching criteria
//! let registry = CapabilityRegistry::new()?;
//! let results = discover_capabilities(&registry, "?cap rdf:type cap:FileReader")?;
//!
//! // Build dynamic CLI from discovered capabilities
//! let cli = RuntimeBuilder::new()
//!     .add_capabilities(&results)
//!     .build()?;
//! ```

mod capability;
mod composition;
mod protocol;
mod registry;
mod runtime;
mod sparql;

pub use capability::{CapabilityMetadata, SEMANTIC_CAPABILITIES};
pub use composition::{CompositionError, CompositionValidator, ValidationResult};
pub use protocol::{McpAdapter, ProtocolAdapter, ProtocolError};
pub use registry::{CapabilityRegistry, RegistryError};
pub use runtime::{RuntimeBuilder, RuntimeError};
pub use sparql::{QueryEngine, QueryError, QueryResult};

/// Discover capabilities matching SPARQL pattern
///
/// # Arguments
///
/// - `registry`: The capability registry to query
/// - `pattern`: SPARQL WHERE pattern (e.g., "?cap rdf:type cap:FileReader")
///
/// # Returns
///
/// Vector of matching `CapabilityMetadata` references
///
/// # Errors
///
/// Returns `QueryError` if SPARQL query fails or pattern is invalid
pub fn discover_capabilities<'a>(
    registry: &'a CapabilityRegistry,
    pattern: impl AsRef<str>,
) -> Result<Vec<&'a CapabilityMetadata>, QueryError> {
    registry.query_capabilities(pattern)
}

/// Validate that capability composition is type-safe
///
/// # Arguments
///
/// - `capabilities`: Slice of capabilities to compose
///
/// # Returns
///
/// `ValidationResult` with success status and any constraint violations
pub fn validate_composition(capabilities: &[&CapabilityMetadata]) -> ValidationResult {
    CompositionValidator::new().validate(capabilities)
}
