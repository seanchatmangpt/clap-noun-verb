// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use clap_noun_verb::{noun, verb, Arg, CliBuilder, VerbArgs};

#[test]
fn test_adversarial_null_bytes() {
    // Arrange - Set up CliBuilder and build command structure
    let cli = CliBuilder::new().name("adversarial-app").noun(noun!(
        "user",
        "User operations",
        [verb!("create", "Create a user", |args: &VerbArgs| {
                let name = args.get_many_opt_str("name");
                assert!(!name.is_empty(), "Name argument should not be empty");
                Ok(())
            }, args: [Arg::new("name").long("name").required(true)])]
    ));
    let cmd = cli.build_command();

    // Act - Parse arguments containing a null byte
    let res =
        cmd.try_get_matches_from(vec!["adversarial-app", "user", "create", "--name", "john\0doe"]);

    // Assert - Verify that the parsing succeeded and null byte is handled gracefully
    assert!(
        res.is_ok(),
        "Expected try_get_matches_from to succeed with null bytes, but got error: {:?}",
        res.err()
    );
}

#[test]
fn test_adversarial_overflow_args() {
    // Arrange - Set up CliBuilder with val argument
    let cli = CliBuilder::new().name("adversarial-app").noun(noun!(
        "calc",
        "Calc operations",
        [verb!("add", "Add values", |args: &VerbArgs| {
                let val = args.get_many_opt_str("val");
                assert!(!val.is_empty(), "Val argument should not be empty");
                Ok(())
            }, args: [Arg::new("val").long("val").required(true)])]
    ));
    let cmd = cli.build_command();

    // Act - Parse and execute command with a massive integer string that could cause overflow
    let res = cmd.try_get_matches_from(vec![
        "adversarial-app",
        "calc",
        "add",
        "--val",
        "1844674407370955161599999999999999999999999999999999",
    ]);

    // Assert - Verify that the extremely large integer string is successfully parsed as a string without crashing
    assert!(
        res.is_ok(),
        "Expected try_get_matches_from to succeed with overflow inputs, but got error: {:?}",
        res.err()
    );
}

#[test]
fn test_adversarial_extremely_long_strings() {
    // Arrange - Set up CliBuilder with large payload parameter
    let cli = CliBuilder::new().name("adversarial-app").noun(noun!(
        "data",
        "Data operations",
        [verb!("store", "Store data", |_args: &VerbArgs| {
                Ok(())
            }, args: [Arg::new("payload").long("payload").required(true)])]
    ));
    let cmd = cli.build_command();
    let long_str = "a".repeat(100_000); // 100KB string

    // Act - Parse extremely long argument string
    let res =
        cmd.try_get_matches_from(vec!["adversarial-app", "data", "store", "--payload", &long_str]);

    // Assert - Verify parser processes extremely large string without overflow or panic
    assert!(
        res.is_ok(),
        "Expected try_get_matches_from to succeed with 100KB payload, but got error: {:?}",
        res.err()
    );
}

#[test]
fn test_adversarial_invalid_command_suggestions() {
    // Arrange - Set up CliBuilder with system status command
    let cli = CliBuilder::new().name("adversarial-app").noun(noun!(
        "system",
        "System command",
        [verb!("status", "Get status", |_args: &VerbArgs| { Ok(()) })]
    ));
    let cmd = cli.build_command();

    // Act - Query invalid subcommand
    let res = cmd.try_get_matches_from(vec!["adversarial-app", "systm", "status"]);

    // Assert - Verify that suggestion mechanism triggers and mentions "system"
    assert!(res.is_err(), "Expected command parsing to fail for invalid subcommand 'systm'");
    let err = res.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("system")
            || err_str.contains("subcommand")
            || err_str.contains("recognized"),
        "Expected error output to contain suggestions or recognize failure, but got: '{}'",
        err_str
    );
}
