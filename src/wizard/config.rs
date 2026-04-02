//! Configuration types for the wizard module
//!
//! This module defines type-safe configuration for AI models and providers.
//! Follows type-first thinking: use enums instead of strings for compile-time safety.

use crate::wizard::error::{WizardError, WizardResult};
use serde::{Deserialize, Serialize};

/// Supported AI model providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// OpenAI (GPT models)
    OpenAI,
    /// Anthropic (Claude models)
    Anthropic,
    /// Google Gemini
    Gemini,
    /// DeepSeek
    DeepSeek,
    /// xAI (Grok)
    #[serde(rename = "xai")]
    XAI,
    /// Groq
    Groq,
    /// Cohere
    Cohere,
    /// Ollama (local models)
    Ollama,
}

impl Provider {
    /// Get the environment variable name for this provider's API key
    pub const fn env_var_name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OPENAI_API_KEY",
            Provider::Anthropic => "ANTHROPIC_API_KEY",
            Provider::Gemini => "GEMINI_API_KEY",
            Provider::DeepSeek => "DEEPSEEK_API_KEY",
            Provider::XAI => "XAI_API_KEY",
            Provider::Groq => "GROQ_API_KEY",
            Provider::Cohere => "COHERE_API_KEY",
            Provider::Ollama => "OLLAMA_HOST", // Ollama uses host, not API key
        }
    }

    /// Get the default API endpoint for this provider (if applicable)
    pub const fn default_endpoint(&self) -> Option<&'static str> {
        match self {
            Provider::Ollama => Some("http://localhost:11434"),
            _ => None, // Most providers use rust-genai's built-in endpoints
        }
    }
}

/// Type-safe model selection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "provider", content = "model")]
pub enum Model {
    /// OpenAI GPT models
    #[serde(rename = "openai")]
    OpenAI(OpenAIModel),
    /// Anthropic Claude models
    #[serde(rename = "anthropic")]
    Anthropic(AnthropicModel),
    /// Google Gemini models
    #[serde(rename = "gemini")]
    Gemini(GeminiModel),
    /// Custom model (provider:model format)
    #[serde(rename = "custom")]
    Custom { provider: String, model: String },
}

impl Model {
    /// Get the provider for this model
    pub const fn provider(&self) -> Provider {
        match self {
            Model::OpenAI(_) => Provider::OpenAI,
            Model::Anthropic(_) => Provider::Anthropic,
            Model::Gemini(_) => Provider::Gemini,
            Model::Custom { .. } => Provider::OpenAI, // Default for custom
        }
    }

    /// Get the model identifier string for rust-genai
    pub fn model_id(&self) -> String {
        match self {
            Model::OpenAI(m) => m.as_str().to_string(),
            Model::Anthropic(m) => m.as_str().to_string(),
            Model::Gemini(m) => m.as_str().to_string(),
            Model::Custom { provider, model } => format!("{}:{}", provider, model),
        }
    }

    /// Get the maximum context window size for this model (in tokens)
    pub const fn max_tokens(&self) -> usize {
        match self {
            Model::OpenAI(OpenAIModel::Gpt4) => 8192,
            Model::OpenAI(OpenAIModel::Gpt4Turbo) => 128000,
            Model::OpenAI(OpenAIModel::Gpt35Turbo) => 16384,
            Model::Anthropic(AnthropicModel::Claude3Opus) => 200000,
            Model::Anthropic(AnthropicModel::Claude3Sonnet) => 200000,
            Model::Anthropic(AnthropicModel::Claude3Haiku) => 200000,
            Model::Gemini(GeminiModel::Gemini15Pro) => 2000000,
            Model::Gemini(GeminiModel::Gemini15Flash) => 1000000,
            Model::Custom { .. } => 8192, // Conservative default
        }
    }
}

/// OpenAI model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpenAIModel {
    /// GPT-4 (8K context)
    Gpt4,
    /// GPT-4 Turbo (128K context)
    Gpt4Turbo,
    /// GPT-3.5 Turbo (16K context)
    Gpt35Turbo,
}

impl OpenAIModel {
    const fn as_str(self) -> &'static str {
        match self {
            OpenAIModel::Gpt4 => "gpt-4",
            OpenAIModel::Gpt4Turbo => "gpt-4-turbo",
            OpenAIModel::Gpt35Turbo => "gpt-3.5-turbo",
        }
    }
}

/// Anthropic Claude model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnthropicModel {
    /// Claude 3 Opus (most capable)
    Claude3Opus,
    /// Claude 3 Sonnet (balanced)
    Claude3Sonnet,
    /// Claude 3 Haiku (fastest)
    Claude3Haiku,
}

impl AnthropicModel {
    const fn as_str(self) -> &'static str {
        match self {
            AnthropicModel::Claude3Opus => "claude-3-opus-20240229",
            AnthropicModel::Claude3Sonnet => "claude-3-sonnet-20240229",
            AnthropicModel::Claude3Haiku => "claude-3-haiku-20240307",
        }
    }
}

/// Google Gemini model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeminiModel {
    /// Gemini 1.5 Pro (2M context)
    Gemini15Pro,
    /// Gemini 1.5 Flash (1M context, faster)
    Gemini15Flash,
}

impl GeminiModel {
    const fn as_str(self) -> &'static str {
        match self {
            GeminiModel::Gemini15Pro => "gemini-1.5-pro",
            GeminiModel::Gemini15Flash => "gemini-1.5-flash",
        }
    }
}

/// Model generation parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    /// The model to use
    pub model: Model,
    /// Temperature (0.0-2.0, higher = more random)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Top-p sampling (0.0-1.0)
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    /// Maximum tokens to generate
    #[serde(default = "default_max_response_tokens")]
    pub max_response_tokens: usize,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_top_p() -> f32 {
    1.0
}

fn default_max_response_tokens() -> usize {
    4096
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: Model::Anthropic(AnthropicModel::Claude3Sonnet),
            temperature: default_temperature(),
            top_p: default_top_p(),
            max_response_tokens: default_max_response_tokens(),
        }
    }
}

impl ModelConfig {
    /// Create a new model configuration
    pub fn new(model: Model) -> Self {
        Self { model, ..Default::default() }
    }

    /// Set the temperature (0.0-2.0)
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// Set the top-p sampling parameter (0.0-1.0)
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p.clamp(0.0, 1.0);
        self
    }

    /// Set the maximum response tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_response_tokens = max_tokens;
        self
    }

    /// Validate that the configuration is valid
    pub fn validate(&self) -> WizardResult<()> {
        // Check if max_response_tokens exceeds model's context window
        if self.max_response_tokens > self.model.max_tokens() {
            return Err(WizardError::TokenLimit {
                requested: self.max_response_tokens,
                max: self.model.max_tokens(),
            });
        }

        Ok(())
    }
}

/// Complete wizard configuration
#[derive(Debug, Clone)]
pub struct WizardConfig {
    /// Model configuration
    pub model_config: ModelConfig,
    /// API key for the provider
    pub api_key: Option<String>,
    /// Custom API endpoint (if applicable)
    pub endpoint: Option<String>,
    /// Enable caching (requires "caching" feature)
    #[cfg(feature = "caching")]
    pub enable_cache: bool,
    /// v2 feature configs
    #[cfg(feature = "wizard")]
    pub streaming_config: Option<crate::wizard::streaming::StreamingConfig>,
    #[cfg(feature = "wizard")]
    pub cache_config: Option<crate::wizard::cache::CacheConfig>,
    #[cfg(feature = "wizard")]
    pub rate_limit_config: Option<crate::wizard::rate_limit::RateLimitConfig>,
    #[cfg(feature = "wizard")]
    pub retry_config: Option<crate::wizard::retry::RetryConfig>,
    #[cfg(feature = "wizard")]
    pub fallback_config: Option<crate::wizard::fallback::FallbackConfig>,
}

impl Default for WizardConfig {
    fn default() -> Self {
        Self {
            model_config: ModelConfig::default(),
            api_key: None,
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
            retry_config: None,
            #[cfg(feature = "wizard")]
            fallback_config: None,
        }
    }
}

impl WizardConfig {
    /// Create a new configuration with the specified model
    pub fn new(model: Model) -> Self {
        Self { model_config: ModelConfig::new(model), ..Default::default() }
    }

    /// Load configuration from environment variables
    ///
    /// Reads:
    /// - `WIZARD_MODEL` - Model identifier (e.g., "anthropic:claude-3-sonnet")
    /// - `{PROVIDER}_API_KEY` - API key for the provider
    /// - `{PROVIDER}_ENDPOINT` - Custom endpoint (optional)
    pub fn from_env() -> WizardResult<Self> {
        // Get model from environment or use default
        let model = std::env::var("WIZARD_MODEL")
            .ok()
            .and_then(|s| Self::parse_model_string(&s))
            .unwrap_or_else(|| Model::Anthropic(AnthropicModel::Claude3Sonnet));

        let provider = model.provider();

        // Get API key from environment
        let api_key = match provider {
            Provider::Ollama => None, // Ollama doesn't require API key
            _ => {
                let key = std::env::var(provider.env_var_name())?;
                Some(key)
            }
        };

        // Get endpoint from environment (optional)
        let endpoint = std::env::var(format!("{}_ENDPOINT", provider.env_var_name()))
            .ok()
            .or_else(|| provider.default_endpoint().map(String::from));

        Ok(Self {
            model_config: ModelConfig::new(model),
            api_key,
            endpoint,
            #[cfg(feature = "caching")]
            enable_cache: std::env::var("WIZARD_ENABLE_CACHE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            #[cfg(feature = "wizard")]
            streaming_config: None,
            #[cfg(feature = "wizard")]
            cache_config: None,
            #[cfg(feature = "wizard")]
            rate_limit_config: None,
            #[cfg(feature = "wizard")]
            retry_config: None,
            #[cfg(feature = "wizard")]
            fallback_config: None,
        })
    }

    /// Parse a model string (e.g., "anthropic:claude-3-sonnet")
    fn parse_model_string(s: &str) -> Option<Model> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.as_slice() {
            ["openai", "gpt-4"] => Some(Model::OpenAI(OpenAIModel::Gpt4)),
            ["openai", "gpt-4-turbo"] => Some(Model::OpenAI(OpenAIModel::Gpt4Turbo)),
            ["openai", "gpt-3.5-turbo"] => Some(Model::OpenAI(OpenAIModel::Gpt35Turbo)),
            ["anthropic", "claude-3-opus"] => Some(Model::Anthropic(AnthropicModel::Claude3Opus)),
            ["anthropic", "claude-3-sonnet"] => {
                Some(Model::Anthropic(AnthropicModel::Claude3Sonnet))
            }
            ["anthropic", "claude-3-haiku"] => Some(Model::Anthropic(AnthropicModel::Claude3Haiku)),
            ["gemini", "gemini-1.5-pro"] => Some(Model::Gemini(GeminiModel::Gemini15Pro)),
            ["gemini", "gemini-1.5-flash"] => Some(Model::Gemini(GeminiModel::Gemini15Flash)),
            [provider, model] => Some(Model::Custom {
                provider: (*provider).to_string(),
                model: (*model).to_string(),
            }),
            _ => None,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> WizardResult<()> {
        self.model_config.validate()?;

        // Check if API key is present (except for Ollama)
        if self.model_config.model.provider() != Provider::Ollama && self.api_key.is_none() {
            return Err(WizardError::Config(format!(
                "API key required for provider '{:?}'",
                self.model_config.model.provider()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_env_var_names() {
        // Arrange + Act + Assert
        assert_eq!(Provider::OpenAI.env_var_name(), "OPENAI_API_KEY");
        assert_eq!(Provider::Anthropic.env_var_name(), "ANTHROPIC_API_KEY");
        assert_eq!(Provider::Ollama.env_var_name(), "OLLAMA_HOST");
    }

    #[test]
    fn test_model_provider() {
        // Arrange
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);

        // Act
        let provider = model.provider();

        // Assert
        assert_eq!(provider, Provider::Anthropic);
    }

    #[test]
    fn test_model_id() {
        // Arrange
        let model = Model::OpenAI(OpenAIModel::Gpt4Turbo);

        // Act
        let id = model.model_id();

        // Assert
        assert_eq!(id, "gpt-4-turbo");
    }

    #[test]
    fn test_model_max_tokens() {
        // Arrange
        let model = Model::Anthropic(AnthropicModel::Claude3Opus);

        // Act
        let max = model.max_tokens();

        // Assert
        assert_eq!(max, 200000);
    }

    #[test]
    fn test_model_config_default() {
        // Arrange + Act
        let config = ModelConfig::default();

        // Assert
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.top_p, 1.0);
        assert_eq!(config.max_response_tokens, 4096);
    }

    #[test]
    fn test_model_config_builder() {
        // Arrange
        let model = Model::OpenAI(OpenAIModel::Gpt4);

        // Act
        let config = ModelConfig::new(model.clone())
            .with_temperature(0.5)
            .with_top_p(0.9)
            .with_max_tokens(2048);

        // Assert
        assert_eq!(config.model, model);
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.max_response_tokens, 2048);
    }

    #[test]
    fn test_model_config_temperature_clamping() {
        // Arrange + Act
        let config = ModelConfig::default().with_temperature(5.0);

        // Assert - should clamp to 2.0
        assert_eq!(config.temperature, 2.0);
    }

    #[test]
    fn test_model_config_validate_token_limit() {
        // Arrange
        let config = ModelConfig::new(Model::OpenAI(OpenAIModel::Gpt4)).with_max_tokens(100000); // Exceeds GPT-4's 8192 limit

        // Act
        let result = config.validate();

        // Assert
        assert!(result.is_err());
        match result {
            Err(WizardError::TokenLimit { requested, max }) => {
                assert_eq!(requested, 100000);
                assert_eq!(max, 8192);
            }
            _ => {
                assert!(false, "Expected TokenLimit error");
            }
        }
    }

    #[test]
    fn test_parse_model_string() {
        // Arrange + Act
        let model = WizardConfig::parse_model_string("anthropic:claude-3-sonnet");

        // Assert
        assert_eq!(model, Some(Model::Anthropic(AnthropicModel::Claude3Sonnet)));
    }

    #[test]
    fn test_parse_custom_model_string() {
        // Arrange + Act
        let model = WizardConfig::parse_model_string("custom:my-model");

        // Assert
        assert_eq!(
            model,
            Some(Model::Custom { provider: "custom".to_string(), model: "my-model".to_string() })
        );
    }
}
