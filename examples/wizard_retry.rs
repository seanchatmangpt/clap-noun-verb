#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Example: Retry logic with exponential backoff
//!
//! This example demonstrates automatic retry with exponential backoff,
//! jitter, and selective retry strategies for handling transient failures.
//!
//! Usage:
//!   cargo run --example wizard_retry --features wizard
//!
//! Environment variables required:
//!   ANTHROPIC_API_KEY=your-api-key

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{
    retry::{RetryClient, RetryConfig},
    types::Prompt,
    WizardConfig,
};

#[cfg(feature = "wizard")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wizard Retry Example ===\n");

    // Load configuration from environment
    let wizard_config = WizardConfig::from_env()?;
    println!("Using model: {:?}\n", wizard_config.model_config.model);

    // Example 1: Default retry configuration
    println!("Example 1: Default retry configuration (3 attempts, exponential backoff)");

    let retry_config = RetryConfig::default();
    println!("Config: {:?}\n", retry_config);

    let mut client = RetryClient::new(wizard_config.clone(), retry_config).await?;

    let prompt = Prompt::new("Explain Rust's borrow checker in one sentence.");
    let response = client.generate(prompt).await?;

    println!("Response: {}", response.text);
    if let Some(usage) = response.usage {
        println!(
            "[Tokens: {} prompt + {} completion = {} total]\n",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    // Example 2: Custom retry configuration
    println!("Example 2: Custom retry configuration (5 attempts, longer delays)");

    let retry_config = RetryConfig::new(5, 2000) // 5 attempts, 2s initial delay
        .with_max_delay(60000) // 60s max delay
        .with_multiplier(1.5) // 1.5x backoff
        .with_jitter(0.3) // 30% jitter
        .with_rate_limit_retry(true);

    println!("Config: {:?}\n", retry_config);

    let mut client = RetryClient::new(wizard_config.clone(), retry_config).await?;

    let prompt = Prompt::new("What are Rust's zero-cost abstractions?");
    let (response, context) = client.generate_with_context(prompt).await?;

    println!("Response: {}", response.text);
    println!("\nRetry context:");
    println!("  Attempts: {}", context.attempts);
    println!("  Total delay: {}ms", context.total_delay_ms);
    if let Some(err) = context.last_error {
        println!("  Last error: {}", err);
    }

    // Example 3: Aggressive retry configuration
    println!("\nExample 3: Aggressive retry (short delays, high jitter)");

    let retry_config = RetryConfig::new(4, 500) // 4 attempts, 500ms initial
        .with_max_delay(5000) // 5s max
        .with_multiplier(2.0) // Double each time
        .with_jitter(0.7); // 70% jitter for randomness

    let mut client = RetryClient::new(wizard_config.clone(), retry_config).await?;

    let prompt = Prompt::new("What is a trait object in Rust?");
    let (response, context) = client.generate_with_context(prompt).await?;

    println!("Response: {}", response.text);
    println!("\nRetry context:");
    println!("  Attempts: {}", context.attempts);
    println!("  Total delay: {}ms", context.total_delay_ms);

    // Example 4: Conservative retry configuration
    println!("\nExample 4: Conservative retry (fewer attempts, longer delays)");

    let retry_config = RetryConfig::new(2, 3000) // Only 2 attempts, 3s initial
        .with_max_delay(30000) // 30s max
        .with_multiplier(3.0) // Triple each time
        .with_jitter(0.1) // Low jitter
        .with_rate_limit_retry(false); // Don't retry rate limits

    let mut client = RetryClient::new(wizard_config, retry_config).await?;

    let prompt = Prompt::new("Explain Rust's async/await in one sentence.");
    let response = client.generate(prompt).await?;

    println!("Response: {}", response.text);

    // Example 5: Show retry delay calculations
    println!("\n\nExample 5: Retry delay calculations");

    let retry_config = RetryConfig::new(5, 1000).with_multiplier(2.0).with_jitter(0.0); // No jitter for predictable output

    println!("Config: initial=1000ms, multiplier=2.0, no jitter");
    println!("Expected delays:");
    for attempt in 0..5 {
        let delay_ms = 1000.0 * 2.0_f64.powi(attempt as i32);
        println!("  Attempt {}: {}ms", attempt, delay_ms);
    }

    Ok(())
}
