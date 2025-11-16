//! CNV Kernel Capabilities
//!
//! This module provides the core kernel capabilities for clap-noun-verb:
//!
//! - **Telemetry Profile**: Standardized verbosity, color, and format control
//! - **Output Pipeline**: Deterministic, structured output for all verbs
//! - **Grammar Model**: Introspectable command structure for agents and tooling
//! - **Manpage Integration**: Thin wrapper around manpage generation
//! - **File IO**: Stream-based input/output abstraction
//! - **Test Harness**: API for robust testing of CNV apps
//!
//! ## Design Philosophy
//!
//! The kernel layer provides deterministic, agent-grade CLI capabilities without
//! coupling to specific implementations. External libraries (clap_mangen, etc.)
//! are used as silent backends.
//!
//! ## Version
//!
//! Kernel capabilities introduced in v3.8.0 (next minor release)

pub mod grammar;
pub mod io;
pub mod manpage;
pub mod output;
pub mod telemetry;
pub mod test_harness;

// Re-export key types for convenience
pub use grammar::{Grammar, GrammarModel, GrammarNode};
pub use io::{FileIO, InputSource, OutputSink};
pub use manpage::ManpageGenerator;
pub use output::{OutputEnvelope, OutputPipeline, StructuredError, StructuredResult};
pub use telemetry::{ColorPolicy, TelemetryProfile, VerbosityLevel};
