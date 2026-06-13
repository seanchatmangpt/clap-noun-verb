// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! How-to: Generate shell completions for a builder-constructed CLI
//!
//! This example shows how to use `.with_completions_subcommand()` to register
//! automatic completions generation subcommands for bash, zsh, fish, and powershell.

use clap_noun_verb::{noun, verb, CliBuilder, Result, VerbArgs};

fn main() -> Result<()> {
    let cli = CliBuilder::new()
        .name("completions_demo")
        .about("A demo application for completions")
        .version("1.0.0")
        .with_completions_subcommand()
        .noun(noun!(
            "services",
            "Manage services",
            [verb!("status", "Show status of services", |_args: &VerbArgs| {
                println!("Services are healthy");
                Ok(())
            }),]
        ));

    cli.run()
}
