// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

// Manual wrapper test to verify the logic

use clap_noun_verb::error::Result;
use clap_noun_verb::logic::{HandlerInput, HandlerOutput};

fn test_opt(opt: Option<String>) -> Result<()> {
    println!("Got: {:?}", opt);
    Ok(())
}

fn test_opt_wrapper(input: HandlerInput) -> Result<HandlerOutput> {
    let opt = input.args.get("opt").cloned();
    test_opt(opt)?;
    HandlerOutput::from_data(())
}

#[test]
fn it_compiles() {
    // Manual wrapper compiles if the logic is correct
    let mut args = std::collections::HashMap::new();
    args.insert("opt".to_string(), "test".to_string());

    let input = HandlerInput {
        args,
        opts: std::collections::HashMap::new(),
        context: clap_noun_verb::logic::HandlerContext {
            noun: Some("test".to_string()),
            verb: "test".to_string(),
            data: std::collections::HashMap::new(),
        },
    };

    let _result = test_opt_wrapper(input);
}
