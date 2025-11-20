//! Integration Layer - Wiring middleware, custom implementations, and configuration
//!
//! This module provides production-grade integration of all v4.3 features:
//!
//! - **Middleware Executor** - Wire middleware into command execution with tracing
//! - **Custom Middlewares** - Domain-specific implementations for real-world use
//! - **Custom Exporters** - Production exporters for observability platforms
//! - **Configuration System** - Load plugins from manifest files with dependency resolution
//!
//! # Advanced Patterns
//!
//! This module demonstrates hyper-advanced Rust patterns:
//! - Type-state pattern for compile-time phase verification
//! - Trait objects with sealed trait pattern
//! - Higher-ranked trait bounds (HRTB) for flexible callbacks
//! - Generic associated types for flexible result handling
//! - Phantom types for zero-cost abstractions

pub mod config;
pub mod executor;
pub mod exporters;
pub mod middlewares;

pub use config::{PluginDependencyGraph, PluginManifestLoader};
pub use executor::{CommandExecutor, ExecutionContext, ExecutionPhase};
pub use exporters::{DatadogExporter, ElasticsearchExporter};
pub use middlewares::{
    DistributedTracingMiddleware, DynamicCachingMiddleware, ObservabilityMiddleware,
    SecurityMiddleware, SmartRetryMiddleware,
};
