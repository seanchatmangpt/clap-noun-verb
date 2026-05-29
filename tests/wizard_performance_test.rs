// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use clap_noun_verb::{noun, verb, Arg, CliBuilder, VerbArgs};
use std::time::Instant;

#[test]
fn test_wizard_parser_overhead_performance() {
    let cli = CliBuilder::new().name("wizard-perf").noun(noun!(
        "wizard",
        "Wizard",
        [verb!("generate", "Generate", |_args: &VerbArgs| Ok(()), args: [
            Arg::new("target").long("target").required(true)
        ])]
    ));

    let cmd = cli.build_command();

    let start = Instant::now();
    for _i in 0..1000 {
        let args = vec!["wizard-perf", "wizard", "generate", "--target", "item"];
        let res = cmd.clone().try_get_matches_from(args);
        assert!(res.is_ok());
    }
    let duration = start.elapsed();

    // Verify parser performs well (1000 parses should be well under 100 milliseconds)
    println!("Parsed 1000 invocations in {:?}", duration);
    let threshold = if cfg!(debug_assertions) { 250 } else { 100 };
    assert!(duration.as_millis() < threshold, "Parsing overhead too high: {:?}", duration);
}
