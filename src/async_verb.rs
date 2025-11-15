//! Async handler support for verbs
//!
//! This module provides utilities for using async operations within verb handlers.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::async_verb::run_async;
//! use clap_noun_verb::VerbArgs;
//! use serde::Serialize;
//! use std::time::Duration;
//!
//! #[derive(Serialize)]
//! struct Output {
//!     message: String,
//! }
//!
//! fn my_handler(args: &VerbArgs) -> clap_noun_verb::Result<Output> {
//!     // Run async code from sync handler
//!     run_async(async {
//!         // Async operations here
//!         tokio::time::sleep(Duration::from_millis(100)).await;
//!         Ok(Output {
//!             message: "Done!".to_string(),
//!         })
//!     })
//! }
//! ```

use crate::Result;

/// Helper for running async code from sync contexts using tokio runtime
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb::async_verb::run_async;
///
/// let result = run_async(async {
///     // Your async code here
///     some_async_operation().await
/// });
/// ```
pub fn run_async<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| crate::error::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;

    rt.block_on(future)
}

/// Create a tokio runtime that can be reused for multiple async operations
///
/// # Example
///
/// ```rust,ignore
/// let runtime = create_runtime()?;
/// let result1 = runtime.block_on(async_op1());
/// let result2 = runtime.block_on(async_op2());
/// ```
pub fn create_runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| crate::error::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_runtime() {
        let rt = create_runtime();
        assert!(rt.is_ok());
    }

    #[tokio::test]
    async fn test_async_execution() {
        let value = 42;
        let result = async { value + 1 }.await;
        assert_eq!(result, 43);
    }
}
