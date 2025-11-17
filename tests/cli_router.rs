//! Tests for CLI router

use clap::Command;
use clap_noun_verb::cli::router::CommandRouter;
use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb::noun::NounCommand;
use clap_noun_verb::verb::{VerbArgs, VerbCommand, VerbContext};

#[test]
fn test_router_new() {
    // Arrange - Create a new router
    let router = CommandRouter::new();

    // Assert - Router should exist (construction succeeds without panic)
    // The router is a working instance that can register nouns
    drop(router); // Explicitly use the router to verify no warnings
}

#[test]
fn test_router_default() {
    // Arrange - Create router using Default trait
    let router = CommandRouter::default();

    // Assert - Default implementation should work (no panic)
    drop(router); // Explicitly use the router
}

// Helper struct for testing
struct TestNoun {
    name: &'static str,
    about: &'static str,
}

impl NounCommand for TestNoun {
    fn name(&self) -> &'static str {
        self.name
    }

    fn about(&self) -> &'static str {
        self.about
    }

    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(TestVerb)]
    }

    fn sub_nouns(&self) -> Vec<Box<dyn NounCommand>> {
        Vec::new()
    }

    fn build_command(&self) -> Command {
        let mut cmd = Command::new(self.name).about(self.about);
        for verb in self.verbs() {
            cmd = cmd.subcommand(verb.build_command());
        }
        cmd
    }
}

struct TestVerb;

impl VerbCommand for TestVerb {
    fn name(&self) -> &'static str {
        "status"
    }

    fn about(&self) -> &'static str {
        "Show status"
    }

    fn run(&self, _args: &VerbArgs) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_router_register_noun() {
    // Arrange - Create router and noun
    let mut router = CommandRouter::new();
    let noun = Box::new(TestNoun { name: "services", about: "Manage services" });

    // Act - Register noun (no panic means successful registration)
    router.register_noun(noun);

    // Assert - Verify router is still usable after registration
    drop(router);
}

#[test]
fn test_router_route_noun_not_found() -> Result<()> {
    // Arrange - Create router with one noun
    let mut router = CommandRouter::new();
    let noun = Box::new(TestNoun { name: "services", about: "Manage services" });
    router.register_noun(noun);

    // Create command with subcommands - include the non-existent one
    let cmd = Command::new("test")
        .subcommand(Command::new("services"))
        .subcommand(Command::new("nonexistent"));

    // Act - Try to route to non-existent noun
    // Assert - Should return error
    let matches = cmd
        .try_get_matches_from(vec!["test", "nonexistent"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result = router.route(&matches);

    // Router should return error because noun "nonexistent" is not registered
    assert!(result.is_err());

    Ok(())
}
