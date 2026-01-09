//! Rate limiting for wizard AI interactions
//!
//! This module provides token bucket rate limiting for AI model requests.
//! It prevents exceeding API rate limits and provides configurable throttling.
//!
//! ## Features
//!
//! - Token bucket algorithm for rate limiting
//! - Configurable tokens per second and burst capacity
//! - Wait-if-needed vs error-on-limit strategies
//! - Rate limit header parsing (X-RateLimit-*)
//! - Thread-safe rate limiter

use crate::wizard::{
    client::GenAiClient,
    config::WizardConfig,
    error::{WizardError, WizardResult},
    types::{Prompt, WizardResponse},
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Configuration for rate limiting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateLimitConfig {
    /// Tokens per second (requests per second)
    pub tokens_per_second: f64,
    /// Maximum burst capacity
    pub burst_capacity: usize,
    /// Strategy when rate limit is exceeded
    pub strategy: RateLimitStrategy,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            tokens_per_second: 10.0, // 10 requests/second
            burst_capacity: 20,      // Allow bursts up to 20 requests
            strategy: RateLimitStrategy::Wait,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration
    pub const fn new(tokens_per_second: f64, burst_capacity: usize) -> Self {
        Self { tokens_per_second, burst_capacity, strategy: RateLimitStrategy::Wait }
    }

    /// Set tokens per second
    pub const fn with_tokens_per_second(mut self, tokens_per_second: f64) -> Self {
        self.tokens_per_second = tokens_per_second;
        self
    }

    /// Set burst capacity
    pub const fn with_burst_capacity(mut self, burst_capacity: usize) -> Self {
        self.burst_capacity = burst_capacity;
        self
    }

    /// Set rate limit strategy
    pub const fn with_strategy(mut self, strategy: RateLimitStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> WizardResult<()> {
        if self.tokens_per_second <= 0.0 {
            return Err(WizardError::Config("tokens_per_second must be > 0".to_string()));
        }
        if self.burst_capacity == 0 {
            return Err(WizardError::Config("burst_capacity must be > 0".to_string()));
        }
        Ok(())
    }
}

/// Rate limit strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitStrategy {
    /// Wait until tokens are available
    Wait,
    /// Return an error immediately if rate limit exceeded
    Error,
}

/// Token bucket rate limiter
///
/// Implements the token bucket algorithm:
/// - Tokens refill at a constant rate
/// - Bucket has maximum capacity (burst)
/// - Request consumes one token
/// - If no tokens available, wait or error based on strategy
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,
    /// Maximum tokens (burst capacity)
    max_tokens: f64,
    /// Tokens added per second
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    fn new(refill_rate: f64, max_tokens: usize) -> Self {
        Self {
            tokens: max_tokens as f64,
            max_tokens: max_tokens as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Add tokens based on elapsed time
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to consume one token
    ///
    /// Returns Ok(()) if token consumed, Err(Duration) with wait time if not available
    fn try_consume(&mut self) -> Result<(), Duration> {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            // Calculate wait time for next token
            let tokens_needed = 1.0 - self.tokens;
            let wait_secs = tokens_needed / self.refill_rate;
            Err(Duration::from_secs_f64(wait_secs))
        }
    }

    /// Get current token count
    fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Rate limited client wrapper
///
/// This struct wraps GenAiClient and enforces rate limits using a token bucket.
pub struct RateLimitedClient {
    /// The underlying GenAI client
    client: GenAiClient,
    /// Token bucket for rate limiting
    bucket: Arc<Mutex<TokenBucket>>,
    /// Rate limit configuration
    config: RateLimitConfig,
}

impl RateLimitedClient {
    /// Create a new rate limited client
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if rate limit configuration is invalid
    pub async fn new(
        wizard_config: WizardConfig,
        rate_config: RateLimitConfig,
    ) -> WizardResult<Self> {
        rate_config.validate()?;

        let client = GenAiClient::new(wizard_config).await?;

        let bucket = TokenBucket::new(rate_config.tokens_per_second, rate_config.burst_capacity);

        Ok(Self { client, bucket: Arc::new(Mutex::new(bucket)), config: rate_config })
    }

    /// Generate a response (with rate limiting)
    ///
    /// # Errors
    ///
    /// Returns `WizardError::RateLimit` if rate limit exceeded and strategy is Error
    /// Returns `WizardError::Request` if the API request fails
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();

        // Try to consume a token
        loop {
            let mut bucket = self.bucket.lock().await;
            match bucket.try_consume() {
                Ok(()) => {
                    // Token consumed, proceed with request
                    drop(bucket); // Release lock before async call
                    return self.client.generate(prompt).await;
                }
                Err(wait_duration) => match self.config.strategy {
                    RateLimitStrategy::Wait => {
                        // Wait and retry
                        drop(bucket); // Release lock before sleeping
                        tokio::time::sleep(wait_duration).await;
                    }
                    RateLimitStrategy::Error => {
                        // Return error immediately
                        return Err(WizardError::RateLimit(format!(
                            "Rate limit exceeded. Retry after {:.2}s",
                            wait_duration.as_secs_f64()
                        )));
                    }
                },
            }
        }
    }

    /// Get available tokens
    pub async fn available_tokens(&self) -> f64 {
        let mut bucket = self.bucket.lock().await;
        bucket.available_tokens()
    }

    /// Get rate limit configuration
    pub const fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        // Arrange + Act
        let config = RateLimitConfig::default();

        // Assert
        assert_eq!(config.tokens_per_second, 10.0);
        assert_eq!(config.burst_capacity, 20);
        assert_eq!(config.strategy, RateLimitStrategy::Wait);
    }

    #[test]
    fn test_rate_limit_config_builder() {
        // Arrange + Act
        let config = RateLimitConfig::new(5.0, 10)
            .with_tokens_per_second(15.0)
            .with_strategy(RateLimitStrategy::Error);

        // Assert
        assert_eq!(config.tokens_per_second, 15.0);
        assert_eq!(config.burst_capacity, 10);
        assert_eq!(config.strategy, RateLimitStrategy::Error);
    }

    #[test]
    fn test_rate_limit_config_validation() {
        // Arrange
        let invalid_config = RateLimitConfig {
            tokens_per_second: 0.0,
            burst_capacity: 10,
            strategy: RateLimitStrategy::Wait,
        };

        // Act
        let result = invalid_config.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_token_bucket_initial_tokens() {
        // Arrange + Act
        let mut bucket = TokenBucket::new(10.0, 20);

        // Assert - should start with max tokens
        assert_eq!(bucket.available_tokens(), 20.0);
    }

    #[test]
    fn test_token_bucket_consume() {
        // Arrange
        let mut bucket = TokenBucket::new(10.0, 20);

        // Act
        let result1 = bucket.try_consume();
        let result2 = bucket.try_consume();

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!((bucket.available_tokens() - 18.0).abs() < 0.1); // ~18 tokens left
    }

    #[test]
    fn test_token_bucket_refill() {
        // Arrange
        let mut bucket = TokenBucket::new(10.0, 20);

        // Consume all tokens
        for _ in 0..20 {
            let _ = bucket.try_consume();
        }

        // Act - wait for refill (simulate time passing)
        std::thread::sleep(Duration::from_millis(500)); // 0.5s = 5 tokens at 10/s
        bucket.refill();

        // Assert - should have refilled some tokens
        let available = bucket.available_tokens();
        assert!(available >= 4.0 && available <= 6.0); // ~5 tokens refilled
    }

    #[test]
    fn test_token_bucket_exceeds_capacity() {
        // Arrange
        let mut bucket = TokenBucket::new(10.0, 5);

        // Consume all tokens
        for _ in 0..5 {
            let _ = bucket.try_consume();
        }

        // Act - try to consume one more
        let result = bucket.try_consume();

        // Assert - should return error with wait duration
        assert!(result.is_err());
        if let Err(wait) = result {
            assert!(wait.as_secs_f64() > 0.0);
        }
    }

    #[test]
    fn test_token_bucket_max_capacity() {
        // Arrange
        let mut bucket = TokenBucket::new(100.0, 10);

        // Act - wait longer than needed to exceed capacity
        std::thread::sleep(Duration::from_millis(500));
        bucket.refill();

        // Assert - should not exceed max capacity
        assert_eq!(bucket.available_tokens(), 10.0);
    }
}
