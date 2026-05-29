use clap_noun_verb::{CliBuilder, noun, verb, VerbArgs, Arg};

#[test]
fn test_wizard_edge_cases_empty_values() {
    let cli = CliBuilder::new()
        .name("wizard-edge")
        .noun(noun!("wizard", "Wizard", [
            verb!("generate", "Generate", |args: &VerbArgs| {
                let target = args.get_many_opt_str("target");
                assert!(!target.is_empty());
                assert_eq!(target[0], "");
                Ok(())
            }, args: [
                Arg::new("target").long("target").required(true)
            ])
        ]));

    let cmd = cli.build_command();

    // Edge case: empty string value
    let res = cmd.try_get_matches_from(vec!["wizard-edge", "wizard", "generate", "--target", ""]);
    assert!(res.is_ok());
    let matches = res.unwrap();
    let sub1 = matches.subcommand_matches("wizard").unwrap();
    let sub2 = sub1.subcommand_matches("generate").unwrap();
    let args = VerbArgs::new(sub2.clone());
    let target = args.get_many_opt_str("target");
    assert_eq!(target, vec!["".to_string()]);
}

#[test]
fn test_wizard_edge_cases_default_fallback() {
    let cli = CliBuilder::new()
        .name("wizard-edge")
        .noun(noun!("wizard", "Wizard", [
            verb!("synthesize", "Synthesize", |args: &VerbArgs| {
                let depth = args.get_many_opt_str("depth");
                assert_eq!(depth, vec!["1".to_string()]);
                Ok(())
            }, args: [
                Arg::new("depth").long("depth").default_value("1")
            ])
        ]));

    let cmd = cli.build_command();

    // Edge case: depth is omitted, falls back to default "1"
    let res = cmd.try_get_matches_from(vec!["wizard-edge", "wizard", "synthesize"]);
    assert!(res.is_ok());
    let matches = res.unwrap();
    let sub1 = matches.subcommand_matches("wizard").unwrap();
    let sub2 = sub1.subcommand_matches("synthesize").unwrap();
    let args = VerbArgs::new(sub2.clone());
    let depth = args.get_many_opt_str("depth");
    assert_eq!(depth, vec!["1".to_string()]);
}
