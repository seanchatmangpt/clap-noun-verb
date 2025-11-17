//! Advanced command executor with middleware integration and distributed tracing.
//!
//! Uses type-state pattern to enforce correct execution flow at compile time.
//! Supports both sync and async command execution with automatic instrumentation.

use crate::middleware::{MiddlewareRequest, MiddlewareResponse, MiddlewarePipeline};
use std::fmt;
use std::marker::PhantomData;
use std::time::Instant;

/// Type-state marker for pre-execution phase.
#[derive(Debug, Copy, Clone)]
pub struct PreExecution;

/// Type-state marker for execution phase.
#[derive(Debug, Copy, Clone)]
pub struct Executing;

/// Type-state marker for post-execution phase.
#[derive(Debug, Copy, Clone)]
pub struct PostExecution;

/// Sealed trait to prevent external implementations.
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::PreExecution {}
    impl Sealed for super::Executing {}
    impl Sealed for super::PostExecution {}
}

/// Execution phase marker trait (sealed).
pub trait ExecutionPhase: sealed::Sealed + fmt::Debug + Copy + Send + Sync {}
impl ExecutionPhase for PreExecution {}
impl ExecutionPhase for Executing {}
impl ExecutionPhase for PostExecution {}

/// Execution context carrying request/response data through middleware chain.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Command name
    command: String,
    /// Command arguments
    args: Vec<String>,
    /// Requester/user identifier
    requester: Option<String>,
    /// Start timestamp
    start_time: Option<std::time::Instant>,
    /// Execution duration (ms)
    duration_ms: Option<u128>,
    /// Result message
    result: Option<String>,
    /// Error message (if any)
    error: Option<String>,
    /// Trace ID for distributed tracing
    trace_id: String,
}

impl ExecutionContext {
    /// Create a new execution context for a command.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            requester: None,
            start_time: None,
            duration_ms: None,
            result: None,
            error: None,
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Add an argument to the context.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set the requester/user.
    pub fn with_requester(mut self, requester: impl Into<String>) -> Self {
        self.requester = Some(requester.into());
        self
    }

    /// Get the command name.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Get the arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Get the requester.
    pub fn requester(&self) -> Option<&str> {
        self.requester.as_deref()
    }

    /// Get the trace ID.
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the execution duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u128> {
        self.duration_ms
    }

    /// Get the result message.
    pub fn result(&self) -> Option<&str> {
        self.result.as_deref()
    }

    /// Get the error message.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Set the result (internal use).
    pub(crate) fn set_result(&mut self, result: String) {
        self.result = Some(result);
    }

    /// Set the error (internal use).
    pub(crate) fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Set the duration (internal use).
    pub(crate) fn set_duration(&mut self, duration_ms: u128) {
        self.duration_ms = Some(duration_ms);
    }

    /// Record start time (internal use).
    pub(crate) fn record_start(&mut self) {
        self.start_time = Some(Instant::now());
    }
}

/// Advanced command executor with type-state and middleware integration.
///
/// Uses phantom types to enforce correct execution flow at compile time.
/// Example:
/// ```ignore
/// let executor = CommandExecutor::new(pipeline)
///     .with_context(ctx)
///     .execute_pre()?  // Returns CommandExecutor<Executing>
///     .execute_command(|| {
///         // Your command logic here
///         Ok("result".to_string())
///     })?
///     .execute_post()? // Returns CommandExecutor<PostExecution>
///     .context()
/// ```
pub struct CommandExecutor<Phase: ExecutionPhase = PreExecution> {
    /// Middleware pipeline
    pipeline: MiddlewarePipeline,
    /// Execution context
    context: ExecutionContext,
    /// Current execution phase (type-state)
    _phase: PhantomData<Phase>,
}

impl CommandExecutor<PreExecution> {
    /// Create a new command executor with a middleware pipeline.
    pub fn new(pipeline: MiddlewarePipeline) -> Self {
        Self {
            pipeline,
            context: ExecutionContext::new("default"),
            _phase: PhantomData,
        }
    }

    /// Set the execution context.
    pub fn with_context(mut self, context: ExecutionContext) -> Self {
        self.context = context;
        self
    }

    /// Execute pre-execution phase and transition to Executing state.
    ///
    /// This phase:
    /// - Creates a tracing span
    /// - Runs all middleware "before" hooks
    /// - Records start time
    ///
    /// # Errors
    ///
    /// Returns an error if any middleware rejects the request.
    pub fn execute_pre(mut self) -> crate::Result<CommandExecutor<Executing>> {
        // Record start time
        self.context.record_start();

        // Create middleware request
        let mut request = MiddlewareRequest::new(self.context.command().to_string());
        for arg in self.context.args() {
            request = request.with_arg(arg.clone());
        }
        if let Some(requester) = self.context.requester() {
            request = request.with_requester(requester.to_string());
        }

        // Execute pre-execution hooks
        self.pipeline.execute_before(&request)?;

        Ok(CommandExecutor {
            pipeline: self.pipeline,
            context: self.context,
            _phase: PhantomData,
        })
    }
}

impl CommandExecutor<Executing> {
    /// Execute the actual command with the provided closure.
    ///
    /// # Errors
    ///
    /// Returns an error if the command closure fails.
    pub fn execute_command<F>(
        mut self,
        command: F,
    ) -> crate::Result<CommandExecutor<PostExecution>>
    where
        F: FnOnce() -> crate::Result<String>,
    {
        match command() {
            Ok(result) => {
                self.context.set_result(result);
                Ok(CommandExecutor {
                    pipeline: self.pipeline,
                    context: self.context,
                    _phase: PhantomData,
                })
            }
            Err(e) => {
                self.context.set_error(e.to_string());
                Err(e)
            }
        }
    }

    /// Execute the command with automatic error recovery.
    ///
    /// Allows middlewares to handle errors and provide recovery messages.
    pub fn execute_command_with_recovery<F>(
        mut self,
        command: F,
    ) -> crate::Result<CommandExecutor<PostExecution>>
    where
        F: FnOnce() -> crate::Result<String>,
    {
        match command() {
            Ok(result) => {
                self.context.set_result(result);
                Ok(CommandExecutor {
                    pipeline: self.pipeline,
                    context: self.context,
                    _phase: PhantomData,
                })
            }
            Err(e) => {
                // Try middleware error recovery
                match self.pipeline.handle_error(&e) {
                    Ok(Some(recovery_msg)) => {
                        self.context.set_result(recovery_msg);
                        Ok(CommandExecutor {
                            pipeline: self.pipeline,
                            context: self.context,
                            _phase: PhantomData,
                        })
                    }
                    Ok(None) => {
                        self.context.set_error(e.to_string());
                        Err(e)
                    }
                    Err(recovery_err) => {
                        self.context.set_error(format!("{} (recovery failed: {})", e, recovery_err));
                        Err(recovery_err)
                    }
                }
            }
        }
    }
}

impl CommandExecutor<PostExecution> {
    /// Execute post-execution phase.
    ///
    /// This phase:
    /// - Records execution duration
    /// - Runs all middleware "after" hooks
    /// - Closes tracing span
    ///
    /// # Errors
    ///
    /// Returns an error if any middleware "after" hook fails.
    pub fn execute_post(mut self) -> crate::Result<Self> {
        // Calculate duration
        if let Some(start) = self.context.start_time {
            let duration_ms = start.elapsed().as_millis();
            self.context.set_duration(duration_ms);
        }

        // Create middleware response
        let response = if let Some(result) = self.context.result() {
            MiddlewareResponse::success(result.to_string())
        } else if let Some(error) = self.context.error() {
            MiddlewareResponse::failure(error.to_string())
        } else {
            MiddlewareResponse::success("completed")
        };

        // Execute post-execution hooks
        self.pipeline.execute_after(&response)?;

        Ok(CommandExecutor {
            pipeline: self.pipeline,
            context: self.context,
            _phase: PhantomData,
        })
    }

    /// Get the execution context after completion.
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Consume and return the execution context.
    pub fn into_context(self) -> ExecutionContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_creation() {
        let ctx = ExecutionContext::new("test");
        assert_eq!(ctx.command(), "test");
        assert!(ctx.requester().is_none());
    }

    #[test]
    fn test_execution_context_with_args() {
        let ctx = ExecutionContext::new("test")
            .with_arg("arg1")
            .with_arg("arg2");
        assert_eq!(ctx.args().len(), 2);
    }

    #[test]
    fn test_executor_creation() {
        let pipeline = MiddlewarePipeline::new();
        let executor = CommandExecutor::new(pipeline);
        assert_eq!(executor.context.command(), "default");
    }
}
