//! Builder pattern for wizard construction
//!
//! This module provides a type-safe builder for creating wizard instances
//! with fluent configuration.

use super::{Model, Result, TokenUsage, WizardConfig, WizardError, WizardResponse};
use std::sync::Arc;

/// Builder for creating wizard instances
pub struct WizardBuilder {
    config: WizardConfig,
    context: Option<String>,
}

impl WizardBuilder {
    /// Create a new wizard builder with default configuration
    pub fn new() -> Self {
        Self { config: WizardConfig::default(), context: None }
    }

    /// Set the AI model
    pub fn with_model(mut self, model: Model) -> Self {
        self.config.model_config.model = model;
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.model_config.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.config.model_config.max_response_tokens = max_tokens;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = Some(api_key.into());
        self
    }

    /// Set configuration directly

    pub fn with_config(mut self, config: WizardConfig) -> Self {
        self.config = config;
        self
    }

    /// Add context for the wizard session
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Build the wizard instance
    pub fn build(self) -> Result<Wizard> {
        // Validate configuration
        self.config.validate().map_err(|e| match e {
            WizardError::Config(msg) => WizardError::Config(msg),
            _ => WizardError::Config(format!("{}", e)),
        })?;

        Ok(Wizard { config: Arc::new(self.config), context: self.context })
    }
}

impl Default for WizardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Wizard instance for AI interactions
pub struct Wizard {
    config: Arc<WizardConfig>,
    context: Option<String>,
}

impl Wizard {
    /// Create a new wizard builder
    pub fn builder() -> WizardBuilder {
        WizardBuilder::new().with_api_key("dummy-key")
    }

    /// Get the wizard configuration
    pub fn config(&self) -> &WizardConfig {
        &self.config
    }

    /// Get the wizard context
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    /// Execute a prompt and get a response
    ///
    /// Note: This is a placeholder implementation. In production, this would
    /// call an actual AI API (OpenAI, Anthropic, etc.)
    pub fn prompt(&self, input: impl Into<String>) -> Result<WizardResponse> {
        let input = input.into();

        if self.config.verbose {
            eprintln!("[WIZARD] Model: {}", self.config.model_config.model);
            eprintln!("[WIZARD] Temperature: {}", self.config.model_config.temperature);
            eprintln!("[WIZARD] Input: {}", input);
        }

        // Placeholder response - in production, this would call the AI API
        let response = WizardResponse::new(
            format!("Echo response to: {}", input),
            self.config.model_config.model.model_id(),
        )
        .with_usage(TokenUsage::new(input.len() / 4, 42));

        if self.config.verbose {
            eprintln!("[WIZARD] Response: {}", response.text);
        }

        Ok(response)
    }

    /// Execute a prompt with custom parameters
    pub fn prompt_with_params(
        &self,
        input: impl Into<String>,
        params: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<WizardResponse> {
        let input = input.into();

        if self.config.verbose {
            eprintln!("[WIZARD] Params: {:?}", params);
        }

        // Merge custom params with config params
        let mut merged_params = self.config.parameters.clone();
        merged_params.extend(params);

        self.prompt(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::config::{AnthropicModel, Model};

    #[test]
    fn test_builder_pattern() {
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);
        let wizard = WizardBuilder::new()
            .with_api_key("dummy-key")
            .with_model(model.clone())
            .with_temperature(0.8)
            .with_max_tokens(1024)
            .build()
            .expect("Failed to build wizard");

        assert_eq!(wizard.config().model_config.model, model);
        assert_eq!(wizard.config().model_config.temperature, 0.8);
        assert_eq!(wizard.config().model_config.max_response_tokens, 1024);
    }

    #[test]
    fn test_builder_with_context() {
        let wizard = WizardBuilder::new()
            .with_api_key("dummy-key")
            .with_context("Test context")
            .build()
            .expect("Failed to build wizard with context");

        assert_eq!(wizard.context(), Some("Test context"));
    }

    #[test]
    fn test_wizard_prompt() {
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);
        let wizard = WizardBuilder::new()
            .with_api_key("dummy-key")
            .with_model(model.clone())
            .build()
            .expect("Failed to build wizard for prompt test");

        let response = wizard.prompt("Hello, wizard!").expect("Failed to get prompt response");

        assert_eq!(response.model, model.model_id());
        assert!(response.usage.is_some());
    }
}
