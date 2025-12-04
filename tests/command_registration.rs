//! Critical Registration Tests
//!
//! Tests that verify the command registration fix is working

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct TestOutput {
    message: String,
}

/// Test command 1
#[verb("test1", "testcli")]
fn test_command_1() -> Result<TestOutput> {
    Ok(TestOutput { message: "Test 1".to_string() })
}

/// Test command 2
#[verb("test2", "testcli")]
fn test_command_2() -> Result<TestOutput> {
    Ok(TestOutput { message: "Test 2".to_string() })
}

#[test]
fn test_distributed_slice_populated() {
    // This test verifies that the linkme distributed_slice actually contains our registration functions
    use clap_noun_verb::cli::registry::{__NOUN_REGISTRY, __VERB_REGISTRY};

    println!("VERB_REGISTRY length: {}", __VERB_REGISTRY.len());
    println!("NOUN_REGISTRY length: {}", __NOUN_REGISTRY.len());

    // After our two #[verb] annotations, we should have at least 2 functions
    assert!(
        __VERB_REGISTRY.len() >= 2,
        "Expected at least 2 verb registration functions, found {}",
        __VERB_REGISTRY.len()
    );
}

#[test]
fn test_commands_registered() {
    // Initialize registry and run registration functions
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let reg = registry.lock().unwrap();

    // Check that commands were registered
    let nouns = reg.get_nouns();
    println!("Registered nouns: {:?}", nouns);

    let noun_names: Vec<&str> = nouns.iter().map(|(name, _)| *name).collect();
    assert!(
        noun_names.contains(&"testcli"),
        "Expected 'testcli' noun to be registered. Found nouns: {:?}",
        noun_names
    );

    // Check that verbs were registered
    let verbs = reg.get_verbs("testcli");
    println!("Registered verbs for 'testcli': {:?}", verbs);

    let verb_names: Vec<&str> = verbs.iter().map(|(name, _)| *name).collect();
    assert!(
        verb_names.contains(&"test1"),
        "Expected 'test1' verb to be registered. Found verbs: {:?}",
        verb_names
    );
    assert!(
        verb_names.contains(&"test2"),
        "Expected 'test2' verb to be registered. Found verbs: {:?}",
        verb_names
    );
}
