// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CLI layer - argument validation and routing only
//!
//! This module validates arguments and delegates to domain handlers. It does
//! not own business computation.

pub mod builder;
pub mod preprocessor;
pub mod registry;
pub mod router;
pub mod validator;
pub(crate) mod value_parser;

pub mod init;

pub use builder::CliBuilder;
pub use init::scaffold_config;
pub use registry::CommandRegistry;
pub use router::CommandRouter;
pub use validator::ArgValidator;

/// Auto-run the CLI with all registered commands.
pub fn run() -> crate::error::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    #[cfg(feature = "otel")]
    let _dispatch_span =
        tracing::info_span!("clap_noun_verb.dispatch", argc = args.len(), entrypoint = "cli::run")
            .entered();

    let registry = registry::CommandRegistry::get();
    let registry = registry.lock().map_err(|error| {
        crate::error::NounVerbError::execution_error(format!("Failed to lock registry: {error}"))
    })?;
    registry.run(args)
}
