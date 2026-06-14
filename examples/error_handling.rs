// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Error Handling Example
//!
//! Demonstrates the complete error model:
//! - `NounVerbError` constructors and their `Display` format
//! - "Did you mean?" Levenshtein suggestion (distance ≤ 3)
//! - Validation helpers: `validation_error`, `validation_range_error`, `validation_length_error`
//! - `StructuredError::from_error` — mapping to machine-readable JSON
//! - `StructuredError::deadline_exceeded` — Critical + TimeoutAdjustment template
//! - CLI error propagation: a verb returning Err bubbles to the runner
//!
//! **Expected output** (run with `cargo run --example error_handling`):
//! ```text
//! CommandNotFound: Command 'usr' not found. Did you mean: user?
//! VerbNotFound: Verb 'lst' not found for noun 'user'. Did you mean: list?
//! ValidationError (port, abc): Argument 'port' has invalid value 'abc': Must be a number
//! RangeError (port, 70000): Argument 'port' value '70000' is out of range [1, 65535]
//! LengthError (name, ): Argument 'name' has invalid length  (min: 1, max: 64)
//! deadline kind=DeadlineExceeded severity=Critical suggested_ms=740
//! verb error propagated: Command execution failed: db unreachable
//! ```
//!
//! **Doc**: docs/reference/error-codes.md
//! **Reference**: docs/reference/api/errors.md

use clap_noun_verb::{
    noun, run_cli_with_args, verb, NounVerbError, Result, StructuredError, VerbArgs,
};

fn main() -> Result<()> {
    // --- Witness 1: CommandNotFound with suggestion ---
    let err = NounVerbError::command_not_found_with_candidates("usr", &["user", "session", "config"]);
    let msg = err.to_string();
    assert!(msg.contains("usr"), "must name the bad input");
    assert!(msg.contains("user"), "Levenshtein match ≤3 must suggest 'user'");
    println!("CommandNotFound: {msg}");

    // --- Witness 2: VerbNotFound with suggestion ---
    let err = NounVerbError::verb_not_found_with_candidates("user", "lst", &["list", "get", "delete"]);
    let msg = err.to_string();
    assert!(msg.contains("lst"), "must name the bad verb");
    assert!(msg.contains("list"), "Levenshtein match must suggest 'list'");
    println!("VerbNotFound: {msg}");

    // --- Witness 3: Validation helpers ---
    let err = NounVerbError::validation_error("port", "abc", Some("Must be a number"));
    let msg = err.to_string();
    assert!(msg.contains("port"), "validation_error must name the arg");
    assert!(msg.contains("abc"), "validation_error must name the bad value");
    println!("ValidationError (port, abc): {msg}");

    let err = NounVerbError::validation_range_error("port", "70000", Some("1"), Some("65535"));
    let msg = err.to_string();
    assert!(msg.contains("70000"), "range_error must name the bad value");
    assert!(msg.contains("65535"), "range_error must name the upper bound");
    println!("RangeError (port, 70000): {msg}");

    let err = NounVerbError::validation_length_error("name", "", Some(1), Some(64));
    let msg = err.to_string();
    assert!(msg.contains("name"), "length_error must name the arg");
    println!("LengthError (name, ): {msg}");

    // --- Witness 4: StructuredError deadline path (kind + severity + action_template) ---
    let se = StructuredError::deadline_exceeded(500, 640);
    let json = serde_json::to_string(&se).expect("StructuredError must serialize");
    assert!(json.contains("DeadlineExceeded"), "kind must be DeadlineExceeded");
    assert!(json.contains("Critical"), "severity must be Critical");
    assert!(json.contains("740"), "TimeoutAdjustment must suggest deadline+20%=740ms");
    println!("deadline kind=DeadlineExceeded severity=Critical suggested_ms=740");

    // --- Witness 5: StructuredError::from_error maps NounVerbError variants ---
    let nve = NounVerbError::execution_error("db unreachable");
    let se = StructuredError::from_error(&nve);
    let json = serde_json::to_string(&se).expect("StructuredError must serialize");
    assert!(json.contains("ExecutionError"), "ExecutionError variant must map to ExecutionError kind");

    // --- Witness 6: verb returning Err propagates through the runner ---
    let result = run_cli_with_args(
        vec!["myapp".into(), "db".into(), "connect".into()],
        |builder| {
            builder.name("myapp").noun(noun!("db", "Database commands", [
                verb!("connect", "Connect to database", |_args: &VerbArgs| {
                    Err(NounVerbError::execution_error("db unreachable"))
                }),
            ]))
        },
    );
    match result {
        Err(NounVerbError::ExecutionError { message }) => {
            assert_eq!(message, "db unreachable", "error message must round-trip through runner");
            println!("verb error propagated: Command execution failed: {message}");
        }
        other => panic!("expected ExecutionError, got {other:?}"),
    }

    Ok(())
}
