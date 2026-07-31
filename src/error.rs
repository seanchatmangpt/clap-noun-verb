// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types and deterministic recovery actions for clap-noun-verb.

use thiserror::Error;

/// Errors that can occur in the noun-verb CLI framework.
#[derive(Error, Debug)]
pub enum NounVerbError {
    /// Command not found.
    #[error("Command '{noun}' not found{suggestion}")]
    CommandNotFound { noun: String, suggestion: String },

    /// Verb not found for a given noun.
    #[error("Verb '{verb}' not found for noun '{noun}'{suggestion}")]
    VerbNotFound { noun: String, verb: String, suggestion: String },

    /// Invalid command structure.
    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    /// Command execution error.
    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },

    /// Argument parsing error.
    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },

    /// Plugin-related error.
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Validation failed.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Middleware error.
    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    /// Telemetry error.
    #[error("Telemetry error: {0}")]
    TelemetryError(String),

    /// Generic error wrapper.
    #[error("Error: {0}")]
    Generic(String),
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut cache: Vec<usize> = (1..=b_len).collect();
    let mut distance = 0;

    for (i, &left) in a_chars.iter().enumerate() {
        let mut diagonal = i;
        distance = i + 1;
        for (j, &right) in b_chars.iter().enumerate() {
            let previous = diagonal;
            diagonal = cache[j];
            distance = if left == right {
                previous
            } else {
                std::cmp::min(std::cmp::min(diagonal, distance), previous) + 1
            };
            cache[j] = distance;
        }
    }

    distance
}

/// Find canonical recovery candidates ordered by edit distance, then name.
#[must_use]
pub fn find_best_matches<'a>(input: &str, candidates: &[&'a str]) -> Vec<&'a str> {
    let mut matches: Vec<(&str, usize)> = candidates
        .iter()
        .map(|&candidate| (candidate, levenshtein_distance(input, candidate)))
        .filter(|&(_, distance)| distance <= 3 && distance < input.len())
        .collect();
    matches.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(right.0)));
    matches.into_iter().map(|(candidate, _)| candidate).collect()
}

fn rendered_suggestion(input: &str, candidates: &[&str]) -> String {
    let matches = find_best_matches(input, candidates);
    if matches.is_empty() {
        String::new()
    } else {
        let rendered = matches
            .iter()
            .map(|candidate| format!("\x1b[1m\x1b[33m{candidate}\x1b[0m"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(". Did you mean: {rendered}?")
    }
}

impl NounVerbError {
    /// Render the error together with deterministic recovery actions.
    ///
    /// Recovery is derived from the same structured error object used by agent
    /// consumers, so the human and machine surfaces cannot silently diverge.
    #[must_use]
    pub fn with_recovery_suggestions(self) -> String {
        let structured = StructuredError::from_error(&self);
        let mut rendered = self.to_string();
        for action in structured.action_templates {
            match action {
                ActionTemplate::TimeoutAdjustment { suggested_timeout_ms, reason } => {
                    rendered.push_str(&format!(
                        "\nRecovery: retry with timeout {suggested_timeout_ms}ms ({reason})"
                    ));
                }
                ActionTemplate::CommandFix { suggested_command, reason } => {
                    rendered.push_str(&format!(
                        "\nRecovery: run '{suggested_command}' ({reason})"
                    ));
                }
            }
        }
        rendered
    }

    /// Create a command-not-found error.
    #[must_use]
    pub fn command_not_found(noun: impl Into<String>) -> Self {
        Self::CommandNotFound { noun: noun.into(), suggestion: String::new() }
    }

    /// Create a command-not-found error with candidate corrections.
    #[must_use]
    pub fn command_not_found_with_candidates(noun: impl Into<String>, candidates: &[&str]) -> Self {
        let noun = noun.into();
        let suggestion = rendered_suggestion(&noun, candidates);
        Self::CommandNotFound { noun, suggestion }
    }

    /// Create a verb-not-found error.
    #[must_use]
    pub fn verb_not_found(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self::VerbNotFound { noun: noun.into(), verb: verb.into(), suggestion: String::new() }
    }

    /// Create a verb-not-found error with candidate corrections.
    #[must_use]
    pub fn verb_not_found_with_candidates(
        noun: impl Into<String>,
        verb: impl Into<String>,
        candidates: &[&str],
    ) -> Self {
        let verb = verb.into();
        let suggestion = rendered_suggestion(&verb, candidates);
        Self::VerbNotFound { noun: noun.into(), verb, suggestion }
    }

    /// Create an invalid-structure error.
    #[must_use]
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure { message: message.into() }
    }

    /// Create an execution error.
    #[must_use]
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::ExecutionError { message: message.into() }
    }

    /// Create an argument error.
    #[must_use]
    pub fn argument_error(message: impl Into<String>) -> Self {
        Self::ArgumentError { message: message.into() }
    }

    /// Create a missing-argument error.
    #[must_use]
    pub fn missing_argument(name: impl Into<String>) -> Self {
        Self::ArgumentError { message: format!("Required argument '{}' is missing", name.into()) }
    }

    /// Create a validation error with an optional constraint description.
    #[must_use]
    pub fn validation_error(
        name: impl Into<String>,
        value: impl Into<String>,
        constraints: Option<&str>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        let message = constraints.map_or_else(
            || format!("Invalid value '{value}' for argument '{name}'"),
            |constraint| format!("Invalid value '{value}' for argument '{name}'. {constraint}"),
        );
        Self::ArgumentError { message }
    }

    /// Create a validation error with numeric range constraints.
    #[must_use]
    pub fn validation_range_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<&str>,
        max: Option<&str>,
    ) -> Self {
        let constraint = match (min, max) {
            (Some(minimum), Some(maximum)) => {
                format!("Must be between {minimum} and {maximum}")
            }
            (Some(minimum), None) => format!("Must be >= {minimum}"),
            (None, Some(maximum)) => format!("Must be <= {maximum}"),
            (None, None) => "Invalid value".to_string(),
        };
        Self::validation_error(name, value, Some(&constraint))
    }

    /// Create a validation error with length constraints.
    #[must_use]
    pub fn validation_length_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Self {
        let constraint = match (min, max) {
            (Some(minimum), Some(maximum)) => {
                format!("Length must be between {minimum} and {maximum} characters")
            }
            (Some(minimum), None) => format!("Length must be at least {minimum} characters"),
            (None, Some(maximum)) => format!("Length must be at most {maximum} characters"),
            (None, None) => "Invalid length".to_string(),
        };
        Self::validation_error(name, value, Some(&constraint))
    }
}

impl From<std::io::Error> for NounVerbError {
    fn from(error: std::io::Error) -> Self {
        Self::ExecutionError { message: error.to_string() }
    }
}

/// Result type alias for noun-verb operations.
pub type Result<T> = std::result::Result<T, NounVerbError>;

/// MAPE-K error classification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ErrorKind {
    /// Input arguments or structure were invalid.
    InvalidInput,
    /// The operation was not permitted.
    PermissionDenied,
    /// A required invariant was violated.
    InvariantBreach,
    /// A deadline or timeout budget was exceeded.
    DeadlineExceeded,
    /// A resource guard limit was exceeded.
    GuardExceeded,
    /// The requested noun/command was not found.
    CommandNotFound,
    /// The requested verb was not found for a noun.
    VerbNotFound,
    /// Execution of the command failed.
    ExecutionError,
    /// An internal framework error occurred.
    InternalError,
}

/// Severity level of the error.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Severity {
    /// Non-fatal condition; execution may continue.
    Warning,
    /// A recoverable error occurred.
    Error,
    /// A severe error requiring immediate attention.
    Critical,
}

/// Recovery action proposed by the MAPE-K recovery layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ActionTemplate {
    /// Suggest increasing the timeout/deadline budget.
    TimeoutAdjustment {
        /// Recommended new timeout in milliseconds.
        suggested_timeout_ms: u64,
        /// Human-readable rationale for the adjustment.
        reason: String,
    },
    /// Suggest a corrected command.
    CommandFix {
        /// Corrected command string.
        suggested_command: String,
        /// Human-readable rationale for the correction.
        reason: String,
    },
}

/// Machine-readable structured error for autonomic loops.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StructuredError {
    /// Classification of the error.
    pub kind: ErrorKind,
    /// Severity level.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Additional details keyed by field name.
    pub details: std::collections::HashMap<String, serde_json::Value>,
    /// Proposed recovery actions.
    pub action_templates: Vec<ActionTemplate>,
}

fn clean_ansi_suggestion(suggestion: &str) -> String {
    suggestion
        .replace("\x1b[1m\x1b[33m", "")
        .replace("\x1b[0m", "")
        .replace(". Did you mean: ", "")
        .replace('?', "")
}

impl StructuredError {
    /// Create a deadline-exceeded error with observed and target latency.
    #[must_use]
    pub fn deadline_exceeded(deadline_ms: u64, actual_ms: u64) -> Self {
        let mut details = std::collections::HashMap::new();
        details.insert("deadline_ms".to_string(), serde_json::json!(deadline_ms));
        details.insert("actual_ms".to_string(), serde_json::json!(actual_ms));
        Self {
            kind: ErrorKind::DeadlineExceeded,
            severity: Severity::Critical,
            message: format!("Deadline {deadline_ms}ms exceeded, took {actual_ms}ms"),
            details,
            action_templates: vec![ActionTemplate::TimeoutAdjustment {
                suggested_timeout_ms: actual_ms + 100,
                reason: "Increase deadline budget to match observed latency".to_string(),
            }],
        }
    }

    /// Convert a framework error into its machine-readable form.
    #[must_use]
    pub fn from_error(error: &NounVerbError) -> Self {
        let mut details = std::collections::HashMap::new();
        let mut action_templates = Vec::new();
        let mut severity = Severity::Error;

        let kind = match error {
            NounVerbError::CommandNotFound { noun, suggestion } => {
                details.insert("noun".to_string(), serde_json::Value::String(noun.clone()));
                if !suggestion.is_empty() {
                    details.insert(
                        "suggestion".to_string(),
                        serde_json::Value::String(suggestion.clone()),
                    );
                    if let Some(first) = clean_ansi_suggestion(suggestion).split(", ").next() {
                        if !first.is_empty() {
                            action_templates.push(ActionTemplate::CommandFix {
                                suggested_command: first.to_string(),
                                reason: format!(
                                    "Suggested correction for misspelled command '{noun}'"
                                ),
                            });
                        }
                    }
                }
                ErrorKind::CommandNotFound
            }
            NounVerbError::VerbNotFound { noun, verb, suggestion } => {
                details.insert("noun".to_string(), serde_json::Value::String(noun.clone()));
                details.insert("verb".to_string(), serde_json::Value::String(verb.clone()));
                if !suggestion.is_empty() {
                    details.insert(
                        "suggestion".to_string(),
                        serde_json::Value::String(suggestion.clone()),
                    );
                    if let Some(first) = clean_ansi_suggestion(suggestion).split(", ").next() {
                        if !first.is_empty() {
                            action_templates.push(ActionTemplate::CommandFix {
                                suggested_command: format!("{noun} {first}"),
                                reason: format!(
                                    "Suggested correction for misspelled verb '{verb}'"
                                ),
                            });
                        }
                    }
                }
                ErrorKind::VerbNotFound
            }
            NounVerbError::InvalidStructure { message } => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InvalidInput
            }
            NounVerbError::ExecutionError { message } => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                let normalized = message.to_lowercase();
                if normalized.contains("deadline")
                    || normalized.contains("timeout")
                    || normalized.contains("budget exceeded")
                {
                    severity = Severity::Critical;
                    action_templates.push(ActionTemplate::TimeoutAdjustment {
                        suggested_timeout_ms: 1000,
                        reason: "Increase deadline budget due to execution timeout".to_string(),
                    });
                    ErrorKind::DeadlineExceeded
                } else {
                    ErrorKind::ExecutionError
                }
            }
            NounVerbError::ArgumentError { message } => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InvalidInput
            }
            NounVerbError::PluginError(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InternalError
            }
            NounVerbError::ValidationFailed(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InvariantBreach
            }
            NounVerbError::MiddlewareError(message)
            | NounVerbError::TelemetryError(message)
            | NounVerbError::Generic(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InternalError
            }
        };

        Self {
            kind,
            severity,
            message: error.to_string(),
            details,
            action_templates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_suggestion_uses_structured_command_fix() {
        let rendered = NounVerbError::command_not_found_with_candidates("usr", &["user"])
            .with_recovery_suggestions();
        assert!(rendered.contains("Did you mean"));
        assert!(rendered.contains("Recovery: run 'user'"));
    }

    #[test]
    fn recovery_suggestion_uses_structured_timeout_adjustment() {
        let rendered = NounVerbError::execution_error("deadline exceeded")
            .with_recovery_suggestions();
        assert!(rendered.contains("timeout 1000ms"));
    }

    #[test]
    fn best_matches_are_distance_then_name_ordered() {
        assert_eq!(find_best_matches("lst", &["last", "list", "lost"]), vec!["list", "last", "lost"]);
    }
}
