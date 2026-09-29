//! `cnv-any` -- a small developer convenience CLI for `clap-noun-verb-any`.
//!
//! Two subcommands:
//! - `cnv-any init <path-to-binary> [--out cnv-any.json]` scaffolds a
//!   *draft* manifest from `<binary> --help` (see
//!   [`clap_noun_verb_any::scaffold`]) and writes it to disk as pretty JSON
//!   for human review and editing.
//! - `cnv-any doctor <path-to-binary> <path-to-manifest>` validates a
//!   manifest + wrapped-executable pairing before deployment (see
//!   [`clap_noun_verb_any::doctor`]), exiting non-zero if any real problem
//!   was found.

use clap_noun_verb_any::doctor::{diagnose, Severity};
use clap_noun_verb_any::scaffold::draft_manifest_from_help;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("cnv-any: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<bool, String> {
    match args.get(1).map(String::as_str) {
        Some("init") => run_init(&args[2..]).map(|()| true),
        Some("doctor") => run_doctor(&args[2..]),
        _ => Err(usage()),
    }
}

fn run_doctor(args: &[String]) -> Result<bool, String> {
    let executable = args.first().ok_or_else(usage)?;
    let manifest = args.get(1).ok_or_else(usage)?;

    let report = diagnose(Path::new(executable), Path::new(manifest));

    for finding in &report.findings {
        let label = match finding.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
        };
        println!("[{label}] {}", finding.message);
    }

    if report.findings.is_empty() {
        println!("cnv-any doctor: no problems found.");
    } else {
        println!(
            "cnv-any doctor: {} error(s), {} warning(s).",
            report.error_count(),
            report.warning_count()
        );
    }

    Ok(report.is_healthy())
}

fn run_init(args: &[String]) -> Result<(), String> {
    let mut binary_path: Option<&str> = None;
    let mut out_path = PathBuf::from("cnv-any.json");

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                let value =
                    args.get(index + 1).ok_or_else(|| "--out requires a path".to_owned())?;
                out_path = PathBuf::from(value);
                index += 2;
            }
            other if binary_path.is_none() => {
                binary_path = Some(other);
                index += 1;
            }
            other => return Err(format!("unrecognized argument '{other}'\n{}", usage())),
        }
    }

    let binary_path = binary_path.ok_or_else(usage)?;
    let schema = draft_manifest_from_help(Path::new(binary_path))
        .map_err(|error| format!("failed to scaffold manifest: {error}"))?;
    let json = serde_json::to_string_pretty(&schema)
        .map_err(|error| format!("failed to render manifest JSON: {error}"))?;
    std::fs::write(&out_path, json)
        .map_err(|error| format!("failed to write {}: {error}", out_path.display()))?;

    println!(
        "Wrote DRAFT manifest for '{binary_path}' to {} -- review and edit before deploying.",
        out_path.display()
    );
    Ok(())
}

fn usage() -> String {
    "usage:\n  \
     cnv-any init <path-to-binary> [--out cnv-any.json]\n  \
     cnv-any doctor <path-to-binary> <path-to-manifest>"
        .to_owned()
}
