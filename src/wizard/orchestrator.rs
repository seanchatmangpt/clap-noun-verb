//! Multi-model orchestrator with intelligent fallback and retry
//!
//! This module coordinates multiple models with automatic fallback on errors,
//! retry logic, and intelligent model selection.
//!
//! ## Features
//!
//! - Automatic model fallback on errors
//! - Integrated retry with exponential backoff
//! - Token budget enforcement across models
//! - Performance metrics tracking
//! - Health checking for models

use crate::wizard::{
    client::GenAiClient,
    config::{ModelConfig, WizardConfig},
    error::{WizardError, WizardResult},
    fallback::{FallbackConfig, SelectionStrategy},
    retry::{RetryConfig, RetryContext},
    token_manager::{TokenBudget, TokenManager},
    types::{Prompt, WizardResponse},
};
use std::time::Instant;

/// Orchestrator for managing multiple models with fallback
pub struct Orchestrator {
    /// Fallback configuration
    fallback_config: FallbackConfig,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Token manager (optional)
    token_manager: Option<TokenManager>,
    /// Current attempt number
    current_attempt: usize,
}

impl Orchestrator {
    /// Create a new orchestrator
    pub fn new(fallback_config: FallbackConfig) -> Self {
        Self {
            fallback_config,
            retry_config: RetryConfig::default(),
            token_manager: None,
            current_attempt: 0,
        }
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Enable token budget management
    pub fn with_token_budget(mut self, budget: TokenBudget) -> Self {
        self.token_manager = Some(TokenManager::new(budget));
        self
    }

    /// Generate response with automatic fallback and retry
    ///
    /// This method attempts to generate a response using the primary model.
    /// If it fails, it automatically falls back to alternative models.
    ///
    /// # Errors
    ///
    /// Returns error if all fallback attempts fail
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();
        self.generate_with_context(prompt).await.map(|(response, _)| response)
    }

    /// Generate response with orchestration context
    ///
    /// Returns both the response and context about fallback/retry attempts
    ///
    /// # Errors
    ///
    /// Returns error if all fallback attempts fail
    pub async fn generate_with_context(
        &mut self,
        prompt: Prompt,
    ) -> WizardResult<(WizardResponse, OrchestrationContext)> {
        let mut context = OrchestrationContext::new();
        let start_time = Instant::now();

        // Check token budget if enabled
        if let Some(token_mgr) = &self.token_manager {
            // Check against primary model
            token_mgr.check_budget(&prompt, &self.fallback_config.primary.model)?;
        }

        // Try each model in fallback chain
        for attempt in 0..self.fallback_config.max_attempts {
            self.current_attempt = attempt;

            // Select model based on strategy
            let model_config = match self.fallback_config.select_model(attempt) {
                Ok(config) => config,
                Err(e) => {
                    context.record_error(attempt, &e);
                    break;
                }
            };

            context.record_attempt(attempt, &model_config.model);

            // Create client for this model
            let wizard_config = WizardConfig {
                model_config: model_config.clone(),
                api_key: None, // API keys loaded from environment
                endpoint: None,
                #[cfg(feature = "caching")]
                enable_cache: false,
                #[cfg(feature = "wizard")]
                streaming_config: None,
                #[cfg(feature = "wizard")]
                cache_config: None,
                #[cfg(feature = "wizard")]
                rate_limit_config: None,
                #[cfg(feature = "wizard")]
                retry_config: Some(self.retry_config),
                #[cfg(feature = "wizard")]
                fallback_config: None,
            };

            let mut client = match GenAiClient::new(wizard_config).await {
                Ok(c) => c,
                Err(e) => {
                    context.record_error(attempt, &e);
                    continue; // Try next fallback
                }
            };

            // Attempt generation with retry
            match self.generate_with_retry(&mut client, prompt.clone()).await {
                Ok(response) => {
                    // Record token usage
                    if let (Some(token_mgr), Some(usage)) = (&mut self.token_manager, &response.usage) {
                        token_mgr.record_usage(usage);
                    }

                    context.total_latency_ms = start_time.elapsed().as_millis() as u64;
                    return Ok((response, context));
                }
                Err(e) => {
                    context.record_error(attempt, &e);
                    // Continue to next fallback
                }
            }
        }

        // All attempts failed
        Err(WizardError::Other(format!(
            "All orchestration attempts failed after {} tries",
            self.fallback_config.max_attempts
        )))
    }

    /// Generate with retry logic
    async fn generate_with_retry(
        &self,
        client: &mut GenAiClient,
        prompt: Prompt,
    ) -> WizardResult<WizardResponse> {
        let mut last_error = None;

        for retry_attempt in 0..self.retry_config.max_attempts {
            match client.generate(prompt.clone()).await {
                Ok(response) => {
                    return Ok(response);
                }
                Err(error) => {
                    // Check if we should retry
                    if !Self::should_retry(&error) {
                        return Err(error);
                    }

                    last_error = Some(error.clone());

                    // Wait before retry (except on last attempt)
                    if retry_attempt + 1 < self.retry_config.max_attempts {
                        let delay = self.retry_config.calculate_delay(retry_attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| WizardError::Other("Retry failed".to_string())))
    }

    /// Determine if an error should trigger retry
    fn should_retry(error: &WizardError) -> bool {
        matches!(
            error,
            WizardError::Request(_) | WizardError::Network(_) | WizardError::Timeout(_) | WizardError::RateLimit(_)
        )
    }

    /// Get current orchestration statistics
    pub fn stats(&self) -> OrchestrationStats {
        OrchestrationStats {
            total_tokens_used: self.token_manager.as_ref().map(|tm| tm.total_tokens_used()).unwrap_or(0),
            remaining_budget: self.token_manager.as_ref().and_then(|tm| tm.remaining_budget()),
            current_attempt: self.current_attempt,
        }
    }
}

/// Context about orchestration attempts
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrchestrationContext {
    /// Models attempted (in order)
    pub models_attempted: Vec<String>,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Total latency across all attempts
    pub total_latency_ms: u64,
}

impl OrchestrationContext {
    fn new() -> Self {
        Self::default()
    }

    fn record_attempt(&mut self, _attempt: usize, model: &crate::wizard::config::Model) {
        self.models_attempted.push(model.model_id());
    }

    fn record_error(&mut self, _attempt: usize, error: &WizardError) {
        self.errors.push(error.to_string());
    }
}

/// Orchestration statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrchestrationStats {
    /// Total tokens used across all requests
    pub total_tokens_used: usize,
    /// Remaining token budget (if enabled)
    pub remaining_budget: Option<usize>,
    /// Current attempt number
    pub current_attempt: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::config::{AnthropicModel, Model, ModelConfig};

    #[test]
    fn test_orchestrator_new() {
        // Arrange
        let primary = ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet));
        let fallback_config = FallbackConfig::new(primary);

        // Act
        let orchestrator = Orchestrator::new(fallback_config);

        // Assert
        assert_eq!(orchestrator.current_attempt, 0);
        assert!(orchestrator.token_manager.is_none());
    }

    #[test]
    fn test_orchestrator_with_token_budget() {
        // Arrange
        let primary = ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet));
        let fallback_config = FallbackConfig::new(primary);
        let budget = TokenBudget::new(1000);

        // Act
        let orchestrator = Orchestrator::new(fallback_config).with_token_budget(budget);

        // Assert
        assert!(orchestrator.token_manager.is_some());
    }

    #[test]
    fn test_should_retry_logic() {
        // Arrange & Act & Assert
        assert!(Orchestrator::should_retry(&WizardError::Request("network error".to_string())));
        assert!(Orchestrator::should_retry(&WizardError::RateLimit("limit exceeded".to_string())));
        assert!(!Orchestrator::should_retry(&WizardError::Config("bad config".to_string())));
        assert!(!Orchestrator::should_retry(&WizardError::Auth("auth failed".to_string())));
    }

    #[test]
    fn test_orchestration_context() {
        // Arrange
        let mut context = OrchestrationContext::new();
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);
        let error = WizardError::Request("test error".to_string());

        // Act
        context.record_attempt(0, &model);
        context.record_error(0, &error);

        // Assert
        assert_eq!(context.models_attempted.len(), 1);
        assert_eq!(context.errors.len(), 1);
        assert!(context.errors[0].contains("test error"));
    }

    #[test]
    fn test_stats() {
        // Arrange
        let primary = ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet));
        let fallback_config = FallbackConfig::new(primary);
        let budget = TokenBudget::new(1000).with_max_total(10000);
        let orchestrator = Orchestrator::new(fallback_config).with_token_budget(budget);

        // Act
        let stats = orchestrator.stats();

        // Assert
        assert_eq!(stats.total_tokens_used, 0);
        assert_eq!(stats.remaining_budget, Some(10000));
        assert_eq!(stats.current_attempt, 0);
    }
}
