#![cfg_attr(not(feature = "wizard"), allow(dead_code))]
//! Interactive wizard session example
//!
//! Demonstrates REPL-style interactive wizard sessions with history management.
//!
//! Run with:
//! ```bash
//! cargo run --example wizard_interactive --features wizard
//! ```
//!
//! Commands in the REPL:
//! - `exit` or `quit` - End the session
//! - `clear` - Clear conversation history
//! - `history` - Show conversation history
//! - Any other input - Send as prompt to the wizard

#[cfg(feature = "wizard")]
use clap_noun_verb::wizard::{InteractiveSession, WizardBuilder};

#[cfg(feature = "wizard")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wizard Interactive Example ===\n");

    // Create wizard with custom configuration
    let wizard = WizardBuilder::new()
        .with_model("gpt-4")
        .with_temperature(0.7)
        .with_system_prompt(
            "You are a helpful Rust programming assistant. Provide clear, concise answers.",
        )
        .build()?;

    // Create interactive session
    let mut session = InteractiveSession::new(wizard).with_prompt_prefix("🧙 > ");

    // Run the interactive REPL
    session.run()?;

    // After session ends, export history
    println!("\n=== Session Summary ===");
    println!("Total interactions: {}", session.history_count());

    if session.history_count() > 0 {
        println!("\nExporting conversation history...");
        let json = session.export_history_json()?;
        println!("\nHistory (JSON):\n{}", json);
    }

    Ok(())
}
