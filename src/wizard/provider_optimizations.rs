//! Provider-specific optimizations for OpenAI, Anthropic, and Gemini
//!
//! This module provides provider-specific features and optimizations that leverage
//! unique capabilities of each AI provider.
//!
//! ## Features
//!
//! - OpenAI: Fine-tuning support, vision models, function calling
//! - Anthropic: Claude-specific features, extended context
//! - Gemini: Multi-modal support, code execution

use crate::wizard::{
    config::{Model, Provider},
    error::{WizardError, WizardResult},
    types::{Message, Prompt, WizardResponse},
};
use serde::{Deserialize, Serialize};

/// Provider-specific optimization configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProviderOptimization {
    /// OpenAI-specific optimizations
    OpenAI(OpenAIOptimizations),
    /// Anthropic-specific optimizations
    Anthropic(AnthropicOptimizations),
    /// Gemini-specific optimizations
    Gemini(GeminiOptimizations),
    /// No provider-specific optimizations
    None,
}

impl Default for ProviderOptimization {
    fn default() -> Self {
        Self::None
    }
}

impl ProviderOptimization {
    /// Create provider-specific optimization for a model
    pub fn for_model(model: &Model) -> Self {
        match model.provider() {
            Provider::OpenAI => Self::OpenAI(OpenAIOptimizations::default()),
            Provider::Anthropic => Self::Anthropic(AnthropicOptimizations::default()),
            Provider::Gemini => Self::Gemini(GeminiOptimizations::default()),
            _ => Self::None,
        }
    }

    /// Apply provider-specific prompt optimizations
    pub fn optimize_prompt(&self, prompt: Prompt) -> WizardResult<Prompt> {
        match self {
            Self::OpenAI(opts) => opts.optimize_prompt(prompt),
            Self::Anthropic(opts) => opts.optimize_prompt(prompt),
            Self::Gemini(opts) => opts.optimize_prompt(prompt),
            Self::None => Ok(prompt),
        }
    }
}

/// OpenAI-specific optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAIOptimizations {
    /// Enable function calling (tool use)
    pub enable_function_calling: bool,
    /// Enable vision capabilities (for GPT-4 Vision)
    pub enable_vision: bool,
    /// Use fine-tuned model (if available)
    pub fine_tuned_model: Option<String>,
    /// JSON mode for structured outputs
    pub json_mode: bool,
    /// Logit bias for token probability adjustment
    pub logit_bias: Option<std::collections::HashMap<String, f32>>,
}

impl Default for OpenAIOptimizations {
    fn default() -> Self {
        Self {
            enable_function_calling: false,
            enable_vision: false,
            fine_tuned_model: None,
            json_mode: false,
            logit_bias: None,
        }
    }
}

impl OpenAIOptimizations {
    /// Optimize prompt for OpenAI models
    fn optimize_prompt(&self, mut prompt: Prompt) -> WizardResult<Prompt> {
        // If JSON mode enabled, add instruction to system prompt
        if self.json_mode {
            let json_instruction = "\nIMPORTANT: Respond only with valid JSON.";
            match &prompt.system {
                Some(system) => {
                    prompt.system = Some(format!("{}{}", system, json_instruction));
                }
                None => {
                    prompt.system = Some(json_instruction.to_string());
                }
            }
        }

        // If fine-tuned model specified, add context optimization
        if let Some(_fine_tuned) = &self.fine_tuned_model {
            // Fine-tuned models may need different prompting strategies
            // Add provider-specific context hints
        }

        Ok(prompt)
    }

    /// Create function calling specification
    pub fn create_function_spec(&self, name: &str, description: &str) -> FunctionSpec {
        FunctionSpec { name: name.to_string(), description: description.to_string(), parameters: Vec::new() }
    }
}

/// Anthropic (Claude) specific optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicOptimizations {
    /// Enable extended context window (100K+)
    pub use_extended_context: bool,
    /// Prefill assistant response (Claude-specific)
    pub prefill_response: Option<String>,
    /// Use XML tags for structure (Claude performs better with XML)
    pub use_xml_tags: bool,
}

impl Default for AnthropicOptimizations {
    fn default() -> Self {
        Self { use_extended_context: false, prefill_response: None, use_xml_tags: false }
    }
}

impl AnthropicOptimizations {
    /// Optimize prompt for Anthropic Claude models
    fn optimize_prompt(&self, mut prompt: Prompt) -> WizardResult<Prompt> {
        // If using XML tags, wrap content in XML structure
        if self.use_xml_tags {
            prompt.text = format!("<user_query>{}</user_query>", prompt.text);

            if let Some(system) = &prompt.system {
                prompt.system = Some(format!("<system_context>{}</system_context>", system));
            }
        }

        // If prefill specified, add assistant message
        if let Some(prefill) = &self.prefill_response {
            prompt.history.push(Message::assistant(prefill));
        }

        Ok(prompt)
    }

    /// Add thinking tags for Chain of Thought
    pub fn with_thinking_tags(mut prompt: Prompt) -> Prompt {
        prompt.text = format!(
            "<thinking>\nFirst, let me think through this step by step.\n</thinking>\n\n{}",
            prompt.text
        );
        prompt
    }
}

/// Gemini-specific optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeminiOptimizations {
    /// Enable multi-modal inputs (images, video)
    pub enable_multimodal: bool,
    /// Enable code execution capability
    pub enable_code_execution: bool,
    /// Use grounding with Google Search
    pub use_grounding: bool,
}

impl Default for GeminiOptimizations {
    fn default() -> Self {
        Self { enable_multimodal: false, enable_code_execution: false, use_grounding: false }
    }
}

impl GeminiOptimizations {
    /// Optimize prompt for Gemini models
    fn optimize_prompt(&self, mut prompt: Prompt) -> WizardResult<Prompt> {
        // If grounding enabled, add instruction
        if self.use_grounding {
            let grounding_instruction = "\nUse Google Search to find current, factual information.";
            match &prompt.system {
                Some(system) => {
                    prompt.system = Some(format!("{}{}", system, grounding_instruction));
                }
                None => {
                    prompt.system = Some(grounding_instruction.to_string());
                }
            }
        }

        // If code execution enabled, mention it
        if self.enable_code_execution {
            let code_instruction = "\nYou can execute Python code to verify calculations.";
            if let Some(system) = &prompt.system {
                prompt.system = Some(format!("{}{}", system, code_instruction));
            }
        }

        Ok(prompt)
    }
}

/// Function calling specification (OpenAI)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSpec {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Function parameters
    pub parameters: Vec<FunctionParameter>,
}

/// Function parameter specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (e.g., "string", "number")
    pub param_type: String,
    /// Parameter description
    pub description: String,
    /// Whether parameter is required
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::config::{AnthropicModel, GeminiModel, Model, OpenAIModel};

    #[test]
    fn test_provider_optimization_for_model() {
        // Arrange
        let openai_model = Model::OpenAI(OpenAIModel::Gpt4);
        let anthropic_model = Model::Anthropic(AnthropicModel::Claude3Sonnet);
        let gemini_model = Model::Gemini(GeminiModel::Gemini15Pro);

        // Act
        let openai_opt = ProviderOptimization::for_model(&openai_model);
        let anthropic_opt = ProviderOptimization::for_model(&anthropic_model);
        let gemini_opt = ProviderOptimization::for_model(&gemini_model);

        // Assert
        assert!(matches!(openai_opt, ProviderOptimization::OpenAI(_)));
        assert!(matches!(anthropic_opt, ProviderOptimization::Anthropic(_)));
        assert!(matches!(gemini_opt, ProviderOptimization::Gemini(_)));
    }

    #[test]
    fn test_openai_json_mode() {
        // Arrange
        let mut opts = OpenAIOptimizations::default();
        opts.json_mode = true;
        let prompt = Prompt::new("Give me data");

        // Act
        let optimized = opts.optimize_prompt(prompt).expect("optimize");

        // Assert
        assert!(optimized.system.is_some());
        assert!(optimized.system.as_ref().expect("system should be Some").contains("JSON"));
    }

    #[test]
    fn test_anthropic_xml_tags() {
        // Arrange
        let mut opts = AnthropicOptimizations::default();
        opts.use_xml_tags = true;
        let prompt = Prompt::new("Hello").with_system("System context");

        // Act
        let optimized = opts.optimize_prompt(prompt).expect("optimize");

        // Assert
        assert!(optimized.text.contains("<user_query>"));
        assert!(optimized.system.as_ref().expect("system should be Some").contains("<system_context>"));
    }

    #[test]
    fn test_anthropic_prefill() {
        // Arrange
        let mut opts = AnthropicOptimizations::default();
        opts.prefill_response = Some("I understand. Let me help:".to_string());
        let prompt = Prompt::new("Help me");

        // Act
        let optimized = opts.optimize_prompt(prompt).expect("optimize");

        // Assert
        assert_eq!(optimized.history.len(), 1);
        assert_eq!(optimized.history[0].role, crate::wizard::types::Role::Assistant);
    }

    #[test]
    fn test_anthropic_thinking_tags() {
        // Arrange
        let prompt = Prompt::new("Solve this problem");

        // Act
        let optimized = AnthropicOptimizations::with_thinking_tags(prompt);

        // Assert
        assert!(optimized.text.contains("<thinking>"));
    }

    #[test]
    fn test_gemini_grounding() {
        // Arrange
        let mut opts = GeminiOptimizations::default();
        opts.use_grounding = true;
        let prompt = Prompt::new("What's the latest news?");

        // Act
        let optimized = opts.optimize_prompt(prompt).expect("optimize");

        // Assert
        assert!(optimized.system.is_some());
        assert!(optimized.system.as_ref().expect("system should be Some").contains("Google Search"));
    }

    #[test]
    fn test_openai_function_spec() {
        // Arrange
        let opts = OpenAIOptimizations::default();

        // Act
        let spec = opts.create_function_spec("get_weather", "Get current weather for a location");

        // Assert
        assert_eq!(spec.name, "get_weather");
        assert_eq!(spec.description, "Get current weather for a location");
        assert_eq!(spec.parameters.len(), 0);
    }
}
