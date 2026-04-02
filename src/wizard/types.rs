//! Type definitions for wizard requests and responses
//!
//! This module defines the core data structures for interacting with AI models.

use serde::{Deserialize, Serialize};

/// A prompt to send to an AI model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    /// The prompt text
    pub text: String,
    /// Optional system prompt
    pub system: Option<String>,
    /// Optional conversation history (for multi-turn conversations)
    pub history: Vec<Message>,
}

impl Prompt {
    /// Create a new simple prompt
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), system: None, history: Vec::new() }
    }

    /// Set the system prompt
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Add a message to the conversation history
    pub fn with_message(mut self, role: Role, content: impl Into<String>) -> Self {
        self.history.push(Message { role, content: content.into() });
        self
    }

    /// Add conversation history
    pub fn with_history(mut self, history: Vec<Message>) -> Self {
        self.history = history;
        self
    }
}

impl From<String> for Prompt {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for Prompt {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

/// A message in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender
    pub role: Role,
    /// The message content
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self { role, content: content.into() }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message
    User,
    /// AI assistant message
    Assistant,
    /// System message
    System,
}

/// Response from an AI model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizardResponse {
    /// The generated text
    pub text: String,
    /// Token usage statistics (if available)
    pub usage: Option<TokenUsage>,
    /// Model that generated the response
    pub model: String,
    /// Response metadata
    pub metadata: ResponseMetadata,
}

impl std::fmt::Display for WizardResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)?;
        if let Some(usage) = &self.usage {
            write!(f, "\n[{} tokens used with {}]", usage.total_tokens, self.model)?;
        }
        Ok(())
    }
}

impl WizardResponse {
    /// Create a new response
    pub fn new(text: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            usage: None,
            model: model.into(),
            metadata: ResponseMetadata::default(),
        }
    }

    /// Set token usage
    pub fn with_usage(mut self, usage: TokenUsage) -> Self {
        self.usage = Some(usage);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: ResponseMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: usize,
    /// Tokens in the completion
    pub completion_tokens: usize,
    /// Total tokens (prompt + completion)
    pub total_tokens: usize,
}

impl TokenUsage {
    /// Create a new token usage statistic
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self { prompt_tokens, completion_tokens, total_tokens: prompt_tokens + completion_tokens }
    }
}

/// Response metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResponseMetadata {
    /// Finish reason (e.g., "stop", "length", "content_filter")
    pub finish_reason: Option<String>,
    /// Whether the response was retrieved from cache
    #[cfg(feature = "caching")]
    pub from_cache: bool,
    /// Response latency in milliseconds
    pub latency_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_new() {
        // Arrange + Act
        let prompt = Prompt::new("Hello, AI!");

        // Assert
        assert_eq!(prompt.text, "Hello, AI!");
        assert!(prompt.system.is_none());
        assert!(prompt.history.is_empty());
    }

    #[test]
    fn test_prompt_with_system() {
        // Arrange + Act
        let prompt =
            Prompt::new("What is Rust?").with_system("You are a helpful programming assistant.");

        // Assert
        assert_eq!(prompt.text, "What is Rust?");
        assert_eq!(prompt.system, Some("You are a helpful programming assistant.".to_string()));
    }

    #[test]
    fn test_prompt_with_history() {
        // Arrange
        let history = vec![Message::user("Hello"), Message::assistant("Hi there! How can I help?")];

        // Act
        let prompt = Prompt::new("Tell me about Rust").with_history(history.clone());

        // Assert
        assert_eq!(prompt.history.len(), 2);
        assert_eq!(prompt.history[0].role, Role::User);
        assert_eq!(prompt.history[1].role, Role::Assistant);
    }

    #[test]
    fn test_message_constructors() {
        // Arrange + Act
        let user_msg = Message::user("Hello");
        let assistant_msg = Message::assistant("Hi");
        let system_msg = Message::system("You are helpful");

        // Assert
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(assistant_msg.role, Role::Assistant);
        assert_eq!(system_msg.role, Role::System);
    }

    #[test]
    fn test_wizard_response_new() {
        // Arrange + Act
        let response = WizardResponse::new("Hello!", "gpt-4");

        // Assert
        assert_eq!(response.text, "Hello!");
        assert_eq!(response.model, "gpt-4");
        assert!(response.usage.is_none());
    }

    #[test]
    fn test_wizard_response_with_usage() {
        // Arrange
        let usage = TokenUsage::new(100, 50);

        // Act
        let response = WizardResponse::new("Response", "claude-3-sonnet").with_usage(usage);

        // Assert
        assert_eq!(response.usage, Some(usage));
        assert_eq!(response.usage.map(|u| u.total_tokens), Some(150));
    }

    #[test]
    fn test_token_usage() {
        // Arrange + Act
        let usage = TokenUsage::new(100, 50);

        // Assert
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_prompt_from_string() {
        // Arrange
        let text = "Hello".to_string();

        // Act
        let prompt: Prompt = text.into();

        // Assert
        assert_eq!(prompt.text, "Hello");
    }

    #[test]
    fn test_prompt_from_str() {
        // Arrange + Act
        let prompt: Prompt = "Hello".into();

        // Assert
        assert_eq!(prompt.text, "Hello");
    }
}
