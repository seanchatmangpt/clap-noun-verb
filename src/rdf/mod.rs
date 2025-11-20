//! RDF/Turtle ontology control layer for CNV v5
//!
//! Provides a graph-native machine interface where:
//! - Σ (ontology): Describes all nouns, verbs, args, guards via Turtle
//! - Δ (invocation): Agents drive verbs by issuing RDF requests
//! - Receipt: Execution results + provenance as RDF with blake3 hashes
//!
//! This layer enables:
//! - Agent introspection via SPARQL queries
//! - Guard validation via SHACL shapes
//! - Full provenance tracking (KGC integration)
//! - Zero-copy streaming for large results

pub mod blake3_hash;
pub mod builder;
pub mod guard_validation;
pub mod invocation;
pub mod kgc_integration;
pub mod lockchain;
pub mod lockchain_receipt;
pub mod macro_integration;
pub mod mcp_server;
pub mod ontology;
pub mod receipt;
pub mod rmcp_handler;
pub mod sparql;
pub mod types;
pub mod validation;

pub use blake3_hash::Blake3Hash;
pub use builder::OntologyBuilder;
pub use guard_validation::{recover_from_error, GuardValidationMiddleware};
pub use invocation::{InvocationError, InvocationParser, ParsedInvocation};
pub use kgc_integration::{AuditEntry, KgcMetadata, KgcPackage, KgcShard};
pub use lockchain::{Lockchain, LockchainEntry};
pub use lockchain_receipt::{LockchainReceipt, ReceiptMetadata};
pub use macro_integration::RdfRegistry;
pub use mcp_server::RdfMcpServer;
pub use ontology::{ClnvOntology, Ontology};
pub use receipt::{Receipt, ReceiptGenerator};
pub use rmcp_handler::RdfMcpHandler;
pub use sparql::SparqlPlanner;
pub use types::{Invocation, RdfTriple, RdfValue};
pub use validation::ShapeValidator;

/// Feature gate: enable RDF control layer (defaults to true)
pub const FEATURE_ENABLED: bool = cfg!(feature = "rdf-control");

/// Ontology namespace
pub const CNV_NAMESPACE: &str = "https://cnv.dev/ontology#";

/// Standard RDF namespaces
pub const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
pub const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";
pub const SHACL_NS: &str = "http://www.w3.org/ns/shacl#";

/// Prelude: import commonly-used types
pub mod prelude {
    pub use crate::rdf::{
        AuditEntry, Blake3Hash, GuardValidationMiddleware, Invocation, InvocationError,
        KgcMetadata, KgcPackage, KgcShard, Lockchain, LockchainEntry, Ontology, OntologyBuilder,
        ParsedInvocation, RdfMcpServer, RdfRegistry, RdfTriple, RdfValue, Receipt, ReceiptMetadata,
        ShapeValidator,
    };
}
