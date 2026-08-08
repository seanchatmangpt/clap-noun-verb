// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! `clap-noun-verb` is a framework for composable noun-verb command-line interfaces.
//!
//! Version 26.8.8 preserves the small core while completing every declared crown
//! capability as a bounded, deterministic implementation surface.
//! Domain computation remains in handlers; the CLI validates and routes.

// =============================================================================
// CORE MODULES
// =============================================================================

pub mod async_verb;
pub mod builder;
pub mod cli;
pub mod error;
pub mod logic;
pub mod macros;
pub mod noun;
#[cfg(feature = "otel")]
pub mod otel;
pub mod registry;
pub mod telemetry;
pub mod tree;
pub mod verb;

// Feature-gated core extension and contributor surfaces.
#[cfg(feature = "contrib")]
pub mod contrib;
#[cfg(feature = "process-data")]
pub mod process_data;

// Autonomic CI/CD policies
pub mod policies;

// Graph operations, capability management, and diagnostics
pub mod capability;
pub mod diagnostics;
pub mod graph;

// Federation support
#[cfg(feature = "federated-network")]
pub mod federation;

// The complete bounded frontier capability surface.
#[cfg(any(
    feature = "meta-framework",
    feature = "rdf-composition",
    feature = "fractal-patterns",
    feature = "discovery-engine",
    feature = "federated-network",
    feature = "learning-trajectories",
    feature = "reflexive-testing",
    feature = "economic-sim",
    feature = "quantum-ready",
    feature = "executable-specs"
))]
pub mod frontier;

// Execution context, deprecation, formatting, shell, and REPL surfaces
pub mod clap_ext;
pub mod context;
pub mod deprecation;
pub mod format;
pub mod repl;
pub mod shell;

// RDF ↔ ggen synchronization
pub mod ggen_to_rdf;
pub mod ontology_sync;
pub mod rdf_to_ggen;

// =============================================================================
// PUBLIC RE-EXPORTS
// =============================================================================

pub use builder::{build_cli, run_cli, run_cli_with_args, CliBuilder};
pub use cli::run;
pub use error::{ActionTemplate, ErrorKind, NounVerbError, Result, Severity, StructuredError};
pub use noun::{CompoundNounCommand, NounCommand, NounContext};
pub use registry::CommandRegistry;
pub use tree::{CommandTree, CommandTreeBuilder};
pub use verb::{VerbArgs, VerbCommand, VerbContext};

#[cfg(feature = "contrib")]
pub use contrib::{Contributor, ContributorRegistry};
#[cfg(feature = "process-data")]
pub use process_data::{ProcessDataPipeline, ProcessDataStep};

pub use policies::{
    GitPhaseDirtyPolicy, PolicyEvent, PolicyMode, PolicySet, PolicyVerdict, TargetPressurePolicy,
    ToolchainMismatchPolicy, TrybuildChangedPolicy,
};

pub use context::AppContext;
pub use deprecation::{Deprecation, DeprecationType};
pub use format::{
    clear_output_validation_hooks, format_output, register_output_validation_hook, OutputFormat,
    OutputValidationHook,
};
pub mod validators;
pub use validators::{
    validate_email, validate_ipv4, validate_ipv6, validate_length, validate_not_empty,
    validate_path_creatable, validate_path_exists, validate_port, validate_regex, validate_url,
};

// Facade re-exports so consumers do not need a direct clap dependency.
pub use clap::{Arg, ArgAction, ArgMatches, Command};

pub use capability::{
    CapabilityPackage, CapabilityRegistry, CapabilityStanding, PackAddedOutput, PackRemovedOutput,
    ProofSurface,
};
pub use diagnostics::{DoctorOutput, HealthIssue};
pub use graph::{Graph, GraphLoadedOutput, QueryResultOutput, Triple, ValidationResultOutput};
pub use repl::Repl;

pub use builder::CliBuilder as Cli;
pub use registry::CommandRegistry as Registry;
pub use tree::CommandTree as Tree;
