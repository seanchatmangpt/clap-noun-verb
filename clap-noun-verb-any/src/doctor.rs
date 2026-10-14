//! `cnv-any doctor` -- validates a manifest + wrapped-executable pairing
//! before deployment, catching real, common mistakes: an executable that
//! doesn't exist or isn't executable, a manifest that doesn't parse, and
//! per-command argument-shape inconsistencies (duplicate ids/flags, a
//! positional argument that also declares a long/short flag, duplicate
//! tool paths) that would otherwise surface later as a confusing runtime
//! error from `wrap()`/`Gateway::execute` instead of a clear diagnosis
//! up front.

use clap_noun_verb_deploy::{CliSchema, CommandSchema, ManifestError};
use std::collections::HashSet;
use std::path::Path;

/// One real problem `diagnose` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Would prevent `wrap()`/deployment from working at all.
    Error,
    /// Works today but is a real, likely-unintended inconsistency.
    Warning,
}

/// The full result of diagnosing one executable + manifest pairing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DoctorReport {
    pub findings: Vec<Finding>,
}

impl DoctorReport {
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        !self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    #[must_use]
    pub fn error_count(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Error).count()
    }

    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Warning).count()
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.findings.push(Finding { severity: Severity::Error, message: message.into() });
    }

    fn push_warning(&mut self, message: impl Into<String>) {
        self.findings.push(Finding { severity: Severity::Warning, message: message.into() });
    }
}

/// Diagnose a wrapped-target pairing: does `executable` exist and is it
/// really executable, does `manifest_path` parse into a real `CliSchema`,
/// and is every admitted command's argument shape internally consistent.
///
/// Never panics; a manifest that fails to parse is reported as a real
/// `Finding`, not propagated as an error -- `doctor`'s whole point is to
/// produce a complete diagnosis in one pass, not stop at the first
/// problem.
#[must_use]
pub fn diagnose(executable: &Path, manifest_path: &Path) -> DoctorReport {
    let mut report = DoctorReport::default();

    diagnose_executable(executable, &mut report);

    match CliSchema::from_manifest_path(manifest_path) {
        Ok(schema) => diagnose_schema(&schema, &mut report),
        Err(ManifestError::Io(io_error)) => {
            report.push_error(format!(
                "manifest {} could not be read: {io_error}",
                manifest_path.display()
            ));
        }
        Err(ManifestError::Json(json_error)) => {
            report.push_error(format!(
                "manifest {} is not valid CliSchema JSON: {json_error}",
                manifest_path.display()
            ));
        }
    }

    report
}

fn diagnose_executable(executable: &Path, report: &mut DoctorReport) {
    match std::fs::metadata(executable) {
        Ok(metadata) => {
            if !metadata.is_file() {
                report.push_error(format!(
                    "{} exists but is not a regular file",
                    executable.display()
                ));
                return;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                if mode & 0o111 == 0 {
                    report.push_error(format!(
                        "{} exists but is not executable (mode {mode:o})",
                        executable.display()
                    ));
                }
            }
        }
        Err(io_error) => {
            report.push_error(format!("{} does not exist: {io_error}", executable.display()));
        }
    }
}

/// Real shape-validation errors in `schema` alone (no filesystem access,
/// no executable check) -- the exact same rules [`diagnose`] applies to a
/// manifest's schema, exposed standalone so a caller that already holds a
/// parsed [`CliSchema`] in memory (like `wrap()`) can validate it without
/// re-reading the manifest file from disk. Empty iff the schema has no
/// shape errors (it may still have warnings, which this deliberately
/// omits -- a warning-only manifest is not malformed).
#[must_use]
pub fn schema_shape_errors(schema: &CliSchema) -> Vec<String> {
    let mut report = DoctorReport::default();
    diagnose_schema(schema, &mut report);
    report
        .findings
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.message)
        .collect()
}

fn diagnose_schema(schema: &CliSchema, report: &mut DoctorReport) {
    if schema.commands.is_empty() {
        report.push_warning("manifest admits zero commands -- nothing will be callable");
    }

    let mut seen_tool_names: HashSet<String> = HashSet::new();
    for command in &schema.commands {
        let tool_name = command.path.join("__");
        if command.path.is_empty() {
            report.push_error("a command has an empty path (no noun/verb segments)");
        } else if !seen_tool_names.insert(tool_name.clone()) {
            report.push_error(format!("duplicate command path (tool name '{tool_name}')"));
        }

        diagnose_command_arguments(command, &tool_name, report);
    }
}

fn diagnose_command_arguments(command: &CommandSchema, tool_name: &str, report: &mut DoctorReport) {
    let mut seen_ids: HashSet<&str> = HashSet::new();
    let mut seen_long: HashSet<&str> = HashSet::new();
    let mut seen_short: HashSet<char> = HashSet::new();

    for argument in &command.arguments {
        if !seen_ids.insert(argument.id.as_str()) {
            report.push_error(format!("'{tool_name}': duplicate argument id '{}'", argument.id));
        }
        if let Some(long) = &argument.long {
            if !seen_long.insert(long.as_str()) {
                report.push_error(format!("'{tool_name}': duplicate long flag '--{long}'"));
            }
        }
        if let Some(short) = argument.short {
            if !seen_short.insert(short) {
                report.push_error(format!("'{tool_name}': duplicate short flag '-{short}'"));
            }
        }
        if argument.positional && (argument.long.is_some() || argument.short.is_some()) {
            report.push_warning(format!(
                "'{tool_name}': argument '{}' is positional but also declares a long/short flag \
                 (the flag will never be reachable)",
                argument.id
            ));
        }
    }
}
