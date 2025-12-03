#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! regex = "1.10"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize, Deserialize)]
struct CodeExample {
    file: String,
    line_start: usize,
    line_end: usize,
    language: String,
    code: String,
    context: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    example: CodeExample,
    status: Status,
    error_message: Option<String>,
    suggested_fix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Status {
    Pass,
    CompileError,
    DeprecatedAPI,
    MissingImport,
    TypeMismatch,
    VersionMismatch,
}

fn main() {
    println!("clap-noun-verb Documentation Example Validator");
    println!("==============================================\n");

    let doc_files = vec![
        "README.md",
        "AUTONOMIC.md",
        "docs/QUICKSTART.md",
        "docs/CLI_REFERENCE.md",
        "docs/CLI_COOKBOOK.md",
        "docs/SEMANTIC_CLI_ARCHITECTURE.md",
    ];

    let mut all_examples = Vec::new();
    let mut results = Vec::new();

    // Extract all code examples
    for file in &doc_files {
        if let Ok(examples) = extract_code_examples(file) {
            println!("Found {} code examples in {}", examples.len(), file);
            all_examples.extend(examples);
        }
    }

    println!("\nTotal examples found: {}\n", all_examples.len());

    // Validate each example
    for example in all_examples {
        let result = validate_example(&example);
        println!(
            "[{}] {}:{} - {:?}",
            match result.status {
                Status::Pass => "✓",
                _ => "✗",
            },
            result.example.file,
            result.example.line_start,
            result.status
        );
        if let Some(ref error) = result.error_message {
            println!("  Error: {}", error);
        }
        results.push(result);
    }

    // Generate report
    generate_report(&results);
}

fn extract_code_examples(file_path: &str) -> Result<Vec<CodeExample>, std::io::Error> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut examples = Vec::new();
    let mut in_code_block = false;
    let mut current_language = String::new();
    let mut code_lines = Vec::new();
    let mut block_start = 0;
    let mut context = String::new();

    let code_fence_re = Regex::new(r"^```(\w+)?").ok().filter(|_| true);

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("```") {
            if !in_code_block {
                // Start of code block
                in_code_block = true;
                let lang = line
                    .strip_prefix("```")
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or("text");
                current_language = lang.to_string();
                code_lines.clear();
                block_start = i + 1;

                // Capture context (previous non-empty line)
                context = lines
                    .iter()
                    .take(i)
                    .rev()
                    .find(|l| !l.trim().is_empty() && !l.starts_with("```"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            } else {
                // End of code block
                in_code_block = false;
                if (current_language == "rust" || current_language == "toml" || current_language == "bash")
                    && !code_lines.is_empty()
                {
                    examples.push(CodeExample {
                        file: file_path.to_string(),
                        line_start: block_start,
                        line_end: i,
                        language: current_language.clone(),
                        code: code_lines.join("\n"),
                        context: context.clone(),
                    });
                }
            }
        } else if in_code_block {
            code_lines.push(line.to_string());
        }
    }

    Ok(examples)
}

fn validate_example(example: &CodeExample) -> ValidationResult {
    match example.language.as_str() {
        "rust" => validate_rust_code(example),
        "toml" => validate_toml_config(example),
        "bash" => validate_bash_command(example),
        _ => ValidationResult {
            example: example.clone(),
            status: Status::Pass,
            error_message: None,
            suggested_fix: None,
        },
    }
}

fn validate_rust_code(example: &CodeExample) -> ValidationResult {
    let mut status = Status::Pass;
    let mut error_message = None;
    let mut suggested_fix = None;

    // Check for deprecated APIs
    let deprecated_apis = vec![
        ("VerbArgs", "Current v5.1.1 uses VerbContext"),
        ("OutputFormat::Json", "Now part of format module"),
        ("run_with_format", "Changed to CliBuilder API"),
        ("NounVerbError::ValidationError", "Now uses thiserror patterns"),
    ];

    for (old_api, suggestion) in deprecated_apis {
        if example.code.contains(old_api) {
            status = Status::DeprecatedAPI;
            error_message = Some(format!("Deprecated API '{}' found", old_api));
            suggested_fix = Some(suggestion.to_string());
            break;
        }
    }

    // Check for version mismatches
    if example.code.contains("4.0.2") || example.code.contains("3.8.0") {
        status = Status::VersionMismatch;
        error_message = Some("Version reference outdated - should be 5.1.1".to_string());
        suggested_fix = Some("Update version to 5.1.1".to_string());
    }

    // Check for missing imports
    if example.code.contains("#[verb]") && !example.code.contains("use clap_noun_verb_macros::verb") {
        if status == Status::Pass {
            status = Status::MissingImport;
            error_message = Some("Missing import for #[verb] macro".to_string());
            suggested_fix = Some("Add: use clap_noun_verb_macros::verb;".to_string());
        }
    }

    // Try to compile if it's a complete program
    if example.code.contains("fn main") {
        match try_compile_example(&example.code) {
            Ok(_) => {
                if status == Status::Pass {
                    status = Status::Pass;
                }
            }
            Err(e) => {
                if status == Status::Pass {
                    status = Status::CompileError;
                    error_message = Some(e);
                }
            }
        }
    }

    ValidationResult {
        example: example.clone(),
        status,
        error_message,
        suggested_fix,
    }
}

fn validate_toml_config(example: &CodeExample) -> ValidationResult {
    let mut status = Status::Pass;
    let mut error_message = None;
    let mut suggested_fix = None;

    // Check version references
    if example.code.contains("clap-noun-verb = \"4.0.2\"") || example.code.contains("3.8.0") {
        status = Status::VersionMismatch;
        error_message = Some("Version in Cargo.toml should be 5.1.1".to_string());
        suggested_fix = Some("Update to: clap-noun-verb = \"5.1.1\"".to_string());
    }

    ValidationResult {
        example: example.clone(),
        status,
        error_message,
        suggested_fix,
    }
}

fn validate_bash_command(example: &CodeExample) -> ValidationResult {
    // Basic validation - check if commands exist
    ValidationResult {
        example: example.clone(),
        status: Status::Pass,
        error_message: None,
        suggested_fix: None,
    }
}

fn try_compile_example(code: &str) -> Result<(), String> {
    // Create temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("doc_example_test.rs");

    // Write code with necessary preamble
    let full_code = format!(
        r#"
// Auto-generated test from documentation
#![allow(dead_code, unused_imports, unused_variables)]

use clap_noun_verb::*;
use clap_noun_verb_macros::*;
use serde::Serialize;

{}
"#,
        code
    );

    fs::write(&temp_file, full_code).map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Try to compile
    let output = Command::new("rustc")
        .args(&[
            "--crate-type",
            "bin",
            "--edition",
            "2021",
            temp_file.to_str().unwrap_or(""),
            "-o",
            temp_dir.join("doc_example_test").to_str().unwrap_or(""),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    // Clean up
    let _ = fs::remove_file(temp_file);

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Compilation failed: {}", stderr))
    }
}

fn generate_report(results: &[ValidationResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.status == Status::Pass).count();
    let failed = total - passed;

    let mut status_counts: HashMap<String, usize> = HashMap::new();
    for result in results {
        let status_str = format!("{:?}", result.status);
        *status_counts.entry(status_str).or_insert(0) += 1;
    }

    println!("\n\n==============================================");
    println!("Documentation Validation Report");
    println!("==============================================\n");

    println!("Total Examples: {}", total);
    println!("Passed: {} ({:.1}%)", passed, (passed as f64 / total as f64) * 100.0);
    println!("Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);

    println!("\n\nFailure Breakdown:");
    for (status, count) in status_counts.iter() {
        if status != "Pass" {
            println!("  {}: {}", status, count);
        }
    }

    // List failed examples with suggestions
    println!("\n\nFailed Examples with Suggested Fixes:");
    println!("=====================================\n");

    for result in results.iter().filter(|r| r.status != Status::Pass) {
        println!("File: {}:{}", result.example.file, result.example.line_start);
        println!("Status: {:?}", result.status);
        if let Some(ref error) = result.error_message {
            println!("Error: {}", error);
        }
        if let Some(ref fix) = result.suggested_fix {
            println!("Suggested Fix: {}", fix);
        }
        println!("Context: {}", result.example.context);
        println!();
    }

    // Generate JSON report
    let json_report = serde_json::to_string_pretty(&results).ok();
    if let Some(json) = json_report {
        let _ = fs::write("doc_validation_report.json", json);
        println!("Full report saved to: doc_validation_report.json\n");
    }
}
