// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed framework errors and deterministic recovery actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors produced by noun-verb parsing, routing, validation, or execution.
#[derive(Error, Debug)]
pub enum NounVerbError {
    /// Command not found.
    #[error("Command '{noun}' not found{suggestion}")]
    CommandNotFound { noun: String, suggestion: String },
    /// Verb not found for a noun.
    #[error("Verb '{verb}' not found for noun '{noun}'{suggestion}")]
    VerbNotFound { noun: String, verb: String, suggestion: String },
    /// Invalid command structure.
    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },
    /// Command execution failure.
    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },
    /// Argument parsing or validation failure.
    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },
    /// Plugin failure.
    #[error("Plugin error: {0}")]
    PluginError(String),
    /// Invariant validation failure.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    /// Middleware failure.
    #[error("Middleware error: {0}")]
    MiddlewareError(String),
    /// Telemetry failure.
    #[error("Telemetry error: {0}")]
    TelemetryError(String),
    /// Generic framework failure.
    #[error("Error: {0}")]
    Generic(String),
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    if left.is_empty() {
        return right.len();
    }
    if right.is_empty() {
        return left.len();
    }

    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_character) in left.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_character) in right.iter().enumerate() {
            let substitution = previous[right_index]
                + usize::from(left_character != right_character);
            let insertion = current[right_index] + 1;
            let deletion = previous[right_index + 1] + 1;
            current[right_index + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

/// Find recovery candidates ordered by distance, then lexicographically.
#[must_use]
pub fn find_best_matches<'a>(input: &str, candidates: &[&'a str]) -> Vec<&'a str> {
    let mut matches: Vec<_> = candidates
        .iter()
        .copied()
        .map(|candidate| (candidate, levenshtein_distance(input, candidate)))
        .filter(|(_, distance)| *distance <= 3 && *distance < input.chars().count())
        .collect();
    matches.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(right.0)));
    matches.into_iter().map(|(candidate, _)| candidate).collect()
}

fn rendered_suggestion(input: &str, candidates: &[&str]) -> String {
    let candidates = find_best_matches(input, candidates);
    if candidates.is_empty() {
        return String::new();
    }
    let rendered = candidates
        .iter()
        .map(|candidate| format!("\x1b[1m\x1b[33m{candidate}\x1b[0m"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(". Did you mean: {rendered}?")
}

impl NounVerbError {
    /// Render the error together with deterministic recovery actions.
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

/// Result type for framework operations.
pub type Result<T> = std::result::Result<T, NounVerbError>;

/// Machine-readable error classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorKind {
    /// Invalid input or command structure.
    InvalidInput,
    /// Operation was not permitted.
    PermissionDenied,
    /// Invariant was violated.
    InvariantBreach,
    /// Deadline or timeout was exceeded.
    DeadlineExceeded,
    /// Resource guard was exceeded.
    GuardExceeded,
    /// Command was not found.
    CommandNotFound,
    /// Verb was not found.
    VerbNotFound,
    /// Command execution failed.
    ExecutionError,
    /// Internal framework failure.
    InternalError,
}

/// Error severity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    /// Non-fatal condition.
    Warning,
    /// Recoverable failure.
    Error,
    /// Severe failure requiring intervention.
    Critical,
}

/// Recovery action proposed by the structured error layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ActionTemplate {
    /// Increase a timeout budget.
    TimeoutAdjustment {
        /// Recommended timeout in milliseconds.
        suggested_timeout_ms: u64,
        /// Rationale.
        reason: String,
    },
    /// Correct a command or route.
    CommandFix {
        /// Suggested command.
        suggested_command: String,
        /// Rationale.
        reason: String,
    },
}

/// Uniform structured error for autonomic consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredError {
    /// Error classification.
    pub kind: ErrorKind,
    /// Error severity.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Structured details.
    pub details: HashMap<String, serde_json::Value>,
    /// Proposed recovery actions.
    pub action_templates: Vec<ActionTemplate>,
}

fn clean_suggestion(suggestion: &str) -> String {
    suggestion
        .replace("\x1b[1m\x1b[33m", "")
        .replace("\x1b[0m", "")
        .replace(". Did you mean: ", "")
        .replace('?', "")
}

impl StructuredError {
    /// Create a deadline-exceeded receipt with observed timing.
    #[must_use]
    pub fn deadline_exceeded(deadline_ms: u64, actual_ms: u64) -> Self {
        let mut details = HashMap::new();
        details.insert("deadline_ms".to_string(), serde_json::json!(deadline_ms));
        details.insert("actual_ms".to_string(), serde_json::json!(actual_ms));
        Self {
            kind: ErrorKind::DeadlineExceeded,
            severity: Severity::Critical,
            message: format!("Deadline {deadline_ms}ms exceeded, took {actual_ms}ms"),
            details,
            action_templates: vec![ActionTemplate::TimeoutAdjustment {
                suggested_timeout_ms: actual_ms.saturating_add(100),
                reason: "Increase deadline budget to match observed latency".to_string(),
            }],
        }
    }

    /// Convert a framework error into its machine-readable form.
    #[must_use]
    pub fn from_error(error: &NounVerbError) -> Self {
        let mut details = HashMap::new();
        let mut actions = Vec::new();
        let mut severity = Severity::Error;

        let kind = match error {
            NounVerbError::CommandNotFound { noun, suggestion } => {
                details.insert("noun".to_string(), serde_json::json!(noun));
                add_command_fix(&mut details, &mut actions, suggestion, noun, None);
                ErrorKind::CommandNotFound
            }
            NounVerbError::VerbNotFound { noun, verb, suggestion } => {
                details.insert("noun".to_string(), serde_json::json!(noun));
                details.insert("verb".to_string(), serde_json::json!(verb));
                add_command_fix(&mut details, &mut actions, suggestion, verb, Some(noun));
                ErrorKind::VerbNotFound
            }
            NounVerbError::InvalidStructure { message }
            | NounVerbError::ArgumentError { message } => {
                details.insert("message".to_string(), serde_json::json!(message));
                ErrorKind::InvalidInput
            }
            NounVerbError::ExecutionError { message } => {
                details.insert("message".to_string(), serde_json::json!(message));
                let normalized = message.to_ascii_lowercase();
                if normalized.contains("deadline")
                    || normalized.contains("timeout")
                    || normalized.contains("budget exceeded")
                {
                    severity = Severity::Critical;
                    actions.push(ActionTemplate::TimeoutAdjustment {
                        suggested_timeout_ms: 1000,
                        reason: "Increase deadline budget due to execution timeout".to_string(),
                    });
                    ErrorKind::DeadlineExceeded
                } else {
                    ErrorKind::ExecutionError
                }
            }
            NounVerbError::ValidationFailed(message) => {
                details.insert("message".to_string(), serde_json::json!(message));
                ErrorKind::InvariantBreach
            }
            NounVerbError::PluginError(message)
            | NounVerbError::MiddlewareError(message)
            | NounVerbError::TelemetryError(message)
            | NounVerbError::Generic(message) => {
                details.insert("message".to_string(), serde_json::json!(message));
                ErrorKind::InternalError
            }
        };

        Self {
            kind,
            severity,
            message: error.to_string(),
            details,
            action_templates: actions,
        }
    }
}

fn add_command_fix(
    details: &mut HashMap<String, serde_json::Value>,
    actions: &mut Vec<ActionTemplate>,
    suggestion: &str,
    misspelled: &str,
    noun: Option<&str>,
) {
    if suggestion.is_empty() {
        return;
    }
    details.insert("suggestion".to_string(), serde_json::json!(suggestion));
    if let Some(first) = clean_suggestion(suggestion).split(", ").next() {
        if !first.is_empty() {
            let command = noun.map_or_else(|| first.to_string(), |noun| format!("{noun} {first}"));
            actions.push(ActionTemplate::CommandFix {
                suggested_command: command,
                reason: format!("Suggested correction for misspelled input '{misspelled}'"),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_uses_structured_command_fix() {
        let rendered = NounVerbError::command_not_found_with_candidates("usr", &["user"])
            .with_recovery_suggestions();
        assert!(rendered.contains("Did you mean"));
        assert!(rendered.contains("Recovery: run 'user'"));
    }

    #[test]
    fn recovery_uses_structured_timeout_adjustment() {
        let rendered =
            NounVerbError::execution_error("deadline exceeded").with_recovery_suggestions();
        assert!(rendered.contains("timeout 1000ms"));
    }

    #[test]
    fn best_matches_are_distance_then_name_ordered() {
        assert_eq!(
            find_best_matches("lst", &["last", "list", "lost"]),
            vec!["last", "list", "lost"]
        );
    }

    #[test]
    fn deadline_receipt_uses_observed_latency() {
        let error = StructuredError::deadline_exceeded(500, 640);
        assert_eq!(
            error.action_templates,
            vec![ActionTemplate::TimeoutAdjustment {
                suggested_timeout_ms: 740,
                reason: "Increase deadline budget to match observed latency".to_string(),
            }]
        );
    }
}
