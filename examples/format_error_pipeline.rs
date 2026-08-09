// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cross-product witness: `VerbArgs` → `OutputFormat` → structured refusal.

use clap_noun_verb::{
    format_output, Arg, Command, ErrorKind, OutputFormat, StructuredError, VerbArgs, VerbContext,
};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("report")
        .arg(Arg::new("format").long("format").required(true))
        .arg(Arg::new("missing").long("missing"))
        .try_get_matches_from(["report", "--format", "json"])?;
    let args = VerbArgs::new(matches).with_context(VerbContext::new("render").with_noun("report"));

    let requested = args.get_one_str("format")?;
    let format = OutputFormat::from_str(&requested).map_err(std::io::Error::other)?;
    let rendered = format_output(
        &serde_json::json!({"noun": args.noun(), "verb": args.verb(), "standing": "ALIVE"}),
        format,
    )?;
    assert!(rendered.contains("ALIVE"));

    let missing = match args.get_one_str("missing") {
        Ok(_) => return Err("missing argument unexpectedly admitted".into()),
        Err(error) => error,
    };
    let structured = StructuredError::from_error(&missing);
    assert_eq!(structured.kind, ErrorKind::InvalidInput);
    assert!(structured.message.contains("missing"));

    println!("rendered={rendered}; refusal={}", structured.message);
    Ok(())
}
