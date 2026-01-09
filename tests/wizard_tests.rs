//! Integration tests for the wizard module
//!
//! These tests use Chicago TDD principles:
//! - State-based testing with real collaborators
//! - AAA pattern (Arrange-Act-Assert)
//! - Behavior verification (observable outputs)
//!
//! Note: These tests require valid API keys in environment variables.
//! Run with: cargo test --features wizard --test wizard_tests
//!
//! To skip API tests (unit tests only):
//! cargo test --features wizard --lib

#![cfg(feature = "wizard")]

use clap_noun_verb::wizard::{
    GenAiClient, Message, Model, ModelConfig, Prompt, Role, WizardConfig,
};

#[test]
fn test_prompt_creation() {
    // Arrange
    let text = "What is Rust?";

    // Act
    let prompt = Prompt::new(text);

    // Assert
    assert_eq!(prompt.text, text);
    assert!(prompt.system.is_none());
    assert!(prompt.history.is_empty());
}

#[test]
fn test_prompt_with_system() {
    // Arrange
    let text = "Explain ownership";
    let system = "You are a Rust expert";

    // Act
    let prompt = Prompt::new(text).with_system(system);

    // Assert
    assert_eq!(prompt.text, text);
    assert_eq!(prompt.system.as_deref(), Some(system));
}

#[test]
fn test_prompt_with_history() {
    // Arrange
    let history = vec![Message::user("Hello"), Message::assistant("Hi! How can I help?")];

    // Act
    let prompt = Prompt::new("Tell me about Rust").with_history(history.clone());

    // Assert
    assert_eq!(prompt.history.len(), 2);
    assert_eq!(prompt.history[0].role, Role::User);
    assert_eq!(prompt.history[0].content, "Hello");
    assert_eq!(prompt.history[1].role, Role::Assistant);
}

#[test]
fn test_model_config_validation() {
    // Arrange
    let config = ModelConfig::new(Model::OpenAI(clap_noun_verb::wizard::config::OpenAIModel::Gpt4))
        .with_max_tokens(100000); // Exceeds GPT-4's limit

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_model_config_valid() {
    // Arrange
    let config = ModelConfig::new(Model::OpenAI(clap_noun_verb::wizard::config::OpenAIModel::Gpt4))
        .with_max_tokens(4096); // Within GPT-4's limit

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_wizard_config_default() {
    // Arrange + Act
    let config = WizardConfig::default();

    // Assert
    assert_eq!(config.model_config.temperature, 0.7);
    assert!(config.api_key.is_none());
}

#[test]
fn test_model_id_generation() {
    // Arrange
    let model = Model::Anthropic(clap_noun_verb::wizard::config::AnthropicModel::Claude3Sonnet);

    // Act
    let model_id = model.model_id();

    // Assert
    assert_eq!(model_id, "claude-3-sonnet-20240229");
}

#[test]
fn test_message_serialization() {
    // Arrange
    let message = Message::user("Hello, AI!");

    // Act
    let json = serde_json::to_string(&message);

    // Assert
    assert!(json.is_ok());
    let json = json.unwrap();
    assert!(json.contains("user"));
    assert!(json.contains("Hello, AI!"));
}

// =============================================================================
// Integration Tests (require API keys)
// =============================================================================
// These tests are marked with #[ignore] by default to prevent CI failures
// Run them manually with: cargo test --features wizard -- --ignored

#[tokio::test]
#[ignore]
async fn test_genai_client_creation_with_env() {
    // Arrange - Requires ANTHROPIC_API_KEY in environment
    let config = WizardConfig::from_env();

    // Skip if no API key configured
    if config.is_err() {
        eprintln!("Skipping: No API key configured");
        return;
    }

    // Act
    let client = GenAiClient::new(config.unwrap()).await;

    // Assert
    assert!(client.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_genai_client_simple_generation() {
    // Arrange - Requires ANTHROPIC_API_KEY in environment
    let config = WizardConfig::from_env();

    if config.is_err() {
        eprintln!("Skipping: No API key configured");
        return;
    }

    let mut client = GenAiClient::new(config.unwrap()).await.unwrap();
    let prompt = Prompt::new("Say 'Hello from Rust!' and nothing else.");

    // Act
    let response = client.generate(prompt).await;

    // Assert
    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.text.is_empty());
    assert!(response.text.to_lowercase().contains("hello"));
}

#[tokio::test]
#[ignore]
async fn test_genai_client_with_system_prompt() {
    // Arrange
    let config = WizardConfig::from_env();

    if config.is_err() {
        eprintln!("Skipping: No API key configured");
        return;
    }

    let mut client = GenAiClient::new(config.unwrap()).await.unwrap();
    let prompt =
        Prompt::new("What is 2+2?").with_system("You are a calculator. Respond only with numbers.");

    // Act
    let response = client.generate(prompt).await;

    // Assert
    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.text.is_empty());
    // Response should contain "4"
    assert!(response.text.contains('4'));
}

#[tokio::test]
#[ignore]
async fn test_genai_client_conversation_history() {
    // Arrange
    let config = WizardConfig::from_env();

    if config.is_err() {
        eprintln!("Skipping: No API key configured");
        return;
    }

    let mut client = GenAiClient::new(config.unwrap()).await.unwrap();
    let prompt = Prompt::new("What was my first question?").with_history(vec![
        Message::user("My name is Alice."),
        Message::assistant("Hello Alice! Nice to meet you."),
    ]);

    // Act
    let response = client.generate(prompt).await;

    // Assert
    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.text.is_empty());
    // Response should reference the name "Alice"
    assert!(response.text.to_lowercase().contains("alice"));
}

#[cfg(feature = "caching")]
#[tokio::test]
#[ignore]
async fn test_genai_client_caching() {
    // Arrange
    let mut config = WizardConfig::from_env().unwrap();
    config.enable_cache = true;

    let mut client = GenAiClient::new(config).await.unwrap();
    let prompt = Prompt::new("Say 'cached' and nothing else.");

    // Act
    let response1 = client.generate(prompt.clone()).await.unwrap();
    let response2 = client.generate(prompt).await.unwrap();

    // Assert
    assert_eq!(response1.text, response2.text);
    assert!(!response1.metadata.from_cache); // First request not cached
    assert!(response2.metadata.from_cache); // Second request from cache

    // Verify cache stats
    let stats = client.cache_stats();
    assert!(stats.is_some());
    let (len, _cap) = stats.unwrap();
    assert_eq!(len, 1); // One entry in cache
}
