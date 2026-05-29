// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

// Test without calling the function

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;

#[verb("test")]
fn handle_option(value: Option<String>) -> Result<String> {
    Ok(format!("Value: {:?}", value))
}

// NO TEST - just checking if macro-generated code compiles
