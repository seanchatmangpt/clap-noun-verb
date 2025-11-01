//! Interceptor trait for cross-cutting concerns
//!
//! Interceptors allow adding cross-cutting concerns like logging,
//! tracing, metrics, etc. without modifying business logic.

use crate::error::Result;
use crate::logic::handler::{HandlerInput, HandlerOutput};

/// Interceptor for cross-cutting concerns
///
/// Interceptors can modify inputs before execution and outputs after
/// execution. They are useful for logging, tracing, metrics, etc.
pub trait Interceptor: Send + Sync {
    /// Called before command execution
    ///
    /// Can modify the input or perform side effects (logging, etc.).
    fn pre_execute(&self, input: HandlerInput) -> Result<HandlerInput>;

    /// Called after command execution
    ///
    /// Can modify the output or perform side effects (logging, etc.).
    fn post_execute(&self, output: HandlerOutput) -> Result<HandlerOutput>;
}

/// No-op interceptor (for testing or default behavior)
pub struct NoOpInterceptor;

impl Interceptor for NoOpInterceptor {
    fn pre_execute(&self, input: HandlerInput) -> Result<HandlerInput> {
        Ok(input)
    }

    fn post_execute(&self, output: HandlerOutput) -> Result<HandlerOutput> {
        Ok(output)
    }
}
