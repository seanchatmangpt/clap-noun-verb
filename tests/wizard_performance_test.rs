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
        let matches = cmd.clone().try_get_matches_from(args).expect("valid invocation must parse");

        // Witness the full noun-verb routing chain and the parsed argument value.
        let (noun_name, wizard_m) =
            matches.subcommand().expect("top-level must route to the wizard noun");
        assert_eq!(noun_name, "wizard");
        let (verb_name, generate_m) =
            wizard_m.subcommand().expect("wizard noun must route to the generate verb");
        assert_eq!(verb_name, "generate");
        let target =
            generate_m.get_one::<String>("target").expect("required --target must be captured");
        assert_eq!(target, "item");
    }
    let duration = start.elapsed();

    // Verify parser performs well (1000 parses should be well under the threshold, adjusted for CPU throttling in virtual environments)
    println!("Parsed 1000 invocations in {:?}", duration);
    let threshold = if cfg!(debug_assertions) { 1500 } else { 500 };
    assert!(duration.as_millis() < threshold, "Parsing overhead too high: {:?}", duration);
}
