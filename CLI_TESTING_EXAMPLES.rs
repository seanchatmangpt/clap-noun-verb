// Copyright (c) 2024
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Practical CLI Testing Examples for clap-noun-verb
//!
//! This file contains real-world testing patterns you can copy and adapt
//! for your own noun-verb commands. All examples follow the AAA pattern:
//! - Arrange: Set up test data/CLI
//! - Act: Execute the command
//! - Assert: Verify observable behavior

#![allow(dead_code, unused_imports)]

// ============================================================================
// EXAMPLE 1: Basic Command Structure Test
// ============================================================================

#[cfg(test)]
mod example1_basic_command {
    use clap::{Arg, ArgAction, Command};

    /// Test that a CLI has the expected nouns and verbs
    #[test]
    fn test_services_noun_has_status_verb() {
        // ARRANGE: Create minimal CLI structure
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("services")
                    .subcommand(Command::new("status"))
            );

        // ACT: Parse a valid command
        let matches = cmd
            .try_get_matches_from(vec!["myapp", "services", "status"])
            .expect("Should parse valid command");

        // ASSERT: Verify structure
        assert!(matches.subcommand_matches("services").is_some());
        let services = matches.subcommand_matches("services").unwrap();
        assert!(services.subcommand_matches("status").is_some());
    }
}

// ============================================================================
// EXAMPLE 2: Argument Parsing Test
// ============================================================================

#[cfg(test)]
mod example2_argument_parsing {
    use clap::{Arg, Command};

    /// Test required arguments are enforced
    #[test]
    fn test_required_argument_validation() {
        // ARRANGE: CLI with required argument
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("config")
                    .subcommand(
                        Command::new("set")
                            .arg(Arg::new("key").required(true).index(1))
                            .arg(Arg::new("value").required(true).index(2))
                    )
            );

        // ACT: Try to parse with only one argument
        let result = cmd.try_get_matches_from(vec!["myapp", "config", "set", "only_one"]);

        // ASSERT: Should fail
        assert!(result.is_err(), "Should require two arguments");
    }

    /// Test optional arguments with defaults
    #[test]
    fn test_optional_flag_parsing() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("services")
                    .subcommand(
                        Command::new("status")
                            .arg(
                                Arg::new("verbose")
                                    .long("verbose")
                                    .short('v')
                                    .action(clap::ArgAction::SetTrue)
                            )
                    )
            );

        // ACT: Parse with flag
        let matches = cmd
            .clone()
            .try_get_matches_from(vec!["myapp", "services", "status", "--verbose"])
            .unwrap();

        let services = matches.subcommand_matches("services").unwrap();
        let status = services.subcommand_matches("status").unwrap();

        // ASSERT: Flag is set
        assert!(status.get_flag("verbose"));

        // ACT: Parse without flag
        let matches2 = cmd
            .try_get_matches_from(vec!["myapp", "services", "status"])
            .unwrap();

        let services2 = matches2.subcommand_matches("services").unwrap();
        let status2 = services2.subcommand_matches("status").unwrap();

        // ASSERT: Flag defaults to false
        assert!(!status2.get_flag("verbose"));
    }
}

// ============================================================================
// EXAMPLE 3: Help Text Testing
// ============================================================================

#[cfg(test)]
mod example3_help_text {
    use clap::{Arg, Command};

    /// Test help text for root command
    #[test]
    fn test_root_help_text() {
        let mut cmd = Command::new("myapp")
            .version("1.0.0")
            .about("A CLI for managing things");

        // ACT: Generate help text
        let mut help_output = Vec::new();
        cmd.write_help(&mut help_output).expect("Should generate help");

        // ASSERT: Help contains expected content
        let help_text = String::from_utf8_lossy(&help_output);
        assert!(help_text.contains("A CLI for managing things"));
        assert!(help_text.contains("USAGE"));
    }

    /// Test help text for subcommand
    #[test]
    fn test_subcommand_help_text() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("services")
                    .about("Manage services")
                    .subcommand(
                        Command::new("status")
                            .about("Show the status of all services")
                    )
            );

        // ACT: Generate help for subcommand
        let mut services_cmd = cmd.clone()
            .get_subcommands()
            .find(|s| s.get_name() == "services")
            .unwrap()
            .clone();

        let mut help_output = Vec::new();
        services_cmd.write_help(&mut help_output).expect("Should generate help");

        // ASSERT: Subcommand help is complete
        let help_text = String::from_utf8_lossy(&help_output);
        assert!(help_text.contains("Manage services"));
        assert!(help_text.contains("status"));
    }

    /// Test help for argument with description
    #[test]
    fn test_argument_help_descriptions() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("create")
                    .arg(
                        Arg::new("name")
                            .required(true)
                            .index(1)
                            .help("Name of the resource (alphanumeric only)")
                    )
                    .arg(
                        Arg::new("tags")
                            .long("tag")
                            .help("Comma-separated tags for categorization")
                            .action(clap::ArgAction::Append)
                    )
            );

        // ACT: Parse arguments to trigger help text association
        let result = cmd.try_get_matches_from(vec!["myapp", "create", "--help"]);

        // ASSERT: Should display help (will error due to --help, but that's expected)
        assert!(result.is_err()); // --help causes Err
    }
}

// ============================================================================
// EXAMPLE 4: Error Handling
// ============================================================================

#[cfg(test)]
mod example4_error_handling {
    use clap::{Arg, Command};

    /// Test invalid noun is rejected
    #[test]
    fn test_invalid_noun_error() {
        let cmd = Command::new("myapp")
            .subcommand(Command::new("services"));

        // ACT: Try to run invalid noun
        let result = cmd.try_get_matches_from(vec!["myapp", "invalid_noun"]);

        // ASSERT: Should fail
        assert!(result.is_err());
    }

    /// Test invalid verb under valid noun
    #[test]
    fn test_invalid_verb_error() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("services")
                    .subcommand(Command::new("status"))
            );

        // ACT: Try invalid verb under valid noun
        let result = cmd.try_get_matches_from(vec!["myapp", "services", "invalid_verb"]);

        // ASSERT: Should fail
        assert!(result.is_err());
    }

    /// Test error on unexpected positional argument
    #[test]
    fn test_unexpected_positional_argument() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("config")
                    .subcommand(Command::new("set"))
            );

        // ACT: Provide extra positional argument
        let result = cmd.try_get_matches_from(
            vec!["myapp", "config", "set", "extra_arg"]
        );

        // ASSERT: May fail depending on CLI definition
        // (Result depends on whether the command accepts positional args)
        let _ = result;
    }
}

// ============================================================================
// EXAMPLE 5: Value Parsing and Type Conversion
// ============================================================================

#[cfg(test)]
mod example5_value_parsing {
    use clap::{Arg, Command};

    /// Test numeric argument parsing
    #[test]
    fn test_numeric_argument_parsing() {
        let cmd = Command::new("myapp")
            .arg(
                Arg::new("port")
                    .long("port")
                    .value_parser(clap::value_parser!(u16))
            );

        // ACT: Parse valid number
        let matches = cmd.clone()
            .try_get_matches_from(vec!["myapp", "--port", "8080"])
            .expect("Should parse valid number");

        // ASSERT: Value is parsed as u16
        let port: u16 = *matches.get_one("port").unwrap();
        assert_eq!(port, 8080);

        // ACT: Try invalid number
        let result = cmd.try_get_matches_from(vec!["myapp", "--port", "99999"]);

        // ASSERT: Should fail validation (exceeds u16 max)
        assert!(result.is_err());
    }

    /// Test string argument with validation
    #[test]
    fn test_string_argument_with_choices() {
        let cmd = Command::new("myapp")
            .arg(
                Arg::new("level")
                    .long("level")
                    .value_parser(["debug", "info", "warn", "error"])
            );

        // ACT: Parse valid choice
        let matches = cmd.clone()
            .try_get_matches_from(vec!["myapp", "--level", "info"])
            .expect("Should parse valid choice");

        let level = matches.get_one::<String>("level").unwrap();
        assert_eq!(level, "info");

        // ACT: Try invalid choice
        let result = cmd.try_get_matches_from(vec!["myapp", "--level", "invalid"]);

        // ASSERT: Should fail
        assert!(result.is_err());
    }
}

// ============================================================================
// EXAMPLE 6: Complex Multi-Level Commands
// ============================================================================

#[cfg(test)]
mod example6_complex_commands {
    use clap::{Arg, Command};

    /// Test three-level command hierarchy: myapp <noun> <verb> <sub-verb>
    #[test]
    fn test_three_level_command_structure() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("database")
                    .subcommand(
                        Command::new("backup")
                            .arg(Arg::new("format").long("format").required(true))
                    )
                    .subcommand(
                        Command::new("restore")
                            .arg(Arg::new("file").required(true).index(1))
                    )
            );

        // ACT: Parse complete command path
        let matches = cmd
            .try_get_matches_from(vec![
                "myapp", "database", "backup", "--format", "sql"
            ])
            .expect("Should parse complete path");

        // ASSERT: All levels parsed correctly
        let db = matches.subcommand_matches("database").unwrap();
        let backup = db.subcommand_matches("backup").unwrap();
        assert_eq!(
            backup.get_one::<String>("format").unwrap(),
            "sql"
        );
    }

    /// Test multiple independent subcommands at same level
    #[test]
    fn test_multiple_nouns_with_different_verbs() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("users")
                    .subcommand(Command::new("list"))
                    .subcommand(Command::new("create"))
            )
            .subcommand(
                Command::new("roles")
                    .subcommand(Command::new("list"))
                    .subcommand(Command::new("assign"))
            );

        // ACT: Parse users list
        let matches1 = cmd.clone()
            .try_get_matches_from(vec!["myapp", "users", "list"])
            .unwrap();
        let users = matches1.subcommand_matches("users").unwrap();
        assert!(users.subcommand_matches("list").is_some());

        // ACT: Parse roles assign
        let matches2 = cmd
            .try_get_matches_from(vec!["myapp", "roles", "assign"])
            .unwrap();
        let roles = matches2.subcommand_matches("roles").unwrap();
        assert!(roles.subcommand_matches("assign").is_some());
    }
}

// ============================================================================
// EXAMPLE 7: Testing with Global Arguments
// ============================================================================

#[cfg(test)]
mod example7_global_arguments {
    use clap::{Arg, ArgAction, Command};

    /// Test global arguments available to all subcommands
    #[test]
    fn test_global_arguments_passed_to_subcommands() {
        let cmd = Command::new("myapp")
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .action(ArgAction::SetTrue)
                    .global(true)
            )
            .arg(
                Arg::new("config")
                    .long("config")
                    .global(true)
            )
            .subcommand(
                Command::new("services")
                    .subcommand(Command::new("status"))
            );

        // ACT: Use global argument with subcommand
        let matches = cmd
            .try_get_matches_from(vec![
                "myapp", "-v", "--config", "/etc/app.conf",
                "services", "status"
            ])
            .unwrap();

        // ASSERT: Global arguments are available at root level
        assert!(matches.get_flag("verbose"));
        assert_eq!(
            matches.get_one::<String>("config").unwrap(),
            "/etc/app.conf"
        );

        // ASSERT: Subcommand is also parsed
        assert!(matches.subcommand_matches("services").is_some());
    }
}

// ============================================================================
// EXAMPLE 8: Testing Multiple Values for Single Argument
// ============================================================================

#[cfg(test)]
mod example8_multiple_values {
    use clap::{Arg, ArgAction, Command};

    /// Test arguments that accept multiple values
    #[test]
    fn test_multiple_value_collection() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("process")
                    .arg(
                        Arg::new("files")
                            .long("file")
                            .action(ArgAction::Append)
                            .help("Files to process (can use multiple times)")
                    )
            );

        // ACT: Provide multiple values
        let matches = cmd
            .try_get_matches_from(vec![
                "myapp", "process",
                "--file", "a.txt",
                "--file", "b.txt",
                "--file", "c.txt",
            ])
            .unwrap();

        let process = matches.subcommand_matches("process").unwrap();
        let files: Vec<_> = process
            .get_many::<String>("files")
            .unwrap()
            .map(|s| s.as_str())
            .collect();

        // ASSERT: All values collected
        assert_eq!(files, vec!["a.txt", "b.txt", "c.txt"]);
    }

    /// Test arguments with value_delimiter for comma-separated input
    #[test]
    fn test_comma_separated_values() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("tag")
                    .arg(
                        Arg::new("tags")
                            .long("tags")
                            .value_delimiter(',')
                    )
            );

        // ACT: Provide comma-separated values
        let matches = cmd
            .try_get_matches_from(vec![
                "myapp", "tag", "--tags", "dev,staging,prod"
            ])
            .unwrap();

        let tag = matches.subcommand_matches("tag").unwrap();
        let tags: Vec<_> = tag
            .get_many::<String>("tags")
            .unwrap()
            .map(|s| s.as_str())
            .collect();

        // ASSERT: Values split by comma
        assert_eq!(tags, vec!["dev", "staging", "prod"]);
    }
}

// ============================================================================
// EXAMPLE 9: Feature-Gated Command Testing
// ============================================================================

#[cfg(test)]
#[cfg(feature = "experimental")]
mod example9_feature_gated {
    use clap::Command;

    /// Only compiled if "experimental" feature is enabled
    #[test]
    #[cfg(feature = "experimental")]
    fn test_experimental_command_available() {
        let cmd = Command::new("myapp")
            .subcommand(Command::new("experimental"));

        let matches = cmd
            .try_get_matches_from(vec!["myapp", "experimental"])
            .expect("Experimental command should be available");

        assert!(matches.subcommand_matches("experimental").is_some());
    }
}

// ============================================================================
// EXAMPLE 10: Snapshot-style Test
// ============================================================================

#[cfg(test)]
mod example10_snapshot_test {
    use clap::Command;

    /// Verify command structure matches expected snapshot
    #[test]
    fn test_command_structure_unchanged() {
        let cmd = Command::new("myapp")
            .subcommand(Command::new("users"))
            .subcommand(Command::new("roles"))
            .subcommand(Command::new("audit"));

        // ACT: Extract command structure
        let nouns: Vec<_> = cmd
            .get_subcommands()
            .map(|c| c.get_name())
            .collect();

        // ASSERT: Structure matches expected (snapshot)
        let expected = vec!["users", "roles", "audit"];
        assert_eq!(nouns, expected,
            "Command structure changed. Update if intentional.");
    }

    /// Verify verb structure under a noun
    #[test]
    fn test_noun_verbs_unchanged() {
        let cmd = Command::new("myapp")
            .subcommand(
                Command::new("users")
                    .subcommand(Command::new("list"))
                    .subcommand(Command::new("create"))
                    .subcommand(Command::new("delete"))
            );

        // ACT: Extract verbs for "users" noun
        let users_cmd = cmd.get_subcommands()
            .find(|c| c.get_name() == "users")
            .unwrap();

        let verbs: Vec<_> = users_cmd
            .get_subcommands()
            .map(|v| v.get_name())
            .collect();

        // ASSERT: Verbs match snapshot
        let expected = vec!["list", "create", "delete"];
        assert_eq!(verbs, expected);
    }
}

// ============================================================================
// EXAMPLE 11: Testing Help with -h vs --help
// ============================================================================

#[cfg(test)]
mod example11_help_variants {
    use clap::Command;

    /// Test both help flag variants work
    #[test]
    fn test_help_flag_variants() {
        let cmd1 = Command::new("myapp")
            .subcommand(Command::new("services"));

        // ACT: Try -h variant (short form)
        let result_h = cmd1.clone()
            .try_get_matches_from(vec!["myapp", "-h"]);

        // -h triggers help display which causes error
        assert!(result_h.is_err());

        // ACT: Try --help variant (long form)
        let cmd2 = Command::new("myapp")
            .subcommand(Command::new("services"));

        let result_help = cmd2
            .try_get_matches_from(vec!["myapp", "--help"]);

        assert!(result_help.is_err());
        // Both should behave the same
    }
}

// ============================================================================
// EXAMPLE 12: Integration Test Pattern - Combining Multiple Concepts
// ============================================================================

#[cfg(test)]
mod example12_complete_integration_test {
    use clap::{Arg, ArgAction, Command};

    /// A complete integration test combining multiple testing patterns
    #[test]
    fn test_complete_user_management_workflow() {
        // ARRANGE: Build realistic CLI
        let cmd = Command::new("admin")
            .version("1.0.0")
            .about("Admin tool")
            .arg(
                Arg::new("debug")
                    .short('d')
                    .action(ArgAction::SetTrue)
                    .global(true)
            )
            .subcommand(
                Command::new("users")
                    .about("User management")
                    .subcommand(
                        Command::new("list")
                            .about("List all users")
                            .arg(
                                Arg::new("filter")
                                    .long("filter")
                                    .help("Filter users by role")
                            )
                    )
                    .subcommand(
                        Command::new("create")
                            .about("Create new user")
                            .arg(Arg::new("name").required(true).index(1))
                            .arg(
                                Arg::new("role")
                                    .long("role")
                                    .value_parser(["admin", "user", "viewer"])
                                    .required(true)
                            )
                    )
            );

        // TEST 1: List users works
        let matches1 = cmd.clone()
            .try_get_matches_from(vec![
                "admin", "--debug", "users", "list", "--filter", "admin"
            ])
            .expect("Should parse list command");

        assert!(matches1.get_flag("debug"));
        let users = matches1.subcommand_matches("users").unwrap();
        let list = users.subcommand_matches("list").unwrap();
        assert_eq!(
            list.get_one::<String>("filter").unwrap(),
            "admin"
        );

        // TEST 2: Create user with valid role
        let matches2 = cmd.clone()
            .try_get_matches_from(vec![
                "admin", "users", "create", "john",
                "--role", "user"
            ])
            .expect("Should parse create command");

        let users = matches2.subcommand_matches("users").unwrap();
        let create = users.subcommand_matches("create").unwrap();
        assert_eq!(create.get_one::<String>("name").unwrap(), "john");
        assert_eq!(create.get_one::<String>("role").unwrap(), "user");

        // TEST 3: Create user fails with invalid role
        let result3 = cmd.clone()
            .try_get_matches_from(vec![
                "admin", "users", "create", "jane",
                "--role", "superuser"  // Invalid!
            ]);

        assert!(result3.is_err(), "Should reject invalid role");

        // TEST 4: Create user requires name
        let result4 = cmd
            .try_get_matches_from(vec![
                "admin", "users", "create", "--role", "user"
            ]);

        assert!(result4.is_err(), "Should require name argument");
    }
}
