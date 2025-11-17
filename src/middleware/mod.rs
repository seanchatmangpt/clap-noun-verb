//! Command Middleware/Interceptors System (Feature 4 - v4.3)
//!
//! This module provides a powerful middleware pipeline for intercepting and processing
//! commands before and after execution, enabling logging, error recovery, metrics, and more.
//!
//! # Architecture
//!
//! - **Middleware trait** - Base interface for sync and async middlewares
//! - **MiddlewarePipeline** - Composable middleware chain
//! - **Built-in middlewares** - Logging, error recovery, auth, profiling, rate limiting, caching
//!
//! # Example
//!
//! ```ignore
//! use clap_noun_verb::middleware::{Middleware, MiddlewarePipeline};
//!
//! let pipeline = MiddlewarePipeline::new()
//!     .add(LoggingMiddleware::new())
//!     .add(ErrorRecoveryMiddleware::new());
//! ```

pub mod builtin;

use std::fmt;

pub use builtin::{
    AuthMiddleware, CachingMiddleware, LoggingMiddleware, ProfilingMiddleware,
    RateLimitingMiddleware, ErrorRecoveryMiddleware,
};

/// Middleware request context.
#[derive(Debug, Clone)]
pub struct MiddlewareRequest {
    /// Command name
    command: String,
    /// Command arguments
    args: Vec<String>,
    /// User/requester identification
    requester: Option<String>,
}

impl MiddlewareRequest {
    /// Create a new middleware request.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            requester: None,
        }
    }

    /// Add an argument.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set the requester.
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
}

/// Middleware response context.
#[derive(Debug, Clone)]
pub struct MiddlewareResponse {
    /// Response status
    success: bool,
    /// Response message
    message: String,
    /// Response metadata
    metadata: std::collections::HashMap<String, String>,
}

impl MiddlewareResponse {
    /// Create a successful response.
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a failed response.
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add metadata to the response.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if the response is successful.
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the metadata.
    pub fn metadata(&self) -> &std::collections::HashMap<String, String> {
        &self.metadata
    }
}

/// Core middleware trait for request/response processing.
pub trait Middleware: Send + Sync {
    /// Get the middleware name.
    fn name(&self) -> &str;

    /// Process a request before command execution.
    ///
    /// Return true to continue, false to abort.
    fn before(&self, _request: &MiddlewareRequest) -> crate::Result<bool> {
        Ok(true)
    }

    /// Process a response after command execution.
    fn after(&self, _response: &MiddlewareResponse) -> crate::Result<()> {
        Ok(())
    }

    /// Handle an error that occurred during execution.
    fn handle_error(&self, _error: &crate::NounVerbError) -> crate::Result<Option<String>> {
        Ok(None)
    }
}

/// Middleware pipeline for composing multiple middlewares.
pub struct MiddlewarePipeline {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl fmt::Debug for MiddlewarePipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiddlewarePipeline")
            .field("middleware_count", &self.middlewares.len())
            .finish()
    }
}

impl MiddlewarePipeline {
    /// Create a new empty middleware pipeline.
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the pipeline.
    pub fn add(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Execute the pipeline's before hooks.
    ///
    /// # Errors
    ///
    /// Returns an error if any middleware returns false or errors.
    pub fn execute_before(&self, request: &MiddlewareRequest) -> crate::Result<()> {
        for middleware in &self.middlewares {
            if !middleware.before(request)? {
                return Err(crate::NounVerbError::MiddlewareError(
                    format!("Middleware '{}' rejected request", middleware.name()),
                ));
            }
        }
        Ok(())
    }

    /// Execute the pipeline's after hooks.
    ///
    /// # Errors
    ///
    /// Returns an error if any middleware errors.
    pub fn execute_after(&self, response: &MiddlewareResponse) -> crate::Result<()> {
        for middleware in &self.middlewares {
            middleware.after(response)?;
        }
        Ok(())
    }

    /// Execute the pipeline's error handlers.
    ///
    /// Returns the first recovery message if any middleware can recover.
    pub fn handle_error(&self, error: &crate::NounVerbError) -> crate::Result<Option<String>> {
        for middleware in &self.middlewares {
            if let Some(recovery) = middleware.handle_error(error)? {
                return Ok(Some(recovery));
            }
        }
        Ok(None)
    }

    /// Get the number of middlewares in the pipeline.
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    /// Check if the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Get a list of middleware names.
    pub fn middleware_names(&self) -> Vec<&str> {
        self.middlewares.iter().map(|m| m.name()).collect()
    }
}

impl Default for MiddlewarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MiddlewarePipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MiddlewarePipeline({})",
            self.middleware_names().join(" -> ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_request_creation() {
        let req = MiddlewareRequest::new("test");
        assert_eq!(req.command(), "test");
        assert!(req.args().is_empty());
    }

    #[test]
    fn test_middleware_request_with_args() {
        let req = MiddlewareRequest::new("test")
            .with_arg("arg1")
            .with_arg("arg2");
        assert_eq!(req.args().len(), 2);
    }

    #[test]
    fn test_middleware_response_success() {
        let resp = MiddlewareResponse::success("OK");
        assert!(resp.is_success());
        assert_eq!(resp.message(), "OK");
    }

    #[test]
    fn test_middleware_response_failure() {
        let resp = MiddlewareResponse::failure("Failed");
        assert!(!resp.is_success());
    }

    #[test]
    fn test_middleware_pipeline_creation() {
        let pipeline = MiddlewarePipeline::new();
        assert_eq!(pipeline.len(), 0);
        assert!(pipeline.is_empty());
    }
}
