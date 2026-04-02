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
    /// Get the prompt text (zero-cost reference)
    #[inline(always)]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the system message if present (zero-cost reference)
    #[inline(always)]
    pub fn system(&self) -> Option<&str> {
        self.system.as_deref()
    }

    /// Get the max tokens setting (zero-cost copy of usize)
    #[inline(always)]
    pub fn max_tokens(&self) -> Option<usize> {
        self.max_tokens
    }

    /// Get the temperature setting (zero-cost copy of f32)
    #[inline(always)]
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Get metadata value by key (zero-cost reference)
    #[inline]
    pub fn metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Get all metadata (zero-cost reference)
    #[inline(always)]
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
    /// Create a new prompt builder with pre-allocated metadata capacity
    pub fn new() -> Self {
        Self::with_metadata_capacity(4)
    }

    /// Create a new prompt builder with specified metadata capacity
    #[inline]
    pub fn with_metadata_capacity(capacity: usize) -> Self {
        Self {
            text: None,
            system: None,
            max_tokens: None,
            temperature: None,
            metadata: HashMap::with_capacity(capacity),
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

    /// Extract variable names from template with capacity pre-allocation
    fn extract_variables(template: &str) -> Vec<String> {
        // Pre-allocate: estimate 1 variable per 20 chars (reasonable default)
        let estimated_capacity = (template.len() / 20).max(2);
        let mut vars = Vec::with_capacity(estimated_capacity);
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                let mut var_name = String::with_capacity(16); // typical variable name length

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

    /// Render template with provided values (optimized to reduce allocations)
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String> {
        // Pre-validate all variables exist before starting replacements
        for var in &self.variables {
            if !values.contains_key(var) {
                return Err(WizardError::InvalidPrompt(format!("Missing template variable: {}", var)));
            }
        }

        // Estimate result size: template + average value length * variable count
        let avg_value_len = values.values().map(|v| v.len()).sum::<usize>() / values.len().max(1);
        let estimated_size = self.template.len() + (avg_value_len * self.variables.len());
        let mut result = String::with_capacity(estimated_size);

        let mut last_end = 0;
        let template_bytes = self.template.as_bytes();
        let mut i = 0;

        // Single-pass rendering to avoid multiple allocations
        while i < template_bytes.len() {
            if i + 1 < template_bytes.len()
                && template_bytes[i] == b'{'
                && template_bytes[i + 1] == b'{' {
                // Found potential variable start
                let var_start = i + 2;
                let mut var_end = var_start;

                // Find variable end
                while var_end + 1 < template_bytes.len() {
                    if template_bytes[var_end] == b'}' && template_bytes[var_end + 1] == b'}' {
                        // Extract variable name
                        let var_name = &self.template[var_start..var_end].trim();

                        // Add text before variable
                        result.push_str(&self.template[last_end..i]);

                        // Add variable value (we validated existence above)
                        if let Some(value) = values.get(*var_name) {
                            result.push_str(value);
                        }

                        i = var_end + 2;
                        last_end = i;
                        break;
                    }
                    var_end += 1;
                }

                if var_end + 1 >= template_bytes.len() {
                    i += 1; // Not a valid variable, move forward
                }
            } else {
                i += 1;
            }
        }

        // Add remaining text
        result.push_str(&self.template[last_end..]);
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
            .build()
            .expect("Failed to build prompt");

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

        let result = template.render(&values)
            .expect("Failed to render template");
        assert_eq!(result, "Hello Alice, your role is Developer");
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

        let prompt = template.to_prompt(&values)
            .expect("Failed to create prompt from template");
        assert_eq!(prompt.text(), "Process delete for file.txt");
    }
}
