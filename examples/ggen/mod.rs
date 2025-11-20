//! ggen CLI - General Projection Engine
//!
//! A production-ready implementation of ggen with enhanced error handling
//! that provides user-friendly, actionable error messages.
//!
//! ## Architecture
//!
//! - `errors` - User-friendly error types with recovery suggestions
//! - `validators` - Input validation with helpful error messages
//! - `ai_commands` - AI generation commands (generate, project, graph, sparql)
//! - `marketplace_commands` - Package marketplace (search, install, list, publish)
//! - `template_commands` - Template operations (generate, render, validate, list)
//!
//! ## Error Handling Philosophy
//!
//! Instead of technical error messages:
//! ```text
//! Error: No such file or directory (os error 2)
//! ```
//!
//! We provide actionable guidance:
//! ```text
//! ❌ Problem: Template file 'my-template.tmpl' not found
//! 💡 Solution: Check the following:
//!   1. Verify the path exists: ls my-template.tmpl
//!   2. Use absolute path or relative to current directory
//!   3. List available templates: ggen template list
//! 📚 Learn more: https://docs.ggen.io/templates
//! ```

pub mod ai_commands;
pub mod errors;
pub mod marketplace_commands;
pub mod template_commands;
pub mod validators;

// Re-export commonly used items
pub use errors::{ErrorCategory, UserError};
pub use validators::*;
