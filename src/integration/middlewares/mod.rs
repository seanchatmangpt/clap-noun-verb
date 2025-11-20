//! Custom domain-specific middlewares for production use.
//!
//! These middlewares demonstrate advanced Rust patterns:
//! - Const generics for compile-time configuration
//! - Phantom types for zero-cost markers
//! - Generic type parameters for flexible implementations

mod caching;
mod observability;
mod retry;
mod security;
mod tracing;

pub use caching::DynamicCachingMiddleware;
pub use observability::ObservabilityMiddleware;
pub use retry::SmartRetryMiddleware;
pub use security::SecurityMiddleware;
pub use tracing::DistributedTracingMiddleware;
