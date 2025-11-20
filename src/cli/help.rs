//! Enhanced help system for improved time-to-first-success
//!
//! This module provides an integrated help system with:
//! - Brief command category descriptions
//! - Popular commands (top 5)
//! - Detailed help with examples
//! - Quick links to quickstart guide

use crate::error::{NounVerbError, Result};
use serde::Serialize;
use std::fmt;

/// Command category for organizing help output
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CommandCategory {
    /// Pack management commands
    Pack,
    /// AI/generation commands
    AI,
    /// Marketplace commands
    Marketplace,
    /// Template commands
    Template,
    /// Configuration commands
    Config,
    /// System commands
    System,
}

impl fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pack => write!(f, "Pack"),
            Self::AI => write!(f, "AI"),
            Self::Marketplace => write!(f, "Marketplace"),
            Self::Template => write!(f, "Template"),
            Self::Config => write!(f, "Config"),
            Self::System => write!(f, "System"),
        }
    }
}

impl CommandCategory {
    /// Get brief description of category
    pub fn description(&self) -> &'static str {
        match self {
            Self::Pack => "Manage and organize code generation packs",
            Self::AI => "AI-powered code generation and analysis",
            Self::Marketplace => "Browse and install community packs",
            Self::Template => "Create and manage code templates",
            Self::Config => "Configure ggen settings and preferences",
            Self::System => "System utilities and diagnostics",
        }
    }

    /// Get all categories
    pub fn all() -> Vec<Self> {
        vec![Self::Pack, Self::AI, Self::Marketplace, Self::Template, Self::Config, Self::System]
    }
}

/// Command metadata for help system
#[derive(Debug, Clone, Serialize)]
pub struct CommandInfo {
    /// Command name (e.g., "pack list")
    pub name: String,
    /// Category this command belongs to
    pub category: CommandCategory,
    /// Brief one-line description
    pub brief: String,
    /// Detailed description
    pub description: String,
    /// Usage examples
    pub examples: Vec<String>,
    /// Popularity score (0-100)
    pub popularity: u8,
}

impl CommandInfo {
    /// Create new command info
    pub fn new(
        name: impl Into<String>,
        category: CommandCategory,
        brief: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            brief: brief.into(),
            description: String::new(),
            examples: Vec::new(),
            popularity: 0,
        }
    }

    /// Set detailed description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add an example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Set popularity score (0-100)
    ///
    /// # Panics
    ///
    /// Panics if score > 100
    pub fn with_popularity(mut self, score: u8) -> Self {
        assert!(score <= 100, "popularity score must be 0-100, got {}", score);
        self.popularity = score;
        self
    }
}

/// Enhanced help system
pub struct HelpSystem {
    /// All registered commands
    commands: Vec<CommandInfo>,
}

impl HelpSystem {
    /// Create new help system
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    /// Register a command
    pub fn register_command(&mut self, info: CommandInfo) {
        self.commands.push(info);
    }

    /// Get top N most popular commands
    pub fn popular_commands(&self, n: usize) -> Vec<&CommandInfo> {
        let mut sorted = self.commands.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        sorted.into_iter().take(n).collect()
    }

    /// Get commands by category
    pub fn commands_by_category(&self, category: &CommandCategory) -> Vec<&CommandInfo> {
        self.commands.iter().filter(|c| &c.category == category).collect()
    }

    /// Find command by name
    pub fn find_command(&self, name: &str) -> Option<&CommandInfo> {
        self.commands.iter().find(|c| c.name == name)
    }

    /// Generate main help output
    pub fn generate_main_help(&self) -> Result<HelpOutput> {
        let categories = CommandCategory::all()
            .into_iter()
            .map(|cat| CategoryHelp {
                name: cat.to_string(),
                description: cat.description().to_string(),
                command_count: self.commands_by_category(&cat).len(),
            })
            .collect();

        let popular = self
            .popular_commands(5)
            .into_iter()
            .map(|cmd| PopularCommand { name: cmd.name.clone(), brief: cmd.brief.clone() })
            .collect();

        Ok(HelpOutput {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            categories,
            popular,
            quickstart_url: "https://docs.rs/clap-noun-verb/latest/clap_noun_verb".to_string(),
        })
    }

    /// Generate detailed help for a command
    pub fn generate_command_help(&self, name: &str) -> Result<CommandHelp> {
        let cmd = self.find_command(name).ok_or_else(|| NounVerbError::command_not_found(name))?;

        Ok(CommandHelp {
            name: cmd.name.clone(),
            category: cmd.category.to_string(),
            brief: cmd.brief.clone(),
            description: cmd.description.clone(),
            examples: cmd.examples.clone(),
        })
    }

    /// Get all commands with brief descriptions
    pub fn all_commands(&self) -> Vec<CommandListItem> {
        self.commands
            .iter()
            .map(|cmd| CommandListItem {
                name: cmd.name.clone(),
                category: cmd.category.to_string(),
                brief: cmd.brief.clone(),
            })
            .collect()
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        let mut system = Self::new();

        // Register popular commands with examples
        system.register_command(
            CommandInfo::new("pack list", CommandCategory::Pack, "List available packs")
                .with_description(
                    "Display all code generation packs available in the current workspace",
                )
                .with_example("ggen pack list")
                .with_example("ggen pack list --category templates")
                .with_popularity(95),
        );

        system.register_command(
            CommandInfo::new("ai generate", CommandCategory::AI, "Generate code with AI")
                .with_description("Use AI to generate code based on natural language descriptions")
                .with_example("ggen ai generate \"Create a REST API handler\"")
                .with_example("ggen ai generate --template rust-api \"User authentication\"")
                .with_popularity(90),
        );

        system.register_command(
            CommandInfo::new(
                "marketplace search",
                CommandCategory::Marketplace,
                "Search marketplace",
            )
            .with_description("Search the community marketplace for packs and templates")
            .with_example("ggen marketplace search \"web framework\"")
            .with_example("ggen marketplace search --category backend")
            .with_popularity(85),
        );

        system.register_command(
            CommandInfo::new("template render", CommandCategory::Template, "Render template")
                .with_description("Render a code template with provided variables")
                .with_example("ggen template render my-template.hbs --vars config.json")
                .with_example("ggen template render api-handler --name UserService")
                .with_popularity(80),
        );

        system.register_command(
            CommandInfo::new("pack install", CommandCategory::Pack, "Install a pack")
                .with_description("Install a code generation pack from marketplace or local path")
                .with_example("ggen pack install rust-web-api")
                .with_example("ggen pack install ./my-pack.tar.gz")
                .with_popularity(75),
        );

        system.register_command(
            CommandInfo::new("config set", CommandCategory::Config, "Set configuration value")
                .with_description("Configure ggen settings and preferences")
                .with_example("ggen config set ai.provider openai")
                .with_example("ggen config set template.default handlebars")
                .with_popularity(70),
        );

        system.register_command(
            CommandInfo::new("template create", CommandCategory::Template, "Create template")
                .with_description("Create a new code generation template")
                .with_example("ggen template create my-template --type handlebars")
                .with_example("ggen template create api-endpoint --from examples/rest.hbs")
                .with_popularity(65),
        );

        system.register_command(
            CommandInfo::new("pack create", CommandCategory::Pack, "Create new pack")
                .with_description("Create a new code generation pack")
                .with_example("ggen pack create my-pack")
                .with_example("ggen pack create web-api --template typescript")
                .with_popularity(60),
        );

        system.register_command(
            CommandInfo::new(
                "marketplace install",
                CommandCategory::Marketplace,
                "Install from marketplace",
            )
            .with_description("Install a pack directly from the marketplace")
            .with_example("ggen marketplace install typescript-backend")
            .with_example("ggen marketplace install react-components --version 2.0")
            .with_popularity(70),
        );

        system.register_command(
            CommandInfo::new("ai analyze", CommandCategory::AI, "Analyze code with AI")
                .with_description("Use AI to analyze existing code and suggest improvements")
                .with_example("ggen ai analyze src/main.rs")
                .with_example("ggen ai analyze . --focus performance")
                .with_popularity(55),
        );

        system
    }
}

/// Main help output structure
#[derive(Debug, Serialize)]
pub struct HelpOutput {
    /// Version string
    pub version: String,
    /// Application description
    pub description: String,
    /// Command categories
    pub categories: Vec<CategoryHelp>,
    /// Popular commands
    pub popular: Vec<PopularCommand>,
    /// Quickstart guide URL
    pub quickstart_url: String,
}

/// Category help information
#[derive(Debug, Serialize)]
pub struct CategoryHelp {
    /// Category name
    pub name: String,
    /// Category description
    pub description: String,
    /// Number of commands in category
    pub command_count: usize,
}

/// Popular command information
#[derive(Debug, Serialize)]
pub struct PopularCommand {
    /// Command name
    pub name: String,
    /// Brief description
    pub brief: String,
}

/// Detailed command help
#[derive(Debug, Serialize)]
pub struct CommandHelp {
    /// Command name
    pub name: String,
    /// Category
    pub category: String,
    /// Brief description
    pub brief: String,
    /// Detailed description
    pub description: String,
    /// Examples
    pub examples: Vec<String>,
}

/// Command list item
#[derive(Debug, Serialize)]
pub struct CommandListItem {
    /// Command name
    pub name: String,
    /// Category
    pub category: String,
    /// Brief description
    pub brief: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_category_display() {
        assert_eq!(CommandCategory::Pack.to_string(), "Pack");
        assert_eq!(CommandCategory::AI.to_string(), "AI");
    }

    #[test]
    fn test_command_category_description() {
        assert!(!CommandCategory::Pack.description().is_empty());
        assert!(!CommandCategory::AI.description().is_empty());
    }

    #[test]
    fn test_command_info_builder() {
        let info = CommandInfo::new("test cmd", CommandCategory::System, "Test command")
            .with_description("Detailed description")
            .with_example("example 1")
            .with_example("example 2")
            .with_popularity(50);

        assert_eq!(info.name, "test cmd");
        assert_eq!(info.brief, "Test command");
        assert_eq!(info.description, "Detailed description");
        assert_eq!(info.examples.len(), 2);
        assert_eq!(info.popularity, 50);
    }

    #[test]
    fn test_help_system_default() {
        let system = HelpSystem::default();
        assert!(!system.commands.is_empty());
    }

    #[test]
    fn test_popular_commands() {
        let system = HelpSystem::default();
        let popular = system.popular_commands(5);

        assert_eq!(popular.len(), 5);
        // Should be sorted by popularity
        for i in 0..popular.len() - 1 {
            assert!(popular[i].popularity >= popular[i + 1].popularity);
        }
    }

    #[test]
    fn test_commands_by_category() {
        let system = HelpSystem::default();
        let pack_commands = system.commands_by_category(&CommandCategory::Pack);

        assert!(!pack_commands.is_empty());
        for cmd in pack_commands {
            assert_eq!(cmd.category, CommandCategory::Pack);
        }
    }

    #[test]
    fn test_find_command() {
        let system = HelpSystem::default();
        let cmd = system.find_command("pack list");

        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "pack list");
    }

    #[test]
    fn test_generate_main_help() {
        let system = HelpSystem::default();
        let help = system.generate_main_help();

        assert!(help.is_ok());
        let output = help.unwrap();
        assert!(!output.version.is_empty());
        assert!(!output.categories.is_empty());
        assert_eq!(output.popular.len(), 5);
    }

    #[test]
    fn test_generate_command_help() {
        let system = HelpSystem::default();
        let help = system.generate_command_help("pack list");

        assert!(help.is_ok());
        let output = help.unwrap();
        assert_eq!(output.name, "pack list");
        assert!(!output.examples.is_empty());
    }

    #[test]
    fn test_all_commands() {
        let system = HelpSystem::default();
        let commands = system.all_commands();

        assert!(!commands.is_empty());
        for cmd in commands {
            assert!(!cmd.name.is_empty());
            assert!(!cmd.brief.is_empty());
        }
    }
}
