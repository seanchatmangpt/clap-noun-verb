//! clap-noun-verb - A framework for building composable CLI patterns
//!
//! This crate provides a high-level, ergonomic API for building noun-verb CLI patterns
//! on top of clap, similar to how Python's Typer provides a simpler interface over Click.
//!
//! ## Minimal Dependencies
//!
//! By default, clap-noun-verb compiles with **only 10 core dependencies** for basic CLI:
//! - `clap` - CLI framework
//! - `clap-noun-verb-macros` - Our proc macros
//! - `linkme` - Auto-discovery
//! - `serde`, `serde_json` - JSON output
//! - `thiserror`, `anyhow` - Error handling
//! - `once_cell`, `lazy_static`, `atty` - Utilities
//!
//! All advanced features are opt-in via cargo features:
//! - `full` - Enable all features
//! - `autonomic` - Agent introspection & telemetry spans
//! - `async` - Async handlers (tokio, futures)
//! - `io` - Advanced I/O (clio)
//! - `crypto` - Cryptographic hashing (sha2, sha3, blake3)
//! - `agent2028` - Trillion-agent ecosystems
//! - `rdf` - RDF/Ontology with MCP
//! - `kernel` - Deterministic execution
//!
//! ## Version 5.3.0 Architecture
//!
//! - **Attribute Macros** (`clap-noun-verb-macros`) - `#[noun]` and `#[verb]` for declarative command registration
//! - **Auto-Discovery** - Commands automatically discovered using `linkme` distributed slices
//! - **Type Inference** - Arguments automatically inferred from function signatures
//! - **JSON Output** - All output automatically serialized to JSON
//!
//! ### Key Principles
//!
//! 1. **Zero Boilerplate** - Just add `#[noun]` and `#[verb]` attributes to functions
//! 2. **Auto-Discovery** - Commands automatically discovered at compile time
//! 3. **Type Inference** - Arguments inferred from function signatures
//! 4. **JSON by Default** - Perfect for agents, MCP, and modern tooling
//! 5. **Minimal Dependencies** - Core CLI needs only 9 crates
//!
//! ## API Stability
//!
//! This crate follows [Semantic Versioning](https://semver.org/). Version 5.3.0 provides:
//!
//! - **Public APIs** are stable within the same major version
//! - **Breaking changes** only in major version bumps
//! - **Feature flags** are stable - won't be removed without deprecation

// =============================================================================
// CORE MODULES - Always available (no feature flags)
// =============================================================================

pub mod builder;
pub mod cli;
pub mod error;
pub mod logic;
pub mod macros;
pub mod noun;
pub mod registry;
pub mod router;
pub mod runtime;
pub mod tree;
pub mod verb;

// Capability Discovery Engine (requires agent2028 feature for swarm optimization)
#[cfg(feature = "agent2028")]
pub mod macros_discovery_engine;

// =============================================================================
// OPTIONAL MODULES - Feature-gated for minimal compile burden
// =============================================================================

// Async verb support (requires "async" feature)
#[cfg(feature = "async")]
pub mod async_verb;

// Shell completion generation (requires "completions" feature)
#[cfg(feature = "completions")]
pub mod completion;

// Configuration formats (requires "config-formats" feature)
#[cfg(feature = "config-formats")]
pub mod config;

// Execution context
pub mod context;

// Deprecation warnings
pub mod deprecation;

// Output formatting
pub mod format;

// Man page generation (requires "mangen" feature)
#[cfg(feature = "mangen")]
pub mod mangen;

// Shell utilities
pub mod shell;

// URL/Regex validators (requires "validators" feature)
#[cfg(feature = "validators")]
pub mod validators;

// Autonomic CLI Layer (requires "autonomic" feature)
#[cfg(feature = "autonomic")]
pub mod autonomic;

// CNV Kernel Capabilities (requires "kernel" feature)
#[cfg(feature = "kernel")]
pub mod kernel;

// I/O Integration (requires "io" feature)
#[cfg(feature = "io")]
pub mod io;

// Advanced clap Integration
pub mod clap;

// Plugin System (requires "full" feature)
#[cfg(feature = "full")]
pub mod plugin;

// Middleware System (requires "full" feature)
#[cfg(feature = "full")]
pub mod middleware;

// Telemetry & Observability (requires "observability" feature)
#[cfg(feature = "observability")]
pub mod telemetry;

// Integration Layer (requires "full" feature)
#[cfg(feature = "full")]
pub mod integration;

// Production Plugins (requires "full" feature)
#[cfg(feature = "full")]
pub mod plugins;

// Agent2028 - Trillion-Agent Ecosystems (requires "agent2028" feature)
#[cfg(feature = "agent2028")]
pub mod agent2028;

// RDF/Ontology Control Layer (requires "rdf" feature)
#[cfg(feature = "rdf")]
pub mod rdf;

// Semantic Agent Coordinator (requires "agent2028" feature + optional "rdf", "autonomic")
#[cfg(feature = "agent2028")]
pub mod agents;

// Semantic CLI Composition (requires "rdf" feature for SPARQL and RDF metadata)
#[cfg(feature = "rdf")]
pub mod semantic;

// Procedural macros are available as attributes: #[clap_noun_verb::noun] and #[clap_noun_verb::verb]
// They don't need to be re-exported - they're used directly as attributes

// =============================================================================
// PUBLIC RE-EXPORTS - Core types always available
// =============================================================================

// Re-export CLI run function for convenience
pub use cli::run;

// Core framework types
pub use builder::{build_cli, run_cli, run_cli_with_args, CliBuilder};
pub use error::{NounVerbError, Result};
pub use noun::{CompoundNounCommand, NounCommand, NounContext};
pub use registry::CommandRegistry;
pub use router::CommandRouter;
pub use tree::{CommandTree, CommandTreeBuilder};
pub use verb::{VerbArgs, VerbCommand, VerbContext};

// Context and formatting (always available)
pub use context::AppContext;
pub use deprecation::{Deprecation, DeprecationType};
pub use format::OutputFormat;

// =============================================================================
// FEATURE-GATED RE-EXPORTS
// =============================================================================

// Async support (requires "async" feature)
#[cfg(feature = "async")]
pub use async_verb::{create_runtime, run_async};

// Shell completion (requires "completions" feature)
#[cfg(feature = "completions")]
pub use completion::{generate_completion, print_completion, Shell};

// Macros are exported at crate root via #[macro_export]

// Framework-level re-exports for easy composition
pub use builder::CliBuilder as Cli;
pub use registry::CommandRegistry as Registry;
pub use tree::CommandTree as Tree;
