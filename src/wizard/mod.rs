//! Wizard - AI integration layer for clap-noun-verb
//!
//! This module provides a type-safe, ergonomic wrapper around rust-genai
//! for multi-provider LLM integration. It follows the clap-noun-verb principles:
//!
//! - **Type-first thinking**: Models are type-safe enums, not strings
//! - **Zero-cost abstractions**: Thin wrapper with no runtime overhead
//! - **Result-based errors**: All operations return `Result<T, WizardError>`
//! - **Environment configuration**: API keys loaded from environment variables
//!
//! ## Features
//!
//! - Multi-provider support (OpenAI, Anthropic, Gemini, etc.)
//! - Type-safe model selection with compile-time guarantees
//! - Async/await integration with tokio
//! - Optional response streaming
//! - Optional caching for common prompts
//! - Builder pattern for wizard construction
//! - Interactive REPL-style sessions
//! - CLI integration with clap noun-verb pattern
//!
//! ## Example
//!
//! ```rust,no_run
//! use clap_noun_verb::wizard::{GenAiClient, ModelConfig, WizardConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment
//!     let config = WizardConfig::from_env()?;
//!
//!     // Create client
//!     let client = GenAiClient::new(config).await?;
//!
//!     // Generate response
//!     let response = client.generate("What is Rust?").await?;
//!     println!("{}", response);
//!
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod interactive;
pub mod types;

// v2 features (feature-gated)
#[cfg(feature = "wizard")]
pub mod cache;
#[cfg(feature = "wizard")]
pub mod fallback;
#[cfg(feature = "wizard")]
pub mod rate_limit;
#[cfg(feature = "wizard")]
pub mod retry;
#[cfg(feature = "wizard")]
pub mod streaming;

pub use builder::{Wizard, WizardBuilder};
pub use cli::{InteractiveArgs, OutputFormat, RunArgs, WizardCli, WizardCommand};
pub use client::GenAiClient;
pub use config::{ModelConfig, WizardConfig};
pub use error::{Result, WizardError, WizardResult};
pub use interactive::InteractiveSession;
pub use types::{Prompt, WizardResponse};

// Re-export v2 features when enabled
#[cfg(feature = "wizard")]
pub use cache::{CacheConfig, CacheStats, CachedClient};
#[cfg(feature = "wizard")]
pub use fallback::{FallbackClient, FallbackConfig, FallbackStats};
#[cfg(feature = "wizard")]
pub use rate_limit::{RateLimitConfig, RateLimitStrategy, RateLimitedClient};
#[cfg(feature = "wizard")]
pub use retry::{RetryClient, RetryConfig, RetryContext};
#[cfg(feature = "wizard")]
pub use streaming::{CancellableStream, StreamChunk, StreamingClient, StreamingConfig};
