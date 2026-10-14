//! Best-effort manifest scaffolding from `<executable> --help` output.
//!
//! **This module produces a DRAFT, not a trusted schema source.** No
//! ecosystem's `--help` output format is reliable enough to parse and trust
//! unreviewed -- argparse, click, Cobra, and Clap all format subcommand
//! lists and flags slightly differently, and many CLIs hand-write their
//! help text with no fixed structure at all. [`draft_manifest_from_help`]
//! applies a few simple, clearly-labeled heuristics to produce a starting
//! point for a human to review and edit into a real `cnv-any.json`
//! manifest -- it is never a substitute for that review, and it is
//! deliberately not extended to parse every CLI framework in existence.

use clap_noun_verb_deploy::{
    ArgumentBehavior, ArgumentKind, ArgumentSchema, CliSchema, CommandSchema,
};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Failure running or parsing `<executable> --help` while scaffolding.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("failed to execute '{0} --help': {1}")]
    Io(String, std::io::Error),
    #[error("'{0} --help' output was not valid UTF-8")]
    NotUtf8(String),
}

/// Run `<executable> --help` and apply simple heuristics to produce a
/// **draft** [`CliSchema`] for human review.
///
/// Heuristics applied (intentionally simple, intentionally incomplete):
/// - A line of the shape `  name   description` (2-4 leading spaces, a
///   bare word, then 2+ spaces before more text) is treated as a
///   subcommand name -- the common shape of an argparse/Click/Cobra/Clap
///   subcommand listing.
/// - A line containing a `--long-flag` or ` -x` token is treated as an
///   optional string argument on the single scaffolded command.
///
/// The result should always be inspected and edited by a human before it is
/// used as a real manifest -- see the module-level documentation.
pub fn draft_manifest_from_help(executable: &Path) -> Result<CliSchema, ScaffoldError> {
    let label = executable.display().to_string();
    let output = Command::new(executable)
        .arg("--help")
        .output()
        .map_err(|error| ScaffoldError::Io(label.clone(), error))?;
    let help_text =
        String::from_utf8(output.stdout).map_err(|_| ScaffoldError::NotUtf8(label.clone()))?;

    let name =
        executable.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or(label);

    let subcommands = extract_subcommands(&help_text);
    let arguments = extract_arguments(&help_text);

    let commands = if subcommands.is_empty() {
        vec![CommandSchema {
            path: vec![name.clone()],
            about: Some("DRAFT: single callable command inferred from --help".to_owned()),
            arguments,
            callable: true,
        }]
    } else {
        subcommands
            .into_iter()
            .map(|subcommand| CommandSchema {
                path: vec![subcommand],
                about: Some("DRAFT: subcommand inferred from --help listing".to_owned()),
                arguments: arguments.clone(),
                callable: true,
            })
            .collect()
    };

    Ok(CliSchema {
        name,
        about: Some("DRAFT manifest scaffolded from --help output -- review before use".to_owned()),
        commands,
    })
}

/// Lines of the shape `  name   description text` (2-4 leading spaces, a
/// bare identifier, then 2+ spaces before more text): the common shape of
/// an argparse/Click/Cobra/Clap subcommand listing.
fn extract_subcommands(help_text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in help_text.lines() {
        let leading_spaces = line.len() - line.trim_start().len();
        if !(2..=4).contains(&leading_spaces) {
            continue;
        }
        let trimmed = line.trim_start();
        let Some(first_word_end) = trimmed.find(char::is_whitespace) else {
            continue;
        };
        let candidate = &trimmed[..first_word_end];
        let rest = &trimmed[first_word_end..];
        let is_identifier = !candidate.is_empty()
            && candidate.chars().next().is_some_and(|c| c.is_alphabetic())
            && candidate.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        let has_gap_before_description = rest.starts_with("  ");
        if is_identifier && has_gap_before_description && !names.contains(&candidate.to_owned()) {
            names.push(candidate.to_owned());
        }
    }
    names
}

/// Lines containing a `--long-flag` or ` -x` token are treated as optional
/// string arguments.
fn extract_arguments(help_text: &str) -> Vec<ArgumentSchema> {
    let mut arguments = Vec::new();
    for token in help_text.split(|c: char| c.is_whitespace() || c == ',') {
        if let Some(long) = token.strip_prefix("--") {
            let long = long.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if long.is_empty()
                || arguments.iter().any(|a: &ArgumentSchema| a.long.as_deref() == Some(long))
            {
                continue;
            }
            arguments.push(ArgumentSchema {
                id: long.to_owned(),
                long: Some(long.to_owned()),
                short: None,
                required: false,
                positional: false,
                kind: ArgumentKind::String,
                behavior: ArgumentBehavior::Value,
            });
        }
    }
    arguments
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_script_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        std::env::temp_dir().join(format!("cnv-any-scaffold-{label}-{nanos}.sh"))
    }

    fn write_executable_script(path: &Path, contents: &str) {
        fs::write(path, contents).expect("write real fixture script");
        let mut permissions = fs::metadata(path).expect("real file metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("mark real fixture script executable");
    }

    #[test]
    fn draft_manifest_extracts_subcommands_and_flags_from_real_help_output() {
        // Arrange: a real, tiny shell script whose --help mimics a common
        // argparse/Click-style listing.
        let path = unique_script_path("help");
        write_executable_script(
            &path,
            "#!/bin/sh\ncat <<'EOF'\nusage: demo [--verbose] <command>\n\nCommands:\n  greet   Greet someone\n  list    List things\n\nOptions:\n  --verbose  Enable verbose output\nEOF\n",
        );

        // Act
        let schema = draft_manifest_from_help(&path).expect("run real --help and scaffold");

        // Assert
        assert!(schema.about.as_deref().unwrap_or_default().contains("DRAFT"));
        let paths: Vec<&str> = schema.commands.iter().map(|c| c.path[0].as_str()).collect();
        assert!(paths.contains(&"greet"));
        assert!(paths.contains(&"list"));
        let has_verbose_flag = schema
            .commands
            .iter()
            .any(|c| c.arguments.iter().any(|a| a.long.as_deref() == Some("verbose")));
        assert!(has_verbose_flag);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn draft_manifest_falls_back_to_single_command_without_subcommand_listing() {
        // Arrange
        let path = unique_script_path("plain");
        write_executable_script(&path, "#!/bin/sh\necho 'usage: demo [--name NAME]'\n");

        // Act
        let schema = draft_manifest_from_help(&path).expect("run real --help and scaffold");

        // Assert: no subcommand-shaped lines -- one draft callable command
        assert_eq!(schema.commands.len(), 1);
        assert!(schema.commands[0].callable);
    }
}
