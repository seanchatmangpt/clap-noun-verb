//! ggen Integration Module
//!
//! This module provides integration between ggen's RDF/Turtle specifications
//! and clap-noun-verb's CLI framework. It enables generating CLI code from
//! Turtle language specifications.
//!
//! ## Architecture
//!
//! The integration follows a three-stage pipeline:
//!
//! 1. **Parser**: Parses Turtle/RDF specifications using Oxigraph
//! 2. **AST**: Intermediate representation of CLI structure
//! 3. **Codegen**: Generates clap-noun-verb Rust code
//!
//! ## Type-First Design
//!
//! All types are designed to encode invariants at compile time:
//! - NonEmpty types prevent empty strings
//! - Result<T, E> for all fallible operations
//! - No unwrap/expect in production code
//!
//! ## Modules
//!
//! - [`ast`] - Abstract Syntax Tree types
//! - [`parser`] - Turtle/RDF parser
//! - [`codegen`] - CLI code generator
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use clap_noun_verb::ggen_integration::{parse_turtle, generate_cli_code};
//! use std::path::Path;
//!
//! # fn main() -> clap_noun_verb::Result<()> {
//! // Parse turtle specification
//! let ast = parse_turtle(Path::new("spec.ttl"))?;
//!
//! // Generate CLI code
//! let code = generate_cli_code(&ast)?;
//!
//! // Write generated code
//! std::fs::write("generated.rs", code)?;
//! # Ok(())
//! # }
//! ```

pub mod ast;
pub mod codegen;
pub mod error;
pub mod parser;

// Re-export main types
pub use ast::{Argument, ArgumentKind, Command, Flag, TypeAnnotation};
pub use codegen::CodeGenerator;
pub use error::{GgenError, GgenResult};
pub use parser::TurtleParser;

use error::GgenResult as Result;
use std::path::Path;

/// Parse a Turtle specification file into an AST
///
/// # Arguments
///
/// * `path` - Path to the Turtle specification file
///
/// # Returns
///
/// * `Result<Vec<Command>>` - Parsed command definitions
///
/// # Errors
///
/// Returns error if:
/// - File cannot be read
/// - Turtle syntax is invalid
/// - Required properties are missing
pub fn parse_turtle(path: &Path) -> Result<Vec<Command>> {
    let parser = TurtleParser::new()?;
    parser.parse_file(path)
}

/// Generate CLI code from AST
///
/// # Arguments
///
/// * `commands` - Command definitions from AST
///
/// # Returns
///
/// * `Result<String>` - Generated Rust code
///
/// # Errors
///
/// Returns error if code generation fails
pub fn generate_cli_code(commands: &[Command]) -> Result<String> {
    let generator = CodeGenerator::new();
    generator.generate(commands)
}

/// End-to-end pipeline: Parse turtle and generate code
///
/// # Arguments
///
/// * `input_path` - Path to Turtle specification
/// * `output_path` - Path to write generated code
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns error if parsing or generation fails
pub fn turtle_to_code(input_path: &Path, output_path: &Path) -> Result<()> {
    let commands = parse_turtle(input_path)?;
    let code = generate_cli_code(&commands)?;
    std::fs::write(output_path, code)?;
    Ok(())
}
