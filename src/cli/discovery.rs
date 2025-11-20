//! Command discovery system for easy command finding
//!
//! This module provides:
//! - Listing all commands with descriptions
//! - Fuzzy search by keyword
//! - Category-based browsing
//! - Command suggestions for typos

use crate::cli::help::{CommandCategory, CommandInfo};
use crate::error::Result;
use serde::Serialize;

/// Command discovery system
pub struct CommandDiscovery {
    /// All registered commands
    commands: Vec<CommandInfo>,
}

impl CommandDiscovery {
    /// Create new discovery system
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    /// Create from command list
    pub fn from_commands(commands: Vec<CommandInfo>) -> Self {
        Self { commands }
    }

    /// Register a command
    pub fn register(&mut self, info: CommandInfo) {
        self.commands.push(info);
    }

    /// List all commands
    pub fn list_all(&self) -> Vec<CommandListItem> {
        self.commands
            .iter()
            .map(|cmd| CommandListItem {
                name: cmd.name.clone(),
                category: cmd.category.to_string(),
                description: cmd.brief.clone(),
            })
            .collect()
    }

    /// List commands by category
    pub fn list_by_category(&self, category: &CommandCategory) -> Vec<CommandListItem> {
        self.commands
            .iter()
            .filter(|cmd| &cmd.category == category)
            .map(|cmd| CommandListItem {
                name: cmd.name.clone(),
                category: cmd.category.to_string(),
                description: cmd.brief.clone(),
            })
            .collect()
    }

    /// Search commands by keyword (fuzzy matching)
    pub fn search(&self, keyword: &str) -> Vec<SearchResult> {
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        for cmd in &self.commands {
            if let Some(score) = self.calculate_match_score(cmd, &keyword_lower) {
                results.push(SearchResult {
                    name: cmd.name.clone(),
                    category: cmd.category.to_string(),
                    description: cmd.brief.clone(),
                    score,
                    match_type: self.determine_match_type(cmd, &keyword_lower),
                });
            }
        }

        // Sort by score (highest first), treating NaN/Inf as lowest priority
        results.sort_by(|a, b| {
            match b.score.partial_cmp(&a.score) {
                Some(ordering) => ordering,
                None => {
                    // Handle NaN/Inf cases - neither is a valid score
                    if a.score.is_nan() && b.score.is_nan() {
                        std::cmp::Ordering::Equal
                    } else if b.score.is_nan() {
                        std::cmp::Ordering::Less // a is "greater" if b is NaN
                    } else {
                        std::cmp::Ordering::Greater // b is "greater" if a is NaN
                    }
                }
            }
        });

        results
    }

    /// Calculate match score for a command
    fn calculate_match_score(&self, cmd: &CommandInfo, keyword: &str) -> Option<f32> {
        let name_lower = cmd.name.to_lowercase();
        let brief_lower = cmd.brief.to_lowercase();

        // Exact match in name
        if name_lower == keyword {
            return Some(100.0);
        }

        // Starts with keyword
        if name_lower.starts_with(keyword) {
            return Some(90.0);
        }

        // Contains exact keyword in name
        if name_lower.contains(keyword) {
            return Some(80.0);
        }

        // Contains keyword in description
        if brief_lower.contains(keyword) {
            return Some(60.0);
        }

        // Category match
        if cmd.category.to_string().to_lowercase().contains(keyword) {
            return Some(50.0);
        }

        // Fuzzy match in name
        let fuzzy_score = self.fuzzy_match(&name_lower, keyword);
        if fuzzy_score > 0.5 {
            return Some(40.0 * fuzzy_score);
        }

        None
    }

    /// Simple fuzzy matching algorithm
    fn fuzzy_match(&self, text: &str, pattern: &str) -> f32 {
        // Empty pattern matches everything (no constraints = perfect match)
        if pattern.is_empty() {
            return 1.0;
        }

        // Empty text cannot match non-empty pattern
        if text.is_empty() {
            return 0.0;
        }

        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();

        let mut pattern_idx = 0;
        let mut matches = 0;

        for text_char in text_chars {
            if pattern_idx < pattern_chars.len() && text_char == pattern_chars[pattern_idx] {
                pattern_idx += 1;
                matches += 1;
            }
        }

        matches as f32 / pattern_chars.len() as f32
    }

    /// Determine match type
    fn determine_match_type(&self, cmd: &CommandInfo, keyword: &str) -> MatchType {
        let name_lower = cmd.name.to_lowercase();
        let brief_lower = cmd.brief.to_lowercase();

        if name_lower == keyword {
            MatchType::ExactName
        } else if name_lower.starts_with(keyword) {
            MatchType::PrefixName
        } else if name_lower.contains(keyword) {
            MatchType::ContainsName
        } else if brief_lower.contains(keyword) {
            MatchType::Description
        } else if cmd.category.to_string().to_lowercase().contains(keyword) {
            MatchType::Category
        } else {
            MatchType::Fuzzy
        }
    }

    /// Suggest commands for unknown input
    pub fn suggest(&self, unknown_command: &str) -> Vec<Suggestion> {
        let search_results = self.search(unknown_command);

        search_results
            .into_iter()
            .take(5) // Top 5 suggestions
            .map(|result| Suggestion {
                command: result.name,
                reason: format!("Similar to your input ({})", result.match_type.description()),
                score: result.score,
            })
            .collect()
    }

    /// Get all categories with command counts
    pub fn categories_summary(&self) -> Vec<CategorySummary> {
        CommandCategory::all()
            .into_iter()
            .map(|cat| {
                let count = self.commands.iter().filter(|cmd| cmd.category == cat).count();
                CategorySummary {
                    name: cat.to_string(),
                    description: cat.description().to_string(),
                    command_count: count,
                }
            })
            .collect()
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Command list item
#[derive(Debug, Clone, Serialize)]
pub struct CommandListItem {
    /// Command name
    pub name: String,
    /// Category
    pub category: String,
    /// Brief description
    pub description: String,
}

/// Search result
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Command name
    pub name: String,
    /// Category
    pub category: String,
    /// Description
    pub description: String,
    /// Match score (0-100)
    pub score: f32,
    /// Type of match
    pub match_type: MatchType,
}

/// Type of match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MatchType {
    /// Exact match in name
    ExactName,
    /// Prefix match in name
    PrefixName,
    /// Contains in name
    ContainsName,
    /// Match in description
    Description,
    /// Match in category
    Category,
    /// Fuzzy match
    Fuzzy,
}

impl MatchType {
    /// Get description of match type
    pub fn description(&self) -> &'static str {
        match self {
            Self::ExactName => "exact match",
            Self::PrefixName => "starts with",
            Self::ContainsName => "contains",
            Self::Description => "description match",
            Self::Category => "category match",
            Self::Fuzzy => "fuzzy match",
        }
    }
}

/// Command suggestion
#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    /// Suggested command
    pub command: String,
    /// Reason for suggestion
    pub reason: String,
    /// Confidence score
    pub score: f32,
}

/// Category summary
#[derive(Debug, Clone, Serialize)]
pub struct CategorySummary {
    /// Category name
    pub name: String,
    /// Category description
    pub description: String,
    /// Number of commands
    pub command_count: usize,
}

/// Generate discovery output for "ggen commands"
pub fn generate_commands_output(discovery: &CommandDiscovery) -> Result<CommandsOutput> {
    Ok(CommandsOutput {
        commands: discovery.list_all(),
        total: discovery.commands.len(),
        categories: discovery.categories_summary(),
    })
}

/// Generate search output for "ggen find <keyword>"
pub fn generate_search_output(discovery: &CommandDiscovery, keyword: &str) -> Result<SearchOutput> {
    let results = discovery.search(keyword);

    if results.is_empty() {
        // Get suggestions instead of failing hard
        let suggestions = discovery.suggest(keyword);

        if !suggestions.is_empty() {
            eprintln!("No exact match for '{}'. Did you mean:", keyword);
            for (i, suggestion) in suggestions.iter().take(3).enumerate() {
                eprintln!("  {}. {} ({})", i + 1, suggestion.command, suggestion.reason);
            }
        } else {
            eprintln!(
                "No commands found matching '{}'. Run 'ggen help' for all commands.",
                keyword
            );
        }

        // Return empty result instead of error, allowing further processing
        return Ok(SearchOutput { keyword: keyword.to_string(), results: vec![], total: 0 });
    }

    let total = results.len();
    Ok(SearchOutput { keyword: keyword.to_string(), results, total })
}

/// Commands output
#[derive(Debug, Serialize)]
pub struct CommandsOutput {
    /// All commands
    pub commands: Vec<CommandListItem>,
    /// Total count
    pub total: usize,
    /// Categories summary
    pub categories: Vec<CategorySummary>,
}

/// Search output
#[derive(Debug, Serialize)]
pub struct SearchOutput {
    /// Search keyword
    pub keyword: String,
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total results
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::help::HelpSystem;

    fn create_test_discovery() -> CommandDiscovery {
        let help_system = HelpSystem::default();
        let commands = help_system
            .all_commands()
            .into_iter()
            .filter_map(|item| {
                help_system.find_command(&item.name).map(|info| CommandInfo {
                    name: info.name.clone(),
                    category: info.category.clone(),
                    brief: info.brief.clone(),
                    description: info.description.clone(),
                    examples: info.examples.clone(),
                    popularity: 0,
                })
            })
            .collect();

        CommandDiscovery::from_commands(commands)
    }

    #[test]
    fn test_list_all() {
        let discovery = create_test_discovery();
        let all = discovery.list_all();

        assert!(!all.is_empty());
        for item in all {
            assert!(!item.name.is_empty());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn test_list_by_category() {
        let discovery = create_test_discovery();
        let pack_commands = discovery.list_by_category(&CommandCategory::Pack);

        assert!(!pack_commands.is_empty());
        for item in pack_commands {
            assert_eq!(item.category, "Pack");
        }
    }

    #[test]
    fn test_search_exact() {
        let discovery = create_test_discovery();
        let results = discovery.search("pack list");

        assert!(!results.is_empty());
        assert_eq!(results[0].name, "pack list");
        assert_eq!(results[0].match_type, MatchType::ExactName);
    }

    #[test]
    fn test_search_prefix() {
        let discovery = create_test_discovery();
        let results = discovery.search("pack");

        assert!(!results.is_empty());
        // Should find "pack list", "pack install", "pack create"
        for result in results.iter().take(3) {
            assert!(result.name.starts_with("pack"));
        }
    }

    #[test]
    fn test_search_fuzzy() {
        let discovery = create_test_discovery();
        let results = discovery.search("pck");

        assert!(!results.is_empty());
        // Should find pack-related commands
    }

    #[test]
    fn test_suggest() {
        let discovery = create_test_discovery();
        let suggestions = discovery.suggest("pak list");

        assert!(!suggestions.is_empty());
        // Should suggest "pack list"
        assert!(suggestions.iter().any(|s| s.command == "pack list"));
    }

    #[test]
    fn test_categories_summary() {
        let discovery = create_test_discovery();
        let summary = discovery.categories_summary();

        assert_eq!(summary.len(), CommandCategory::all().len());
        for cat in summary {
            assert!(!cat.name.is_empty());
            assert!(!cat.description.is_empty());
        }
    }

    #[test]
    fn test_fuzzy_match() {
        let discovery = CommandDiscovery::new();

        // Perfect match
        assert_eq!(discovery.fuzzy_match("pack list", "pack list"), 1.0);

        // Partial match
        assert!(discovery.fuzzy_match("pack list", "pack") > 0.0);

        // No match
        assert_eq!(discovery.fuzzy_match("pack list", "xyz"), 0.0);
    }

    #[test]
    fn test_generate_commands_output() {
        let discovery = create_test_discovery();
        let output = generate_commands_output(&discovery);

        assert!(output.is_ok());
        let out = output.unwrap();
        assert!(!out.commands.is_empty());
        assert_eq!(out.total, out.commands.len());
    }

    #[test]
    fn test_generate_search_output() {
        let discovery = create_test_discovery();
        let output = generate_search_output(&discovery, "pack");

        assert!(output.is_ok());
        let out = output.unwrap();
        assert_eq!(out.keyword, "pack");
        assert!(!out.results.is_empty());
    }

    #[test]
    fn test_search_no_results() {
        let discovery = create_test_discovery();
        let output = generate_search_output(&discovery, "nonexistent_xyz_123");

        // Should return OK with empty results instead of error (improved UX)
        assert!(output.is_ok());
        let result = output.unwrap();
        assert_eq!(result.total, 0);
        assert!(result.results.is_empty());
    }
}
