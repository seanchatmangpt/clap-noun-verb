//! JTBD End-to-End Execution Tests
//!
//! These tests actually execute CLI commands and verify argument extraction
//! works correctly through the entire pipeline.
//!
//! Focus areas:
//! - Numeric argument extraction (previously panicked on type mismatch)
//! - Boolean flag extraction
//! - Optional argument fallback
//! - Count action arguments

use clap::Arg;
use clap_noun_verb::error::Result;
use clap_noun_verb::{noun, verb};
use clap_noun_verb::{Cli, VerbArgs};

// =============================================================================
// Test 1: Numeric Argument Extraction (Critical - Previously Panicked)
// =============================================================================

#[test]
fn test_numeric_args_extraction_port() -> Result<()> {
    // Arrange: CLI with numeric port argument
    let cli = Cli::new().name("numeric-test").noun(noun!(
        "server",
        "Server commands",
        [verb!("start", "Start server", |args: &VerbArgs| {
            let port = args.get_one_str_opt("port").unwrap_or_else(|| "8080".to_string());
            assert_eq!(port, "9000", "Port should be extracted as '9000'");
            Ok(())
        }, args: [
            Arg::new("port")
                .long("port")
                .short('p')
                .value_name("PORT")
                .default_value("8080")
                .value_parser(clap::value_parser!(u16)),
        ]),]
    ));

    // Act: Run with --port 9000
    cli.run_with_args(vec![
        "numeric-test".to_string(),
        "server".to_string(),
        "start".to_string(),
        "--port".to_string(),
        "9000".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_numeric_args_extraction_timeout() -> Result<()> {
    // Arrange: CLI with u64 timeout argument
    let cli = Cli::new().name("timeout-test").noun(noun!(
        "request",
        "Request commands",
        [verb!("send", "Send request", |args: &VerbArgs| {
            let timeout = args.get_one_str_opt("timeout").unwrap_or_else(|| "30".to_string());
            assert_eq!(timeout, "60", "Timeout should be extracted as '60'");
            Ok(())
        }, args: [
            Arg::new("timeout")
                .long("timeout")
                .short('t')
                .value_name("SECONDS")
                .default_value("30")
                .value_parser(clap::value_parser!(u64)),
        ]),]
    ));

    // Act: Run with --timeout 60
    cli.run_with_args(vec![
        "timeout-test".to_string(),
        "request".to_string(),
        "send".to_string(),
        "--timeout".to_string(),
        "60".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_numeric_args_extraction_workers() -> Result<()> {
    // Arrange: CLI with usize workers argument
    let cli = Cli::new().name("workers-test").noun(noun!(
        "pool",
        "Worker pool commands",
        [verb!("scale", "Scale workers", |args: &VerbArgs| {
            let workers = args.get_one_str_opt("workers").unwrap_or_else(|| "4".to_string());
            assert_eq!(workers, "16", "Workers should be extracted as '16'");
            Ok(())
        }, args: [
            Arg::new("workers")
                .long("workers")
                .short('w')
                .value_name("COUNT")
                .default_value("4")
                .value_parser(clap::value_parser!(usize)),
        ]),]
    ));

    // Act: Run with --workers 16
    cli.run_with_args(vec![
        "workers-test".to_string(),
        "pool".to_string(),
        "scale".to_string(),
        "--workers".to_string(),
        "16".to_string(),
    ])?;

    Ok(())
}

// =============================================================================
// Test 2: Boolean Flag Extraction
// =============================================================================

#[test]
fn test_boolean_flag_extraction_verbose() -> Result<()> {
    // Arrange: CLI with boolean verbose flag
    let cli = Cli::new().name("flag-test").noun(noun!(
        "task",
        "Task commands",
        [verb!("run", "Run task", |args: &VerbArgs| {
            let verbose = args.is_flag_set("verbose");
            assert!(verbose, "Verbose flag should be set");
            Ok(())
        }, args: [
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue),
        ]),]
    ));

    // Act: Run with --verbose
    cli.run_with_args(vec![
        "flag-test".to_string(),
        "task".to_string(),
        "run".to_string(),
        "--verbose".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_boolean_flag_extraction_force() -> Result<()> {
    // Arrange: CLI with boolean force flag
    let cli = Cli::new().name("force-test").noun(noun!(
        "deploy",
        "Deploy commands",
        [verb!("push", "Push deployment", |args: &VerbArgs| {
            let force = args.is_flag_set("force");
            assert!(force, "Force flag should be set");
            Ok(())
        }, args: [
            Arg::new("force")
                .long("force")
                .short('f')
                .action(clap::ArgAction::SetTrue),
        ]),]
    ));

    // Act: Run with -f (short form)
    cli.run_with_args(vec![
        "force-test".to_string(),
        "deploy".to_string(),
        "push".to_string(),
        "-f".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_boolean_flag_not_set() -> Result<()> {
    // Arrange: CLI with boolean flag not provided
    let cli = Cli::new().name("no-flag-test").noun(noun!(
        "action",
        "Action commands",
        [verb!("do", "Do action", |args: &VerbArgs| {
            let verbose = args.is_flag_set("verbose");
            assert!(!verbose, "Verbose flag should NOT be set");
            Ok(())
        }, args: [
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue),
        ]),]
    ));

    // Act: Run WITHOUT --verbose
    cli.run_with_args(vec!["no-flag-test".to_string(), "action".to_string(), "do".to_string()])?;

    Ok(())
}

// =============================================================================
// Test 3: Optional Argument Fallback
// =============================================================================

#[test]
fn test_optional_arg_with_default() -> Result<()> {
    // Arrange: CLI with optional argument using default
    let cli = Cli::new().name("default-test").noun(noun!(
        "config",
        "Config commands",
        [verb!("show", "Show config", |args: &VerbArgs| {
            let format = args.get_one_str_opt("format").unwrap_or_else(|| "json".to_string());
            assert_eq!(format, "json", "Default format should be 'json'");
            Ok(())
        }, args: [
            Arg::new("format")
                .long("format")
                .default_value("json"),
        ]),]
    ));

    // Act: Run WITHOUT --format (use default)
    cli.run_with_args(vec!["default-test".to_string(), "config".to_string(), "show".to_string()])?;

    Ok(())
}

#[test]
fn test_optional_arg_with_override() -> Result<()> {
    // Arrange: CLI with optional argument being overridden
    let cli = Cli::new().name("override-test").noun(noun!(
        "output",
        "Output commands",
        [verb!("format", "Format output", |args: &VerbArgs| {
            let format = args.get_one_str_opt("format").unwrap_or_else(|| "json".to_string());
            assert_eq!(format, "yaml", "Format should be overridden to 'yaml'");
            Ok(())
        }, args: [
            Arg::new("format")
                .long("format")
                .default_value("json"),
        ]),]
    ));

    // Act: Run with --format yaml (override default)
    cli.run_with_args(vec![
        "override-test".to_string(),
        "output".to_string(),
        "format".to_string(),
        "--format".to_string(),
        "yaml".to_string(),
    ])?;

    Ok(())
}

// =============================================================================
// Test 4: Count Action Arguments (-vvv verbosity)
// =============================================================================

#[test]
fn test_count_action_single() -> Result<()> {
    // Arrange: CLI with count action for verbosity
    let cli = Cli::new().name("count-single-test").noun(noun!(
        "log",
        "Log commands",
        [verb!("show", "Show logs", |args: &VerbArgs| {
            let verbosity = args.get_flag_count("verbose");
            assert_eq!(verbosity, 1, "Verbosity should be 1 for single -v");
            Ok(())
        }, args: [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        ]),]
    ));

    // Act: Run with -v (single)
    cli.run_with_args(vec![
        "count-single-test".to_string(),
        "log".to_string(),
        "show".to_string(),
        "-v".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_count_action_double() -> Result<()> {
    // Arrange: CLI with count action for verbosity
    let cli = Cli::new().name("count-double-test").noun(noun!(
        "trace",
        "Trace commands",
        [verb!("dump", "Dump trace", |args: &VerbArgs| {
            let verbosity = args.get_flag_count("verbose");
            assert_eq!(verbosity, 2, "Verbosity should be 2 for -vv");
            Ok(())
        }, args: [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        ]),]
    ));

    // Act: Run with -vv (double)
    cli.run_with_args(vec![
        "count-double-test".to_string(),
        "trace".to_string(),
        "dump".to_string(),
        "-vv".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_count_action_triple() -> Result<()> {
    // Arrange: CLI with count action for verbosity
    let cli = Cli::new().name("count-triple-test").noun(noun!(
        "debug",
        "Debug commands",
        [verb!("analyze", "Analyze debug info", |args: &VerbArgs| {
            let verbosity = args.get_flag_count("verbose");
            assert_eq!(verbosity, 3, "Verbosity should be 3 for -vvv");
            Ok(())
        }, args: [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        ]),]
    ));

    // Act: Run with -vvv (triple)
    cli.run_with_args(vec![
        "count-triple-test".to_string(),
        "debug".to_string(),
        "analyze".to_string(),
        "-vvv".to_string(),
    ])?;

    Ok(())
}

#[test]
fn test_count_action_zero() -> Result<()> {
    // Arrange: CLI with count action not provided
    let cli = Cli::new().name("count-zero-test").noun(noun!(
        "quiet",
        "Quiet commands",
        [verb!("run", "Run quietly", |args: &VerbArgs| {
            let verbosity = args.get_flag_count("verbose");
            assert_eq!(verbosity, 0, "Verbosity should be 0 when not provided");
            Ok(())
        }, args: [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        ]),]
    ));

    // Act: Run WITHOUT -v
    cli.run_with_args(vec!["count-zero-test".to_string(), "quiet".to_string(), "run".to_string()])?;

    Ok(())
}

// =============================================================================
// Test 5: Mixed Numeric and String Arguments
// =============================================================================

#[test]
fn test_mixed_numeric_string_args() -> Result<()> {
    // Arrange: CLI with both numeric and string arguments
    let cli = Cli::new()
        .name("mixed-test")
        .noun(noun!(
            "connection",
            "Connection commands",
            [verb!("open", "Open connection", |args: &VerbArgs| {
                let host = args.get_one_str_opt("host").unwrap_or_else(|| "localhost".to_string());
                let port = args.get_one_str_opt("port").unwrap_or_else(|| "8080".to_string());

                assert_eq!(host, "example.com", "Host should be 'example.com'");
                assert_eq!(port, "443", "Port should be '443'");
                Ok(())
            }, args: [
Arg::new("host")
                    .long("host")
                    .default_value("localhost"),
                Arg::new("port")
                    .long("port")
                    .short('p')
                    .default_value("8080")
                    .value_parser(clap::value_parser!(u16)),
            ]),]
        ));

    // Act: Run with both --host and --port
    cli.run_with_args(vec![
        "mixed-test".to_string(),
        "connection".to_string(),
        "open".to_string(),
        "--host".to_string(),
        "example.com".to_string(),
        "--port".to_string(),
        "443".to_string(),
    ])?;

    Ok(())
}

// =============================================================================
// Test 6: I64 Negative Numbers
// =============================================================================

#[test]
fn test_negative_numeric_arg() -> Result<()> {
    // Arrange: CLI with i64 argument that can be negative
    let cli = Cli::new().name("negative-test").noun(noun!(
        "offset",
        "Offset commands",
        [verb!("set", "Set offset", |args: &VerbArgs| {
            let offset = args.get_one_str_opt("value").unwrap_or_else(|| "0".to_string());
            assert_eq!(offset, "-100", "Offset should be '-100'");
            Ok(())
        }, args: [
            Arg::new("value")
                .long("value")
                .allow_negative_numbers(true)
                .value_parser(clap::value_parser!(i64)),
        ]),]
    ));

    // Act: Run with negative value
    cli.run_with_args(vec![
        "negative-test".to_string(),
        "offset".to_string(),
        "set".to_string(),
        "--value".to_string(),
        "-100".to_string(),
    ])?;

    Ok(())
}

// =============================================================================
// Test 7: Float Arguments
// =============================================================================

#[test]
fn test_float_numeric_arg() -> Result<()> {
    // Arrange: CLI with f64 argument
    let cli = Cli::new().name("float-test").noun(noun!(
        "scale",
        "Scale commands",
        [verb!("apply", "Apply scale", |args: &VerbArgs| {
            let factor = args.get_one_str_opt("factor").unwrap_or_else(|| "1.0".to_string());
            assert_eq!(factor, "2.5", "Factor should be '2.5'");
            Ok(())
        }, args: [
            Arg::new("factor")
                .long("factor")
                .default_value("1.0")
                .value_parser(clap::value_parser!(f64)),
        ]),]
    ));

    // Act: Run with float value
    cli.run_with_args(vec![
        "float-test".to_string(),
        "scale".to_string(),
        "apply".to_string(),
        "--factor".to_string(),
        "2.5".to_string(),
    ])?;

    Ok(())
}
