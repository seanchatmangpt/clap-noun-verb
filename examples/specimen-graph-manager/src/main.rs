// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Open Ontologies Graph Manager CLI - Entry Point
//!
//! A specimen implementation demonstrating all proven v26.6.1 clap-noun-verb APIs
//! for building noun-verb CLI patterns with auto-discovery and JSON output.
//!
//! ## Architecture
//!
//! The CLI uses the following proven patterns:
//!
//! 1. **Command Registration via Macros** - Commands are declared with `#[verb]` macros
//!    scattered across command modules. The compiler collects them via `linkme` distributed slices.
//!
//! 2. **Auto-Discovery** - No manual command registration needed. The `CommandRegistry`
//!    automatically discovers all `#[verb]` declarations at compile time.
//!
//! 3. **Type-Safe Arguments** - Arguments are inferred from function signatures:
//!    - `String` = required argument
//!    - `Option<T>` = optional argument
//!    - `bool` = flag
//!
//! 4. **JSON Output** - All command return types implement `Serialize` for automatic
//!    JSON formatting via `OutputFormat`.
//!
//! 5. **Error Handling** - Commands return `Result<T>` with structured error messages.
//!
//! ## Entry Point Pattern
//!
//! The main function simply calls `clap_noun_verb::run()` which:
//! - Collects all `#[verb]` declarations
//! - Builds a clap Command tree
//! - Parses CLI arguments
//! - Dispatches to the correct handler
//! - Serializes output to JSON
//! - Handles errors with exit codes

mod commands {
    pub mod doctor_check;
    pub mod graph_load;
    pub mod graph_query;
    pub mod graph_validate;
    pub mod pack_add;
    pub mod pack_remove;
}

mod graph_model;
mod output_models;

fn main() -> clap_noun_verb::Result<()> {
    // Auto-discover all registered commands and run
    // This invokes the command routing system which:
    // 1. Finds all #[verb] declarations
    // 2. Builds the clap Command tree
    // 3. Parses arguments
    // 4. Routes to the correct handler
    // 5. Serializes output to JSON (default)
    clap_noun_verb::run()
}
