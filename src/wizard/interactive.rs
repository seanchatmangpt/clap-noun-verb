//! Interactive session handler for wizard CLI
//!
//! This module provides a REPL-style interface for interactive
//! AI conversations with session history management.

use super::{Result, Wizard, WizardError, WizardResponse};
use std::io::{self, BufRead, Write};

/// Interactive session for wizard interactions
pub struct InteractiveSession {
    wizard: Wizard,
    history: Vec<(String, WizardResponse)>,
    prompt_prefix: String,
}

impl InteractiveSession {
    /// Create a new interactive session
    pub fn new(wizard: Wizard) -> Self {
        Self { wizard, history: Vec::new(), prompt_prefix: "> ".to_string() }
    }

    /// Set custom prompt prefix
    pub fn with_prompt_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prompt_prefix = prefix.into();
        self
    }

    /// Get session history
    pub fn history(&self) -> &[(String, WizardResponse)] {
        &self.history
    }

    /// Clear session history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Run the interactive session
    ///
    /// This starts a REPL loop that reads user input, sends it to the wizard,
    /// and displays responses. Type 'exit' or 'quit' to end the session.
    pub fn run(&mut self) -> Result<()> {
        println!("Wizard Interactive Session");
        println!("Model: {}", self.wizard.config().model);
        println!("Type 'exit' or 'quit' to end session");
        println!("Type 'clear' to clear history");
        println!("Type 'history' to show conversation history");
        println!();

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // Display prompt
            print!("{}", self.prompt_prefix);
            stdout.flush().map_err(|e| WizardError::Io(e))?;

            // Read user input
            let mut input = String::new();
            let mut handle = stdin.lock();
            handle.read_line(&mut input).map_err(|e| WizardError::Io(e))?;

            let input = input.trim();

            // Handle special commands
            match input.to_lowercase().as_str() {
                "" => continue,
                "exit" | "quit" => {
                    println!("Goodbye!");
                    break;
                }
                "clear" => {
                    self.clear_history();
                    println!("History cleared");
                    continue;
                }
                "history" => {
                    self.display_history();
                    continue;
                }
                _ => {}
            }

            // Send to wizard and display response
            match self.prompt(input.to_string()) {
                Ok(response) => {
                    println!("{}", response);
                    println!();
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    println!();
                }
            }
        }

        Ok(())
    }

    /// Send a prompt to the wizard and store in history
    pub fn prompt(&mut self, input: String) -> Result<WizardResponse> {
        let response = self.wizard.prompt(&input)?;
        self.history.push((input, response.clone()));
        Ok(response)
    }

    /// Display conversation history
    fn display_history(&self) {
        println!("\n=== Conversation History ===");
        if self.history.is_empty() {
            println!("(empty)");
        } else {
            for (i, (input, response)) in self.history.iter().enumerate() {
                println!("\n[{}] User: {}", i + 1, input);
                println!("    Wizard: {}", response.content);
            }
        }
        println!("===========================\n");
    }

    /// Export history as JSON
    pub fn export_history_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.history).map_err(|e| WizardError::Json(e))
    }

    /// Get history count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::WizardBuilder;

    #[test]
    fn test_session_creation() {
        let wizard =
            WizardBuilder::new().build().expect("Failed to build wizard for session creation test");
        let session = InteractiveSession::new(wizard);

        assert_eq!(session.history().len(), 0);
        assert_eq!(session.prompt_prefix, "> ");
    }

    #[test]
    fn test_session_prompt() {
        let wizard = WizardBuilder::new().build().expect("Failed to build wizard for prompt test");
        let mut session = InteractiveSession::new(wizard);

        let result = session.prompt("Test input".to_string());
        assert!(result.is_ok());
        assert_eq!(session.history_count(), 1);
    }

    #[test]
    fn test_clear_history() {
        let wizard =
            WizardBuilder::new().build().expect("Failed to build wizard for clear history test");
        let mut session = InteractiveSession::new(wizard);

        let _response = session.prompt("Test".to_string());
        assert_eq!(session.history_count(), 1);

        session.clear_history();
        assert_eq!(session.history_count(), 0);
    }

    #[test]
    fn test_export_history() {
        let wizard =
            WizardBuilder::new().build().expect("Failed to build wizard for export history test");
        let mut session = InteractiveSession::new(wizard);

        let _response = session.prompt("Test".to_string());
        let json = session.export_history_json().expect("Failed to export history as JSON");

        assert!(json.contains("Test"));
    }
}
