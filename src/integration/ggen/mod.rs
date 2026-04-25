//! # ggen Integration Module
//!
//! Ontology-driven code generation integration for clap-noun-verb.
//!
//! **NOTE**: This module is currently non-functional as the ggen_core and ggen_domain
//! dependencies are not available. The feature exists for future integration.

#![cfg(feature = "ggen")]

// Compile error to prevent use of non-functional ggen integration
// compile_error!("The ggen integration is not yet available. The ggen_core and ggen_domain dependencies are marked as FUTURE in Cargo.toml. Disable the 'ggen' feature or add the required dependencies.");

// NOTE: The following modules are commented out because they depend on ggen_core/ggen_domain
// which are not available. When those dependencies are added, uncomment these modules.
//
// pub mod config;
// pub mod error;
// pub mod generator;
// pub mod graph;
// pub mod pipeline;
// pub mod receipt;
//
// // Re-export commonly used types for convenience
// pub use config::{GgenConfig, GgenConfigBuilder};
// pub use error::{GgenError, GgenResult};
// pub use generator::GgenGenerator;
// pub use graph::GgenGraph;
// pub use pipeline::GgenPipeline;
// pub use receipt::GenerationReceipt;

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
