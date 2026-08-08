// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # VerbArgs — Argument Access Example
//!
//! Demonstrates the `VerbArgs` API: the bridge between parsed CLI arguments
//! and domain handler logic.
//!
//! Capabilities witnessed:
//! - `get_one_str` — required string argument, errors on missing
//! - `get_one_str_opt` — optional string argument
//! - `get_one::<T>` — typed argument (bool flag)
//! - `trailing()` — ordered trailing positionals (e.g. `explain ISSUE-001 ISSUE-002`)
//! - `verb()` / `noun()` — context metadata on the dispatch
//!
//! **Expected output** (run with `cargo run --example verb_args`):
//! ```text
//! deploy --service api --verbose: service=api verbose=true
//! explain ISSUE-001 ISSUE-002: trailing=["ISSUE-001", "ISSUE-002"]
//! verb()=status noun()=Some("services")
//! ```
//!
//! **Doc**: docs/reference/api/verb-macro.md, docs/reference/api/arg-attributes.md
//! **Reference**: docs/reference/api-catalog.md

use clap::{Arg, ArgAction, Command};
use clap_noun_verb::{noun, run_cli_with_args, verb, Result, VerbArgs, VerbContext};

fn main() -> Result<()> {
    // --- Witness 1: get_one_str (required) and get_one::<bool> (flag) ---
    run_cli_with_args(
        vec![
            "myapp".into(),
            "deploy".into(),
            "start".into(),
            "--service".into(),
            "api".into(),
            "--verbose".into(),
        ],
        |builder| {
            builder
                .name("myapp")
                .about("VerbArgs demo")
                .noun(noun!("deploy", "Deployment commands", [
                    verb!(
                        "start",
                        "Start a deployment",
                        |args: &VerbArgs| {
                            let service = args.get_one_str("service")?;
                            let verbose: bool = args
                                .get_one::<bool>("verbose")
                                .unwrap_or(false);
                            assert_eq!(service, "api", "get_one_str must return the --service value");
                            assert!(verbose, "get_one::<bool> must return true when flag is set");
                            println!("deploy --service api --verbose: service={service} verbose={verbose}");
                            Ok(())
                        },
                        args: [
                            Arg::new("service").long("service").required(true),
                            Arg::new("verbose").long("verbose").action(ArgAction::SetTrue),
                        ]
                    ),
                ]))
        },
    )?;

    // --- Witness 2: trailing() — positional var-args after the verb ---
    run_cli_with_args(
        vec![
            "myapp".into(),
            "explain".into(),
            "issue".into(),
            "ISSUE-001".into(),
            "ISSUE-002".into(),
        ],
        |builder| {
            builder.name("myapp").about("VerbArgs trailing demo").noun(noun!(
                "explain",
                "Explain commands",
                [verb!(
                    "issue",
                    "Explain one or more issues",
                    |args: &VerbArgs| {
                        let trailing = args.trailing();
                        assert_eq!(
                            trailing,
                            vec!["ISSUE-001".to_string(), "ISSUE-002".to_string()],
                            "trailing() must return positionals in order"
                        );
                        println!("explain ISSUE-001 ISSUE-002: trailing={trailing:?}");
                        Ok(())
                    },
                    args: [
                        Arg::new("trailing").num_args(0..).trailing_var_arg(true),
                    ]
                ),]
            ))
        },
    )?;

    // --- Witness 3: verb() / noun() context from VerbArgs ---
    {
        let context = VerbContext::new("status").with_noun("services");
        let matches = Command::new("test").get_matches_from(["test"]);
        let args = VerbArgs::new(matches).with_context(context);
        assert_eq!(args.verb(), "status", "verb() must return context verb name");
        assert_eq!(args.noun(), Some("services"), "noun() must return context noun name");
        println!("verb()=status noun()=Some(\"services\")");
    }

    Ok(())
}
