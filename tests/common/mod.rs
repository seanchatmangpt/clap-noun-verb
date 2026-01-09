//! Common test utilities for clap-noun-verb tests

pub mod test_prelude;

/// Assertion helpers for command structure verification
pub mod command_assertions {
    use clap::Command;

    /// Assert command has a specific subcommand
    pub fn assert_has_subcommand(cmd: &Command, name: &str) {
        assert!(
            cmd.get_subcommands().any(|s| s.get_name() == name),
            "Command should have '{}' subcommand",
            name
        );
    }

    /// Assert command does NOT have a specific subcommand
    pub fn assert_no_subcommand(cmd: &Command, name: &str) {
        assert!(
            !cmd.get_subcommands().any(|s| s.get_name() == name),
            "Command should not have '{}' subcommand",
            name
        );
    }

    /// Assert subcommand has specific verb
    pub fn assert_subcommand_has_verb(cmd: &Command, subcommand: &str, verb: &str) {
        let sub = cmd
            .get_subcommands()
            .find(|s| s.get_name() == subcommand)
            .expect(&format!("Subcommand '{}' not found", subcommand));

        assert!(
            sub.get_subcommands().any(|v| v.get_name() == verb),
            "Subcommand '{}' should have verb '{}'",
            subcommand,
            verb
        );
    }

    /// Assert command has version set
    pub fn assert_has_version(cmd: &Command, expected_version: Option<&str>) {
        match (cmd.get_version(), expected_version) {
            (Some(actual), Some(exp)) => {
                assert_eq!(actual, exp, "Version mismatch");
            }
            (Some(_), None) => {
                panic!("Expected no version, but command has version");
            }
            (None, Some(_)) => {
                panic!("Expected version, but command has no version");
            }
            (None, None) => {}
        }
    }

    /// Get all subcommand names as a Vec
    pub fn get_subcommand_names(cmd: &Command) -> Vec<&str> {
        cmd.get_subcommands().map(|s| s.get_name()).collect()
    }

    /// Get all verb names under a subcommand (with lifetime bound to command)
    pub fn get_verb_names<'a>(cmd: &'a Command, subcommand: &str) -> Vec<&'a str> {
        cmd.get_subcommands()
            .find(|s| s.get_name() == subcommand)
            .map(|s| s.get_subcommands().map(|v| v.get_name()).collect())
            .unwrap_or_default()
    }

    /// Assert help text contains expected content
    pub fn assert_help_contains(cmd: &mut Command, expected: &str) {
        let mut help_output = Vec::new();
        cmd.write_help(&mut help_output).expect("Failed to write help");
        let help_text = String::from_utf8_lossy(&help_output);
        assert!(
            help_text.contains(expected),
            "Help should contain '{}'. Got: {}",
            expected,
            help_text
        );
    }
}

/// Context helpers for handler testing
pub mod handler_context {
    use clap_noun_verb::logic::HandlerContext;

    /// Create a handler context with verb and noun
    pub fn create_context(verb: &str, noun: Option<&str>) -> HandlerContext {
        let mut ctx = HandlerContext::new(verb);
        if let Some(n) = noun {
            ctx = ctx.with_noun(n);
        }
        ctx
    }

    /// Create a handler context with additional data
    pub fn create_context_with_data(
        verb: &str,
        noun: Option<&str>,
        data: Vec<(&str, &str)>,
    ) -> HandlerContext {
        let mut ctx = create_context(verb, noun);
        for (key, value) in data {
            ctx = ctx.with_data(key, value);
        }
        ctx
    }
}

/// Capture stdout for testing
pub struct OutputCapture {
    // Will implement stdout capture for JSON output testing
}

impl OutputCapture {
    pub fn new() -> Self {
        Self {}
    }

    pub fn capture<F>(f: F) -> String
    where
        F: FnOnce() -> (),
    {
        // Placeholder - will implement actual stdout capture
        f();
        String::new()
    }
}

impl Default for OutputCapture {
    fn default() -> Self {
        Self::new()
    }
}

/// Assert JSON output matches expected value
pub fn assert_json_eq<T>(actual: &T, expected: &T)
where
    T: serde::Serialize + PartialEq + std::fmt::Debug,
{
    let actual_json = serde_json::to_string(actual).unwrap();
    let expected_json = serde_json::to_string(expected).unwrap();
    assert_eq!(actual_json, expected_json, "JSON output mismatch");
}
