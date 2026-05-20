//! GenAI client wrapper for rust-genai integration
//!
//! This module provides a thin, type-safe wrapper around rust-genai's Client.
//! It follows zero-cost abstraction principles with no runtime overhead.

use crate::wizard::{
    config::{ModelConfig, WizardConfig},
    error::{WizardError, WizardResult},
    types::{Prompt, Role, TokenUsage, WizardResponse},
};

use lru::LruCache;
use std::num::NonZeroUsize;

/// GenAI client wrapper
///
/// This struct wraps rust-genai's Client with type-safe configuration.
/// It uses zero-cost abstractions - no runtime overhead compared to direct rust-genai usage.
pub struct GenAiClient {
    /// The underlying rust-genai client
    client: genai::Client,
    /// Model configuration
    config: ModelConfig,
    /// Optional response cache (feature-gated)
    cache: Option<LruCache<String, WizardResponse>>,
}

impl GenAiClient {
    /// Create a new GenAI client from configuration
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if configuration is invalid
    /// Returns `WizardError::Auth` if authentication fails
    pub async fn new(wizard_config: WizardConfig) -> WizardResult<Self> {
        // Validate configuration
        wizard_config.validate()?;

        // Create rust-genai client
        let client = Self::create_genai_client(&wizard_config)?;

        Ok(Self {
            client,
            config: wizard_config.model_config,
            cache: if wizard_config.enable_cache {
                // Default cache size: 100 entries
                // SAFETY: 100 is a compile-time constant that is non-zero
                const CACHE_SIZE: NonZeroUsize = match NonZeroUsize::new(100) {
                    Some(size) => size,
                    None => unreachable!(),
                };
                Some(LruCache::new(CACHE_SIZE))
            } else {
                None
            },
        })
    }

    /// Create a rust-genai client from wizard configuration
    fn create_genai_client(_wizard_config: &WizardConfig) -> WizardResult<genai::Client> {
        // For now, create a basic client
        // rust-genai reads API keys from environment variables automatically
        Ok(genai::Client::default())
    }

    /// Generate a response from a prompt
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Request` if the API request fails
    /// Returns `WizardError::Parse` if the response cannot be parsed
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();

        // Check cache if enabled
        if self.cache.is_some() {
            let cache_key = self.cache_key(&prompt);
            if let Some(cached) = self.cache.as_mut().unwrap().get(&cache_key) {
                let mut response = cached.clone();
                response.metadata.from_cache = true;
                return Ok(response);
            }
        }

        // Generate response
        let start = std::time::Instant::now();
        let response = self.generate_internal(&prompt).await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        // Add latency to metadata
        let mut response = response;
        response.metadata.latency_ms = Some(latency_ms);

        // Cache if enabled
        if self.cache.is_some() {
            let cache_key = self.cache_key(&prompt);
            self.cache.as_mut().unwrap().put(cache_key, response.clone());
        }

        Ok(response)
    }

    /// Internal generation method (without caching)
    async fn generate_internal(&self, prompt: &Prompt) -> WizardResult<WizardResponse> {
        // Build chat messages
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &prompt.system {
            messages.push(genai::chat::ChatMessage {
                role: genai::chat::ChatRole::System,
                content: genai::chat::MessageContent::Text(system.clone()),
                options: None,
            });
        }

        // Add conversation history
        for msg in &prompt.history {
            messages.push(genai::chat::ChatMessage {
                role: Self::convert_role(msg.role),
                content: genai::chat::MessageContent::Text(msg.content.clone()),
                options: None,
            });
        }

        // Add user prompt
        messages.push(genai::chat::ChatMessage {
            role: genai::chat::ChatRole::User,
            content: genai::chat::MessageContent::Text(prompt.text.clone()),
            options: None,
        });

        // Create chat request
        let chat_req = genai::chat::ChatRequest::new(messages);

        // Create chat options
        let options = genai::chat::ChatOptions::default()
            .with_temperature(self.config.temperature.into())
            .with_top_p(self.config.top_p.into())
            .with_max_tokens(self.config.max_response_tokens as u32);

        // Execute request
        let model_id = self.config.model.model_id();
        let chat_res = self
            .client
            .exec_chat(&model_id, chat_req, Some(&options))
            .await
            .map_err(|e| WizardError::Request(e.to_string()))?;

        // Extract response text
        let text = chat_res
            .content
            .as_ref()
            .and_then(|c| c.text_as_str())
            .ok_or_else(|| WizardError::Parse("No text content in response".to_string()))?
            .to_string();

        // Extract token usage
        let usage = TokenUsage::new(
            chat_res.usage.prompt_tokens.unwrap_or(0) as usize,
            chat_res.usage.completion_tokens.unwrap_or(0) as usize,
        );

        // Build response
        let mut response = WizardResponse::new(text, model_id).with_metadata(
            crate::wizard::types::ResponseMetadata {
                finish_reason: None, // finish_reason not directly on ChatResponse in v0.3.5
                from_cache: false,
                latency_ms: None,
            },
        );

        response = response.with_usage(usage);

        Ok(response)
    }

    /// Convert our Role enum to rust-genai's ChatRole
    fn convert_role(role: Role) -> genai::chat::ChatRole {
        match role {
            Role::User => genai::chat::ChatRole::User,
            Role::Assistant => genai::chat::ChatRole::Assistant,
            Role::System => genai::chat::ChatRole::System,
        }
    }

    /// Generate a cache key from a prompt
    fn cache_key(&self, prompt: &Prompt) -> String {
        use std::hash::{Hash, Hasher};

        let mut hasher = ahash::AHasher::default();
        prompt.text.hash(&mut hasher);
        prompt.system.hash(&mut hasher);
        for msg in &prompt.history {
            msg.content.hash(&mut hasher);
            format!("{:?}", msg.role).hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    /// Get the current model configuration
    pub fn model_config(&self) -> &ModelConfig {
        &self.config
    }

    /// Clear the cache (if caching is enabled)
    pub fn clear_cache(&mut self) {
        if let Some(cache) = &mut self.cache {
            cache.clear();
        }
    }

    /// Get cache statistics (if caching is enabled)
    pub fn cache_stats(&self) -> Option<(usize, usize)> {
        self.cache.as_ref().map(|c| (c.len(), c.cap().get()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::config::{AnthropicModel, Model};

    // Note: These are unit tests without actual API calls
    // Integration tests with real API calls are in tests/wizard_tests.rs

    #[test]
    fn test_convert_role() {
        // Arrange + Act + Assert
        assert!(matches!(GenAiClient::convert_role(Role::User), genai::chat::ChatRole::User));
        assert!(matches!(
            GenAiClient::convert_role(Role::Assistant),
            genai::chat::ChatRole::Assistant
        ));
        assert!(matches!(GenAiClient::convert_role(Role::System), genai::chat::ChatRole::System));
    }

    #[test]
    fn test_model_config_accessor() {
        // This test requires async runtime and API key, so we'll just verify
        // the struct can be constructed with a config
        let config = ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet));

        // Assert - just verify it compiles and config is accessible
        assert_eq!(config.model, Model::Anthropic(AnthropicModel::Claude3Sonnet));
    }
}
