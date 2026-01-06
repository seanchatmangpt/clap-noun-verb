//! # ggen Integration Module
//!
//! Ontology-driven code generation integration for clap-noun-verb.
//!
//! This module provides a type-safe, ergonomic wrapper around ggen's code
//! generation capabilities, following clap-noun-verb's API design principles:
//!
//! - **Type-first thinking**: Types encode invariants at compile time
//! - **Zero-cost abstractions**: Generics and references for performance
//! - **Result-based error handling**: No unwrap() or expect() in production
//! - **Deterministic outputs**: Reproducible code generation with receipts
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use clap_noun_verb::integration::ggen::GgenGenerator;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let receipt = GgenGenerator::new()
//!     .template("templates/rust-cli.tera")?
//!     .output("output/my-cli")?
//!     .variable("name", "my-cli")?
//!     .generate()
//!     .await?;
//!
//! println!("Generated: {}", receipt.output_hash);
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`config`] - Configuration types and builders
//! - [`error`] - Error types with Result aliases
//! - [`generator`] - High-level generator API
//! - [`graph`] - RDF graph integration
//! - [`pipeline`] - Template pipeline integration
//! - [`receipt`] - Generation receipts for determinism

#![cfg(feature = "ggen")]

pub mod config;
pub mod error;
pub mod generator;
pub mod graph;
pub mod pipeline;
pub mod receipt;

// Re-export commonly used types for convenience
pub use config::{GgenConfig, GgenConfigBuilder};
pub use error::{GgenError, GgenResult};
pub use generator::GgenGenerator;
pub use graph::GgenGraph;
pub use pipeline::GgenPipeline;
pub use receipt::GenerationReceipt;

/// Re-export core ggen types for advanced usage
pub mod core {
    pub use ggen_core::{Generator as CoreGenerator, Pipeline as CorePipeline, Template};
}

/// Re-export domain ggen types for advanced usage
pub mod domain {
    pub use ggen_domain::{
        generation as domain_generation, project as domain_project, template as domain_template,
    };
}

/// Type-state marker for GgenGenerator
pub mod state {
    /// Type-state pattern for compile-time validation
    pub trait State {}

    /// Generator is configured but not yet executed
    pub struct Configured;
    impl State for Configured {}

    /// Generator has been executed and produced results
    pub struct Generated;
    impl State for Generated {}
}
