// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! E2E tests for automatic completions generation using compiled examples.

use assert_cmd::Command;

#[test]
fn test_completions_e2e_bash() {
    let mut cmd = Command::cargo_bin("completions_demo")
        .expect("Failed to get command for completions_demo example");
    cmd.arg("completions").arg("bash");

    let assert = cmd.assert().success();
    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("Output is not valid UTF-8");

    assert!(
        stdout.contains("# completions_demo completion script for bash"),
        "Output missing header for bash"
    );
    assert!(stdout.contains("complete -o bashdefault"), "Output missing complete command for bash");
    assert!(stdout.contains("services status"), "Output missing services status subcommands");
}

#[test]
fn test_completions_e2e_zsh() {
    let mut cmd = Command::cargo_bin("completions_demo")
        .expect("Failed to get command for completions_demo example");
    cmd.arg("completions").arg("zsh");

    let assert = cmd.assert().success();
    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("Output is not valid UTF-8");

    assert!(stdout.contains("#compdef completions_demo"), "Output missing compdef header for zsh");
    assert!(
        stdout.contains("# completions_demo completion script for zsh"),
        "Output missing header for zsh"
    );
    assert!(stdout.contains("_arguments"), "Output missing arguments specification for zsh");
}

#[test]
fn test_completions_e2e_fish() {
    let mut cmd = Command::cargo_bin("completions_demo")
        .expect("Failed to get command for completions_demo example");
    cmd.arg("completions").arg("fish");

    let assert = cmd.assert().success();
    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("Output is not valid UTF-8");

    assert!(
        stdout.contains("# completions_demo completion script for fish"),
        "Output missing header for fish"
    );
    assert!(
        stdout.contains("complete -c completions_demo"),
        "Output missing complete command for fish"
    );
    assert!(stdout.contains("services status"), "Output missing services status subcommands");
}

#[test]
fn test_completions_e2e_powershell() {
    let mut cmd = Command::cargo_bin("completions_demo")
        .expect("Failed to get command for completions_demo example");
    cmd.arg("completions").arg("powershell");

    let assert = cmd.assert().success();
    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("Output is not valid UTF-8");

    assert!(
        stdout.contains("Register-ArgumentCompleter"),
        "Output missing Register-ArgumentCompleter for powershell"
    );
    assert!(stdout.contains("completions_demo"), "Output missing binary name for powershell");
}
