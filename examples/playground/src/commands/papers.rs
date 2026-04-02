//! Academic paper operations
//!
//! Generate, list, and validate academic papers using Tera templates.
//!
//! Following the golden rule: CLI validates, domain computes, integration connects.

use clap_noun_verb_macros::verb;
use clap_noun_verb::{NounVerbError, Result};

use std::str::FromStr;
use clap_noun_verb::{OutputFormat, format_output};
use crate::domain::{Paper, PaperFamily};
use crate::integration::{ensure_output_dir, get_template_engine, render_paper_latex, write_paper};
use crate::outputs::PaperGeneratedOutput;

/// Generate academic paper with Tera templates
///
/// Creates a LaTeX paper using Tera templates for the specified thesis family.
///
/// # Arguments
/// * `family` - Paper family [default: imrad]
/// * `output` - Output file path [value_hint: FilePath]
#[verb("generate")]
fn generate_paper(
    family: Option<String>,
    output: Option<String>,
) -> Result<PaperGeneratedOutput> {
    // 1. Validate inputs (CLI validates)
    let family_str = family.unwrap_or_else(|| "IMRaD".to_string());
    let family = PaperFamily::from_str(&family_str)
        .ok_or_else(|| NounVerbError::validation_error(
            "family".to_string(),
            family_str.clone(),
            Some(&format!("Valid options: {}", PaperFamily::valid_values().join(", ")))
        ))?;

    // 2. Call domain logic (pure, testable)
    let paper = Paper::new(family.clone(), None, None);

    // 3. Call integration layer (I/O side effects)
    let tera = get_template_engine()
        .map_err(|e| NounVerbError::execution_error(e))?;
    let latex = render_paper_latex(&paper, tera)
        .map_err(|e| NounVerbError::execution_error(e))?;

    // 4. Determine output path
    let path = if let Some(output_path) = output {
        output_path
    } else {
        ensure_output_dir("output")
            .map_err(|e| NounVerbError::execution_error(e))?;
        format!("output/{}-paper.tex", family.name().to_lowercase())
    };

    write_paper(&path, &latex)
        .map_err(|e| NounVerbError::execution_error(e))?;

    // 5. Return structured output
    Ok(PaperGeneratedOutput {
        family: family.name().to_string(),
        output_path: path,
        template_engine: "Tera 1.20".to_string(),
        sections: paper.sections.len(),
    })
}

/// List available paper families
///
/// Shows all 7 thesis families with descriptions.
///
/// # Arguments
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("list")]
fn list_families(format: Option<String>) -> Result<()> {
    // 1. Parse format arg (CLI validates)
    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    // 2. Delegate to domain logic
    let families: Vec<_> = PaperFamily::all()
        .iter()
        .map(|f| crate::outputs::PaperFamilyOutput {
            name: f.name().to_string(),
            description: f.description().to_string(),
        })
        .collect();

    // 3. Format and print output
    let output = format_output(&families, fmt)
        .map_err(|e| NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}

/// Validate paper structure
///
/// Validates a LaTeX paper file against HTF guidelines.
///
/// # Arguments
/// * `file` - Paper file to validate [default: output/imrad-paper.tex] [value_hint: FilePath]
/// * `format` - Output format (json, yaml, table, plain) [default: json-pretty]
#[verb("validate")]
fn validate_paper(file: Option<String>, format: Option<String>) -> Result<()> {
    // Default to sample file if none provided
    let file_path = file.unwrap_or_else(|| "output/imrad-paper.tex".to_string());

    // Parse format arg
    let fmt = format
        .as_deref()
        .and_then(|s| OutputFormat::from_str(s).ok())
        .unwrap_or(OutputFormat::JsonPretty);

    // Delegate to domain logic
    let result = crate::domain::papers::ValidationResult::validate_path(&file_path);

    let output = format_output(
        &crate::outputs::ValidationResultOutput {
            is_valid: result.is_valid,
            structure_valid: result.structure_valid,
            citations_valid: result.citations_valid,
            formatting_valid: result.formatting_valid,
            errors: result.errors,
        },
        fmt,
    )
    .map_err(|e| NounVerbError::execution_error(e.to_string()))?;
    println!("{}", output);
    Ok(())
}
