//! Custom domain-specific middlewares for production use.
//!
//! These middlewares demonstrate advanced Rust patterns:
//! - Const generics for compile-time configuration
//! - Phantom types for zero-cost markers
//! - Generic type parameters for flexible implementations

mod observability;
mod caching;
mod retry;
mod tracing;
mod security;

pub use observability::ObservabilityMiddleware;
pub use caching::DynamicCachingMiddleware;
pub use retry::SmartRetryMiddleware;
pub use tracing::DistributedTracingMiddleware;
pub use security::SecurityMiddleware;
