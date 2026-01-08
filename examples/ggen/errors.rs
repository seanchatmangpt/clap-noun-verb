//! User-friendly error handling for ggen CLI
//!
//! This module provides actionable error messages with recovery suggestions
//! to reduce support requests and improve user experience.

use std::fmt;

/// User-facing error with problem description and actionable recovery steps
#[derive(Debug, Clone)]
pub struct UserError {
    /// What went wrong (user-friendly description)
    pub problem: String,
    /// How to fix it (actionable steps)
    pub solution: String,
    /// Documentation or help link
    pub learn_more: Option<String>,
    /// Error category for metrics
    pub category: ErrorCategory,
}

/// Error categories for tracking and metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Invalid input or arguments
    Validation,
    /// File or resource not found
    NotFound,
    /// Missing configuration
    Configuration,
    /// Network or API errors
    Network,
    /// System or internal errors
    Internal,
}

impl UserError {
    /// Create a new user error with problem and solution
    pub fn new(
        category: ErrorCategory,
        problem: impl Into<String>,
        solution: impl Into<String>,
    ) -> Self {
        Self { problem: problem.into(), solution: solution.into(), learn_more: None, category }
    }

    /// Add a documentation link
    pub fn with_docs(mut self, link: impl Into<String>) -> Self {
        self.learn_more = Some(link.into());
        self
    }

    /// Format error with emoji markers for better readability
    pub fn format_pretty(&self) -> String {
        let mut output = format!("❌ Problem: {}\n💡 Solution: {}", self.problem, self.solution);

        if let Some(link) = &self.learn_more {
            output.push_str(&format!("\n📚 Learn more: {}", link));
        }

        output
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_pretty())
    }
}

impl std::error::Error for UserError {}

/// Validation error for model names
pub fn invalid_model_name(name: &str) -> UserError {
    let problem = format!("Model '{}' is not recognized", name);
    let solution = format!(
        "Use one of the supported models:\n  \
        - gpt-4-turbo (recommended for complex tasks)\n  \
        - gpt-3.5-turbo (faster, good for simple tasks)\n  \
        - claude-3-opus (best quality, slower)\n  \
        - claude-3-sonnet (balanced speed/quality)\n\n  \
        Example: ggen ai generate --model gpt-4-turbo -d 'your prompt'"
    );

    UserError::new(ErrorCategory::Validation, problem, solution)
        .with_docs("https://docs.ggen.io/models")
}

/// Validation error for missing API key
pub fn missing_api_key(provider: &str) -> UserError {
    let problem = format!("{} API key is not configured", provider);
    let solution = format!(
        "Set the environment variable:\n  \
        export {}_API_KEY='your-api-key-here'\n\n  \
        Or add it to your ~/.ggen/config.toml:\n  \
        [{}]\n  \
        api_key = 'your-api-key-here'",
        provider.to_uppercase().replace('-', "_"),
        provider.to_lowercase()
    );

    UserError::new(ErrorCategory::Configuration, problem, solution)
        .with_docs("https://docs.ggen.io/configuration")
}

/// Validation error for empty or invalid prompts
pub fn invalid_prompt(reason: &str) -> UserError {
    let problem = format!("Invalid prompt: {}", reason);
    let solution = "Provide a clear description using -d or --description:\n  \
        ggen ai generate -d 'Create a REST API handler for user authentication'\n\n  \
        Tips for good prompts:\n  \
        - Be specific about what you want to generate\n  \
        - Include technology stack (e.g., 'using Rust and Axum')\n  \
        - Mention any constraints or requirements"
        .to_string();

    UserError::new(ErrorCategory::Validation, problem, solution)
        .with_docs("https://docs.ggen.io/prompts")
}

/// Validation error for pack path issues
pub fn invalid_pack_path(path: &str, reason: &str) -> UserError {
    let problem = format!("Cannot access pack at '{}': {}", path, reason);
    let solution = format!(
        "Check the following:\n  \
        1. Verify the path exists: ls {}\n  \
        2. Check file permissions: ls -la {}\n  \
        3. Use absolute path or relative to current directory\n\n  \
        Example: ggen pack list --source ./my-packs/",
        path, path
    );

    UserError::new(ErrorCategory::NotFound, problem, solution)
        .with_docs("https://docs.ggen.io/packs")
}

/// Validation error for marketplace search with no results
pub fn no_search_results(query: &str) -> UserError {
    let problem = format!("No packages found matching '{}'", query);
    let solution = "Try these alternatives:\n  \
        - Broaden your search terms\n  \
        - Check spelling\n  \
        - Browse all packages: ggen marketplace list\n  \
        - Search by category: ggen marketplace search --category rust\n\n  \
        Popular categories:\n  \
        - rust (Rust templates)\n  \
        - web (Web frameworks)\n  \
        - api (API templates)"
        .to_string();

    UserError::new(ErrorCategory::NotFound, problem, solution)
        .with_docs("https://marketplace.ggen.io")
}

/// Validation error for template variable issues
pub fn missing_template_vars(
    template: &str,
    required: &[String],
    provided: &[String],
) -> UserError {
    let missing: Vec<_> = required.iter().filter(|r| !provided.contains(r)).collect();

    let problem = format!(
        "Template '{}' requires {} variable(s) that were not provided",
        template,
        missing.len()
    );

    let vars_list =
        missing.iter().map(|v| format!("  {}=<value>", v)).collect::<Vec<_>>().join("\n");

    let solution = format!(
        "Provide the following variables:\n{}\n\n  \
        Example: ggen template generate {} {}",
        vars_list,
        template,
        missing.iter().map(|v| format!("{}=example", v)).collect::<Vec<_>>().join(" ")
    );

    UserError::new(ErrorCategory::Validation, problem, solution)
        .with_docs("https://docs.ggen.io/templates")
}

/// Validation error for invalid template variable format
pub fn invalid_var_format(var: &str) -> UserError {
    let problem = format!("Invalid variable format: '{}'", var);
    let solution = "Variables must be in key=value format:\n  \
        ✓ Correct: name=MyProject version=1.0.0\n  \
        ✗ Wrong: name:MyProject, name\n\n  \
        Example: ggen template generate my-template.tmpl \\\n    \
        project_name=MyApp \\\n    \
        author='John Doe' \\\n    \
        version=1.0.0"
        .to_string();

    UserError::new(ErrorCategory::Validation, problem, solution)
}

/// Network error for API failures
pub fn api_request_failed(provider: &str, status: u16, message: &str) -> UserError {
    let problem = format!("{} API request failed (HTTP {}): {}", provider, status, message);

    let solution = match status {
        401 => "Check your API key:\n  \
            1. Verify the key is correct in your config\n  \
            2. Check if the key has expired\n  \
            3. Ensure you have an active subscription"
            .to_string(),
        429 => "Rate limit exceeded:\n  \
            1. Wait a few minutes before retrying\n  \
            2. Consider upgrading your API plan\n  \
            3. Use --retry-after flag to auto-retry"
            .to_string(),
        500..=599 => format!(
            "{} service is experiencing issues:\n  \
            1. Try again in a few minutes\n  \
            2. Check status: https://status.{}.com\n  \
            3. Use alternative provider with --model flag",
            provider,
            provider.to_lowercase()
        ),
        _ => "Check your network connection and try again:\n  \
            1. Verify internet connectivity\n  \
            2. Check firewall settings\n  \
            3. Try with --verbose for detailed logs"
            .to_string(),
    };

    UserError::new(ErrorCategory::Network, problem, solution)
        .with_docs("https://docs.ggen.io/troubleshooting")
}

/// File I/O error with helpful context
pub fn file_error(path: &str, operation: &str, error: &str) -> UserError {
    let problem = format!("Cannot {} file '{}': {}", operation, path, error);
    let solution = format!(
        "Try the following:\n  \
        1. Check if the file exists: ls -la {}\n  \
        2. Verify permissions: chmod 644 {}\n  \
        3. Ensure directory exists: mkdir -p $(dirname {})\n  \
        4. Check disk space: df -h",
        path, path, path
    );

    UserError::new(ErrorCategory::Internal, problem, solution)
}

/// Package not found error
pub fn package_not_found(package: &str) -> UserError {
    let problem = format!("Package '{}' not found in marketplace", package);
    let solution = format!(
        "Search for available packages:\n  \
        ggen marketplace search '{}'\n\n  \
        Or browse all packages:\n  \
        ggen marketplace list\n\n  \
        Package names are case-sensitive and use reverse domain notation:\n  \
        Example: io.ggen.rust.axum",
        package.split('.').last().unwrap_or(package)
    );

    UserError::new(ErrorCategory::NotFound, problem, solution)
        .with_docs("https://marketplace.ggen.io")
}

/// Configuration file error
pub fn invalid_config(path: &str, error: &str) -> UserError {
    let problem = format!("Invalid configuration file '{}': {}", path, error);
    let solution = format!(
        "Fix your configuration file:\n  \
        1. Check TOML syntax: https://toml.io/en/\n  \
        2. View example config: ggen config example\n  \
        3. Validate config: ggen config validate {}\n\n  \
        Or reset to defaults:\n  \
        ggen config reset",
        path
    );

    UserError::new(ErrorCategory::Configuration, problem, solution)
        .with_docs("https://docs.ggen.io/configuration")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_error_display() {
        let error = UserError::new(ErrorCategory::Validation, "Test problem", "Test solution");

        let display = error.to_string();
        assert!(display.contains("❌ Problem: Test problem"));
        assert!(display.contains("💡 Solution: Test solution"));
    }

    #[test]
    fn test_user_error_with_docs() {
        let error = UserError::new(ErrorCategory::Validation, "Test problem", "Test solution")
            .with_docs("https://example.com");

        let display = error.to_string();
        assert!(display.contains("📚 Learn more: https://example.com"));
    }

    #[test]
    fn test_invalid_model_name() {
        let error = invalid_model_name("invalid-model");
        assert!(error.problem.contains("invalid-model"));
        assert!(error.solution.contains("gpt-4-turbo"));
        assert!(error.solution.contains("claude-3-opus"));
        assert_eq!(error.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_missing_api_key() {
        let error = missing_api_key("openai");
        assert!(error.problem.contains("openai"));
        assert!(error.solution.contains("OPENAI_API_KEY"));
        assert_eq!(error.category, ErrorCategory::Configuration);
    }

    #[test]
    fn test_invalid_prompt() {
        let error = invalid_prompt("empty prompt");
        assert!(error.problem.contains("empty prompt"));
        assert!(error.solution.contains("-d"));
        assert_eq!(error.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_no_search_results() {
        let error = no_search_results("nonexistent");
        assert!(error.problem.contains("nonexistent"));
        assert!(error.solution.contains("ggen marketplace list"));
        assert_eq!(error.category, ErrorCategory::NotFound);
    }

    #[test]
    fn test_missing_template_vars() {
        let required = vec!["name".to_string(), "version".to_string()];
        let provided = vec!["name".to_string()];
        let error = missing_template_vars("template.tmpl", &required, &provided);

        assert!(error.problem.contains("template.tmpl"));
        assert!(error.solution.contains("version="));
        assert_eq!(error.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_api_request_failed_401() {
        let error = api_request_failed("OpenAI", 401, "Unauthorized");
        assert!(error.problem.contains("401"));
        assert!(error.solution.contains("API key"));
        assert_eq!(error.category, ErrorCategory::Network);
    }

    #[test]
    fn test_api_request_failed_429() {
        let error = api_request_failed("OpenAI", 429, "Rate limit");
        assert!(error.solution.contains("Rate limit"));
        assert!(error.solution.contains("retry"));
    }

    #[test]
    fn test_package_not_found() {
        let error = package_not_found("io.ggen.nonexistent");
        assert!(error.problem.contains("io.ggen.nonexistent"));
        assert!(error.solution.contains("ggen marketplace search"));
        assert_eq!(error.category, ErrorCategory::NotFound);
    }

    #[test]
    fn test_error_categories() {
        let categories = [
            ErrorCategory::Validation,
            ErrorCategory::NotFound,
            ErrorCategory::Configuration,
            ErrorCategory::Network,
            ErrorCategory::Internal,
        ];

        // Ensure all categories are distinct
        for (i, cat1) in categories.iter().enumerate() {
            for (j, cat2) in categories.iter().enumerate() {
                if i == j {
                    assert_eq!(cat1, cat2);
                } else {
                    assert_ne!(cat1, cat2);
                }
            }
        }
    }
}
