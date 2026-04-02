#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Example: Streaming responses with wizard
//!
//! This example demonstrates token-by-token streaming responses
//! using the wizard streaming client.
//!
//! Usage:
//!   cargo run --example wizard_streaming --features wizard
//!
//! Environment variables required:
//!   ANTHROPIC_API_KEY=your-api-key

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{
    streaming::{StreamingClient, StreamingConfig},
    types::Prompt,
    WizardConfig,
};
#[cfg(feature = "wizard")]
use futures::StreamExt;

#[cfg(feature = "wizard")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wizard Streaming Example ===\n");

    // Load configuration from environment
    let wizard_config = WizardConfig::from_env()?;
    println!("Using model: {:?}\n", wizard_config.model_config.model);

    // Create streaming client with custom buffer size
    let streaming_config = StreamingConfig::new(64).with_backpressure(true);

    let client = StreamingClient::new(wizard_config).await?.with_streaming_config(streaming_config);

    // Example 1: Stream a simple response
    println!("Example 1: Simple streaming");
    println!("Prompt: Tell me a short story about Rust\n");

    let prompt = Prompt::new("Tell me a short story about Rust in exactly 3 sentences.");

    let mut stream = client.generate_stream(prompt).await?;

    print!("Response: ");
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                print!("{}", chunk.text);
                std::io::Write::flush(&mut std::io::stdout())?;

                if chunk.is_final {
                    if let Some(usage) = chunk.usage {
                        println!(
                            "\n\n[Tokens: {} prompt + {} completion = {} total]",
                            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }

    println!("\n");

    // Example 2: Stream with conversation history
    println!("Example 2: Streaming with conversation history");

    let prompt = Prompt::new("What is a trait?")
        .with_system("You are a helpful Rust programming assistant.")
        .with_message(
            clap_noun_verb::wizard::types::Role::User,
            "What is a struct in Rust?",
        )
        .with_message(
            clap_noun_verb::wizard::types::Role::Assistant,
            "A struct is a custom data type that lets you name and package together multiple related values.",
        );

    println!("Prompt: What is a trait? (with conversation history)\n");

    let mut stream = client.generate_stream(prompt).await?;

    print!("Response: ");
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                print!("{}", chunk.text);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }

    println!("\n");

    // Example 3: Collect complete response
    println!("Example 3: Collect complete response from stream");

    let prompt = Prompt::new("What are the benefits of Rust's ownership system?");
    let response = client.generate_complete(prompt).await?;

    println!("Response: {}\n", response.text);
    if let Some(usage) = response.usage {
        println!(
            "[Tokens: {} prompt + {} completion = {} total]",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}

#[cfg(not(feature = "wizard"))]
fn main() {
    println!("This example requires the 'wizard' feature to be enabled.");
    println!("Run with: cargo run --example wizard_streaming --features wizard");
}
