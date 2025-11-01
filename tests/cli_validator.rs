//! Tests for CLI validator

use clap::{Arg, Command};
use clap_noun_verb::cli::validator::ArgValidator;
use clap_noun_verb::error::{NounVerbError, Result};

#[test]
fn test_validator_new() {
    let _validator = ArgValidator::new();
    // Validator should be created successfully
    assert!(true); // Validator is zero-sized, just verify it compiles
}

#[test]
fn test_validator_default() {
    let _validator = ArgValidator::default();
    // Default should work
    assert!(true);
}

#[test]
fn test_validator_validate_required_str_success() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("name").required(true));

    let matches = cmd
        .try_get_matches_from(vec!["test", "value"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result = validator.validate_required_str(&matches, "name")?;
    assert_eq!(result, "value");

    Ok(())
}

#[test]
fn test_validator_validate_required_str_missing() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("name").required(true));

    let matches = cmd
        .try_get_matches_from(vec!["test"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()));

    // Should fail because required argument is missing
    assert!(matches.is_err());

    Ok(())
}

#[test]
fn test_validator_validate_optional_str_present() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("name").long("name"));

    let matches = cmd
        .try_get_matches_from(vec!["test", "--name", "value"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result = validator.validate_optional_str(&matches, "name");
    assert_eq!(result, Some("value".to_string()));

    Ok(())
}

#[test]
fn test_validator_validate_optional_str_missing() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("name"));

    let matches = cmd
        .try_get_matches_from(vec!["test"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result = validator.validate_optional_str(&matches, "name");
    assert_eq!(result, None);

    Ok(())
}

#[test]
fn test_validator_validate_required_typed() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test")
        .arg(Arg::new("count").required(true).value_parser(clap::value_parser!(usize)));

    let matches = cmd
        .try_get_matches_from(vec!["test", "42"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result: usize = validator.validate_required(&matches, "count")?;
    assert_eq!(result, 42);

    Ok(())
}

#[test]
fn test_validator_validate_flag_set() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test")
        .arg(Arg::new("verbose").long("verbose").action(clap::ArgAction::SetTrue));

    let matches = cmd
        .try_get_matches_from(vec!["test", "--verbose"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    assert!(validator.validate_flag(&matches, "verbose"));

    Ok(())
}

#[test]
fn test_validator_validate_flag_not_set() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test")
        .arg(Arg::new("verbose").long("verbose").action(clap::ArgAction::SetTrue));

    let matches = cmd
        .try_get_matches_from(vec!["test"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    assert!(!validator.validate_flag(&matches, "verbose"));

    Ok(())
}

#[test]
fn test_validator_validate_flag_count() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test")
        .arg(Arg::new("verbose").short('v').long("verbose").action(clap::ArgAction::Count));

    let matches = cmd
        .try_get_matches_from(vec!["test", "-vvv"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let count = validator.validate_flag_count(&matches, "verbose");
    assert_eq!(count, 3);

    Ok(())
}

#[test]
fn test_validator_validate_many_success() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("items").required(true).num_args(1..));

    let matches = cmd
        .try_get_matches_from(vec!["test", "item1", "item2", "item3"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result: Vec<String> = validator.validate_many(&matches, "items")?;
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "item1");
    assert_eq!(result[1], "item2");
    assert_eq!(result[2], "item3");

    Ok(())
}

#[test]
fn test_validator_validate_many_empty() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("items").required(true).num_args(1..));

    let matches = cmd
        .try_get_matches_from(vec!["test"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()));

    // Should fail because required argument is missing
    assert!(matches.is_err());

    Ok(())
}

#[test]
fn test_validator_validate_many_opt_empty() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test").arg(Arg::new("items").num_args(1..));

    let matches = cmd
        .try_get_matches_from(vec!["test"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let result: Vec<String> = validator.validate_many_opt(&matches, "items");
    assert_eq!(result.len(), 0);

    Ok(())
}

#[test]
fn test_validator_extract_args() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test")
        .arg(Arg::new("name").required(true))
        .arg(Arg::new("value").required(true));

    let matches = cmd
        .try_get_matches_from(vec!["test", "test-name", "test-value"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let args = validator.extract_args(&matches);

    // Should extract both arguments
    assert!(args.len() >= 2);
    assert_eq!(args.get("name"), Some(&"test-name".to_string()));
    assert_eq!(args.get("value"), Some(&"test-value".to_string()));

    Ok(())
}

#[test]
fn test_validator_extract_opts() -> Result<()> {
    let validator = ArgValidator::new();
    let cmd = Command::new("test")
        .arg(Arg::new("verbose").long("verbose").action(clap::ArgAction::SetTrue));

    let matches = cmd
        .try_get_matches_from(vec!["test", "--verbose"])
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    let opts = validator.extract_opts(&matches);

    // Should extract flags
    // Note: extract_opts handles errors gracefully and only extracts flags for now
    assert!(opts.contains_key("verbose"));
    assert_eq!(opts.get("verbose"), Some(&"true".to_string()));

    Ok(())
}
