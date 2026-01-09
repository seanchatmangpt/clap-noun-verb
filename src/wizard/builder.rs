//! Builder pattern for wizard construction
//!
//! This module provides a type-safe builder for creating wizard instances
//! with fluent configuration.

use super::{Result, WizardConfig, WizardError, WizardResponse};
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
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
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
        self.config.validate().map_err(WizardError::Config)?;

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
        WizardBuilder::new()
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
            eprintln!("[WIZARD] Model: {}", self.config.model);
            eprintln!("[WIZARD] Temperature: {}", self.config.temperature);
            eprintln!("[WIZARD] Input: {}", input);
        }

        // Placeholder response - in production, this would call the AI API
        let response =
            WizardResponse::new(format!("Echo response to: {}", input), self.config.model.clone())
                .with_tokens(42);

        if self.config.verbose {
            eprintln!("[WIZARD] Response: {}", response.content);
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

    #[test]
    fn test_builder_pattern() {
        let wizard = WizardBuilder::new()
            .with_model("gpt-4")
            .with_temperature(0.8)
            .with_max_tokens(1024)
            .build();

        assert!(wizard.is_ok());
        let wizard = wizard.unwrap();
        assert_eq!(wizard.config().model, "gpt-4");
        assert_eq!(wizard.config().temperature, 0.8);
        assert_eq!(wizard.config().max_tokens, 1024);
    }

    #[test]
    fn test_builder_with_context() {
        let wizard = WizardBuilder::new().with_context("Test context").build();

        assert!(wizard.is_ok());
        let wizard = wizard.unwrap();
        assert_eq!(wizard.context(), Some("Test context"));
    }

    #[test]
    fn test_wizard_prompt() {
        let wizard = WizardBuilder::new().with_model("test-model").build().unwrap();

        let response = wizard.prompt("Hello, wizard!");
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.model, "test-model");
        assert!(response.tokens.is_some());
    }

    #[test]
    fn test_invalid_config() {
        let result = WizardBuilder::new().with_model("").build();

        assert!(result.is_err());
    }
}
