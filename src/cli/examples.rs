//! Built-in examples for common workflows
//!
//! This module provides working examples with expected output
//! for the most-used commands to accelerate learning.

use crate::error::Result;
use serde::Serialize;

/// Example with expected output
#[derive(Debug, Clone, Serialize)]
pub struct Example {
    /// Example title
    pub title: String,
    /// Brief description
    pub description: String,
    /// Command to run
    pub command: String,
    /// Expected output (or description of output)
    pub expected_output: String,
    /// Common variations
    pub variations: Vec<Variation>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Command variation
#[derive(Debug, Clone, Serialize)]
pub struct Variation {
    /// Variation description
    pub description: String,
    /// Command
    pub command: String,
}

impl Example {
    /// Create new example
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        command: impl Into<String>,
        expected_output: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            command: command.into(),
            expected_output: expected_output.into(),
            variations: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Add a variation
    pub fn with_variation(mut self, desc: impl Into<String>, cmd: impl Into<String>) -> Self {
        self.variations.push(Variation { description: desc.into(), command: cmd.into() });
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Examples registry
pub struct ExamplesRegistry {
    /// All examples
    examples: Vec<Example>,
}

impl ExamplesRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self { examples: Vec::new() }
    }

    /// Register an example
    pub fn register(&mut self, example: Example) {
        self.examples.push(example);
    }

    /// Get all examples
    pub fn all(&self) -> &[Example] {
        &self.examples
    }

    /// Get examples by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&Example> {
        self.examples.iter().filter(|e| e.tags.contains(&tag.to_string())).collect()
    }

    /// Search examples by keyword
    pub fn search(&self, keyword: &str) -> Vec<&Example> {
        let keyword_lower = keyword.to_lowercase();
        self.examples
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&keyword_lower)
                    || e.description.to_lowercase().contains(&keyword_lower)
                    || e.command.to_lowercase().contains(&keyword_lower)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&keyword_lower))
            })
            .collect()
    }

    /// Generate examples output
    pub fn generate_output(&self) -> Result<ExamplesOutput> {
        Ok(ExamplesOutput { examples: self.examples.clone(), total: self.examples.len() })
    }
}

impl Default for ExamplesRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Example 1: List packs
        registry.register(
            Example::new(
                "List Available Packs",
                "Display all code generation packs in your workspace",
                "ggen pack list",
                r#"{
  "packs": [
    {"name": "rust-web-api", "version": "1.0.0"},
    {"name": "typescript-frontend", "version": "2.1.0"}
  ]
}"#,
            )
            .with_variation("Filter by category", "ggen pack list --category templates")
            .with_variation("Show detailed info", "ggen pack list --verbose")
            .with_tags(vec!["pack".into(), "list".into(), "beginner".into()]),
        );

        // Example 2: AI Generate
        registry.register(
            Example::new(
                "Generate Code with AI",
                "Use AI to generate code from natural language descriptions",
                r#"ggen ai generate "Create a REST API handler for user authentication""#,
                r#"{
  "generated_files": [
    "src/handlers/auth.rs",
    "tests/handlers/auth_test.rs"
  ],
  "status": "success"
}"#,
            )
            .with_variation(
                "Use specific template",
                r#"ggen ai generate --template rust-api "User login endpoint""#,
            )
            .with_variation(
                "Generate with context",
                r#"ggen ai generate --context src/models "CRUD operations for Product""#,
            )
            .with_tags(vec!["ai".into(), "generate".into(), "advanced".into()]),
        );

        // Example 3: Marketplace Search
        registry.register(
            Example::new(
                "Search Marketplace",
                "Find community packs and templates",
                r#"ggen marketplace search "web framework""#,
                r#"{
  "results": [
    {"name": "actix-web-template", "downloads": 5420, "rating": 4.8},
    {"name": "rocket-api-starter", "downloads": 3210, "rating": 4.6}
  ]
}"#,
            )
            .with_variation("Search by category", "ggen marketplace search --category backend")
            .with_variation("Search with filters", "ggen marketplace search rust --min-rating 4.5")
            .with_tags(vec!["marketplace".into(), "search".into(), "discovery".into()]),
        );

        // Example 4: Template Render
        registry.register(
            Example::new(
                "Render Template",
                "Render a code template with variables",
                "ggen template render api-handler --name UserService --model User",
                r#"{
  "output_file": "src/handlers/user_service.rs",
  "status": "success"
}"#,
            )
            .with_variation(
                "Use variables file",
                "ggen template render my-template --vars config.json",
            )
            .with_variation(
                "Render to stdout",
                "ggen template render api-handler --name Auth --stdout",
            )
            .with_tags(vec!["template".into(), "render".into(), "intermediate".into()]),
        );

        // Example 5: Pack Install
        registry.register(
            Example::new(
                "Install Pack",
                "Install a code generation pack",
                "ggen pack install rust-web-api",
                r#"{
  "pack": "rust-web-api",
  "version": "1.0.0",
  "installed": true,
  "templates": 12
}"#,
            )
            .with_variation("Install from local path", "ggen pack install ./my-pack.tar.gz")
            .with_variation("Install specific version", "ggen pack install rust-web-api@2.0.0")
            .with_tags(vec!["pack".into(), "install".into(), "beginner".into()]),
        );

        // Example 6: Config Set
        registry.register(
            Example::new(
                "Configure Settings",
                "Set ggen configuration values",
                "ggen config set ai.provider openai",
                r#"{
  "key": "ai.provider",
  "value": "openai",
  "previous": null
}"#,
            )
            .with_variation("Set API key", "ggen config set ai.api_key $OPENAI_API_KEY")
            .with_variation("Set template engine", "ggen config set template.engine handlebars")
            .with_tags(vec!["config".into(), "settings".into(), "setup".into()]),
        );

        // Example 7: Template Create
        registry.register(
            Example::new(
                "Create Template",
                "Create a new code generation template",
                "ggen template create my-template --type handlebars",
                r#"{
  "template": "my-template",
  "path": "templates/my-template.hbs",
  "type": "handlebars"
}"#,
            )
            .with_variation(
                "Create from existing",
                "ggen template create api-endpoint --from examples/rest.hbs",
            )
            .with_variation(
                "Interactive creation",
                "ggen template create my-template --interactive",
            )
            .with_tags(vec!["template".into(), "create".into(), "advanced".into()]),
        );

        // Example 8: Pack Create
        registry.register(
            Example::new(
                "Create Pack",
                "Create a new code generation pack",
                "ggen pack create my-pack",
                r#"{
  "pack": "my-pack",
  "path": "packs/my-pack",
  "manifest": "packs/my-pack/pack.toml"
}"#,
            )
            .with_variation(
                "Create with template",
                "ggen pack create web-api --template typescript",
            )
            .with_variation(
                "Create with metadata",
                r#"ggen pack create my-pack --author "John Doe" --license MIT"#,
            )
            .with_tags(vec!["pack".into(), "create".into(), "advanced".into()]),
        );

        // Example 9: Marketplace Install
        registry.register(
            Example::new(
                "Install from Marketplace",
                "Install pack directly from marketplace",
                "ggen marketplace install typescript-backend",
                r#"{
  "pack": "typescript-backend",
  "version": "1.5.0",
  "source": "marketplace",
  "installed": true
}"#,
            )
            .with_variation(
                "Install specific version",
                "ggen marketplace install react-components --version 2.0",
            )
            .with_variation(
                "Install with dependencies",
                "ggen marketplace install full-stack --with-deps",
            )
            .with_tags(vec![
                "marketplace".into(),
                "install".into(),
                "intermediate".into(),
            ]),
        );

        // Example 10: AI Analyze
        registry.register(
            Example::new(
                "Analyze Code",
                "Use AI to analyze code and suggest improvements",
                "ggen ai analyze src/main.rs",
                r#"{
  "file": "src/main.rs",
  "suggestions": [
    {"type": "performance", "line": 45, "message": "Consider using a HashMap instead of Vec"},
    {"type": "style", "line": 82, "message": "Function complexity is high, consider refactoring"}
  ]
}"#,
            )
            .with_variation("Analyze with focus", "ggen ai analyze . --focus performance")
            .with_variation("Analyze directory", "ggen ai analyze src/ --recursive")
            .with_tags(vec!["ai".into(), "analyze".into(), "code-quality".into()]),
        );

        registry
    }
}

/// Examples output structure
#[derive(Debug, Serialize)]
pub struct ExamplesOutput {
    /// All examples
    pub examples: Vec<Example>,
    /// Total count
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_builder() {
        let example = Example::new("Test", "Description", "cmd", "output")
            .with_variation("Var 1", "cmd --flag")
            .with_tags(vec!["tag1".into(), "tag2".into()]);

        assert_eq!(example.title, "Test");
        assert_eq!(example.variations.len(), 1);
        assert_eq!(example.tags.len(), 2);
    }

    #[test]
    fn test_registry_default() {
        let registry = ExamplesRegistry::default();
        assert_eq!(registry.examples.len(), 10);
    }

    #[test]
    fn test_registry_by_tag() {
        let registry = ExamplesRegistry::default();
        let beginner_examples = registry.by_tag("beginner");

        assert!(!beginner_examples.is_empty());
        for example in beginner_examples {
            assert!(example.tags.contains(&"beginner".to_string()));
        }
    }

    #[test]
    fn test_registry_search() {
        let registry = ExamplesRegistry::default();
        let results = registry.search("pack");

        assert!(!results.is_empty());
        for example in results {
            let has_keyword = example.title.to_lowercase().contains("pack")
                || example.description.to_lowercase().contains("pack")
                || example.command.to_lowercase().contains("pack")
                || example.tags.iter().any(|t| t.to_lowercase().contains("pack"));
            assert!(has_keyword);
        }
    }

    #[test]
    fn test_registry_search_case_insensitive() {
        let registry = ExamplesRegistry::default();
        let results1 = registry.search("PACK");
        let results2 = registry.search("pack");

        assert_eq!(results1.len(), results2.len());
    }

    #[test]
    fn test_generate_output() {
        let registry = ExamplesRegistry::default();
        let output = registry.generate_output();

        assert!(output.is_ok());
        let out = output.unwrap();
        assert_eq!(out.total, 10);
        assert_eq!(out.examples.len(), 10);
    }

    #[test]
    fn test_all_examples_have_content() {
        let registry = ExamplesRegistry::default();
        for example in registry.all() {
            assert!(!example.title.is_empty());
            assert!(!example.description.is_empty());
            assert!(!example.command.is_empty());
            assert!(!example.expected_output.is_empty());
            assert!(!example.tags.is_empty());
        }
    }
}
