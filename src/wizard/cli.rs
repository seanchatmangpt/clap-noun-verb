//! CLI integration for wizard commands
//!
//! This module provides clap integration for wizard commands using
//! the noun-verb pattern.

use super::{InteractiveSession, Result, WizardBuilder, WizardConfig, WizardError};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

/// Wizard CLI commands
#[derive(Debug, Subcommand)]
pub enum WizardCommand {
    /// Start an interactive wizard session
    Interactive(InteractiveArgs),

    /// Run a single wizard prompt
    Run(RunArgs),

    /// Show wizard version and configuration
    Version,

    /// Show available models
    Models,
}

/// Arguments for interactive command
#[derive(Debug, Args)]
pub struct InteractiveArgs {
    /// Model to use
    #[arg(long, default_value = "gpt-4")]
    pub model: String,

    /// Temperature (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,

    /// System prompt
    #[arg(long)]
    pub system_prompt: Option<String>,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,
}

/// Arguments for run command
#[derive(Debug, Args)]
pub struct RunArgs {
    /// The prompt to execute
    pub prompt: String,

    /// Model to use
    #[arg(long, default_value = "gpt-4")]
    pub model: String,

    /// Temperature (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,

    /// Maximum tokens to generate
    #[arg(long, default_value = "2048")]
    pub max_tokens: u32,

    /// Output format (json, text)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,
}

/// Output format for wizard responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Invalid format: {}. Use 'text' or 'json'", s)),
        }
    }
}

/// Wizard CLI interface
pub struct WizardCli;

impl WizardCli {
    /// Execute a wizard command
    pub fn execute(command: WizardCommand) -> Result<()> {
        match command {
            WizardCommand::Interactive(args) => Self::run_interactive(args),
            WizardCommand::Run(args) => Self::run_prompt(args),
            WizardCommand::Version => Self::show_version(),
            WizardCommand::Models => Self::show_models(),
        }
    }

    /// Run interactive session
    fn run_interactive(args: InteractiveArgs) -> Result<()> {
        let mut config = WizardConfig::new()
            .with_model(args.model)
            .with_temperature(args.temperature)
            .with_verbose(args.verbose);

        if let Some(prompt) = args.system_prompt {
            config = config.with_system_prompt(prompt);
        }

        let wizard = WizardBuilder::new().with_config(config).build()?;

        let mut session = InteractiveSession::new(wizard);
        session.run()
    }

    /// Run a single prompt
    fn run_prompt(args: RunArgs) -> Result<()> {
        let config = WizardConfig::new()
            .with_model(args.model)
            .with_temperature(args.temperature)
            .with_max_tokens(args.max_tokens)
            .with_verbose(args.verbose);

        let wizard = WizardBuilder::new().with_config(config).build()?;

        let response = wizard.prompt(args.prompt)?;

        match args.format {
            OutputFormat::Text => {
                println!("{}", response);
            }
            OutputFormat::Json => {
                let json =
                    serde_json::to_string_pretty(&response).map_err(|e| WizardError::Json(e))?;
                println!("{}", json);
            }
        }

        Ok(())
    }

    /// Show version information
    fn show_version() -> Result<()> {
        println!("Wizard CLI v{}", env!("CARGO_PKG_VERSION"));
        println!("Part of clap-noun-verb framework");
        Ok(())
    }

    /// Show available models
    fn show_models() -> Result<()> {
        println!("Available Models:");
        println!("  - gpt-4 (OpenAI GPT-4)");
        println!("  - gpt-3.5-turbo (OpenAI GPT-3.5)");
        println!("  - claude-3-opus (Anthropic Claude 3 Opus)");
        println!("  - claude-3-sonnet (Anthropic Claude 3 Sonnet)");
        println!();
        println!("Note: API keys must be configured via environment variables");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_parsing() {
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert!("invalid".parse::<OutputFormat>().is_err());
    }
}
