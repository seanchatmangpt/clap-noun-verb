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
//! - **Attribute Macros** (`clap-noun-verb-macros`) - `#[verb]` for declarative command registration
//! - **Auto-Discovery** - Commands automatically discovered using `linkme` distributed slices
//! - **Type Inference** - Arguments automatically inferred from function signatures
//! - **JSON Output** - All output automatically serialized to JSON
//!
//! ### Key Principles
//!
//! 1. **Zero Boilerplate** - Just add `#[verb]` attributes to functions
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

pub mod async_verb;
pub mod builder;
pub mod cli;
pub mod error;
pub mod logic;
pub mod macros;
pub mod noun;
pub mod registry;
pub mod telemetry;
pub mod tree;
pub mod verb;

// =============================================================================
// OPTIONAL MODULES - Feature-gated for minimal compile burden
// =============================================================================

// Execution context
pub mod context;

// Deprecation warnings
pub mod deprecation;

// Output formatting
pub mod format;

// Shell utilities
pub mod shell;

// Advanced clap Integration
pub mod clap_ext;

// Interactive REPL shell
pub mod repl;

// Procedural macros are available as attributes: #[clap_noun_verb::noun] and #[clap_noun_verb::verb]
// They don't need to be re-exported - they're used directly as attributes

// =============================================================================
// PUBLIC RE-EXPORTS - Core types always available
// =============================================================================

// Re-export CLI run function for convenience
pub use cli::run;

// Core framework types
pub use builder::{build_cli, run_cli, run_cli_with_args, CliBuilder};
pub use error::{ActionTemplate, ErrorKind, NounVerbError, Result, Severity, StructuredError};
pub use noun::{CompoundNounCommand, NounCommand, NounContext};
pub use registry::CommandRegistry;
pub use tree::{CommandTree, CommandTreeBuilder};
pub use verb::{VerbArgs, VerbCommand, VerbContext};

// Context and formatting (always available)
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

// Re-export clap types so users don't need clap as a direct dependency
// This follows the facade pattern used by serde, tokio, and tracing
// Note: These are from clap's builder module (the main clap crate re-exports these)
pub use clap::{Arg, ArgAction, ArgMatches, Command};

// =============================================================================
// FEATURE-GATED RE-EXPORTS
// =============================================================================

// Macros are exported at crate root via #[macro_export]

pub use repl::Repl;

// Framework-level re-exports for easy composition
pub use builder::CliBuilder as Cli;
pub use registry::CommandRegistry as Registry;
pub use tree::CommandTree as Tree;
