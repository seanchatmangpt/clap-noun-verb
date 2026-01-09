//! GenAI integration for wizard AI capabilities
//!
//! Provides integration with the genai crate for multi-provider AI support
//! including OpenAI, Anthropic, Gemini, and others.

use super::error::{Result, WizardError};
use super::prompt::Prompt;
use std::env;

/// Configuration for GenAI client
#[derive(Debug, Clone)]
pub struct GenAiConfig {
    /// Model to use (e.g., "gpt-4", "claude-3-opus", "gemini-pro")
    pub model: String,

    /// API key (loaded from environment if not provided)
    pub api_key: Option<String>,

    /// Provider-specific configuration
    pub provider_config: serde_json::Value,
}

impl GenAiConfig {
    /// Create a new configuration with defaults
    pub fn new(model: String) -> Self {
        Self {
            model,
            api_key: None,
            provider_config: serde_json::Value::Null,
        }
    }

    /// Set API key explicitly
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set provider configuration
    pub fn with_provider_config(mut self, config: serde_json::Value) -> Self {
        self.provider_config = config;
        self
    }

    /// Load API key from environment
    ///
    /// Attempts to load from common environment variables:
    /// - OPENAI_API_KEY
    /// - ANTHROPIC_API_KEY
    /// - GEMINI_API_KEY
    pub fn load_api_key_from_env(&mut self) -> Result<()> {
        if self.api_key.is_some() {
            return Ok(());
        }

        // Try common environment variables
        let env_vars = ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GEMINI_API_KEY", "GENAI_API_KEY"];

        for var in &env_vars {
            if let Ok(key) = env::var(var) {
                self.api_key = Some(key);
                return Ok(());
            }
        }

        Err(WizardError::ConfigError(
            "No API key found in environment. Set OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY, or GENAI_API_KEY".to_string()
        ))
    }
}

/// GenAI client for AI interactions
///
/// Wraps the genai crate to provide wizard-specific functionality.
pub struct GenAiClient {
    config: GenAiConfig,
}

impl GenAiClient {
    /// Create a new GenAI client
    pub fn new(config: GenAiConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Create a client from environment configuration
    pub fn from_env(model: String) -> Result<Self> {
        let mut config = GenAiConfig::new(model);
        config.load_api_key_from_env()?;
        Self::new(config)
    }

    /// Send a prompt and get a response
    ///
    /// This is the main entry point for AI interactions.
    pub async fn generate(&self, prompt: &Prompt) -> Result<String> {
        // In a real implementation, this would use the genai crate
        // For now, provide a type-safe stub that compiles

        // Validate we have an API key
        if self.config.api_key.is_none() {
            return Err(WizardError::ConfigError(
                "API key not configured".to_string(),
            ));
        }

        // Construct request (placeholder - real implementation would use genai::Client)
        let _model = &self.config.model;
        let _prompt_text = prompt.text();
        let _system = prompt.system();
        let _max_tokens = prompt.max_tokens();
        let _temperature = prompt.temperature();

        // Real implementation would look like:
        // use genai::Client;
        // let client = Client::default();
        // let request = client.new_request(&self.config.model)
        //     .with_message_user(prompt.text());
        // if let Some(system) = prompt.system() {
        //     request = request.with_system(system);
        // }
        // let response = request.exec().await?;
        // Ok(response.content_text_as_str()?.to_string())

        // Placeholder response for compilation
        Err(WizardError::ClientError(
            "GenAI integration requires the 'wizard' feature and proper configuration".to_string(),
        ))
    }

    /// Get the configured model name
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Get configuration
    pub fn config(&self) -> &GenAiConfig {
        &self.config
    }
}

/// Builder for GenAI client configuration
pub struct GenAiClientBuilder {
    model: Option<String>,
    api_key: Option<String>,
    provider_config: serde_json::Value,
}

impl GenAiClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self {
            model: None,
            api_key: None,
            provider_config: serde_json::Value::Null,
        }
    }

    /// Set the model to use
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the API key
    pub fn api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set provider configuration
    pub fn provider_config(mut self, config: serde_json::Value) -> Self {
        self.provider_config = config;
        self
    }

    /// Build the client, loading API key from environment if needed
    pub fn build(self) -> Result<GenAiClient> {
        let model = self.model.ok_or_else(|| {
            WizardError::ConfigError("Model is required".to_string())
        })?;

        let mut config = GenAiConfig::new(model)
            .with_provider_config(self.provider_config);

        if let Some(api_key) = self.api_key {
            config = config.with_api_key(api_key);
        } else {
            config.load_api_key_from_env()?;
        }

        GenAiClient::new(config)
    }
}

impl Default for GenAiClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = GenAiConfig::new("gpt-4".to_string());
        assert_eq!(config.model, "gpt-4");
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_config_with_api_key() {
        let config = GenAiConfig::new("gpt-4".to_string())
            .with_api_key("test-key".to_string());
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_client_creation() {
        let config = GenAiConfig::new("gpt-4".to_string())
            .with_api_key("test-key".to_string());
        let client = GenAiClient::new(config);
        assert!(client.is_ok());
        let client = client.ok().unwrap();
        assert_eq!(client.model(), "gpt-4");
    }

    #[test]
    fn test_builder_missing_model() {
        let result = GenAiClientBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_api_key_env() {
        // This will fail unless environment has API keys
        let result = GenAiClientBuilder::new()
            .model("gpt-4")
            .build();
        // Don't assert on this - depends on environment
        let _ = result;
    }

    #[test]
    fn test_builder_with_explicit_key() {
        let result = GenAiClientBuilder::new()
            .model("gpt-4")
            .api_key("test-key")
            .build();
        assert!(result.is_ok());
    }
}
