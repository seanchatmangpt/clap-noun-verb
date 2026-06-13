#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use clap::{Arg, ArgAction, Command};
use clap_noun_verb_utils::display_json::{arg_matches_to_json, extract_command_schema};
use clap_noun_verb_utils::number_parsing::{
    decimal_range, maybe_hex, maybe_hex_range, parse_bytes, parse_duration, parse_percentage,
};
use serde_json::Value;
use std::time::Duration;

// =========================================================================
// 1. Number Parsing Bounds Checking (`clap-num` wrapper)
// =========================================================================

#[test]
fn test_decimal_range_adversarial() {
    // Test with unsigned u8
    let parse_u8 = decimal_range(10u8, 20u8);

    // Bounds check
    assert_eq!(parse_u8("10").unwrap(), 10);
    assert_eq!(parse_u8("20").unwrap(), 20);

    // Empty and invalid formats
    assert!(parse_u8("").is_err());
    assert!(parse_u8(" ").is_err());
    assert!(parse_u8("abc").is_err());
    assert!(parse_u8("10a").is_err());
    assert!(parse_u8("15.5").is_err());
    assert!(parse_u8("0x0A").is_err());

    // Overflow bounds
    assert!(parse_u8("21").is_err());
    assert!(parse_u8("9").is_err());
    assert!(parse_u8("256").is_err()); // u8 overflow
    assert!(parse_u8("-1").is_err()); // negative on unsigned
    assert!(parse_u8("18446744073709551615").is_err()); // huge overflow

    // Signed integer test
    let parse_i32 = decimal_range(-10, 10);
    assert_eq!(parse_i32("-10").unwrap(), -10);
    assert_eq!(parse_i32("0").unwrap(), 0);
    assert_eq!(parse_i32("10").unwrap(), 10);
    assert!(parse_i32("-11").is_err());
    assert!(parse_i32("11").is_err());
    assert!(parse_i32("-2147483649").is_err()); // i32 underflow
    assert!(parse_i32("2147483648").is_err()); // i32 overflow

    // Min > Max scenario - what happens when range is invalid?
    let parse_invalid = decimal_range(100, 0);
    assert!(parse_invalid("50").is_err(), "Expected Err when calling decimal_range with min > max");
}

#[test]
fn test_maybe_hex_adversarial() {
    // Normal cases
    assert_eq!(maybe_hex::<u32>("10").unwrap(), 10);
    assert_eq!(maybe_hex::<u32>("0x10").unwrap(), 16);
    assert_eq!(maybe_hex::<u32>("0X10").unwrap(), 16);
    assert_eq!(maybe_hex::<u32>("0").unwrap(), 0);
    assert_eq!(maybe_hex::<u32>("0x0").unwrap(), 0);

    // Mixed casing
    assert_eq!(maybe_hex::<u32>("0xAbCd").unwrap(), 0xABCD);

    // Extreme values
    assert_eq!(maybe_hex::<u64>("0xffffffffffffffff").unwrap(), u64::MAX);

    // Overflow bounds
    assert!(maybe_hex::<u64>("0x10000000000000000").is_err()); // overflow u64
    assert!(maybe_hex::<u8>("256").is_err()); // overflow u8
    assert!(maybe_hex::<u8>("0x100").is_err()); // overflow u8 (256)

    // Empty and invalid formats
    assert!(maybe_hex::<u32>("").is_err());
    assert!(maybe_hex::<u32>(" ").is_err());
    assert!(maybe_hex::<u32>("0x").is_err());
    assert!(maybe_hex::<u32>("0X").is_err());
    assert!(maybe_hex::<u32>("0xG").is_err());
    assert!(maybe_hex::<u32>("0x12G").is_err());
    assert!(maybe_hex::<u32>(" 0x10").is_err());
    assert!(maybe_hex::<u32>("0x10 ").is_err());
    assert!(maybe_hex::<u32>("+0x10").is_err());
    assert!(maybe_hex::<u32>("-0x10").is_err());
    assert!(maybe_hex::<u32>("0x1.0").is_err());
}

#[test]
fn test_maybe_hex_range_adversarial() {
    let parse = maybe_hex_range(10u8, 20u8);

    // Within bounds (dec and hex)
    assert_eq!(parse("15").unwrap(), 15);
    assert_eq!(parse("0x0f").unwrap(), 15);
    assert_eq!(parse("0X14").unwrap(), 20); // 20

    // Out of bounds
    assert!(parse("9").is_err());
    assert!(parse("21").is_err());
    assert!(parse("0x09").is_err());
    assert!(parse("0x15").is_err()); // 21

    // Invalid formats
    assert!(parse("").is_err());
    assert!(parse("abc").is_err());
    assert!(parse("0x").is_err());
}

#[test]
fn test_parse_percentage_adversarial() {
    // Valid cases
    assert_eq!(parse_percentage("0%").unwrap(), 0.0);
    assert_eq!(parse_percentage("100%").unwrap(), 1.0);
    assert_eq!(parse_percentage("50.0%").unwrap(), 0.5);
    assert_eq!(parse_percentage("12.345%").unwrap(), 0.12345);

    // Empty / whitespace
    assert!(parse_percentage("").is_err());
    assert!(parse_percentage("%").is_err());
    assert!(parse_percentage(" %").is_err());
    assert!(parse_percentage("50% ").is_err());
    assert!(parse_percentage(" 50%").is_err());
    assert!(parse_percentage("50 %").is_err());

    // Out of bounds (0% to 100%)
    assert!(parse_percentage("-0.01%").is_err());
    assert!(parse_percentage("100.01%").is_err());
    assert!(parse_percentage("1000%").is_err());
    assert!(parse_percentage("-50%").is_err());

    // Malformed numeric parts
    assert!(parse_percentage("abc%").is_err());
    assert!(parse_percentage("50%%").is_err());
    assert!(parse_percentage("50").is_err());
    assert!(parse_percentage("NaN%").is_err());
    assert!(parse_percentage("inf%").is_err());
    assert!(parse_percentage("-inf%").is_err());
}

#[test]
fn test_parse_bytes_adversarial() {
    // Normal cases (case insensitivity and units)
    assert_eq!(parse_bytes("0").unwrap(), 0);
    assert_eq!(parse_bytes("10").unwrap(), 10);
    assert_eq!(parse_bytes("10b").unwrap(), 10);
    assert_eq!(parse_bytes("10B").unwrap(), 10);
    assert_eq!(parse_bytes("1kb").unwrap(), 1024);
    assert_eq!(parse_bytes("1KB").unwrap(), 1024);
    assert_eq!(parse_bytes("1Kb").unwrap(), 1024);
    assert_eq!(parse_bytes("1kB").unwrap(), 1024);
    assert_eq!(parse_bytes("1k").unwrap(), 1024);
    assert_eq!(parse_bytes("1m").unwrap(), 1024 * 1024);
    assert_eq!(parse_bytes("1g").unwrap(), 1024 * 1024 * 1024);
    assert_eq!(parse_bytes("1t").unwrap(), 1024 * 1024 * 1024 * 1024);

    // Whitespace handling
    assert_eq!(parse_bytes(" 10 kb ").unwrap(), 10240);
    assert_eq!(parse_bytes("10\tmb").unwrap(), 10485760);

    // Empty & malformed
    assert!(parse_bytes("").is_err());
    assert!(parse_bytes(" ").is_err());
    assert!(parse_bytes("kb").is_err());
    assert!(parse_bytes("10kbx").is_err());
    assert!(parse_bytes("10z").is_err());
    assert!(parse_bytes("1.5mb").is_err()); // floats not supported by parse::<u64>

    // Overflow cases
    assert!(parse_bytes("18446744073709551616").is_err()); // overflows u64 base
                                                           // Multiplying causes overflow
    assert!(parse_bytes("18446744073709551615kb").is_err()); // u64::MAX * 1024 overflows
    assert!(parse_bytes("18446744073709551615t").is_err());
}

#[test]
fn test_parse_duration_adversarial() {
    // Normal cases
    assert_eq!(parse_duration("0s").unwrap(), Duration::from_secs(0));
    assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_duration("1h 15m").unwrap(), Duration::from_secs(4500));
    assert_eq!(parse_duration("1d 2h 3m 4s").unwrap(), Duration::from_secs(86400 + 7200 + 180 + 4));

    // Whitespace variations
    assert_eq!(parse_duration("  30s  ").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_duration("1h\t15m\n30s").unwrap(), Duration::from_secs(4530));

    // Malformed segments
    assert!(parse_duration("").is_err());
    assert!(parse_duration(" ").is_err());
    assert!(parse_duration("30").is_err());
    assert!(parse_duration("s").is_err());
    assert!(parse_duration("30s 5").is_err());
    assert!(parse_duration("30s5m").is_err()); // Single segment with multiple chars
    assert!(parse_duration("30x").is_err());

    // Negative duration?
    assert!(parse_duration("-5s").is_err());

    // Overflow test (This might panic! We will run it and check)
    // u64::MAX in seconds
    assert_eq!(parse_duration("18446744073709551615s").unwrap(), Duration::from_secs(u64::MAX));

    // Now let's try u64::MAX in minutes, which is u64::MAX * 60 (unchecked in code!)
    assert!(
        parse_duration("18446744073709551615m").is_err(),
        "Expected Err on parse_duration overflow for u64::MAX minutes"
    );
}

// =========================================================================
// 2. JSON Serialization and Formatting
// =========================================================================

#[test]
fn test_command_schema_extreme_nesting() {
    // Build a command nested 50 levels deep (within serde_json's recursion limit of 128)
    let mut cmd = Command::new("level_0");
    for i in 1..=50 {
        let name = Box::leak(format!("level_{}", i).into_boxed_str()) as &str;
        cmd = Command::new(name).subcommand(cmd);
    }

    // Extract schema
    let schema = extract_command_schema(&cmd);
    assert_eq!(schema.name, "level_50");

    // Serialize to JSON
    let json_str = serde_json::to_string(&schema);
    assert!(json_str.is_ok());
    let json_val: Value = serde_json::from_str(&json_str.unwrap()).unwrap();

    // Traverse down and check the depth
    let mut current = &json_val;
    for i in (0..=50).rev() {
        assert_eq!(current["name"], format!("level_{}", i));
        if i > 0 {
            current = &current["subcommands"][0];
        }
    }
}

#[test]
fn test_command_schema_deep_nesting_stack_safety() {
    // Build a command nested 1000 levels deep to test stack safety during extraction and serialization
    let mut cmd = Command::new("level_0");
    for i in 1..=1000 {
        let name = Box::leak(format!("level_{}", i).into_boxed_str()) as &str;
        cmd = Command::new(name).subcommand(cmd);
    }

    // Extract schema (recursion depth 1000)
    let schema = extract_command_schema(&cmd);
    assert_eq!(schema.name, "level_1000");

    // Serialization (to_string does not hit recursion limit by default or does not panic)
    let json_str = serde_json::to_string(&schema);
    assert!(json_str.is_ok());
}

#[test]
fn test_command_schema_empty_minimal() {
    let cmd = Command::new("minimal");
    let schema = extract_command_schema(&cmd);

    assert_eq!(schema.name, "minimal");
    assert!(schema.version.is_none());
    assert!(schema.author.is_none());
    assert!(schema.about.is_none());
    assert!(schema.subcommands.is_empty());
    assert!(schema.arguments.is_empty());

    let json_str = serde_json::to_string(&schema).unwrap();
    let expected = r#"{"name":"minimal","version":null,"author":null,"about":null,"subcommands":[],"arguments":[]}"#;
    assert_eq!(json_str, expected);
}

#[test]
fn test_command_schema_weird_characters() {
    let cmd = Command::new("weird-command 🚀")
        .version("v1.0.0-beta.1+build.123")
        .about("About with \n newlines, \t tabs, and \"quotes\".")
        .arg(
            Arg::new("arg with spaces")
                .long("long-name-with-dashes")
                .help("Help with emoji 🤔 and symbols: @#$%^&*()"),
        );

    let schema = extract_command_schema(&cmd);
    assert_eq!(schema.name, "weird-command 🚀");
    assert_eq!(schema.about.as_deref(), Some("About with \n newlines, \t tabs, and \"quotes\"."));

    let json_str = serde_json::to_string_pretty(&schema).unwrap();
    println!("Weird characters JSON:\n{}", json_str);

    // Verify it parses back
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["name"], "weird-command 🚀");
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CustomArgType(String);

impl std::str::FromStr for CustomArgType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CustomArgType(s.to_string()))
    }
}

#[test]
fn test_arg_matches_to_json_custom_type() {
    // Create a command with an argument that uses a custom type parser or different actions.
    let cmd = Command::new("custom-app")
        .arg(
            Arg::new("custom")
                .long("custom")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(CustomArgType)),
        )
        .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue));

    // 1. When raw string value is passed on CLI
    let matches = cmd
        .clone()
        .try_get_matches_from(vec!["custom-app", "--custom", "hello-world", "--flag"])
        .unwrap();
    let json = arg_matches_to_json(&matches);

    assert_eq!(json["custom"], Value::String("hello-world".to_string()));
    assert_eq!(json["flag"], Value::Bool(true));

    // 2. When argument has a default value of custom type but was not passed.
    // Let's create an argument with a default value.
    let cmd_default = Command::new("custom-app").arg(
        Arg::new("custom")
            .long("custom")
            .action(ArgAction::Set)
            .default_value("default-val")
            .value_parser(clap::value_parser!(CustomArgType)),
    );

    let matches_default = cmd_default.try_get_matches_from(vec!["custom-app"]).unwrap();
    let json_default = arg_matches_to_json(&matches_default);

    // Since get_raw(name) will be Some("default-val") because default_value is set,
    // it parses "default-val" as a string.
    assert_eq!(json_default["custom"], Value::String("default-val".to_string()));
}
