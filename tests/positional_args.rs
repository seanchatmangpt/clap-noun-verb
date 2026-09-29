// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Tests for positional argument support
//!
//! These tests verify that arguments with `#[arg(index = N)]` attributes
//! are correctly parsed and applied as positional arguments.
//!
//! Regression coverage for the Design FMEA finding (RPN 810, severity 9 /
//! occurrence 9 / detection 10): the crate documents and models `index` as
//! 0-based (`#[arg(index = 0)]` for the first positional -- see
//! `examples/tutorial/positional.rs`), but the value used to be passed
//! straight through, unconverted, to clap's `Arg::index()`, which is
//! 1-based. With >=2 positional args (`index = 0`, `index = 1`) that made
//! clap register two arguments at its own index 1, which clap's
//! `debug_asserts` rejects with a real runtime panic:
//! "Found positional argument whose index is 1 but there are only 2
//! positional arguments defined". The prior version of this test file used
//! `clone_repository(url: String, destination: Option<String>)` with *zero*
//! `#[arg]` attributes at all, so it built an all-named-argument command and
//! could not have caught the bug despite the file's docstring promising
//! `#[arg(index = N)]` coverage. This file now actually attaches
//! `#[arg(index = N)]` and drives the command through real CLI arg parsing.

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
struct Repo {
    url: String,
    destination: Option<String>,
}

fn clone_repo(url: String, destination: Option<String>) -> Repo {
    Repo { url, destination }
}

/// Clone a repository
///
/// This mirrors `examples/tutorial/positional.rs::clone_repo` exactly: the
/// crate's own canonical, documented usage of `#[arg(index = N)]` for two
/// positional arguments.
#[verb("clone", "git")]
fn clone_repository(
    #[arg(index = 0)] url: String,
    #[arg(index = 1)] destination: Option<String>,
) -> Result<Repo> {
    Ok(clone_repo(url, destination))
}

#[test]
fn test_positional_args_registered() -> Result<()> {
    // Test: Arguments can be registered as positional

    // Arrange: clone_repository has arguments
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    // Act: Find git -> clone command
    let git_cmd = cmd.get_subcommands().find(|s| s.get_name() == "git");
    assert!(git_cmd.is_some(), "git noun should be registered");

    let clone_cmd = git_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "clone");
    assert!(clone_cmd.is_some(), "clone verb should be registered");

    // Assert: Arguments should exist
    let clone_cmd = clone_cmd.unwrap();
    let args: Vec<_> = clone_cmd.get_arguments().collect();

    let url_arg = args.iter().find(|a| a.get_id().as_str() == "url");
    let dest_arg = args.iter().find(|a| a.get_id().as_str() == "destination");

    assert!(url_arg.is_some(), "url argument should exist");
    assert!(dest_arg.is_some(), "destination argument should exist");

    Ok(())
}

/// Reproduces the exact panicking invocation from the FMEA report:
/// `cargo run --example tutorial_positional -- git clone
/// https://example.com/repo.git`. Building the command (which triggers
/// clap's `debug_assert`-gated positional-index verification) and getting
/// matches back must not panic, and the parsed values must actually reflect
/// the crate's 0-based `#[arg(index = N)]` convention: `index = 0` (url) is
/// the *first* token after `clone`, `index = 1` (destination) is the
/// second/optional one -- not clap's own 1-based numbering applied a second
/// time on top of it.
#[test]
fn test_positional_args_use_documented_zero_based_index() -> Result<()> {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();

    // Building the command runs clap's debug_asserts::assert_app, which is
    // exactly what panicked before the fix
    // ("Found positional argument whose index is 1 but there are only 2
    // positional arguments defined").
    let cmd = registry.build_command();

    // Required positional only (index = 0), matching the FMEA's exact repro.
    let matches = cmd
        .clone()
        .try_get_matches_from([
            "clap-noun-verb",
            "git",
            "clone",
            "https://example.com/repo.git",
        ])
        .expect("parsing a single positional (index = 0) must succeed");
    let clone_matches = matches.subcommand_matches("git").unwrap().subcommand_matches("clone").unwrap();
    assert_eq!(
        clone_matches.get_one::<String>("url").map(String::as_str),
        Some("https://example.com/repo.git"),
        "index = 0 (url) must bind to the first positional token"
    );
    assert_eq!(
        clone_matches.get_one::<String>("destination"),
        None,
        "optional index = 1 (destination) must be absent when not supplied"
    );

    // Both positionals supplied: index = 0 (url) then index = 1 (destination),
    // in that documented order.
    let matches = cmd
        .clone()
        .try_get_matches_from([
            "clap-noun-verb",
            "git",
            "clone",
            "https://example.com/repo.git",
            "/path/to/dest",
        ])
        .expect("parsing two positionals (index = 0, index = 1) must succeed");
    let clone_matches = matches.subcommand_matches("git").unwrap().subcommand_matches("clone").unwrap();
    assert_eq!(
        clone_matches.get_one::<String>("url").map(String::as_str),
        Some("https://example.com/repo.git"),
        "index = 0 (url) must still bind to the first positional token"
    );
    assert_eq!(
        clone_matches.get_one::<String>("destination").map(String::as_str),
        Some("/path/to/dest"),
        "index = 1 (destination) must bind to the second positional token"
    );

    Ok(())
}
