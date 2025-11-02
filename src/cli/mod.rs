//! CLI layer - argument validation and routing only
//!
//! This module contains the CLI interface layer that validates arguments
//! and delegates to business logic. It contains NO business logic.
//!
//! ## Design Principle
//!
//! CLI code ONLY validates arguments and options, then delegates to
//! business logic functions. No business logic is allowed in this layer.

pub mod builder;
pub mod registry;
pub mod router;
pub mod validator;
pub(crate) mod value_parser;

pub use builder::CliBuilder;
pub use registry::CommandRegistry;
pub use router::CommandRouter;
pub use validator::ArgValidator;

/// Auto-run CLI with all registered commands
///
/// This function automatically discovers all functions marked with
/// `#[noun]` and `#[verb]` attributes and runs the CLI.
///
/// These attribute macros are provided by the `clap-noun-verb-macros` crate.
pub fn run() -> crate::error::Result<()> {
    let registry = registry::CommandRegistry::get();
    let registry = registry.lock().map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Failed to lock registry: {}", e))
    })?;
    let args: Vec<String> = std::env::args().collect();
    registry.run(args)
}
