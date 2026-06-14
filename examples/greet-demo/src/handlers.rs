// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// EXAMPLE-SCOPED HANDLERS — NOT PACK API.
//
// This file is the consumer side of the delegation seam for the greet-demo
// example only. The clap-noun-verb pack is authoritative for the verb INTERFACE
// (rendered into src/verbs/*.rs by `ggen sync`); it never ships handler logic.
// These trivial bodies exist solely to prove dispatch end-to-end: each handler
// prints a deterministic, verb-distinguishing line derived from its verb name
// (the held R6 convention: `tool <verb>` emits uppercase(<verb>)), so that a
// known input proves the rendered wrapper routed to ITS handler and no other.
//
// A real consumer replaces these with domain logic; the signatures are fixed by
// the rendered wrappers, which call `crate::handlers::<verb>(..)`.

use clap_noun_verb::Result;

/// greet handler — distinguishing output: "GREET".
pub fn greet(name: Option<String>) -> Result<()> {
    let who = name.unwrap_or_else(|| "world".to_string());
    println!("GREET hello {who}");
    Ok(())
}

/// convert handler — distinguishing output: "CONVERT".
/// `r#type` is the Rust raw-identifier for the CLI flag `--type` (a keyword);
/// `dry_run` is the snake-cased form of the kebab flag `--dry-run`.
pub fn convert(dry_run: Option<bool>, r#type: String) -> Result<()> {
    let dry = dry_run.unwrap_or(false);
    println!("CONVERT to={} dry_run={dry}", r#type);
    Ok(())
}
