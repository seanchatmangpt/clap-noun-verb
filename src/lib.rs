//! clap-noun-verb - A framework for building composable CLI patterns
//!
//! This crate provides a high-level, ergonomic API for building noun-verb CLI patterns
//! on top of clap, similar to how Python's Typer provides a simpler interface over Click.
//!
//! ## Version 3.2.0 Architecture
//!
//! Version 3.0.0 introduced attribute macros for zero-boilerplate command registration.
//! Version 3.2.0 adds complete clap feature support:
//!
//! - **Attribute Macros** (`clap-noun-verb-macros`) - `#[noun]` and `#[verb]` for declarative command registration
//! - **Auto-Discovery** - Commands automatically discovered using `linkme` distributed slices
//! - **Type Inference** - Arguments automatically inferred from function signatures
//! - **JSON Output** - All output automatically serialized to JSON
//! - **Environment Variables** - `#[arg(env = "VAR_NAME")]` for env var fallback
//! - **Positional Arguments** - `#[arg(index = 0)]` for positional args
//! - **Enhanced Actions** - Count, SetFalse with auto-inference (`usize` → Count, `bool` → SetTrue)
//! - **Argument Groups** - Groups, requires, conflicts_with support
//! - **Better Help** - long_about, hide, help_heading support
//!
//! ### Key Principles
//!
//! 1. **Zero Boilerplate** - Just add `#[noun]` and `#[verb]` attributes to functions
//! 2. **Auto-Discovery** - Commands automatically discovered at compile time
//! 3. **Type Inference** - Arguments inferred from function signatures
//! 4. **JSON by Default** - Perfect for agents, MCP, and modern tooling
//!
//! ## Framework Philosophy
//!
//! Instead of providing specific compositions, this crate provides a framework that allows
//! users to compose their own CLI patterns. Key features:
//!
//! - **Composable Command Structure**: Easy composition of nouns and verbs
//! - **Separation of Concerns**: CLI validates, logic is separate and reusable
//! - **Type-Safe Composition**: Compile-time verification of command structure
//! - **Zero-Cost Abstractions**: Thin wrapper over clap with no runtime overhead
//!
//! ## API Stability
//!
//! This crate follows [Semantic Versioning](https://semver.org/). Version 3.2.0 and above
//! provide API stability guarantees:
//!
//! - **Public APIs** are stable and will not change in a breaking way within the same major version
//! - **Breaking changes** will only occur in major version bumps (4.0.0, 5.0.0, etc.)
//! - **Deprecations** will be announced at least one minor version before removal
//! - **Private APIs** (non-pub items) are not subject to stability guarantees
//!
//! All public types, traits, and functions documented in this crate are considered stable.

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

// New in v3.6.0
pub mod async_verb;
pub mod completion;
pub mod config;
pub mod context;
pub mod deprecation;
pub mod format;
pub mod mangen;
pub mod shell;
pub mod validators;

// New in v3.8.0 - Autonomic CLI Layer
pub mod autonomic;

// New in v3.8.0 - CNV Kernel Capabilities
pub mod kernel;

// New in v4.0 - I/O Integration
pub mod io;

// Procedural macros are available as attributes: #[clap_noun_verb::noun] and #[clap_noun_verb::verb]
// They don't need to be re-exported - they're used directly as attributes

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

// New in v3.6.0
pub use async_verb::{create_runtime, run_async};
pub use completion::{generate_completion, print_completion, Shell};
pub use context::AppContext;
pub use deprecation::{Deprecation, DeprecationType};
pub use format::OutputFormat;

// Macros are exported at crate root via #[macro_export]

// Framework-level re-exports for easy composition
pub use builder::CliBuilder as Cli;
pub use registry::CommandRegistry as Registry;
pub use tree::CommandTree as Tree;
