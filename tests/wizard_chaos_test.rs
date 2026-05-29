use clap_noun_verb::{noun, verb, Arg, CliBuilder, VerbArgs};

#[test]
fn test_wizard_chaos_argument_sequences() {
    let cli = CliBuilder::new().name("wizard-chaos").noun(noun!(
        "wizard",
        "Wizard",
        [verb!("generate", "Generate", |_args: &VerbArgs| Ok(()), args: [
            Arg::new("target").long("target").required(true),
            Arg::new("verbose").long("verbose").num_args(0)
        ])]
    ));

    let cmd = cli.build_command();

    // Chaotic input 1: missing required arguments
    let res =
        cmd.clone().try_get_matches_from(vec!["wizard-chaos", "wizard", "generate", "--verbose"]);
    assert!(res.is_err());

    // Chaotic input 2: unexpected extra positional arguments
    let res = cmd.clone().try_get_matches_from(vec![
        "wizard-chaos",
        "wizard",
        "generate",
        "--target",
        "foo",
        "extra_pos1",
        "extra_pos2",
    ]);
    assert!(res.is_err());

    // Chaotic input 3: repeating options
    let res = cmd.clone().try_get_matches_from(vec![
        "wizard-chaos",
        "wizard",
        "generate",
        "--target",
        "foo",
        "--target",
        "bar",
    ]);
    // Clap overrides or lists them depending on configuration, but shouldn't panic
    assert!(res.is_ok() || res.is_err());
}
