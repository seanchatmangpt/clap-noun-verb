//! Fallback chain for model switching on failure
//!
//! This module provides fallback chains for automatic model switching when
//! a primary model fails. Useful for handling rate limits, outages, or cost optimization.
//!
//! ## Features
//!
//! - Sequential fallback strategy (try models in order)
//! - Configurable fallback decision criteria
//! - Error aggregation across attempts
//! - Fallback statistics tracking
//! - Per-model configuration support

use crate::wizard::{
    config::{Model, ModelConfig, WizardConfig},
    error::{WizardError, WizardResult},
    types::{Prompt, WizardResponse},
};

/// Configuration for fallback behavior
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackConfig {
    /// Chain of models to try (in order)
    pub model_chain: Vec<ModelConfig>,
    /// Whether to stop on first success
    pub stop_on_success: bool,
    /// Whether to aggregate errors from all attempts
    pub aggregate_errors: bool,
}

impl FallbackConfig {
    /// Create a new fallback configuration
    pub fn new(model_chain: Vec<ModelConfig>) -> Self {
        Self { model_chain, stop_on_success: true, aggregate_errors: true }
    }

    /// Set whether to stop on first success
    pub fn with_stop_on_success(mut self, stop: bool) -> Self {
        self.stop_on_success = stop;
        self
    }

    /// Set whether to aggregate errors
    pub fn with_aggregate_errors(mut self, aggregate: bool) -> Self {
        self.aggregate_errors = aggregate;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> WizardResult<()> {
        if self.model_chain.is_empty() {
            return Err(WizardError::Config("Fallback model chain cannot be empty".to_string()));
        }
        Ok(())
    }
}

/// Fallback attempt result
#[derive(Debug, Clone)]
pub struct FallbackAttempt {
    /// The model that was tried
    pub model: Model,
    /// The result of the attempt
    pub result: Result<WizardResponse, String>,
    /// Attempt number (0-indexed)
    pub attempt_index: usize,
}

impl FallbackAttempt {
    fn success(model: Model, response: WizardResponse, attempt_index: usize) -> Self {
        Self { model, result: Ok(response), attempt_index }
    }

    fn failure(model: Model, error: String, attempt_index: usize) -> Self {
        Self { model, result: Err(error), attempt_index }
    }

    /// Check if this attempt was successful
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if this attempt failed
    pub fn is_failure(&self) -> bool {
        self.result.is_err()
    }
}

/// Fallback statistics
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FallbackStats {
    /// Total fallback attempts made
    pub total_attempts: u64,
    /// Number of times fallback succeeded
    pub successes: u64,
    /// Number of times all models failed
    pub failures: u64,
    /// Number of times primary model succeeded (no fallback needed)
    pub primary_successes: u64,
}

impl FallbackStats {
    /// Calculate fallback rate (how often fallback was needed)
    pub fn fallback_rate(&self) -> f64 {
        let total = self.successes + self.failures + self.primary_successes;
        if total == 0 {
            0.0
        } else {
            (self.successes + self.failures) as f64 / total as f64
        }
    }

    /// Calculate success rate (including fallback successes)
    pub fn success_rate(&self) -> f64 {
        let total = self.successes + self.failures + self.primary_successes;
        if total == 0 {
            0.0
        } else {
            (self.successes + self.primary_successes) as f64 / total as f64
        }
    }
}

/// Fallback client with model chain
///
/// This struct manages a chain of models and automatically falls back to
/// the next model in the chain if the current one fails.
pub struct FallbackClient {
    /// Wizard configuration for client creation
    wizard_config: WizardConfig,
    /// Fallback configuration
    fallback_config: FallbackConfig,
    /// Fallback statistics
    stats: FallbackStats,
}

impl FallbackClient {
    /// Create a new fallback client
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if fallback configuration is invalid
    pub fn new(wizard_config: WizardConfig, fallback_config: FallbackConfig) -> WizardResult<Self> {
        fallback_config.validate()?;

        Ok(Self { wizard_config, fallback_config, stats: FallbackStats::default() })
    }

    /// Generate a response with fallback
    ///
    /// Tries each model in the chain until one succeeds or all fail.
    ///
    /// # Errors
    ///
    /// Returns error if all models in the chain fail
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();
        self.generate_with_attempts(prompt).await.map(|(response, _)| response)
    }

    /// Generate a response with detailed attempt information
    ///
    /// Returns both the response and all attempts made (for debugging/monitoring)
    ///
    /// # Errors
    ///
    /// Returns error if all models in the chain fail
    pub async fn generate_with_attempts(
        &mut self,
        prompt: Prompt,
    ) -> WizardResult<(WizardResponse, Vec<FallbackAttempt>)> {
        let mut attempts = Vec::new();
        let mut errors = Vec::new();

        for (index, model_config) in self.fallback_config.model_chain.iter().enumerate() {
            // Create a temporary client for this model
            let mut temp_config = self.wizard_config.clone();
            temp_config.model_config = model_config.clone();

            // Create client (may fail if API key missing, etc.)
            let mut client = match crate::wizard::client::GenAiClient::new(temp_config).await {
                Ok(client) => client,
                Err(error) => {
                    errors.push(error.to_string());
                    attempts.push(FallbackAttempt::failure(
                        model_config.model.clone(),
                        error.to_string(),
                        index,
                    ));
                    continue;
                }
            };

            // Try to generate with this model
            self.stats.total_attempts += 1;

            match client.generate(prompt.clone()).await {
                Ok(response) => {
                    // Success!
                    attempts.push(FallbackAttempt::success(
                        model_config.model.clone(),
                        response.clone(),
                        index,
                    ));

                    if index == 0 {
                        self.stats.primary_successes += 1;
                    } else {
                        self.stats.successes += 1;
                    }

                    return Ok((response, attempts));
                }
                Err(error) => {
                    // Failure, record and try next model
                    errors.push(error.to_string());
                    attempts.push(FallbackAttempt::failure(
                        model_config.model.clone(),
                        error.to_string(),
                        index,
                    ));

                    // Check if we should continue trying
                    if !Self::should_retry(&error) {
                        // Non-retryable error, stop trying
                        self.stats.failures += 1;
                        return Err(error);
                    }
                }
            }
        }

        // All models failed
        self.stats.failures += 1;

        if self.fallback_config.aggregate_errors {
            let aggregated_error = format!(
                "All {} models in fallback chain failed: [{}]",
                self.fallback_config.model_chain.len(),
                errors.join("; ")
            );
            Err(WizardError::Fallback(aggregated_error))
        } else {
            // Return the last error
            Err(WizardError::Fallback(
                errors.last().cloned().unwrap_or_else(|| "All models failed".to_string()),
            ))
        }
    }

    /// Determine if an error should trigger fallback
    fn should_retry(error: &WizardError) -> bool {
        match error {
            // Retry transient errors
            WizardError::Request(_)
            | WizardError::Timeout(_)
            | WizardError::RateLimit(_)
            | WizardError::Network(_) => true,

            // Don't retry auth, config, or parse errors (likely to fail on all models)
            WizardError::Auth(_) | WizardError::Config(_) | WizardError::Parse(_) => false,

            // Retry other errors by default
            _ => true,
        }
    }

    /// Get fallback statistics
    pub const fn stats(&self) -> &FallbackStats {
        &self.stats
    }

    /// Get fallback configuration
    pub const fn config(&self) -> &FallbackConfig {
        &self.fallback_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::config::{AnthropicModel, OpenAIModel};

    #[test]
    fn test_fallback_config_validation() {
        // Arrange
        let empty_config = FallbackConfig::new(vec![]);

        // Act
        let result = empty_config.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_fallback_config_builder() {
        // Arrange
        let models = vec![
            ModelConfig::new(Model::OpenAI(OpenAIModel::Gpt4)),
            ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet)),
        ];

        // Act
        let config = FallbackConfig::new(models.clone())
            .with_stop_on_success(false)
            .with_aggregate_errors(false);

        // Assert
        assert_eq!(config.model_chain.len(), 2);
        assert!(!config.stop_on_success);
        assert!(!config.aggregate_errors);
    }

    #[test]
    fn test_fallback_attempt_success() {
        // Arrange
        let model = Model::OpenAI(OpenAIModel::Gpt4);
        let response = WizardResponse::new("test", "gpt-4");

        // Act
        let attempt = FallbackAttempt::success(model, response, 0);

        // Assert
        assert!(attempt.is_success());
        assert!(!attempt.is_failure());
    }

    #[test]
    fn test_fallback_attempt_failure() {
        // Arrange
        let model = Model::OpenAI(OpenAIModel::Gpt4);

        // Act
        let attempt = FallbackAttempt::failure(model, "test error".to_string(), 0);

        // Assert
        assert!(!attempt.is_success());
        assert!(attempt.is_failure());
    }

    #[test]
    fn test_fallback_stats_rates() {
        // Arrange
        let stats = FallbackStats {
            total_attempts: 100,
            successes: 20,
            failures: 10,
            primary_successes: 70,
        };

        // Act
        let fallback_rate = stats.fallback_rate();
        let success_rate = stats.success_rate();

        // Assert
        assert!((fallback_rate - 0.3).abs() < 0.001); // 30% needed fallback
        assert!((success_rate - 0.9).abs() < 0.001); // 90% success rate
    }

    #[test]
    fn test_fallback_stats_zero_requests() {
        // Arrange
        let stats = FallbackStats::default();

        // Act
        let fallback_rate = stats.fallback_rate();
        let success_rate = stats.success_rate();

        // Assert
        assert_eq!(fallback_rate, 0.0);
        assert_eq!(success_rate, 0.0);
    }

    #[test]
    fn test_should_retry_logic() {
        // Arrange + Act & Assert
        // Should retry transient errors
        assert!(FallbackClient::should_retry(&WizardError::Request("network error".to_string())));
        assert!(FallbackClient::should_retry(&WizardError::RateLimit(
            "limit exceeded".to_string()
        )));

        // Should not retry auth/config errors
        assert!(!FallbackClient::should_retry(&WizardError::Auth("invalid key".to_string())));
        assert!(!FallbackClient::should_retry(&WizardError::Config("bad config".to_string())));
    }
}
