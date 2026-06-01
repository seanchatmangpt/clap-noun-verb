// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Open Ontologies Graph Manager - A specimen CLI built with clap-noun-verb v26.6.1
//!
//! This specimen demonstrates all proven v26.6.1 APIs:
//! - `#[verb]` macro for command registration
//! - `VerbCommand` trait (used implicitly via macro)
//! - `CommandRegistry` auto-discovery via linkme
//! - `OutputFormat` for JSON serialization
//! - `validators` module for argument validation
//! - All return types implement `serde::Serialize`
//!
//! ## Commands
//!
//! - `graph load <path>` - Load RDF file
//! - `graph query <pattern>` - Query graph with pattern matching
//! - `graph validate <path>` - Validate RDF syntax
//! - `doctor check` - Health check of system
//! - `pack add <name> <version>` - Register capability package
//! - `pack remove <id>` - Unregister capability package

pub mod commands;
pub mod graph_model;
pub mod output_models;

// Re-export key types for consumer convenience
pub use graph_model::{CapabilityPackage, CapabilityRegistry, RdfGraph, Triple};
pub use output_models::{DoctorOutput, GraphLoadedOutput, PackAddedOutput, QueryResultOutput, RemovalStatus, ValidationResultOutput};

// Re-export Result type from clap-noun-verb
pub use clap_noun_verb::Result;
