//! CLI Integration Tests
//!
//! Chicago TDD approach: State-based testing with real CLI execution.
//! Tests verify CLI commands produce correct observable outputs.

use std::process::Command;
use std::str;

#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    /// Test: `claude-config agent list` returns all agents
    ///
    /// AAA Pattern:
    /// - Arrange: Ensure RDF config file exists
    /// - Act: Execute `claude-config agent list` command
    /// - Assert: Verify output contains all 54+ agent names
    #[test]
    fn test_cli_agent_list_returns_all_agents() {
        // Arrange
        let config_file = "tests/fixtures/claude_config.ttl";
        setup_test_config_file(config_file);

        // Act
        let output = Command::new("cargo")
            .args(&["run", "--bin", "claude-config", "--", "agent", "list"])
            .env("CLAUDE_CONFIG_PATH", config_file)
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(
            output.status.success(),
            "Command should succeed: {:?}",
            str::from_utf8(&output.stderr)
        );

        let stdout = str::from_utf8(&output.stdout).expect("Output should be valid UTF-8");

        // Verify hyper-advanced agents present
        let expected_agents = vec![
            "production-validator",
            "code-analyzer",
            "system-architect",
            "performance-benchmarker",
            "backend-dev",
            "task-orchestrator",
        ];

        for agent in expected_agents {
            assert!(stdout.contains(agent), "Output should contain agent: {}", agent);
        }

        // Verify agent count
        let agent_count = stdout
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with("Total:"))
            .count();

        assert!(agent_count >= 54, "Should list at least 54 agents, found {}", agent_count);
    }

    /// Test: `claude-config agent describe production-validator` shows correct details
    ///
    /// AAA Pattern:
    /// - Arrange: Ensure RDF config file exists
    /// - Act: Execute `claude-config agent describe production-validator`
    /// - Assert: Verify output shows name, type, capabilities, use case
    #[test]
    fn test_cli_agent_describe_shows_correct_details() {
        // Arrange
        let config_file = "tests/fixtures/claude_config.ttl";
        setup_test_config_file(config_file);

        // Act
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "claude-config",
                "--",
                "agent",
                "describe",
                "production-validator",
            ])
            .env("CLAUDE_CONFIG_PATH", config_file)
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(
            output.status.success(),
            "Command should succeed: {:?}",
            str::from_utf8(&output.stderr)
        );

        let stdout = str::from_utf8(&output.stdout).expect("Output should be valid UTF-8");

        // Verify agent details present
        assert!(stdout.contains("Name: production-validator"), "Should show agent name");
        assert!(stdout.contains("Type: hyper-advanced"), "Should show agent type");
        assert!(
            stdout.contains("production_readiness") || stdout.contains("dependency_validation"),
            "Should show capabilities"
        );
        assert!(
            stdout.contains("Validating deployments") || stdout.contains("release readiness"),
            "Should show use case"
        );
    }

    /// Test: `claude-config rules list --category absolute` shows 9 rules
    ///
    /// AAA Pattern:
    /// - Arrange: Ensure RDF config file with absolute rules
    /// - Act: Execute `claude-config rules list --category absolute`
    /// - Assert: Verify output shows exactly 9 absolute rules with mandatory=true
    #[test]
    fn test_cli_rules_list_absolute_shows_nine_rules() {
        // Arrange
        let config_file = "tests/fixtures/claude_config.ttl";
        setup_test_config_file(config_file);

        // Act
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "claude-config",
                "--",
                "rules",
                "list",
                "--category",
                "absolute",
            ])
            .env("CLAUDE_CONFIG_PATH", config_file)
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(
            output.status.success(),
            "Command should succeed: {:?}",
            str::from_utf8(&output.stderr)
        );

        let stdout = str::from_utf8(&output.stdout).expect("Output should be valid UTF-8");

        // Verify absolute rules present
        assert!(
            stdout.contains("cargo make") || stdout.contains("NEVER USE DIRECT CARGO"),
            "Should show cargo make rule"
        );

        let rule_count = stdout
            .lines()
            .filter(|line| line.contains("mandatory: true") || line.contains("[MANDATORY]"))
            .count();

        assert_eq!(rule_count, 9, "Should show exactly 9 absolute rules");
    }

    /// Test: `claude-config slo list` shows 5 SLOs
    ///
    /// AAA Pattern:
    /// - Arrange: Ensure RDF config file with SLOs
    /// - Act: Execute `claude-config slo list`
    /// - Assert: Verify output shows compilation, test, CLI execution, memory SLOs
    #[test]
    fn test_cli_slo_list_shows_performance_targets() {
        // Arrange
        let config_file = "tests/fixtures/claude_config.ttl";
        setup_test_config_file(config_file);

        // Act
        let output = Command::new("cargo")
            .args(&["run", "--bin", "claude-config", "--", "slo", "list"])
            .env("CLAUDE_CONFIG_PATH", config_file)
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(
            output.status.success(),
            "Command should succeed: {:?}",
            str::from_utf8(&output.stderr)
        );

        let stdout = str::from_utf8(&output.stdout).expect("Output should be valid UTF-8");

        // Verify SLOs present
        let expected_slos =
            vec!["Compilation", "Unit tests", "Integration tests", "CLI execution", "Memory usage"];

        for slo in expected_slos {
            assert!(stdout.contains(slo), "Output should contain SLO: {}", slo);
        }

        // Verify performance targets
        assert!(stdout.contains("2s") || stdout.contains("2000ms"), "Should show compilation SLO");
        assert!(stdout.contains("10s") || stdout.contains("10000ms"), "Should show unit test SLO");
        assert!(stdout.contains("100ms"), "Should show CLI execution SLO");
    }

    /// Test: `claude-config query sparql` executes SPARQL correctly
    ///
    /// AAA Pattern:
    /// - Arrange: Create SPARQL query file
    /// - Act: Execute `claude-config query sparql --file query.rq`
    /// - Assert: Verify query results in JSON format
    #[test]
    fn test_cli_query_sparql_executes_correctly() {
        // Arrange
        let config_file = "tests/fixtures/claude_config.ttl";
        let query_file = "tests/fixtures/agent_query.rq";
        setup_test_config_file(config_file);
        setup_test_sparql_query(query_file);

        // Act
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "claude-config",
                "--",
                "query",
                "sparql",
                "--file",
                query_file,
                "--format",
                "json",
            ])
            .env("CLAUDE_CONFIG_PATH", config_file)
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(
            output.status.success(),
            "Command should succeed: {:?}",
            str::from_utf8(&output.stderr)
        );

        let stdout = str::from_utf8(&output.stdout).expect("Output should be valid UTF-8");

        // Verify JSON output
        let json: serde_json::Value =
            serde_json::from_str(stdout).expect("Output should be valid JSON");

        assert!(json.get("results").is_some(), "JSON should have 'results' field");

        let results = json["results"]["bindings"].as_array().expect("Results should be an array");

        assert!(results.len() > 0, "Query should return results");
    }

    /// Test: CLI error handling for invalid commands
    ///
    /// AAA Pattern:
    /// - Arrange: None (testing error path)
    /// - Act: Execute invalid command
    /// - Assert: Verify non-zero exit code and helpful error message
    #[test]
    fn test_cli_error_handling_for_invalid_commands() {
        // Act
        let output = Command::new("cargo")
            .args(&["run", "--bin", "claude-config", "--", "invalid", "command"])
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(!output.status.success(), "Invalid command should fail");

        let stderr = str::from_utf8(&output.stderr).expect("Error output should be valid UTF-8");

        assert!(
            stderr.contains("error:") || stderr.contains("Error:"),
            "Should show error message"
        );
    }

    /// Test: CLI help output shows all commands
    ///
    /// AAA Pattern:
    /// - Arrange: None
    /// - Act: Execute `claude-config --help`
    /// - Assert: Verify help text shows agent, rules, slo, query commands
    #[test]
    fn test_cli_help_output_shows_all_commands() {
        // Act
        let output = Command::new("cargo")
            .args(&["run", "--bin", "claude-config", "--", "--help"])
            .output()
            .expect("Failed to execute claude-config command");

        // Assert
        assert!(output.status.success(), "Help command should succeed");

        let stdout = str::from_utf8(&output.stdout).expect("Output should be valid UTF-8");

        // Verify subcommands present
        let expected_commands = vec!["agent", "rules", "slo", "query"];

        for command in expected_commands {
            assert!(stdout.contains(command), "Help should mention command: {}", command);
        }
    }

    // Helper functions

    fn setup_test_config_file(_path: &str) {
        // In real implementation, would create/copy test fixture file
        // For now, placeholder
    }

    fn setup_test_sparql_query(_path: &str) {
        // In real implementation, would create test SPARQL query file
        // For now, placeholder
    }
}
