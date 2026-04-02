#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Example: Fallback chain for model switching
//!
//! This example demonstrates automatic fallback to alternative models
//! when the primary model fails or is unavailable.
//!
//! Usage:
//!   cargo run --example wizard_fallback --features wizard
//!
//! Environment variables required:
//!   ANTHROPIC_API_KEY=your-api-key (or other provider keys)

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{
    config::{AnthropicModel, GeminiModel, Model, ModelConfig, OpenAIModel},
    fallback::{FallbackClient, FallbackConfig},
    types::Prompt,
    WizardConfig,
};

#[cfg(feature = "wizard")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wizard Fallback Example ===\n");

    // Load base configuration from environment
    let wizard_config = WizardConfig::from_env()?;
    println!("Base model: {:?}\n", wizard_config.model_config.model);

    // Example 1: Simple fallback chain (Anthropic models)
    println!("Example 1: Simple fallback chain (Claude models)");

    let fallback_config = FallbackConfig::new(vec![
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Opus)),
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet)),
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Haiku)),
    ]);

    println!("Fallback chain:");
    for (i, model_config) in fallback_config.model_chain.iter().enumerate() {
        println!("  {}: {:?}", i + 1, model_config.model);
    }
    println!();

    let mut client = FallbackClient::new(wizard_config.clone(), fallback_config)?;

    let prompt = Prompt::new("What is Rust's ownership system?");
    let response = client.generate(prompt).await?;

    println!("Response: {}", response.text);
    println!("Model used: {}", response.model);
    println!("Stats: {:?}\n", client.stats());

    // Example 2: Multi-provider fallback
    println!("Example 2: Multi-provider fallback chain");

    let fallback_config = FallbackConfig::new(vec![
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet)),
        ModelConfig::new(Model::OpenAI(OpenAIModel::Gpt4)),
        ModelConfig::new(Model::Gemini(GeminiModel::Gemini15Flash)),
    ])
    .with_aggregate_errors(true);

    println!("Fallback chain:");
    for (i, model_config) in fallback_config.model_chain.iter().enumerate() {
        println!("  {}: {:?}", i + 1, model_config.model);
    }
    println!();

    let mut client = FallbackClient::new(wizard_config.clone(), fallback_config)?;

    let prompt = Prompt::new("Explain Rust traits in one sentence.");
    match client.generate(prompt).await {
        Ok(response) => {
            println!("Response: {}", response.text);
            println!("Model used: {}", response.model);
        }
        Err(e) => {
            println!("All models failed: {}", e);
        }
    }
    println!("Stats: {:?}\n", client.stats());

    // Example 3: Fallback with attempt details
    println!("Example 3: Fallback with detailed attempt information");

    let fallback_config = FallbackConfig::new(vec![
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Haiku)),
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet)),
    ]);

    let mut client = FallbackClient::new(wizard_config.clone(), fallback_config)?;

    let prompt = Prompt::new("What are Rust's zero-cost abstractions?");
    let (response, attempts) = client.generate_with_attempts(prompt).await?;

    println!("Response: {}", response.text);
    println!("\nAttempt details:");
    for attempt in attempts {
        println!("  Attempt {}: {:?}", attempt.attempt_index, attempt.model);
        match &attempt.result {
            Ok(resp) => println!("    Success: {} chars", resp.text.len()),
            Err(err) => println!("    Failed: {}", err),
        }
    }
    println!("\nStats: {:?}", client.stats());

    // Example 4: Performance vs cost tradeoff
    println!("\nExample 4: Performance vs cost tradeoff");

    let fallback_config = FallbackConfig::new(vec![
        // Try fast/cheap model first
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Haiku)),
        // Fall back to balanced model
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Sonnet)),
        // Last resort: most capable model
        ModelConfig::new(Model::Anthropic(AnthropicModel::Claude3Opus)),
    ]);

    let mut client = FallbackClient::new(wizard_config, fallback_config)?;

    let prompts = vec![
        Prompt::new("What is a struct?"),
        Prompt::new("What is a trait?"),
        Prompt::new("What is a lifetime?"),
    ];

    for (i, prompt) in prompts.iter().enumerate() {
        match client.generate(prompt.clone()).await {
            Ok(response) => {
                println!("Request {}: Success with {}", i + 1, response.model);
            }
            Err(e) => {
                println!("Request {}: Failed - {}", i + 1, e);
            }
        }
    }

    println!("\nFinal stats: {:?}", client.stats());
    println!("Fallback rate: {:.2}%", client.stats().fallback_rate() * 100.0);
    println!("Success rate: {:.2}%", client.stats().success_rate() * 100.0);

    Ok(())
}

#[cfg(not(feature = "wizard"))]
fn main() {
    println!("This example requires the 'wizard' feature to be enabled.");
    println!("Run with: cargo run --example wizard_fallback --features wizard");
}
