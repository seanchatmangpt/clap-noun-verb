//! Deprecation tracking and migration helpers
//!
//! This module provides utilities for marking verbs and arguments as deprecated,
//! showing helpful migration messages to users.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::deprecation::{Deprecation, DeprecationType};
//!
//! let deprecation = Deprecation::new(DeprecationType::Verb)
//!     .since("3.6.0")
//!     .note("This command has been replaced with 'new-name'")
//!     .suggestion("Use 'myapp services new-name' instead");
//! ```

use std::fmt;

/// Type of deprecated entity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeprecationType {
    /// Deprecated noun (command group)
    Noun,
    /// Deprecated verb (subcommand)
    Verb,
    /// Deprecated argument/flag
    Argument,
}

/// Information about a deprecated item with migration guidance
#[derive(Debug, Clone)]
pub struct Deprecation {
    /// Type of deprecated item
    pub item_type: DeprecationType,
    /// Version in which deprecation was introduced
    pub since: Option<String>,
    /// Removal version (when it will be deleted)
    pub removed_in: Option<String>,
    /// Additional context about deprecation
    pub note: Option<String>,
    /// Suggested alternative
    pub suggestion: Option<String>,
}

impl Deprecation {
    /// Create a new deprecation with a given type
    pub fn new(item_type: DeprecationType) -> Self {
        Self { item_type, since: None, removed_in: None, note: None, suggestion: None }
    }

    /// Set the version when deprecation was introduced
    pub fn since(mut self, version: impl Into<String>) -> Self {
        self.since = Some(version.into());
        self
    }

    /// Set the version when this will be removed
    pub fn removed_in(mut self, version: impl Into<String>) -> Self {
        self.removed_in = Some(version.into());
        self
    }

    /// Set additional context/reason for deprecation
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Set the suggested replacement
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Format deprecation as a warning message
    pub fn warning_message(&self, item_name: &str) -> String {
        let mut message = format!("⚠️  {} '{}' is deprecated", self.item_type, item_name);

        if let Some(since) = &self.since {
            message.push_str(&format!(" since v{}", since));
        }

        if let Some(removed) = &self.removed_in {
            message.push_str(&format!(" (will be removed in v{})", removed));
        }

        message.push('\n');

        if let Some(note) = &self.note {
            message.push_str(&format!("\n  {}\n", note));
        }

        if let Some(suggestion) = &self.suggestion {
            message.push_str(&format!("\n  💡 Suggestion: {}\n", suggestion));
        }

        message
    }

    /// Format deprecation as help text
    pub fn help_text(&self, item_name: &str) -> String {
        match (self.since.as_ref(), self.suggestion.as_ref()) {
            (Some(since), Some(suggestion)) => {
                format!("[DEPRECATED since v{}] {} → {}", since, item_name, suggestion)
            }
            (Some(since), None) => {
                format!("[DEPRECATED since v{}] {}", since, item_name)
            }
            (None, Some(suggestion)) => {
                format!("[DEPRECATED] {} → {}", item_name, suggestion)
            }
            (None, None) => format!("[DEPRECATED] {}", item_name),
        }
    }

    /// Check if this deprecation is ready for removal
    pub fn is_removable(&self, current_version: &str) -> bool {
        if let Some(removed) = &self.removed_in {
            current_version.as_bytes() >= removed.as_bytes()
        } else {
            false
        }
    }
}

impl fmt::Display for DeprecationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noun => write!(f, "Noun"),
            Self::Verb => write!(f, "Verb"),
            Self::Argument => write!(f, "Argument"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecation_message() {
        let dep = Deprecation::new(DeprecationType::Verb)
            .since("3.5.0")
            .removed_in("4.0.0")
            .note("This verb has been renamed for clarity")
            .suggestion("Use 'new-verb' instead");

        let msg = dep.warning_message("old-verb");
        assert!(msg.contains("old-verb"));
        assert!(msg.contains("deprecated"));
        assert!(msg.contains("3.5.0"));
        assert!(msg.contains("4.0.0"));
        assert!(msg.contains("new-verb"));
    }

    #[test]
    fn test_help_text() {
        let dep = Deprecation::new(DeprecationType::Argument).since("3.6.0");

        let text = dep.help_text("--old-flag");
        assert!(text.contains("DEPRECATED"));
        assert!(text.contains("--old-flag"));
    }

    #[test]
    fn test_removable() {
        let dep = Deprecation::new(DeprecationType::Verb).removed_in("4.0.0");

        assert!(!dep.is_removable("3.9.0"));
        assert!(dep.is_removable("4.0.0"));
        assert!(dep.is_removable("4.1.0"));
    }
}
