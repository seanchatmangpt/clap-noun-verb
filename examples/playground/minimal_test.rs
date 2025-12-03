//! Minimal test to isolate the macro issue

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct Output {
    message: String,
}

/// Simple test command
#[verb("test")]
fn test_command() -> Result<Output> {
    Ok(Output { message: "Hello".to_string() })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
