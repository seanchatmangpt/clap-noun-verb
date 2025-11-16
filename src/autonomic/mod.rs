//! Autonomic CLI Layer - Machine-grade interface for clap-noun-verb
//!
//! This module provides capabilities for building CLI applications that can be
//! introspected, analyzed, and driven by autonomic systems, MAPE-K loops, and agents.
//!
//! ## Key Features
//!
//! - **Introspection**: Discover commands, arguments, and metadata at runtime
//! - **Effect Modeling**: Declare read-only vs mutating operations
//! - **Plane Integration**: O/Σ/Q/ΔΣ metadata for ontology-driven systems
//! - **Guards & Budgets**: Latency and resource constraints
//! - **Receipts**: Structured execution records for audit and analysis
//! - **Autonomic Mode**: MAPE-K loop compatible invocation

pub mod cli;
pub mod effects;
pub mod errors;
pub mod guards;
pub mod introspection;
pub mod planes;
pub mod receipts;

// Re-export key types
pub use cli::{AutonomicCli, AutonomicNounCommand, AutonomicVerbCommand};
pub use effects::{EffectMetadata, EffectType, Sensitivity};
pub use errors::{ErrorKind, ErrorResponse, StructuredError};
pub use guards::{GuardConfig, GuardResult, GuardStatus};
pub use introspection::{
    AppMetadata, ArgumentMetadata, CommandCapabilities, CommandGraph, CommandMetadata,
    GraphEdge, GraphNode, IntrospectionResponse, NounMetadata, VerbMetadata,
};
pub use planes::{InteractionType, Plane, PlaneInteraction};
pub use receipts::{ExecutionReceipt, ReceiptConfig, ReceiptWithOutput};

/// Version of the autonomic CLI schema
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Features supported by this implementation
pub const SUPPORTED_FEATURES: &[&str] = &[
    "introspect",
    "capabilities",
    "effects",
    "planes",
    "guards",
    "receipts",
    "errors",
];
