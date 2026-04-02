//! Thesis structure operations
//!
//! Commands for exploring HTF framework and thesis families.
//!
//! Following the golden rule: CLI validates, domain computes, integration connects.

use clap_noun_verb_macros::verb;
use clap_noun_verb::{NounVerbError, Result};

use std::str::FromStr;
use clap_noun_verb::{OutputFormat, format_output};
use crate::domain::{PaperFamily, ThesisFamily, ThesisSchedule, ThesisStructure};

/// Show thesis structure (HTF framework)
///
/// Displays the Hyper-Thesis Framework structure including Δ-Shards,
/// Λ-Scheduling, Π-Profiling, and Γ-Globalization.
///
/// # Arguments
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("structure")]
fn show_structure(format: Option<String>) -> Result<()> {
    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    let output = format_output(&ThesisStructure::get(), fmt)
        .map_err(|e| NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}

/// List all thesis families
///
/// Shows detailed information about all 7 thesis families.
///
/// # Arguments
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("families")]
fn list_families(format: Option<String>) -> Result<()> {
    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    let output = format_output(&ThesisFamily::all(), fmt)
        .map_err(|e| NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}

/// Show Λ-schedule for family
///
/// Displays optimal writing order for the specified thesis family.
///
/// # Arguments
/// * `family` - Thesis family [default: IMRaD]
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("schedule")]
fn show_schedule(family: Option<String>, format: Option<String>) -> Result<()> {
    // 1. Validate inputs (CLI validates)
    let family_str = family.unwrap_or_else(|| "IMRaD".to_string());
    let family = PaperFamily::from_str(&family_str)
        .ok_or_else(|| NounVerbError::validation_error(
            "family".to_string(),
            family_str.clone(),
            Some("Unknown thesis family")
        ))?;

    // 2. Parse format
    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    // 3. Format and print
    let output = format_output(&ThesisSchedule::for_family(&family), fmt)
        .map_err(|e| NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}
