// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Probe: bare `Vec<T>` #[verb] parameter, no `#[arg(action = "append")]` override.
//!
//! Regression probe for FMEA finding RPN=648 (Severity=9 x Occurrence=8 x
//! Detection=9): a bare `Vec<T>` #[verb] parameter -- the default way to declare
//! a repeatable flag, with no explicit `#[arg(action = "append")]` override --
//! silently dropped every occurrence but the first when the flag was repeated.
//!
//! `examples/ggen/template_commands.rs`'s `template_render(template: String,
//! vars: Vec<String>)` is the real, live example that carried this exact bug
//! shape. This probe isolates the same shape (a bare, unadorned `Vec<String>`
//! parameter) into a minimal, directly-runnable binary so the bug -- and the
//! fix -- can be demonstrated end-to-end through the real macro + registry
//! pipeline, not just unit-tested in isolation.
//!
//! ```bash
//! cargo run --quiet --example vec_multi_value_probe -- probe tags --tags a --tags b --tags c
//! ```

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct TagsOutput {
    tags: Vec<String>,
    count: usize,
}

/// Probe repeated-flag extraction for a bare Vec<T> parameter
///
/// # Arguments
/// * `tags` - Repeatable --tags flag (no #[arg(action = "append")] override --
///   this is the default, most common way to declare a Vec<T> #[verb] param)
#[verb("tags", "probe")]
fn probe_tags(tags: Vec<String>) -> Result<TagsOutput> {
    let count = tags.len();
    Ok(TagsOutput { tags, count })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
