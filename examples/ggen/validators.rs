//! Input validation helpers for ggen CLI
//!
//! Provides validation functions that return user-friendly errors
//! with actionable recovery suggestions.

use super::errors::{ErrorCategory, UserError};

/// Supported AI model identifiers
const SUPPORTED_MODELS: &[&str] = &[
    "gpt-4-turbo",
    "gpt-4",
    "gpt-3.5-turbo",
    "claude-3-opus",
    "claude-3-sonnet",
    "claude-3-haiku",
];

/// Validate model name and provide suggestions for common mistakes
pub fn validate_model_name(name: &str) -> Result<String, UserError> {
    // Normalize input
    let normalized = name.trim().to_lowercase();

    // Exact match
    if SUPPORTED_MODELS.iter().any(|m| m.to_lowercase() == normalized) {
        return Ok(normalized);
    }

    // Check for common typos and suggest corrections
    let suggestion = match normalized.as_str() {
        n if n.contains("gpt4") => Some("gpt-4-turbo"),
        n if n.contains("gpt3") => Some("gpt-3.5-turbo"),
        n if n.contains("claude") && n.contains("3") => Some("claude-3-sonnet"),
        n if n.contains("turbo") => Some("gpt-4-turbo"),
        n if n.contains("opus") => Some("claude-3-opus"),
        n if n.contains("sonnet") => Some("claude-3-sonnet"),
        n if n.contains("haiku") => Some("claude-3-haiku"),
        _ => None,
    };

    if let Some(suggested) = suggestion {
        let error = UserError::new(
            ErrorCategory::Validation,
            format!("Model '{}' not recognized. Did you mean '{}'?", name, suggested),
            format!(
                "Use the correct model name:\n  \
                ggen ai generate --model {} -d 'your prompt'\n\n  \
                Supported models:\n{}",
                suggested,
                format_model_list()
            ),
        )
        .with_docs("https://docs.ggen.io/models");

        return Err(error);
    }

    Err(super::errors::invalid_model_name(name))
}

/// Validate pack path exists and is accessible
pub fn validate_pack_path(path: &str) -> Result<String, UserError> {
    use std::path::Path;

    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(super::errors::invalid_pack_path(path, "path does not exist"));
    }

    if !path_obj.is_dir() {
        return Err(super::errors::invalid_pack_path(path, "path is not a directory"));
    }

    // Check for pack manifest
    let manifest_path = path_obj.join("pack.toml");
    if !manifest_path.exists() {
        let error = UserError::new(
            ErrorCategory::Validation,
            format!("Directory '{}' is not a valid pack", path),
            format!(
                "A valid pack requires a pack.toml manifest file.\n\n  \
                Create a new pack:\n  \
                ggen pack init {}\n\n  \
                Or specify a different directory:\n  \
                ggen pack list --source <path>",
                path
            ),
        )
        .with_docs("https://docs.ggen.io/packs");

        return Err(error);
    }

    Ok(path.to_string())
}

/// Validate template variables are in key=value format
pub fn validate_template_vars(vars: &[String]) -> Result<Vec<(String, String)>, UserError> {
    let mut parsed = Vec::new();

    for var in vars {
        if let Some((key, value)) = var.split_once('=') {
            if key.trim().is_empty() {
                return Err(UserError::new(
                    ErrorCategory::Validation,
                    format!("Variable '{}' has empty key", var),
                    "Variable keys must not be empty:\n  \
                    ✓ Correct: name=value\n  \
                    ✗ Wrong: =value"
                        .to_string(),
                ));
            }

            parsed.push((key.trim().to_string(), value.trim().to_string()));
        } else {
            return Err(super::errors::invalid_var_format(var));
        }
    }

    Ok(parsed)
}

/// Validate prompt is not empty and has minimum quality
pub fn validate_prompt(prompt: &str) -> Result<String, UserError> {
    let trimmed = prompt.trim();

    if trimmed.is_empty() {
        return Err(super::errors::invalid_prompt("prompt is empty"));
    }

    if trimmed.len() < 10 {
        return Err(super::errors::invalid_prompt("prompt is too short (minimum 10 characters)"));
    }

    // Check for placeholder text
    let lowercase = trimmed.to_lowercase();
    if lowercase.contains("todo") || lowercase.contains("tbd") || lowercase.contains("xxx") {
        let error = UserError::new(
            ErrorCategory::Validation,
            "Prompt appears to contain placeholder text",
            "Replace placeholder text with a clear description:\n  \
            ✗ Wrong: 'TODO: add description'\n  \
            ✓ Correct: 'Create a REST API handler for user authentication'"
                .to_string(),
        );

        return Err(error);
    }

    Ok(trimmed.to_string())
}

/// Validate package identifier format
pub fn validate_package_id(package_id: &str) -> Result<String, UserError> {
    let parts: Vec<&str> = package_id.split('.').collect();

    if parts.len() < 3 {
        let error = UserError::new(
            ErrorCategory::Validation,
            format!("Package identifier '{}' is invalid", package_id),
            "Package identifiers use reverse domain notation:\n  \
            Format: <domain>.<namespace>.<package>\n  \
            Examples:\n  \
              io.ggen.rust.axum\n  \
              com.myorg.templates.api\n\n  \
            Search for packages:\n  \
            ggen marketplace search <keyword>"
                .to_string(),
        )
        .with_docs("https://docs.ggen.io/packages");

        return Err(error);
    }

    // Validate each part
    for part in &parts {
        if part.is_empty() {
            return Err(UserError::new(
                ErrorCategory::Validation,
                format!("Package identifier '{}' has empty component", package_id),
                "All parts of the package identifier must be non-empty:\n  \
                ✗ Wrong: io..rust\n  \
                ✓ Correct: io.ggen.rust"
                    .to_string(),
            ));
        }

        if !part.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(UserError::new(
                ErrorCategory::Validation,
                format!("Package identifier '{}' contains invalid characters", package_id),
                "Package parts can only contain letters, numbers, hyphens, and underscores:\n  \
                ✗ Wrong: io.ggen.my@package\n  \
                ✓ Correct: io.ggen.my-package"
                    .to_string(),
            ));
        }
    }

    Ok(package_id.to_string())
}

/// Validate output path for generated files
pub fn validate_output_path(path: &str) -> Result<String, UserError> {
    use std::path::Path;

    let path_obj = Path::new(path);

    // Check if parent directory exists
    if let Some(parent) = path_obj.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let error = UserError::new(
                ErrorCategory::NotFound,
                format!("Output directory '{}' does not exist", parent.display()),
                format!(
                    "Create the directory first:\n  \
                    mkdir -p {}\n\n  \
                    Or use a different output path:\n  \
                    ggen ai generate -d 'prompt' --output ./existing/path/file.txt",
                    parent.display()
                ),
            );

            return Err(error);
        }
    }

    // Check if file already exists and warn (not error)
    if path_obj.exists() {
        // This is a warning case - we'll allow overwrite but could add --force flag
        // For now, just validate the path is writable
    }

    Ok(path.to_string())
}

/// Format list of supported models for help text
fn format_model_list() -> String {
    SUPPORTED_MODELS
        .iter()
        .map(|m| format!("  - {} {}", m, get_model_description(m)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get description for a model
fn get_model_description(model: &str) -> &str {
    match model {
        "gpt-4-turbo" => "(recommended for complex tasks)",
        "gpt-4" => "(highest quality, slower)",
        "gpt-3.5-turbo" => "(faster, good for simple tasks)",
        "claude-3-opus" => "(best quality, slower)",
        "claude-3-sonnet" => "(balanced speed/quality)",
        "claude-3-haiku" => "(fastest, good for simple tasks)",
        _ => "",
    }
}

/// Check if API key is configured for a provider
pub fn validate_api_key(provider: &str) -> Result<String, UserError> {
    let env_var = format!("{}_API_KEY", provider.to_uppercase().replace('-', "_"));

    std::env::var(&env_var).map_err(|_| super::errors::missing_api_key(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model_name_success() {
        assert!(validate_model_name("gpt-4-turbo").is_ok());
        assert!(validate_model_name("GPT-4-TURBO").is_ok());
        assert!(validate_model_name("  gpt-4-turbo  ").is_ok());
    }

    #[test]
    fn test_validate_model_name_suggestions() {
        let result = validate_model_name("gpt4");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.problem.contains("gpt-4-turbo"));
    }

    #[test]
    fn test_validate_model_name_invalid() {
        let result = validate_model_name("invalid-model");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_template_vars_success() {
        let vars = vec!["name=test".to_string(), "version=1.0".to_string()];
        let result = validate_template_vars(&vars);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], ("name".to_string(), "test".to_string()));
        assert_eq!(parsed[1], ("version".to_string(), "1.0".to_string()));
    }

    #[test]
    fn test_validate_template_vars_invalid_format() {
        let vars = vec!["invalid".to_string()];
        let result = validate_template_vars(&vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_template_vars_empty_key() {
        let vars = vec!["=value".to_string()];
        let result = validate_template_vars(&vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prompt_success() {
        assert!(validate_prompt("Create a REST API handler").is_ok());
    }

    #[test]
    fn test_validate_prompt_empty() {
        assert!(validate_prompt("").is_err());
        assert!(validate_prompt("   ").is_err());
    }

    #[test]
    fn test_validate_prompt_too_short() {
        assert!(validate_prompt("short").is_err());
    }

    #[test]
    fn test_validate_prompt_placeholder() {
        assert!(validate_prompt("TODO: add this later").is_err());
        assert!(validate_prompt("TBD something").is_err());
    }

    #[test]
    fn test_validate_package_id_success() {
        assert!(validate_package_id("io.ggen.rust").is_ok());
        assert!(validate_package_id("com.myorg.templates.api").is_ok());
    }

    #[test]
    fn test_validate_package_id_too_short() {
        assert!(validate_package_id("io.ggen").is_err());
        assert!(validate_package_id("single").is_err());
    }

    #[test]
    fn test_validate_package_id_empty_component() {
        assert!(validate_package_id("io..rust").is_err());
    }

    #[test]
    fn test_validate_package_id_invalid_chars() {
        assert!(validate_package_id("io.ggen.my@package").is_err());
        assert!(validate_package_id("io.ggen.my package").is_err());
    }

    #[test]
    fn test_format_model_list() {
        let list = format_model_list();
        assert!(list.contains("gpt-4-turbo"));
        assert!(list.contains("claude-3-opus"));
    }
}
