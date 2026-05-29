use clap_noun_verb::{CliBuilder, noun, verb, VerbArgs};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_wizard_fuzz_inputs(
        ref target in "\\PC*",
        ref depth in "\\d+",
        ref option in "[a-zA-Z0-9]*"
    ) {
        let cli = CliBuilder::new()
            .name("wizard-fuzz")
            .about("Fuzz testing CLI")
            .noun(noun!("wizard", "Wizard commands", [
                verb!("generate", "Generate command", |args: &VerbArgs| {
                    let target_vals = args.get_many_opt_str("target");
                    let depth_vals = args.get_many_opt_str("depth");
                    for val in &target_vals {
                        assert!(val.len() <= 10000);
                    }
                    for val in &depth_vals {
                        assert!(val.len() <= 100);
                    }
                    Ok(())
                }),
            ]));

        let cmd = cli.build_command();

        // Test running with the fuzzed inputs
        let args = vec![
            "wizard-fuzz".to_string(),
            "wizard".to_string(),
            "generate".to_string(),
            "--target".to_string(),
            target.clone(),
            "--depth".to_string(),
            depth.clone(),
            "--option".to_string(),
            option.clone(),
        ];

        // Should parse or fail gracefully, never panic
        let _ = cmd.try_get_matches_from(args);
    }
}
