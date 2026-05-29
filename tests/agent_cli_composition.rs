use clap_noun_verb::{CliBuilder, noun, verb, VerbArgs};

#[test]
fn test_cli_composition_basic() {
    let cli = CliBuilder::new()
        .name("composed-app")
        .about("A composed application")
        .noun(noun!("auth", "Authentication commands", [
            verb!("login", "Login user", |_args: &VerbArgs| { Ok(()) }),
            verb!("logout", "Logout user", |_args: &VerbArgs| { Ok(()) }),
        ]))
        .noun(noun!("config", "Configuration commands", [
            verb!("get", "Get config value", |_args: &VerbArgs| { Ok(()) }),
            verb!("set", "Set config value", |_args: &VerbArgs| { Ok(()) }),
        ]));

    let structure = cli.command_structure();
    assert_eq!(structure.len(), 2);
    
    let auth_verbs = structure.get("auth").unwrap();
    assert_eq!(auth_verbs.len(), 2);
    assert!(auth_verbs.contains(&"login".to_string()));
    assert!(auth_verbs.contains(&"logout".to_string()));

    let config_verbs = structure.get("config").unwrap();
    assert_eq!(config_verbs.len(), 2);
    assert!(config_verbs.contains(&"get".to_string()));
    assert!(config_verbs.contains(&"set".to_string()));
}

#[test]
fn test_cli_composition_nested_nouns() {
    // Composing deep hierarchies: nouns that have sub-nouns
    use clap_noun_verb::{NounCommand, VerbCommand};

    struct SubNoun;
    impl NounCommand for SubNoun {
        fn name(&self) -> &'static str { "profile" }
        fn about(&self) -> &'static str { "User profile settings" }
        fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
            vec![]
        }
    }

    struct RootNoun;
    impl NounCommand for RootNoun {
        fn name(&self) -> &'static str { "user" }
        fn about(&self) -> &'static str { "User management" }
        fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
            vec![]
        }
        fn sub_nouns(&self) -> Vec<Box<dyn NounCommand>> {
            vec![Box::new(SubNoun)]
        }
    }

    let cli = CliBuilder::new()
        .name("nested-app")
        .noun(RootNoun);

    let cmd = cli.build_command();
    let user_cmd = cmd.get_subcommands().find(|s| s.get_name() == "user").unwrap();
    let profile_cmd = user_cmd.get_subcommands().find(|s| s.get_name() == "profile").unwrap();
    
    assert_eq!(profile_cmd.get_name(), "profile");
}
