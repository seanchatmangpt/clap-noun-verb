//! Runtime layer - execution infrastructure
//!
//! This module contains execution infrastructure including interceptors
//! for cross-cutting concerns (logging, tracing, metrics, etc.).

pub mod executor;
pub mod interceptor;

pub use executor::Executor;
pub use interceptor::Interceptor;
