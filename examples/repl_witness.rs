// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Non-interactive witness for the feature-gated REPL surface.

use clap_noun_verb::repl::split_shell_words;
use clap_noun_verb::{CommandRegistry, Repl};
use std::path::PathBuf;

fn main() {
    let repl = Repl::new(CommandRegistry::new().name("repl-witness"))
        .with_history_file(PathBuf::from("target/repl-witness.history"));
    assert_eq!(repl.registry().build_command().get_name(), "repl-witness");

    assert_eq!(
        split_shell_words("pack install \"verified package\""),
        Some(vec!["pack".to_string(), "install".to_string(), "verified package".to_string(),])
    );
    assert!(split_shell_words("pack install \"unterminated").is_none());

    println!("REPL construction and shell-word parsing admitted");
}
