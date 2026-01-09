//! Prompt types and builders for wizard interactions
//!
//! Provides type-safe prompt construction with validation and template support.

use super::error::{Result, WizardError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A validated prompt for AI interaction
///
/// Prompts are immutable once created and must pass validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// The prompt text
    text: String,

    /// System message (optional)
    system: Option<String>,

    /// Maximum tokens for response
    max_tokens: Option<usize>,

    /// Temperature for response generation (0.0-1.0)
    temperature: Option<f32>,

    /// Additional metadata
    metadata: HashMap<String, String>,
}

impl Prompt {
    /// Get the prompt text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the system message if present
    pub fn system(&self) -> Option<&str> {
        self.system.as_deref()
    }

    /// Get the max tokens setting
    pub fn max_tokens(&self) -> Option<usize> {
        self.max_tokens
    }

    /// Get the temperature setting
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Get metadata value by key
    pub fn metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Get all metadata
    pub fn all_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Validate prompt constraints
    fn validate(&self) -> Result<()> {
        // Check text is not empty
        if self.text.trim().is_empty() {
            return Err(WizardError::InvalidPrompt(
                "Prompt text cannot be empty".to_string(),
            ));
        }

        // Validate temperature range
        if let Some(temp) = self.temperature {
            if !(0.0..=1.0).contains(&temp) {
                return Err(WizardError::InvalidPrompt(format!(
                    "Temperature must be between 0.0 and 1.0, got {}",
                    temp
                )));
            }
        }

        // Validate max_tokens is reasonable
        if let Some(tokens) = self.max_tokens {
            if tokens == 0 {
                return Err(WizardError::InvalidPrompt(
                    "max_tokens must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Builder for constructing validated prompts
///
/// Uses the builder pattern for ergonomic prompt construction
/// with compile-time guarantees about required fields.
pub struct PromptBuilder {
    text: Option<String>,
    system: Option<String>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    metadata: HashMap<String, String>,
}

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new() -> Self {
        Self {
            text: None,
            system: None,
            max_tokens: None,
            temperature: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the prompt text (required)
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set the system message (optional)
    pub fn system<S: Into<String>>(mut self, system: S) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set maximum tokens for response
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Set temperature for response generation (0.0-1.0)
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Add metadata key-value pair
    pub fn metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build and validate the prompt
    pub fn build(self) -> Result<Prompt> {
        let text = self
            .text
            .ok_or_else(|| WizardError::InvalidPrompt("Prompt text is required".to_string()))?;

        let prompt = Prompt {
            text,
            system: self.system,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            metadata: self.metadata,
        };

        // Validate before returning
        prompt.validate()?;
        Ok(prompt)
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Template for creating prompts with variable substitution
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    template: String,
    variables: Vec<String>,
}

impl PromptTemplate {
    /// Create a new prompt template
    ///
    /// Template uses `{{variable}}` syntax for variable placeholders
    pub fn new<S: Into<String>>(template: S) -> Self {
        let template = template.into();
        let variables = Self::extract_variables(&template);
        Self {
            template,
            variables,
        }
    }

    /// Extract variable names from template
    fn extract_variables(template: &str) -> Vec<String> {
        let mut vars = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                let mut var_name = String::new();

                while let Some(c) = chars.next() {
                    if c == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second '}'
                        vars.push(var_name.trim().to_string());
                        break;
                    }
                    var_name.push(c);
                }
            }
        }

        vars
    }

    /// Get list of required variables
    pub fn variables(&self) -> &[String] {
        &self.variables
    }

    /// Render template with provided values
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String> {
        let mut result = self.template.clone();

        for var in &self.variables {
            let value = values.get(var).ok_or_else(|| {
                WizardError::InvalidPrompt(format!("Missing template variable: {}", var))
            })?;

            let placeholder = format!("{{{{{}}}}}", var);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Create a prompt from this template
    pub fn to_prompt(&self, values: &HashMap<String, String>) -> Result<Prompt> {
        let text = self.render(values)?;
        PromptBuilder::new().text(text).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_success() {
        let prompt = PromptBuilder::new()
            .text("What is the meaning of life?")
            .system("You are a helpful assistant")
            .max_tokens(100)
            .temperature(0.7)
            .metadata("user_id", "123")
            .build();

        assert!(prompt.is_ok());
        let prompt = prompt.ok().unwrap();
        assert_eq!(prompt.text(), "What is the meaning of life?");
        assert_eq!(prompt.system(), Some("You are a helpful assistant"));
        assert_eq!(prompt.max_tokens(), Some(100));
        assert_eq!(prompt.temperature(), Some(0.7));
        assert_eq!(prompt.metadata("user_id"), Some("123"));
    }

    #[test]
    fn test_prompt_builder_missing_text() {
        let result = PromptBuilder::new()
            .system("System message")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_validation_empty_text() {
        let result = PromptBuilder::new().text("   ").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_validation_invalid_temperature() {
        let result = PromptBuilder::new()
            .text("Test")
            .temperature(1.5)
            .build();
        assert!(result.is_err());

        let result = PromptBuilder::new()
            .text("Test")
            .temperature(-0.1)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_validation_zero_tokens() {
        let result = PromptBuilder::new()
            .text("Test")
            .max_tokens(0)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_template_variable_extraction() {
        let template = PromptTemplate::new("Hello {{name}}, your age is {{age}}");
        let vars = template.variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"age".to_string()));
    }

    #[test]
    fn test_template_render() {
        let template = PromptTemplate::new("Hello {{name}}, your role is {{role}}");
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());
        values.insert("role".to_string(), "Developer".to_string());

        let result = template.render(&values);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), "Hello Alice, your role is Developer");
    }

    #[test]
    fn test_template_missing_variable() {
        let template = PromptTemplate::new("Hello {{name}}");
        let values = HashMap::new();

        let result = template.render(&values);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_to_prompt() {
        let template = PromptTemplate::new("Process {{action}} for {{item}}");
        let mut values = HashMap::new();
        values.insert("action".to_string(), "delete".to_string());
        values.insert("item".to_string(), "file.txt".to_string());

        let result = template.to_prompt(&values);
        assert!(result.is_ok());
        let prompt = result.ok().unwrap();
        assert_eq!(prompt.text(), "Process delete for file.txt");
    }
}
