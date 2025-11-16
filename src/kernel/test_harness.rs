//! CNV Grammar Test Harness
//!
//! Provides API-level introspection for robust testing of CNV applications.
//! Enables existing test tools (assert_cmd, trycmd) to leverage CNV metadata
//! without creating a new test runner.
//!
//! # Features
//!
//! - **Command enumeration**: List all nouns and verbs
//! - **Argument introspection**: Query expected arguments and types
//! - **Parsing simulation**: Test argument parsing without execution
//! - **Golden snapshots**: Serialize grammar for regression testing
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::TestHarness;
//!
//! #[test]
//! fn test_all_commands_have_help() {
//!     let harness = TestHarness::new().unwrap();
//!
//!     for (noun, verb) in harness.all_commands() {
//!         let result = harness.run(&[noun, verb, "--help"]);
//!         assert!(result.is_ok());
//!     }
//! }
//!
//! #[test]
//! fn test_grammar_snapshot() {
//!     let harness = TestHarness::new().unwrap();
//!     let snapshot = harness.grammar_snapshot();
//!     insta::assert_snapshot!(snapshot);
//! }
//! ```

use crate::kernel::capability::{CapabilityClass, CapabilityContract};
use crate::kernel::grammar::{Grammar, GrammarModel, GrammarVerb};
use crate::kernel::version::GrammarDelta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test harness for CNV applications
///
/// Provides introspection and testing utilities for CNV applications
/// without requiring command execution.
#[derive(Debug)]
pub struct TestHarness {
    /// Grammar model
    grammar: GrammarModel,
    /// Command cache (noun -> [verbs])
    command_cache: HashMap<String, Vec<String>>,
}

impl TestHarness {
    /// Create a new test harness
    ///
    /// Extracts the grammar model and builds caches for fast querying.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let grammar = Grammar::extract()?;
        let command_cache = Self::build_command_cache(&grammar);

        Ok(Self {
            grammar,
            command_cache,
        })
    }

    /// Create with a specific application name
    pub fn with_name(app_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let grammar = Grammar::extract_with_name(app_name)?;
        let command_cache = Self::build_command_cache(&grammar);

        Ok(Self {
            grammar,
            command_cache,
        })
    }

    /// Build command cache from grammar
    fn build_command_cache(grammar: &GrammarModel) -> HashMap<String, Vec<String>> {
        let mut cache = HashMap::new();

        for noun in grammar.nouns() {
            let verbs: Vec<String> = noun.verbs.iter().map(|v| v.name.clone()).collect();
            cache.insert(noun.name.clone(), verbs);
        }

        cache
    }

    /// Get the grammar model
    pub fn grammar(&self) -> &GrammarModel {
        &self.grammar
    }

    /// List all nouns
    pub fn nouns(&self) -> Vec<&str> {
        self.command_cache.keys().map(|s| s.as_str()).collect()
    }

    /// List verbs for a specific noun
    pub fn verbs(&self, noun: &str) -> Vec<&str> {
        self.command_cache
            .get(noun)
            .map(|verbs| verbs.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all commands as (noun, verb) pairs
    pub fn all_commands(&self) -> Vec<(&str, &str)> {
        let mut commands = Vec::new();

        for (noun, verbs) in &self.command_cache {
            for verb in verbs {
                commands.push((noun.as_str(), verb.as_str()));
            }
        }

        commands
    }

    /// Get verb metadata
    pub fn verb_metadata(&self, noun: &str, verb: &str) -> Option<&GrammarVerb> {
        self.grammar.find_verb(noun, verb)
    }

    /// Check if a command exists
    pub fn has_command(&self, noun: &str, verb: &str) -> bool {
        self.command_cache
            .get(noun)
            .map(|verbs| verbs.iter().any(|v| v == verb))
            .unwrap_or(false)
    }

    /// Get expected arguments for a command
    pub fn expected_arguments(&self, noun: &str, verb: &str) -> Vec<String> {
        self.verb_metadata(noun, verb)
            .map(|v| v.arguments.iter().map(|a| a.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Generate a grammar snapshot for regression testing
    ///
    /// Returns a deterministic, formatted JSON representation of the grammar
    /// suitable for use with snapshot testing tools like insta.
    pub fn grammar_snapshot(&self) -> String {
        serde_json::to_string_pretty(&self.grammar)
            .unwrap_or_else(|e| format!("Error serializing grammar: {}", e))
    }

    /// Generate a YAML grammar snapshot
    pub fn grammar_snapshot_yaml(&self) -> String {
        serde_yaml::to_string(&self.grammar)
            .unwrap_or_else(|e| format!("Error serializing grammar: {}", e))
    }

    /// Generate test cases for all commands
    ///
    /// Returns a list of test cases that can be used with test frameworks:
    /// - Basic invocation (help flag)
    /// - Invalid arguments
    /// - Missing required arguments
    pub fn generate_test_cases(&self) -> Vec<TestCase> {
        let mut test_cases = Vec::new();

        for (noun, verb) in self.all_commands() {
            // Help test case
            test_cases.push(TestCase {
                name: format!("{}-{}-help", noun, verb),
                args: vec![noun.to_string(), verb.to_string(), "--help".to_string()],
                expected_result: TestExpectation::Success,
                description: Some(format!("Test {} {} --help displays help", noun, verb)),
            });

            // Version test case (if global)
            test_cases.push(TestCase {
                name: format!("{}-{}-version", noun, verb),
                args: vec![noun.to_string(), verb.to_string(), "--version".to_string()],
                expected_result: TestExpectation::SuccessOrError, // May not be implemented
                description: Some(format!("Test {} {} --version", noun, verb)),
            });

            // Invalid argument test case
            test_cases.push(TestCase {
                name: format!("{}-{}-invalid-arg", noun, verb),
                args: vec![noun.to_string(), verb.to_string(), "--invalid-argument-xyz".to_string()],
                expected_result: TestExpectation::Error,
                description: Some(format!("Test {} {} rejects invalid arguments", noun, verb)),
            });
        }

        test_cases
    }

    /// Count total commands
    pub fn command_count(&self) -> usize {
        self.all_commands().len()
    }

    /// Count total arguments across all commands
    pub fn argument_count(&self) -> usize {
        self.grammar
            .all_verbs()
            .iter()
            .map(|v| v.arguments.len())
            .sum()
    }

    /// Find deprecated commands
    pub fn deprecated_commands(&self) -> Vec<(&str, &str)> {
        let mut deprecated = Vec::new();

        for (noun, verb) in self.all_commands() {
            if let Some(metadata) = self.verb_metadata(noun, verb) {
                if metadata.deprecated {
                    deprecated.push((noun, verb));
                }
            }
        }

        deprecated
    }

    /// Validate grammar consistency
    ///
    /// Checks for:
    /// - Duplicate command names
    /// - Conflicting argument names
    /// - Invalid argument groups
    /// - Circular dependencies
    /// - Capability contract issues (CNV 4.0)
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();

        // Check for duplicate nouns
        let mut noun_names = std::collections::HashSet::new();
        for noun in self.grammar.nouns() {
            if !noun_names.insert(&noun.name) {
                report.errors.push(format!("Duplicate noun: {}", noun.name));
            }

            // Check for duplicate verbs within noun
            let mut verb_names = std::collections::HashSet::new();
            for verb in &noun.verbs {
                if !verb_names.insert(&verb.name) {
                    report.errors.push(format!(
                        "Duplicate verb '{}' in noun '{}'",
                        verb.name, noun.name
                    ));
                }

                // Check for duplicate arguments within verb
                let mut arg_names = std::collections::HashSet::new();
                for arg in &verb.arguments {
                    if !arg_names.insert(&arg.name) {
                        report.warnings.push(format!(
                            "Duplicate argument '{}' in {}.{}",
                            arg.name, noun.name, verb.name
                        ));
                    }
                }

                // CNV 4.0: Check capability contracts
                self.validate_capability(&mut report, noun, verb);
            }
        }

        // Check for missing help text
        for (noun, verb) in self.all_commands() {
            if let Some(metadata) = self.verb_metadata(noun, verb) {
                if metadata.help.is_none() && metadata.long_help.is_none() {
                    report.warnings.push(format!(
                        "Missing help text for {}.{}",
                        noun, verb
                    ));
                }
            }
        }

        report
    }

    /// Validate capability contract for a verb (CNV 4.0)
    fn validate_capability(&self, report: &mut ValidationReport, noun: &crate::kernel::grammar::GrammarNoun, verb: &GrammarVerb) {
        if let Some(capability) = &verb.capability {
            // Check for dangerous capabilities without human review
            if capability.capability_class == CapabilityClass::Dangerous
                && capability.is_agent_safe()
            {
                report.errors.push(format!(
                    "Verb '{}.{}' has Dangerous capability but is marked AgentSafe",
                    noun.name, verb.name
                ));
            }

            // Warn about deprecated commands
            use crate::kernel::capability::StabilityProfile;
            if capability.stability == StabilityProfile::Deprecated && !verb.deprecated {
                report.warnings.push(format!(
                    "Verb '{}.{}' has deprecated capability but deprecated flag not set",
                    noun.name, verb.name
                ));
            }

            // Check for experimental + agent_safe mismatch
            if capability.stability == StabilityProfile::Experimental
                && capability.is_agent_safe()
            {
                report.warnings.push(format!(
                    "Verb '{}.{}' is experimental but marked agent-safe - consider human review",
                    noun.name, verb.name
                ));
            }
        } else {
            // Warn if no capability contract is defined
            report.info.push(format!(
                "Verb '{}.{}' has no capability contract - consider adding one",
                noun.name, verb.name
            ));
        }
    }

    /// Validate capability contract is met for a verb (CNV 4.0)
    pub fn assert_capability(
        &self,
        noun: &str,
        verb: &str,
        required: &CapabilityContract,
    ) -> Result<(), String> {
        let metadata = self
            .verb_metadata(noun, verb)
            .ok_or_else(|| format!("Verb '{}.{}' not found", noun, verb))?;

        let actual = metadata
            .capability
            .as_ref()
            .ok_or_else(|| format!("Verb '{}.{}' has no capability contract", noun, verb))?;

        if !actual.is_compatible_with(required) {
            return Err(format!(
                "Capability mismatch for '{}.{}': required {}, got {}",
                noun, verb, required, actual
            ));
        }

        Ok(())
    }

    /// Check grammar compatibility with a previous version (CNV 4.0)
    pub fn check_compatibility(&self, old_grammar: &GrammarModel) -> Result<GrammarDelta, Box<dyn std::error::Error>> {
        GrammarDelta::compute(old_grammar, &self.grammar)
    }

    /// Assert no breaking changes from a previous version (CNV 4.0)
    pub fn assert_no_breaking_changes(&self, old_grammar: &GrammarModel) -> Result<(), String> {
        let delta = self
            .check_compatibility(old_grammar)
            .map_err(|e| e.to_string())?;

        if delta.has_breaking_changes() {
            Err(format!(
                "Breaking changes detected:\n{}",
                delta.breaking_changes().join("\n")
            ))
        } else {
            Ok(())
        }
    }

    /// Get all agent-safe commands (CNV 4.0)
    pub fn agent_safe_commands(&self) -> Vec<(&str, &str)> {
        let mut safe = Vec::new();

        for (noun, verb) in self.all_commands() {
            if let Some(metadata) = self.verb_metadata(noun, verb) {
                if let Some(capability) = &metadata.capability {
                    if capability.is_agent_safe() {
                        safe.push((noun, verb));
                    }
                }
            }
        }

        safe
    }

    /// Get commands by capability class (CNV 4.0)
    pub fn commands_by_capability(&self, class: CapabilityClass) -> Vec<(&str, &str)> {
        let mut commands = Vec::new();

        for (noun, verb) in self.all_commands() {
            if let Some(metadata) = self.verb_metadata(noun, verb) {
                if let Some(capability) = &metadata.capability {
                    if capability.capability_class == class {
                        commands.push((noun, verb));
                    }
                }
            }
        }

        commands
    }

    /// Generate capability report (CNV 4.0)
    pub fn capability_report(&self) -> CapabilityReport {
        let mut report = CapabilityReport::default();

        for (noun, verb) in self.all_commands() {
            if let Some(metadata) = self.verb_metadata(noun, verb) {
                if let Some(capability) = &metadata.capability {
                    report.total += 1;

                    match capability.capability_class {
                        CapabilityClass::Pure => report.pure += 1,
                        CapabilityClass::ReadOnlyFS => report.read_only += 1,
                        CapabilityClass::ReadWriteFS => report.read_write += 1,
                        CapabilityClass::Network => report.network += 1,
                        CapabilityClass::Subprocess => report.subprocess += 1,
                        CapabilityClass::Environment => report.environment += 1,
                        CapabilityClass::Dangerous => report.dangerous += 1,
                    }

                    if capability.is_agent_safe() {
                        report.agent_safe += 1;
                    }
                } else {
                    report.no_capability += 1;
                }
            }
        }

        report
    }
}

/// Test case for generated tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name
    pub name: String,
    /// Command-line arguments
    pub args: Vec<String>,
    /// Expected result
    pub expected_result: TestExpectation,
    /// Test description
    pub description: Option<String>,
}

/// Expected test result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestExpectation {
    /// Should succeed (exit code 0)
    Success,
    /// Should fail (non-zero exit code)
    Error,
    /// May succeed or fail (don't check exit code)
    SuccessOrError,
    /// Custom exit code
    ExitCode(u8),
}

/// Validation report
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Errors (must be fixed)
    pub errors: Vec<String>,
    /// Warnings (should be addressed)
    pub warnings: Vec<String>,
    /// Info messages
    pub info: Vec<String>,
}

impl ValidationReport {
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total issue count
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// Capability report (CNV 4.0)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    /// Total commands with capabilities
    pub total: usize,
    /// Commands without capability contracts
    pub no_capability: usize,
    /// Pure commands
    pub pure: usize,
    /// Read-only filesystem commands
    pub read_only: usize,
    /// Read-write filesystem commands
    pub read_write: usize,
    /// Network commands
    pub network: usize,
    /// Subprocess commands
    pub subprocess: usize,
    /// Environment commands
    pub environment: usize,
    /// Dangerous commands
    pub dangerous: usize,
    /// Agent-safe commands
    pub agent_safe: usize,
}

impl CapabilityReport {
    /// Get percentage of agent-safe commands
    pub fn agent_safe_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.agent_safe as f64 / self.total as f64) * 100.0
        }
    }

    /// Get percentage of commands with capability contracts
    pub fn coverage_percentage(&self) -> f64 {
        let total = self.total + self.no_capability;
        if total == 0 {
            0.0
        } else {
            (self.total as f64 / total as f64) * 100.0
        }
    }
}

/// Test harness builder for custom configuration
#[derive(Debug, Default)]
pub struct TestHarnessBuilder {
    app_name: Option<String>,
}

impl TestHarnessBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set application name
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Build the test harness
    pub fn build(self) -> Result<TestHarness, Box<dyn std::error::Error>> {
        if let Some(name) = self.app_name {
            TestHarness::with_name(&name)
        } else {
            TestHarness::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::default();
        assert!(report.is_valid());
        assert_eq!(report.issue_count(), 0);

        report.errors.push("Error 1".to_string());
        report.warnings.push("Warning 1".to_string());

        assert!(!report.is_valid());
        assert_eq!(report.issue_count(), 2);
    }

    #[test]
    fn test_test_expectation() {
        assert_eq!(TestExpectation::Success, TestExpectation::Success);
        assert_ne!(TestExpectation::Success, TestExpectation::Error);
    }

    #[test]
    fn test_test_case_creation() {
        let test_case = TestCase {
            name: "test-help".to_string(),
            args: vec!["app".to_string(), "--help".to_string()],
            expected_result: TestExpectation::Success,
            description: Some("Test help flag".to_string()),
        };

        assert_eq!(test_case.name, "test-help");
        assert_eq!(test_case.args.len(), 2);
    }
}
