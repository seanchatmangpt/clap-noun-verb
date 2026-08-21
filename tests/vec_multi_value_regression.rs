// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Regression coverage for the Design FMEA finding (RPN 648, severity 9 /
//! occurrence 8 / detection 9): a bare `Vec<T>` #[verb] parameter -- with no
//! `#[arg(action = "append")]` override, the default and most common way to
//! declare a repeatable flag -- silently dropped every occurrence but the
//! first when the flag was repeated.
//!
//! Root cause: `generate_verb_registration`
//! (`clap-noun-verb-macros/src/lib.rs`) never inferred `ArgAction::Append`
//! for Vec-typed params, so the generated `ArgMetadata.action` stayed `None`.
//! `build_argument` (`src/cli/registry.rs`) built the *real* `clap::Arg` with
//! `ArgAction::Append` anyway, from the separate `ArgMetadata.multiple` field
//! -- so clap itself parsed `--tags a --tags b --tags c` correctly -- but
//! `extract_args` (same file) branched on `ArgMetadata.action`, not
//! `multiple`, so its `ArgAction::Append` extraction arm was unreachable and
//! it silently fell through to single-value extraction, keeping only the
//! first occurrence.
//!
//! This mirrors the exact bug shape that shipped in the real, live
//! `examples/ggen/template_commands.rs::template_render(template: String,
//! vars: Vec<String>)` -- a bare, unadorned `Vec<String>` parameter with no
//! attribute override.
//!
//! This test drives the *entire* real pipeline -- the real `#[verb]` macro
//! expansion, the real `CommandRegistry`, real `clap` argument parsing, and
//! the real `extract_args` dispatch -- via `execute_single_step`, exactly as
//! a CLI invocation would. No mocks: this is the same singleton registry and
//! dispatch path `clap_noun_verb::run()` uses.

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
struct TagsOutput {
    tags: Vec<String>,
    count: usize,
}

/// Probe repeated-flag extraction for a bare Vec<T> parameter
///
/// # Arguments
/// * `tags` - Repeatable --tags flag (no #[arg(action = "append")] override)
#[verb("tags", "probe")]
fn probe_tags(tags: Vec<String>) -> Result<TagsOutput> {
    let count = tags.len();
    Ok(TagsOutput { tags, count })
}

/// Registration-level check: the bare `Vec<T>` param must build a real
/// `clap::Arg` wired for `Append` (multiple occurrences), matching what
/// `build_argument` already did before this fix -- this was never the
/// broken half, but pins it so a future change can't regress it silently.
#[test]
fn test_vec_param_registers_as_multi_value_arg() {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    let cmd = registry.build_command();

    let probe_cmd = cmd.get_subcommands().find(|s| s.get_name() == "probe");
    assert!(probe_cmd.is_some(), "probe noun should be registered");

    let tags_cmd = probe_cmd.unwrap().get_subcommands().find(|s| s.get_name() == "tags");
    assert!(tags_cmd.is_some(), "tags verb should be registered");

    let tags_cmd = tags_cmd.unwrap();
    let tags_arg = tags_cmd
        .get_arguments()
        .find(|a| a.get_id().as_str() == "tags")
        .expect("tags argument should exist");

    assert!(
        matches!(tags_arg.get_action(), clap::ArgAction::Append),
        "a bare Vec<T> param must build a real clap::Arg with ArgAction::Append \
         so clap itself accepts repeated --tags occurrences, got {:?}",
        tags_arg.get_action()
    );
}

/// The actual FMEA repro: `probe tags --tags a --tags b --tags c` must
/// extract all 3 occurrences end-to-end, not silently keep only the first.
///
/// Before the fix: `extract_args` fell through to single-value extraction
/// (`ArgMetadata.action == None`, despite `ArgMetadata.multiple == true`),
/// so `tags == ["a"]` and `count == 1`.
///
/// After the fix: the macro infers `ArgAction::Append` for the bare Vec<T>
/// param, `ArgMetadata.action` and `ArgMetadata.multiple` agree, and
/// `extract_args`'s existing `ArgAction::Append` arm is reached, so
/// `tags == ["a", "b", "c"]` and `count == 3`.
#[test]
fn test_vec_param_extracts_all_repeated_occurrences() -> Result<()> {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());

    let output = registry.execute_single_step(vec![
        "clap-noun-verb".to_string(),
        "probe".to_string(),
        "tags".to_string(),
        "--tags".to_string(),
        "a".to_string(),
        "--tags".to_string(),
        "b".to_string(),
        "--tags".to_string(),
        "c".to_string(),
    ])?;

    assert_eq!(
        output.data["count"], 3,
        "all 3 repeated --tags occurrences must be extracted, not just the first: {:?}",
        output.data
    );
    assert_eq!(
        output.data["tags"],
        serde_json::json!(["a", "b", "c"]),
        "extracted tags must preserve every occurrence in order: {:?}",
        output.data
    );

    Ok(())
}

/// A single occurrence of a Vec<T> flag must still work (regression guard:
/// the fix must not turn a single value into anything unexpected).
#[test]
fn test_vec_param_extracts_single_occurrence() -> Result<()> {
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap_or_else(|e| e.into_inner());

    let output = registry.execute_single_step(vec![
        "clap-noun-verb".to_string(),
        "probe".to_string(),
        "tags".to_string(),
        "--tags".to_string(),
        "only-one".to_string(),
    ])?;

    assert_eq!(output.data["count"], 1);
    assert_eq!(output.data["tags"], serde_json::json!(["only-one"]));

    Ok(())
}
