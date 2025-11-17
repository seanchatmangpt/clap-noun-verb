//! # clap-noun-verb - A framework for building composable CLI patterns
//!
//! This crate provides a high-level, ergonomic API for building noun-verb CLI patterns
//! on top of clap, similar to how Python's Typer provides a simpler interface over Click.
//!
//! ## Quick Start
//!
//! ```rust
//! use clap_noun_verb::{noun, verb, Result};
//!
//! #[noun]
//! fn user() {}
//!
//! #[verb(noun = "user")]
//! fn create(name: String, email: String) -> Result<()> {
//!     println!("Creating user: {} ({})", name, email);
//!     Ok(())
//! }
//!
//! #[verb(noun = "user")]
//! fn delete(id: u32) -> Result<()> {
//!     println!("Deleting user: {}", id);
//!     Ok(())
//! }
//! ```
//!
//! ## Version 4.0.0 Features
//!
//! Version 4.0.0 introduces major new capabilities for production-grade CLIs:
//!
//! ### Core Features
//!
//! - **Attribute Macros** - `#[noun]` and `#[verb]` for declarative command registration
//! - **Auto-Discovery** - Commands automatically discovered using `linkme` distributed slices
//! - **Type Inference** - Arguments automatically inferred from function signatures
//! - **JSON Output** - All output automatically serialized to JSON
//! - **Environment Variables** - `#[arg(env = "VAR_NAME")]` for env var fallback
//! - **Positional Arguments** - `#[arg(index = 0)]` for positional args
//! - **Enhanced Actions** - Count, SetFalse with auto-inference (`usize` → Count, `bool` → SetTrue)
//! - **Argument Groups** - Groups, requires, conflicts_with support
//!
//! ### New in v4.0.0
//!
//! - **I/O Integration** - Native file I/O with `clio` integration for seamless stdin/stdout/file handling
//! - **Plugin System** - Dynamic plugin loading and discovery with manifest support
//! - **Middleware System** - Request/response interceptors for logging, auth, metrics, caching
//! - **Telemetry & Observability** - Built-in tracing, metrics, and observability with `tracing`
//! - **Production Plugins** - 10 ready-to-use plugins with comprehensive Chicago-TDD testing
//! - **Async I/O** - Full async/await support with tokio integration
//! - **Type Validation** - Enhanced type validation for I/O operations
//! - **Security Hardening** - Path validation, PII redaction, removed unmaintained dependencies
//!
//! ## Core Concepts
//!
//! ### 1. Nouns and Verbs
//!
//! The noun-verb pattern organizes CLI commands hierarchically:
//!
//! ```rust
//! use clap_noun_verb::{noun, verb, Result};
//!
//! // Define a noun (entity)
//! #[noun]
//! fn database() {}
//!
//! // Define verbs (actions) for that noun
//! #[verb(noun = "database")]
//! fn backup(path: String) -> Result<()> {
//!     println!("Backing up database to: {}", path);
//!     Ok(())
//! }
//!
//! #[verb(noun = "database")]
//! fn restore(path: String) -> Result<()> {
//!     println!("Restoring database from: {}", path);
//!     Ok(())
//! }
//! ```
//!
//! This creates a CLI with commands like:
//! - `mycli database backup /path/to/backup`
//! - `mycli database restore /path/to/backup`
//!
//! ### 2. CommandRegistry
//!
//! The [`CommandRegistry`] is the central hub for managing commands:
//!
//! ```rust
//! use clap_noun_verb::CommandRegistry;
//!
//! let registry = CommandRegistry::new();
//! println!("Registered commands: {}", registry.len());
//! ```
//!
//! Commands are automatically registered at compile time using the `#[noun]` and `#[verb]` macros.
//!
//! ### 3. Result Types
//!
//! All operations return [`Result<T>`](Result), which is an alias for `std::result::Result<T, NounVerbError>`:
//!
//! ```rust
//! use clap_noun_verb::{Result, NounVerbError};
//!
//! fn my_command() -> Result<()> {
//!     // Your logic here
//!     Ok(())
//! }
//!
//! fn with_error() -> Result<()> {
//!     Err(NounVerbError::ValidationError("Invalid input".to_string()))
//! }
//! ```
//!
//! ### 4. I/O Types (v4.0+)
//!
//! Native I/O integration for seamless file and stream handling:
//!
//! ```rust
//! use clap_noun_verb::io::{InputSource, OutputDestination};
//! use clap_noun_verb::{verb, Result};
//!
//! #[verb(noun = "file")]
//! fn convert(
//!     #[arg(value_parser)] input: InputSource,
//!     #[arg(value_parser)] output: OutputDestination,
//! ) -> Result<()> {
//!     // InputSource handles: stdin, files, URLs
//!     // OutputDestination handles: stdout, files
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! ### Core Modules
//!
//! - [`builder`] - CLI builder for constructing command-line interfaces
//! - [`cli`] - Main CLI execution and runtime
//! - [`error`] - Error types and handling
//! - [`noun`] - Noun command definitions and traits
//! - [`verb`] - Verb command definitions and traits
//! - [`registry`] - Command registry for managing registered commands
//! - [`router`] - Command routing and dispatch
//! - [`tree`] - Command tree structure for hierarchical commands
//!
//! ### Advanced Modules (v3.6+)
//!
//! - [`async_verb`] - Async/await support for verb commands
//! - [`completion`] - Shell completion generation
//! - [`config`] - Configuration file support
//! - [`context`] - Application context and state management
//! - [`deprecation`] - Deprecation warnings and migration helpers
//! - [`format`] - Output formatting (JSON, YAML, TOML)
//! - [`validators`] - Input validation helpers
//!
//! ### Kernel Modules (v3.8+)
//!
//! - [`autonomic`] - Autonomic CLI layer for self-management
//! - [`kernel`] - CNV kernel capabilities for deterministic execution
//!
//! ### New in v4.0
//!
//! - [`io`] - I/O integration with clio for file and stream handling
//! - [`plugin`] - Plugin system for dynamic extensibility
//! - [`middleware`] - Middleware system for request/response processing
//! - [`telemetry`] - Telemetry and observability with tracing
//! - [`integration`] - Integration layer for custom implementations
//! - [`plugins`] - Production-ready plugins collection
//!
//! ## API Examples
//!
//! ### Example 1: Basic noun and verb
//!
//! ```rust
//! use clap_noun_verb::{noun, verb, Result};
//!
//! #[noun]
//! fn server() {}
//!
//! #[verb(noun = "server")]
//! fn start(port: u16) -> Result<()> {
//!     println!("Starting server on port {}", port);
//!     Ok(())
//! }
//!
//! #[verb(noun = "server")]
//! fn stop() -> Result<()> {
//!     println!("Stopping server");
//!     Ok(())
//! }
//! ```
//!
//! ### Example 2: Using the noun macro
//!
//! The [`noun!`] macro defines a command entity:
//!
//! ```rust
//! use clap_noun_verb::noun;
//!
//! #[noun]
//! fn project() {}
//!
//! #[noun]
//! fn config() {}
//! ```
//!
//! ### Example 3: Using the verb macro
//!
//! The [`verb!`] macro defines command actions:
//!
//! ```rust
//! use clap_noun_verb::{noun, verb, Result};
//!
//! #[noun]
//! fn file() {}
//!
//! #[verb(noun = "file")]
//! fn read(path: String) -> Result<()> {
//!     println!("Reading file: {}", path);
//!     Ok(())
//! }
//!
//! #[verb(noun = "file")]
//! fn write(path: String, content: String) -> Result<()> {
//!     println!("Writing to file: {}", path);
//!     Ok(())
//! }
//! ```
//!
//! ### Example 4: CommandRegistry usage
//!
//! ```rust
//! use clap_noun_verb::CommandRegistry;
//!
//! let registry = CommandRegistry::new();
//!
//! // Check if a command exists
//! if registry.contains("user") {
//!     println!("User command is registered");
//! }
//!
//! // Get command count
//! println!("Total commands: {}", registry.len());
//! ```
//!
//! ### Example 5: Error handling with Result
//!
//! ```rust
//! use clap_noun_verb::{Result, NounVerbError};
//!
//! fn validate_input(value: &str) -> Result<()> {
//!     if value.is_empty() {
//!         return Err(NounVerbError::ValidationError(
//!             "Value cannot be empty".to_string()
//!         ));
//!     }
//!     Ok(())
//! }
//!
//! fn process_data() -> Result<String> {
//!     validate_input("test")?;
//!     Ok("Processed".to_string())
//! }
//! ```
//!
//! ### Example 6: I/O Integration (v4.0+)
//!
//! ```rust
//! use clap_noun_verb::io::{InputSource, OutputDestination};
//! use clap_noun_verb::{verb, Result};
//!
//! #[verb(noun = "data")]
//! fn process(
//!     #[arg(value_parser)] input: InputSource,
//!     #[arg(value_parser)] output: OutputDestination,
//! ) -> Result<()> {
//!     // Handles stdin/files automatically
//!     // Handles stdout/files automatically
//!     Ok(())
//! }
//! ```
//!
//! ### Example 7: Middleware (v4.0+)
//!
//! ```rust
//! use clap_noun_verb::middleware::{
//!     MiddlewarePipeline,
//!     LoggingMiddleware,
//!     ErrorRecoveryMiddleware,
//! };
//!
//! let pipeline = MiddlewarePipeline::new()
//!     .add(Box::new(LoggingMiddleware::new()))
//!     .add(Box::new(ErrorRecoveryMiddleware::new()));
//! ```
//!
//! ### Example 8: Plugin System (v4.0+)
//!
//! ```rust
//! use clap_noun_verb::plugin::{PluginRegistry, PluginLoader};
//!
//! let mut registry = PluginRegistry::new();
//! let mut loader = PluginLoader::new("./plugins");
//!
//! // Discover plugins from directory
//! let discovered = loader.discover();
//! ```
//!
//! ## Key Principles
//!
//! 1. **Zero Boilerplate** - Just add `#[noun]` and `#[verb]` attributes to functions
//! 2. **Auto-Discovery** - Commands automatically discovered at compile time
//! 3. **Type Inference** - Arguments inferred from function signatures
//! 4. **JSON by Default** - Perfect for agents, MCP, and modern tooling
//! 5. **Production Ready** - Security, observability, and extensibility built-in
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
//! - **Extensible Architecture**: Plugin system and middleware for custom behavior
//!
//! ## API Stability
//!
//! This crate follows [Semantic Versioning](https://semver.org/). Version 4.0.0 and above
//! provide API stability guarantees:
//!
//! - **Public APIs** are stable and will not change in a breaking way within the same major version
//! - **Breaking changes** will only occur in major version bumps (5.0.0, 6.0.0, etc.)
//! - **Deprecations** will be announced at least one minor version before removal
//! - **Private APIs** (non-pub items) are not subject to stability guarantees
//!
//! All public types, traits, and functions documented in this crate are considered stable.
//!
//! ## Migration from v3.x
//!
//! See the [Migration Guide](https://github.com/seanchatmangpt/clap-noun-verb/blob/main/MIGRATION_v3_to_v4.md)
//! for detailed instructions on upgrading from v3.x to v4.0.
//!
//! ## Performance
//!
//! clap-noun-verb is designed for zero-cost abstractions:
//!
//! - Command registration happens at compile time
//! - No runtime overhead for command dispatch
//! - Optimized I/O buffering for file operations
//! - Lazy plugin loading for minimal startup time
//!
//! ## Security
//!
//! v4.0 includes security hardening:
//!
//! - Path canonicalization prevents directory traversal attacks
//! - PII redaction in middleware for sensitive data
//! - Removed unmaintained dependencies (atty)
//! - Input validation for all I/O operations

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

// New in v4.3 - Advanced clap Integration (Phase 7)
pub mod clap;

// New in v4.3 - Plugin System (Feature 1)
pub mod plugin;

// New in v4.3 - Middleware System (Feature 4)
pub mod middleware;

// New in v4.3 - Telemetry & Observability (Feature 5)
pub mod telemetry;

// New in v4.3 - Integration Layer (Middleware executor, custom implementations, configuration)
pub mod integration;

// New in v4.3 - 10 Production Plugins with Chicago-TDD Testing
pub mod plugins;

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
