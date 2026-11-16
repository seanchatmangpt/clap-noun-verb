// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for every `OutputFormat` variant and parser alias.

use clap_noun_verb::{format_output, OutputFormat};
use serde::Serialize;
use std::str::FromStr;

#[derive(Serialize)]
struct ServiceStatus {
    name: &'static str,
    replicas: u32,
    healthy: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = ServiceStatus { name: "api", replicas: 3, healthy: true };
    let formats = [
        OutputFormat::Json,
        OutputFormat::JsonPretty,
        OutputFormat::Yaml,
        OutputFormat::Table,
        OutputFormat::Plain,
        OutputFormat::Tsv,
        OutputFormat::Quiet,
    ];

    assert_eq!(OutputFormat::available_formats().len(), formats.len());
    for format in formats {
        let rendered = format_output(&value, format)?;
        if format == OutputFormat::Quiet {
            assert!(rendered.is_empty());
        } else {
            assert!(!rendered.is_empty(), "{format} must render output");
        }
    }

    assert_eq!(
        OutputFormat::from_str("pretty").map_err(std::io::Error::other)?,
        OutputFormat::JsonPretty
    );
    assert_eq!(OutputFormat::from_str("yml").map_err(std::io::Error::other)?, OutputFormat::Yaml);
    assert!(OutputFormat::from_str("binary").is_err());

    println!("Output formats admitted: {:?}", OutputFormat::available_formats());
    Ok(())
}
