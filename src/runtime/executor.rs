//! Command executor - executes business logic with interceptors
//!
//! The executor runs business logic functions through interceptors
//! for cross-cutting concerns like logging, tracing, metrics, etc.

use crate::error::Result;
use crate::logic::handler::{HandlerInput, HandlerOutput};
use crate::runtime::interceptor::Interceptor;

/// Executor for command handlers with interceptor support
pub struct Executor {
    /// Interceptors for cross-cutting concerns
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Self { interceptors: Vec::new() }
    }

    /// Create an executor with interceptors
    pub fn with_interceptors(interceptors: Vec<Box<dyn Interceptor>>) -> Self {
        Self { interceptors }
    }

    /// Add an interceptor
    pub fn add_interceptor(&mut self, interceptor: Box<dyn Interceptor>) {
        self.interceptors.push(interceptor);
    }

    /// Execute a handler function with interceptors
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler function to execute
    /// * `input` - Validated input from CLI layer
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    pub fn execute<F>(&self, handler: F, input: HandlerInput) -> Result<HandlerOutput>
    where
        F: FnOnce(HandlerInput) -> Result<HandlerOutput>,
    {
        // Run pre-execution interceptors
        let input = self.run_pre_interceptors(input)?;

        // Execute the handler
        let mut output = handler(input)?;

        // Run post-execution interceptors
        output = self.run_post_interceptors(output)?;

        Ok(output)
    }

    /// Run pre-execution interceptors
    fn run_pre_interceptors(&self, mut input: HandlerInput) -> Result<HandlerInput> {
        for interceptor in &self.interceptors {
            input = interceptor.pre_execute(input)?;
        }
        Ok(input)
    }

    /// Run post-execution interceptors
    fn run_post_interceptors(&self, mut output: HandlerOutput) -> Result<HandlerOutput> {
        for interceptor in &self.interceptors {
            output = interceptor.post_execute(output)?;
        }
        Ok(output)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
