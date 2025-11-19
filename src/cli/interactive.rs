//! Interactive help mode for first-time users
//!
//! This module provides a guided, menu-driven interface
//! for common tasks to help new users get started quickly.

use crate::cli::help::CommandCategory;
use crate::error::Result;
use serde::Serialize;
use std::io::{self, Write};

/// Interactive help menu
pub struct InteractiveHelp {
    /// Menu options
    options: Vec<MenuOption>,
}

/// Menu option
#[derive(Debug, Clone, Serialize)]
pub struct MenuOption {
    /// Option key (e.g., "1", "2")
    pub key: String,
    /// Display text
    pub text: String,
    /// Action to perform
    pub action: MenuAction,
}

/// Menu action
#[derive(Debug, Clone, Serialize)]
pub enum MenuAction {
    /// Show category commands
    ShowCategory(CommandCategory),
    /// Show example
    ShowExample(String),
    /// Run guided setup
    GuidedSetup,
    /// Show quickstart
    Quickstart,
    /// Exit
    Exit,
}

impl InteractiveHelp {
    /// Create new interactive help
    pub fn new() -> Self {
        Self {
            options: vec![
                MenuOption {
                    key: "1".to_string(),
                    text: "Show me how to list available packs".to_string(),
                    action: MenuAction::ShowExample("pack list".to_string()),
                },
                MenuOption {
                    key: "2".to_string(),
                    text: "Show me how to generate code with AI".to_string(),
                    action: MenuAction::ShowExample("ai generate".to_string()),
                },
                MenuOption {
                    key: "3".to_string(),
                    text: "Show me how to search the marketplace".to_string(),
                    action: MenuAction::ShowExample("marketplace search".to_string()),
                },
                MenuOption {
                    key: "4".to_string(),
                    text: "Show all pack management commands".to_string(),
                    action: MenuAction::ShowCategory(CommandCategory::Pack),
                },
                MenuOption {
                    key: "5".to_string(),
                    text: "Show all AI commands".to_string(),
                    action: MenuAction::ShowCategory(CommandCategory::AI),
                },
                MenuOption {
                    key: "6".to_string(),
                    text: "Run guided setup".to_string(),
                    action: MenuAction::GuidedSetup,
                },
                MenuOption {
                    key: "q".to_string(),
                    text: "Quickstart guide".to_string(),
                    action: MenuAction::Quickstart,
                },
                MenuOption {
                    key: "x".to_string(),
                    text: "Exit".to_string(),
                    action: MenuAction::Exit,
                },
            ],
        }
    }

    /// Display the menu
    pub fn display_menu(&self) -> Result<InteractiveOutput> {
        Ok(InteractiveOutput {
            title: "Welcome to ggen Interactive Help".to_string(),
            subtitle: "What would you like to do?".to_string(),
            options: self.options.clone(),
        })
    }

    /// Run interactive session (for actual CLI usage)
    #[allow(dead_code)] // Used in actual CLI, not tests
    pub fn run(&self) -> Result<()> {
        loop {
            self.print_menu();

            let choice = self.read_input()?;

            if let Some(option) = self.options.iter().find(|o| o.key == choice) {
                match &option.action {
                    MenuAction::Exit => {
                        println!("\nThank you for using ggen! Run 'ggen help' anytime for assistance.");
                        break;
                    }
                    MenuAction::ShowExample(example) => {
                        self.show_example(example)?;
                    }
                    MenuAction::ShowCategory(category) => {
                        self.show_category(category)?;
                    }
                    MenuAction::GuidedSetup => {
                        self.run_guided_setup()?;
                    }
                    MenuAction::Quickstart => {
                        self.show_quickstart()?;
                    }
                }
            } else {
                println!("\nInvalid choice. Please try again.");
            }

            println!();
        }

        Ok(())
    }

    /// Print the menu
    fn print_menu(&self) {
        println!("\n=== Welcome to ggen Interactive Help ===\n");
        println!("What would you like to do?\n");

        for option in &self.options {
            println!("  [{}] {}", option.key, option.text);
        }

        print!("\nEnter your choice: ");
        io::stdout().flush().ok();
    }

    /// Read user input
    fn read_input(&self) -> Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!("Failed to read input: {}", e))
        })?;
        Ok(input.trim().to_string())
    }

    /// Show example
    fn show_example(&self, example_name: &str) -> Result<()> {
        use crate::cli::examples::ExamplesRegistry;

        let registry = ExamplesRegistry::default();
        let results = registry.search(example_name);

        if let Some(example) = results.first() {
            println!("\n=== {} ===", example.title);
            println!("\n{}", example.description);
            println!("\n$ {}", example.command);
            println!("\nExpected output:");
            println!("{}", example.expected_output);

            if !example.variations.is_empty() {
                println!("\nCommon variations:");
                for var in &example.variations {
                    println!("  • {}: {}", var.description, var.command);
                }
            }
        } else {
            println!("\nExample not found: {}", example_name);
        }

        Ok(())
    }

    /// Show category commands
    fn show_category(&self, category: &CommandCategory) -> Result<()> {
        use crate::cli::help::HelpSystem;

        let help_system = HelpSystem::default();
        let commands = help_system.commands_by_category(category);

        println!("\n=== {} Commands ===", category);
        println!("\n{}", category.description());
        println!("\nAvailable commands:\n");

        for cmd in commands {
            println!("  • {}: {}", cmd.name, cmd.brief);
        }

        Ok(())
    }

    /// Run guided setup
    fn run_guided_setup(&self) -> Result<()> {
        println!("\n=== Guided Setup ===\n");
        println!("This will help you configure ggen for first use.\n");

        // Step 1: AI Provider
        println!("Step 1: AI Provider");
        println!("Which AI provider would you like to use?");
        println!("  [1] OpenAI");
        println!("  [2] Anthropic");
        println!("  [3] Local (Ollama)");
        println!("  [4] Skip for now");
        print!("\nChoice: ");
        io::stdout().flush().ok();

        let provider_choice = self.read_input()?;
        let provider = match provider_choice.as_str() {
            "1" => "openai",
            "2" => "anthropic",
            "3" => "ollama",
            _ => {
                println!("Skipping AI provider configuration.");
                "none"
            }
        };

        if provider != "none" {
            println!("\nTo configure this provider, run:");
            println!("  $ ggen config set ai.provider {}", provider);
            println!("  $ ggen config set ai.api_key YOUR_API_KEY");
        }

        // Step 2: Template Engine
        println!("\n\nStep 2: Template Engine");
        println!("Which template engine would you like to use?");
        println!("  [1] Handlebars (recommended)");
        println!("  [2] Tera");
        println!("  [3] Askama");
        print!("\nChoice: ");
        io::stdout().flush().ok();

        let template_choice = self.read_input()?;
        let template_engine = match template_choice.as_str() {
            "1" => "handlebars",
            "2" => "tera",
            "3" => "askama",
            _ => "handlebars",
        };

        println!("\nTo set template engine, run:");
        println!("  $ ggen config set template.engine {}", template_engine);

        // Summary
        println!("\n\n=== Setup Summary ===");
        println!("You're all set! Here are the next steps:");
        println!("\n1. List available packs:");
        println!("     $ ggen pack list");
        println!("\n2. Install a pack:");
        println!("     $ ggen pack install rust-web-api");
        println!("\n3. Generate code:");
        println!("     $ ggen ai generate \"Create a user model\"");
        println!("\nRun 'ggen help' anytime for more information.");

        Ok(())
    }

    /// Show quickstart guide
    fn show_quickstart(&self) -> Result<()> {
        println!("\n=== Quickstart Guide ===\n");
        println!("Getting started with ggen in 3 steps:\n");

        println!("1. List available packs:");
        println!("     $ ggen pack list\n");

        println!("2. Install a pack:");
        println!("     $ ggen pack install rust-web-api\n");

        println!("3. Generate code:");
        println!("     $ ggen ai generate \"Create a REST API handler\"\n");

        println!("For more examples, run:");
        println!("     $ ggen examples\n");

        println!("For command reference, run:");
        println!("     $ ggen commands\n");

        println!("Full documentation:");
        println!("     https://docs.rs/clap-noun-verb/latest/clap_noun_verb");

        Ok(())
    }
}

impl Default for InteractiveHelp {
    fn default() -> Self {
        Self::new()
    }
}

/// Interactive output structure
#[derive(Debug, Serialize)]
pub struct InteractiveOutput {
    /// Menu title
    pub title: String,
    /// Subtitle
    pub subtitle: String,
    /// Menu options
    pub options: Vec<MenuOption>,
}

/// Guided setup step
#[derive(Debug, Serialize)]
pub struct SetupStep {
    /// Step number
    pub step: usize,
    /// Step title
    pub title: String,
    /// Step description
    pub description: String,
    /// Available choices
    pub choices: Vec<String>,
}

/// Generate interactive help output
pub fn generate_interactive_output() -> Result<InteractiveOutput> {
    let help = InteractiveHelp::new();
    help.display_menu()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_help_new() {
        let help = InteractiveHelp::new();
        assert!(!help.options.is_empty());
    }

    #[test]
    fn test_menu_has_exit_option() {
        let help = InteractiveHelp::new();
        let has_exit = help.options.iter().any(|o| matches!(o.action, MenuAction::Exit));
        assert!(has_exit);
    }

    #[test]
    fn test_menu_has_categories() {
        let help = InteractiveHelp::new();
        let has_category = help
            .options
            .iter()
            .any(|o| matches!(o.action, MenuAction::ShowCategory(_)));
        assert!(has_category);
    }

    #[test]
    fn test_menu_has_examples() {
        let help = InteractiveHelp::new();
        let has_example =
            help.options.iter().any(|o| matches!(o.action, MenuAction::ShowExample(_)));
        assert!(has_example);
    }

    #[test]
    fn test_menu_has_guided_setup() {
        let help = InteractiveHelp::new();
        let has_setup =
            help.options.iter().any(|o| matches!(o.action, MenuAction::GuidedSetup));
        assert!(has_setup);
    }

    #[test]
    fn test_display_menu() {
        let help = InteractiveHelp::new();
        let output = help.display_menu();

        assert!(output.is_ok());
        let out = output.unwrap();
        assert!(!out.title.is_empty());
        assert!(!out.options.is_empty());
    }

    #[test]
    fn test_generate_interactive_output() {
        let output = generate_interactive_output();

        assert!(output.is_ok());
        let out = output.unwrap();
        assert_eq!(out.title, "Welcome to ggen Interactive Help");
        assert!(!out.options.is_empty());
    }

    #[test]
    fn test_all_options_have_keys() {
        let help = InteractiveHelp::new();
        for option in &help.options {
            assert!(!option.key.is_empty());
            assert!(!option.text.is_empty());
        }
    }

    #[test]
    fn test_menu_action_serialization() {
        let action = MenuAction::ShowExample("test".to_string());
        let json = serde_json::to_string(&action);
        assert!(json.is_ok());
    }
}
