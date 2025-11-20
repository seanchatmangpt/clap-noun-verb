//! clap-noun-verb Feature Implementation
//!
//! Core features for noun-verb CLI command patterns with full test coverage.
//! Built using test-driven development (TDD) methodology.

pub mod crud;
pub mod builder;
pub mod executor;
pub mod error_handler;
pub mod async_runner;
pub mod middleware_chain;
pub mod route_registry;
pub mod test_runner;

pub use crud::{NounVerb, CrudOperation, OperationResult};
pub use builder::CommandBuilder;
pub use executor::VerbExecutor;
pub use error_handler::ErrorHandler;
pub use async_runner::AsyncRunner;
pub use middleware_chain::{MiddlewareChain, Middleware};
pub use route_registry::RouteRegistry;
pub use test_runner::TestRunner;

#[cfg(test)]
mod tests;
