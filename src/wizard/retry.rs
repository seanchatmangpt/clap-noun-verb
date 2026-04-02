//! Retry logic with exponential backoff for wizard AI interactions
//!
//! This module provides configurable retry logic with exponential backoff,
//! jitter, and selective retry strategies for transient failures.
//!
//! ## Features
//!
//! - Exponential backoff with jitter
//! - Configurable attempt count and initial delay
//! - Selective retry (don't retry auth errors)
//! - Retry context and metrics tracking
//! - Per-error-type retry policies

use crate::wizard::{
    client::GenAiClient,
    config::WizardConfig,
    error::{WizardError, WizardResult},
    types::{Prompt, WizardResponse},
};
use rand::Rng;
use std::time::Duration;

/// Configuration for retry logic
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial backoff delay
    pub initial_delay_ms: u64,
    /// Maximum backoff delay
    pub max_delay_ms: u64,
    /// Backoff multiplier (typically 2.0 for exponential)
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 = no jitter, 1.0 = full jitter)
    pub jitter_factor: f64,
    /// Whether to retry on rate limit errors
    pub retry_on_rate_limit: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,  // 1 second
            max_delay_ms: 30000,     // 30 seconds
            backoff_multiplier: 2.0, // Exponential
            jitter_factor: 0.5,      // 50% jitter
            retry_on_rate_limit: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub const fn new(max_attempts: usize, initial_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.5,
            retry_on_rate_limit: true,
        }
    }

    /// Set maximum retry attempts
    pub const fn with_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set initial delay in milliseconds
    pub const fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Set maximum delay in milliseconds
    pub const fn with_max_delay(mut self, max_delay_ms: u64) -> Self {
        self.max_delay_ms = max_delay_ms;
        self
    }

    /// Set backoff multiplier
    pub const fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set jitter factor (0.0 to 1.0)
    pub const fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter_factor = jitter;
        self
    }

    /// Set whether to retry on rate limit errors
    pub const fn with_rate_limit_retry(mut self, retry: bool) -> Self {
        self.retry_on_rate_limit = retry;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> WizardResult<()> {
        if self.max_attempts == 0 {
            return Err(WizardError::Config("max_attempts must be > 0".to_string()));
        }
        if self.initial_delay_ms == 0 {
            return Err(WizardError::Config("initial_delay_ms must be > 0".to_string()));
        }
        if self.max_delay_ms < self.initial_delay_ms {
            return Err(WizardError::Config(
                "max_delay_ms must be >= initial_delay_ms".to_string(),
            ));
        }
        if self.backoff_multiplier <= 0.0 {
            return Err(WizardError::Config("backoff_multiplier must be > 0".to_string()));
        }
        if !(0.0..=1.0).contains(&self.jitter_factor) {
            return Err(WizardError::Config(
                "jitter_factor must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(())
    }

    /// Calculate the delay for a given attempt (with jitter)
    fn calculate_delay(&self, attempt: usize) -> Duration {
        // Base exponential backoff: initial_delay * multiplier^attempt
        let base_delay_ms =
            self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);

        // Cap at max delay
        let capped_delay_ms = base_delay_ms.min(self.max_delay_ms as f64);

        // Add jitter: randomize between (1 - jitter) * delay and delay
        let jitter_range = capped_delay_ms * self.jitter_factor;
        let min_delay = capped_delay_ms - jitter_range;

        let mut rng = rand::thread_rng();
        let jittered_delay_ms = rng.gen_range(min_delay..=capped_delay_ms);

        Duration::from_millis(jittered_delay_ms as u64)
    }
}

/// Retry context tracking
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetryContext {
    /// Number of attempts made
    pub attempts: usize,
    /// Total time spent in retries
    pub total_delay_ms: u64,
    /// Last error encountered
    pub last_error: Option<String>,
}

impl RetryContext {
    fn new() -> Self {
        Self::default()
    }

    fn record_attempt(&mut self, delay_ms: u64, error: &WizardError) {
        self.attempts += 1;
        self.total_delay_ms += delay_ms;
        self.last_error = Some(error.to_string());
    }
}

/// Retry client wrapper
///
/// This struct wraps GenAiClient and provides automatic retry with exponential backoff.
pub struct RetryClient {
    /// The underlying GenAI client
    client: GenAiClient,
    /// Retry configuration
    config: RetryConfig,
}

impl RetryClient {
    /// Create a new retry client
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if retry configuration is invalid
    pub async fn new(wizard_config: WizardConfig, retry_config: RetryConfig) -> WizardResult<Self> {
        retry_config.validate()?;

        let client = GenAiClient::new(wizard_config).await?;

        Ok(Self { client, config: retry_config })
    }

    /// Generate a response (with retry)
    ///
    /// # Errors
    ///
    /// Returns the last error if all retry attempts fail
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();
        self.generate_with_context(prompt).await.map(|(response, _)| response)
    }

    /// Generate a response with retry context
    ///
    /// Returns both the response and retry context (attempts, delays, errors)
    ///
    /// # Errors
    ///
    /// Returns the last error if all retry attempts fail
    pub async fn generate_with_context(
        &mut self,
        prompt: Prompt,
    ) -> WizardResult<(WizardResponse, RetryContext)> {
        let mut context = RetryContext::new();
        let mut last_error = None;

        for attempt in 0..self.config.max_attempts {
            match self.client.generate(prompt.clone()).await {
                Ok(response) => {
                    return Ok((response, context));
                }
                Err(error) => {
                    // Check if we should retry this error
                    if !Self::should_retry(&error, &self.config) {
                        return Err(error);
                    }

                    last_error = Some(error.clone());

                    // If this is not the last attempt, wait and retry
                    if attempt + 1 < self.config.max_attempts {
                        let delay = self.config.calculate_delay(attempt);
                        context.record_attempt(delay.as_millis() as u64, &error);
                        tokio::time::sleep(delay).await;
                    } else {
                        // Last attempt failed, record it
                        context.record_attempt(0, &error);
                    }
                }
            }
        }

        // All attempts failed
        Err(last_error
            .unwrap_or_else(|| WizardError::Other("All retry attempts failed".to_string())))
    }

    /// Determine if an error should be retried
    fn should_retry(error: &WizardError, config: &RetryConfig) -> bool {
        match error {
            // Don't retry authentication or configuration errors
            WizardError::Auth(_) | WizardError::Config(_) => false,

            // Retry rate limits based on config
            WizardError::RateLimit(_) => config.retry_on_rate_limit,

            // Retry transient errors (network, timeout, etc.)
            WizardError::Request(_) | WizardError::Timeout(_) | WizardError::Network(_) => true,

            // Don't retry other errors by default
            _ => false,
        }
    }

    /// Get retry configuration
    pub const fn config(&self) -> &RetryConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        // Arrange + Act
        let config = RetryConfig::default();

        // Assert
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.jitter_factor, 0.5);
    }

    #[test]
    fn test_retry_config_builder() {
        // Arrange + Act
        let config =
            RetryConfig::new(5, 500).with_max_delay(60000).with_multiplier(1.5).with_jitter(0.3);

        // Assert
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 60000);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.jitter_factor, 0.3);
    }

    #[test]
    fn test_retry_config_validation() {
        // Arrange
        let invalid_config = RetryConfig { max_attempts: 0, ..Default::default() };

        // Act
        let result = invalid_config.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_calculate_delay() {
        // Arrange
        let config = RetryConfig {
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
            jitter_factor: 0.0, // No jitter for predictable test
            ..Default::default()
        };

        // Act
        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);

        // Assert - exponential backoff: 1000, 2000, 4000
        assert_eq!(delay0.as_millis(), 1000);
        assert_eq!(delay1.as_millis(), 2000);
        assert_eq!(delay2.as_millis(), 4000);
    }

    #[test]
    fn test_retry_config_delay_capped() {
        // Arrange
        let config = RetryConfig {
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 5000,
            jitter_factor: 0.0,
            ..Default::default()
        };

        // Act
        let delay10 = config.calculate_delay(10); // Would be 1024000ms without cap

        // Assert - should be capped at max_delay
        assert_eq!(delay10.as_millis(), 5000);
    }

    #[test]
    fn test_retry_config_delay_with_jitter() {
        // Arrange
        let config = RetryConfig {
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
            jitter_factor: 0.5, // 50% jitter
            ..Default::default()
        };

        // Act - calculate multiple delays to check jitter range
        let delays: Vec<u64> =
            (0..10).map(|_| config.calculate_delay(0).as_millis() as u64).collect();

        // Assert - all delays should be within jitter range [500, 1000]
        for delay in delays {
            assert!(delay >= 500 && delay <= 1000);
        }
    }

    #[test]
    fn test_retry_context() {
        // Arrange
        let mut context = RetryContext::new();
        let error = WizardError::Request("test error".to_string());

        // Act
        context.record_attempt(1000, &error);
        context.record_attempt(2000, &error);

        // Assert
        assert_eq!(context.attempts, 2);
        assert_eq!(context.total_delay_ms, 3000);
        assert!(context.last_error.is_some());
    }

    #[test]
    fn test_should_retry_logic() {
        // Arrange
        let config = RetryConfig::default();

        // Act & Assert - should not retry auth errors
        assert!(!RetryClient::should_retry(&WizardError::Auth("auth failed".to_string()), &config));

        // Should retry request errors
        assert!(RetryClient::should_retry(
            &WizardError::Request("network error".to_string()),
            &config
        ));

        // Should retry rate limits based on config
        assert!(RetryClient::should_retry(
            &WizardError::RateLimit("limit exceeded".to_string()),
            &config
        ));

        // Test with rate limit retry disabled
        let config_no_rate_retry = RetryConfig { retry_on_rate_limit: false, ..Default::default() };
        assert!(!RetryClient::should_retry(
            &WizardError::RateLimit("limit exceeded".to_string()),
            &config_no_rate_retry
        ));
    }
}
