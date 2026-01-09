//! Basic wizard usage example
//!
//! Demonstrates simple wizard configuration and single prompt execution.
//!
//! Run with:
//! ```bash
//! cargo run --example wizard_basic
//! ```

use clap_noun_verb::wizard::{WizardBuilder, WizardConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wizard Basic Example ===\n");

    // Example 1: Using default configuration
    println!("Example 1: Default Configuration");
    let wizard = WizardBuilder::new().build()?;
    println!("Created wizard with model: {}", wizard.config().model);
    println!();

    // Example 2: Custom configuration with builder
    println!("Example 2: Custom Configuration");
    let wizard = WizardBuilder::new()
        .with_model("gpt-4")
        .with_temperature(0.8)
        .with_max_tokens(1024)
        .build()?;

    let response = wizard.prompt("What is Rust?")?;
    println!("Response: {}", response);
    println!();

    // Example 3: Configuration from WizardConfig
    println!("Example 3: WizardConfig Builder");
    let config = WizardConfig::new()
        .with_model("claude-3-sonnet")
        .with_temperature(0.7)
        .with_system_prompt("You are a helpful programming assistant.");

    let wizard = WizardBuilder::new().with_config(config).build()?;

    let response = wizard.prompt("Explain ownership in Rust")?;
    println!("Response: {}", response);
    println!();

    // Example 4: With context
    println!("Example 4: With Context");
    let wizard = WizardBuilder::new()
        .with_model("gpt-3.5-turbo")
        .with_context("User is learning Rust")
        .build()?;

    println!("Wizard context: {:?}", wizard.context());
    let response = wizard.prompt("What should I learn first?")?;
    println!("Response: {}", response);
    println!();

    println!("=== All examples completed successfully ===");

    Ok(())
}
