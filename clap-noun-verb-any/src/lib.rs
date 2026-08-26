//! Wrap any executable -- Rust or not -- as a `clap-noun-verb-deploy`
//! deployable CLI.
//!
//! `clap-noun-verb-deploy`'s [`Deploy`]/[`Gateway`]/[`ProcessExecutor`] and
//! every protocol surface (`mcp`/`http`/`kubernetes`/`container`) are already
//! built on top of [`CliSchema`] + [`Executor`], not on Rust or Clap
//! specifically -- [`ProcessExecutor`] spawns an arbitrary [`std::ffi::OsString`]
//! executable and never inspects what produced it. The entire gap this crate
//! closes is schema *acquisition*: a manifest file that deserializes directly
//! into the existing [`CliSchema`] shape (via
//! [`CliSchema::from_manifest_path`]), so any wrapped target gets the same
//! `Deploy`/`Gateway`/MCP/HTTP/Kubernetes/OCI plumbing a native
//! `clap-noun-verb` binary gets, with **zero changes** to `mcp.rs`/`http.rs`/
//! `kubernetes.rs`/`container.rs`.
//!
//! This crate also gives a wrapped foreign binary **OCEL parity**: since it
//! cannot self-emit an OCEL 2.0 event the way a native `clap-noun-verb`
//! binary does, [`OcelExecutor`] records one on the wrapped binary's behalf
//! after every execution, using the manifest's command path as the
//! noun/verb, so the resulting event is structurally identical to a native
//! one.

pub mod doctor;
pub mod multi;
pub mod scaffold;

pub use multi::{merge_schemas, MultiExecutor, MultiExecutorError};

use clap_noun_verb_deploy::{
    CliSchema, Deploy, Execution, Executor, Invocation, ManifestError, ProcessExecutor,
};
use std::ffi::OsString;
use std::path::Path;
use std::time::Instant;
use thiserror::Error;

/// Failure wrapping an executable behind a manifest-derived [`CliSchema`].
#[derive(Debug, Error)]
pub enum WrapError {
    #[error("failed to load manifest: {0}")]
    Manifest(#[from] ManifestError),
    /// The manifest parsed as valid JSON but its shape is internally
    /// inconsistent (see [`doctor::schema_shape_errors`]) -- refused
    /// before any process is ever spawned, the same fail-closed
    /// discipline every ggen admission gate in this ecosystem uses.
    #[error("manifest failed shape validation: {}", .0.join("; "))]
    InvalidShape(Vec<String>),
}

/// An [`Executor`] wrapper that records an OCEL 2.0 `cli_invocation` event
/// after every execution, on behalf of an inner executor whose target cannot
/// self-emit one (a wrapped foreign binary).
///
/// The noun/verb attributed to the event are derived from the manifest's
/// command path: the first path segment is the noun, and the remaining
/// segments (or the same segment, for a single-segment command) are joined
/// as the verb. This mirrors how a native `clap-noun-verb` binary derives
/// its own noun/verb pair at the `CommandRegistry` boundary.
#[derive(Debug)]
pub struct OcelExecutor<E> {
    inner: E,
    schema: CliSchema,
}

impl<E> OcelExecutor<E> {
    /// Wrap `inner`, deriving noun/verb attribution from `schema`.
    #[must_use]
    pub const fn new(inner: E, schema: CliSchema) -> Self {
        Self { inner, schema }
    }

    /// The wrapped executor, discarding OCEL recording.
    #[must_use]
    pub fn into_inner(self) -> E {
        self.inner
    }

    fn noun_verb_for(&self, args: &[String]) -> (String, String) {
        let command = self
            .schema
            .commands
            .iter()
            .filter(|command| command.callable)
            .find(|command| args.starts_with(command.path.as_slice()));

        let Some(command) = command else {
            return (self.schema.name.clone(), "unknown".to_owned());
        };
        let noun = command.path.first().cloned().unwrap_or_else(|| self.schema.name.clone());
        let verb = if command.path.len() > 1 { command.path[1..].join("__") } else { noun.clone() };
        (noun, verb)
    }
}

impl<E: Executor> Executor for OcelExecutor<E> {
    type Error = E::Error;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
        let started = Instant::now();
        let result = self.inner.execute(invocation);
        let duration_ms = started.elapsed().as_millis();
        let (noun, verb) = self.noun_verb_for(&invocation.args);
        let success = result.as_ref().map(Execution::success).unwrap_or(false);
        clap_noun_verb::ocel::record_invocation(&noun, &verb, success, duration_ms);
        result
    }
}

/// A foreign executable wrapped behind a manifest-derived [`CliSchema`],
/// ready to be handed to any `clap-noun-verb-deploy` protocol surface.
#[derive(Debug)]
pub struct Wrapped {
    deploy: Deploy,
    executor: OcelExecutor<ProcessExecutor>,
}

impl Wrapped {
    /// The transport-neutral deployment projection (same type every native
    /// `clap-noun-verb` binary produces via `Deploy::from_registry`).
    #[must_use]
    pub const fn deploy(&self) -> &Deploy {
        &self.deploy
    }

    /// The OCEL-recording executor to hand to `McpServer::new` /
    /// `HttpServer::new` / any other `Executor` consumer, unchanged.
    #[must_use]
    pub const fn executor(&self) -> &OcelExecutor<ProcessExecutor> {
        &self.executor
    }

    /// Consume this wrapper, returning its parts.
    #[must_use]
    pub fn into_parts(self) -> (Deploy, OcelExecutor<ProcessExecutor>) {
        (self.deploy, self.executor)
    }
}

/// Wrap `executable` using the [`CliSchema`] manifest at `manifest_path`.
///
/// Loads the manifest via [`CliSchema::from_manifest_path`], builds a
/// [`ProcessExecutor`] pinned to `executable`, and returns the same
/// [`Deploy`] type `clap-noun-verb-deploy` already exposes -- so every
/// existing protocol surface (`mcp`/`http`/`kubernetes`/`container`) works
/// unchanged against a wrapped target exactly as it does against a native
/// `clap-noun-verb` binary.
pub fn wrap(executable: impl Into<OsString>, manifest_path: &Path) -> Result<Wrapped, WrapError> {
    let schema = CliSchema::from_manifest_path(manifest_path)?;
    let shape_errors = doctor::schema_shape_errors(&schema);
    if !shape_errors.is_empty() {
        return Err(WrapError::InvalidShape(shape_errors));
    }
    let executor = OcelExecutor::new(ProcessExecutor::new(executable.into()), schema.clone());
    let deploy = Deploy::from_schema(schema);
    Ok(Wrapped { deploy, executor })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        std::env::temp_dir().join(format!("cnv-any-lib-{label}-{nanos}.json"))
    }

    fn manifest_json() -> &'static str {
        r#"{
            "name": "greet-fixture",
            "about": null,
            "commands": [
                {
                    "path": ["greet"],
                    "about": "Greet someone",
                    "arguments": [
                        {
                            "id": "name",
                            "long": null,
                            "short": null,
                            "required": true,
                            "positional": true,
                            "kind": "string",
                            "behavior": "value"
                        }
                    ],
                    "callable": true
                }
            ]
        }"#
    }

    #[test]
    fn wrap_loads_manifest_into_deploy_schema() {
        // Arrange
        let path = unique_temp_path("wrap");
        fs::write(&path, manifest_json()).expect("write real manifest file");

        // Act
        let wrapped = wrap("/bin/echo", &path).expect("wrap a real executable path");

        // Assert
        assert_eq!(wrapped.deploy().schema().name, "greet-fixture");
        assert_eq!(wrapped.deploy().schema().commands.len(), 1);
        assert!(wrapped.deploy().schema().commands[0].callable);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn wrap_reports_manifest_error_for_missing_manifest() {
        let path = unique_temp_path("missing");
        let error = wrap("/bin/echo", &path).expect_err("missing manifest must error");
        assert!(matches!(error, WrapError::Manifest(_)));
    }

    #[test]
    fn ocel_executor_derives_noun_verb_from_manifest_command_path() {
        // Arrange
        let path = unique_temp_path("noun-verb");
        fs::write(&path, manifest_json()).expect("write real manifest file");
        let schema = CliSchema::from_manifest_path(&path).expect("parse real manifest");
        let executor =
            OcelExecutor::new(clap_noun_verb_deploy::ProcessExecutor::new("noop"), schema);

        // Act
        let (noun, verb) = executor.noun_verb_for(&["greet".to_owned(), "World".to_owned()]);

        // Assert: single-segment command path -- noun and verb both "greet"
        assert_eq!(noun, "greet");
        assert_eq!(verb, "greet");

        fs::remove_file(&path).ok();
    }
}
