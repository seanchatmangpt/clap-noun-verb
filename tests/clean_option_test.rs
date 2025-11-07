// Clean test with macro to verify fix

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;

#[verb("test")]
fn handle_option(value: Option<String>) -> Result<String> {
    Ok(format!("Value: {:?}", value))
}

#[test]
fn test_option_parameter() {
    let result1 = handle_option(Some("hello".to_string()));
    assert!(result1.is_ok());

    let result2 = handle_option(None);
    assert!(result2.is_ok());
}
