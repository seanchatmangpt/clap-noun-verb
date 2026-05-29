use clap_noun_verb::CommandRegistry;
use clap_noun_verb::noun::NounCommand;
use clap_noun_verb::verb::{VerbCommand, VerbArgs};
use clap_noun_verb::error::Result;
use clap::Command;

struct TestNoun {
    name: &'static str,
}

impl NounCommand for TestNoun {
    fn name(&self) -> &'static str {
        self.name
    }
    fn about(&self) -> &'static str {
        "test noun"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        if self.name == "service" {
            vec![
                Box::new(TestVerb { name: "status" }),
                Box::new(TestVerb { name: "restart" }),
            ]
        } else {
            vec![]
        }
    }
}

#[derive(Clone)]
struct TestVerb {
    name: &'static str,
}

impl VerbCommand for TestVerb {
    fn name(&self) -> &'static str {
        self.name
    }
    fn about(&self) -> &'static str {
        "test verb"
    }
    fn run(&self, _args: &VerbArgs) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_typo_check_noun_suggestions() {
    let registry = CommandRegistry::new()
        .register_noun(TestNoun { name: "service" })
        .register_noun(TestNoun { name: "database" });

    // Build a mock command structure that allows parsing the typo "servise"
    let mock_cmd = Command::new("cli")
        .subcommand(Command::new("servise"));
    
    let matches = mock_cmd.try_get_matches_from(vec!["cli", "servise"]);
    assert!(matches.is_ok());
    if let Ok(matches) = matches {
        let result = registry.route(&matches);
        
        assert!(result.is_err());
        if let Err(err) = result {
            let err_msg = err.to_string();
            println!("Err message: {}", err_msg);
            assert!(err_msg.contains("Command 'servise' not found"));
            assert!(err_msg.contains("Did you mean:"));
            // Highlighted with ANSI color
            assert!(err_msg.contains("\x1b[1m\x1b[33mservice\x1b[0m"));
        }
    }
}

#[test]
fn test_typo_check_verb_suggestions() {
    let registry = CommandRegistry::new()
        .register_noun(TestNoun { name: "service" });

    // Build a mock command structure that allows parsing the typo "statos"
    let mock_cmd = Command::new("cli")
        .subcommand(Command::new("service")
            .subcommand(Command::new("statos")));

    let matches = mock_cmd.try_get_matches_from(vec!["cli", "service", "statos"]);
    assert!(matches.is_ok());
    if let Ok(matches) = matches {
        let result = registry.route(&matches);
        
        assert!(result.is_err());
        if let Err(err) = result {
            let err_msg = err.to_string();
            println!("Err message: {}", err_msg);
            assert!(err_msg.contains("Verb 'statos' not found for noun 'service'"));
            assert!(err_msg.contains("Did you mean:"));
            assert!(err_msg.contains("\x1b[1m\x1b[33mstatus\x1b[0m"));
        }
    }
}
