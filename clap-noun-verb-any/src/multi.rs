//! Serve several distinct `cnv-any`-wrapped executables from a single
//! [`Executor`] -- the seam that lets one `clap_noun_verb_deploy::mcp::McpServer`
//! (or any other single-`Executor` protocol surface) admit tools from N
//! wrapped foreign binaries at once, instead of one server per binary.
//!
//! [`MultiExecutor`] dispatches an [`Invocation`] to the correct inner
//! [`OcelExecutor<ProcessExecutor>`] by matching the invocation's first
//! argument (its command's leading path segment / noun) against the noun
//! each entry was registered under. The corresponding merged [`CliSchema`]
//! ([`merge_schemas`]) is what a caller hands to `McpServer::new` alongside
//! a `MultiExecutor` -- so `tools/list` legitimately lists every wrapped
//! target's tools, and `tools/call` routes each call to the right process.

use crate::OcelExecutor;
use clap_noun_verb_deploy::{
    CliSchema, Execution, Executor, Invocation, ProcessExecutionError, ProcessExecutor,
};
use thiserror::Error;

/// Failure dispatching through a [`MultiExecutor`].
#[derive(Debug, Error)]
pub enum MultiExecutorError {
    /// No registered entry's noun matches this invocation's first argument
    /// (or the invocation had no arguments at all).
    #[error("no wrapped target admits a command starting with '{0}'")]
    UnknownNoun(String),
    /// The matched entry's own process execution failed.
    #[error(transparent)]
    Process(#[from] ProcessExecutionError),
}

/// Dispatches an [`Invocation`] to one of several wrapped executors, keyed
/// by each command's noun (its path's first segment).
#[derive(Default)]
pub struct MultiExecutor {
    entries: Vec<(String, OcelExecutor<ProcessExecutor>)>,
}

impl MultiExecutor {
    /// An empty dispatcher; add entries with [`Self::add`].
    #[must_use]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Register `executor` to handle every invocation whose first argument
    /// is exactly `noun`.
    pub fn add(&mut self, noun: impl Into<String>, executor: OcelExecutor<ProcessExecutor>) {
        self.entries.push((noun.into(), executor));
    }

    /// The nouns currently registered, in registration order.
    #[must_use]
    pub fn nouns(&self) -> Vec<&str> {
        self.entries.iter().map(|(noun, _)| noun.as_str()).collect()
    }
}

impl Executor for MultiExecutor {
    type Error = MultiExecutorError;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
        let Some(first) = invocation.args.first() else {
            return Err(MultiExecutorError::UnknownNoun(String::new()));
        };
        let Some((_, executor)) = self.entries.iter().find(|(noun, _)| noun == first) else {
            return Err(MultiExecutorError::UnknownNoun(first.clone()));
        };
        Ok(executor.execute(invocation)?)
    }
}

/// Merge several `(noun, schema)` pairs into one [`CliSchema`] whose
/// `commands` is the union of every input schema's commands -- the
/// counterpart callers pass to `McpServer::new` alongside a
/// [`MultiExecutor`] built from the same wrapped targets, in the same
/// order. `name`/`about` describe the merged deployment as a whole.
#[must_use]
pub fn merge_schemas(name: impl Into<String>, about: Option<String>, schemas: &[CliSchema]) -> CliSchema {
    let commands = schemas.iter().flat_map(|schema| schema.commands.iter().cloned()).collect();
    CliSchema { name: name.into(), about, commands }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrap;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn multi_executor_routes_each_invocation_to_its_own_wrapped_target() {
        let fixtures = fixtures_dir();
        let greet =
            wrap(fixtures.join("greet.sh").into_os_string(), &fixtures.join("cnv-any.json"))
                .expect("wrap real greet.sh fixture");
        let calc = wrap(fixtures.join("calc.sh").into_os_string(), &fixtures.join("calc.json"))
            .expect("wrap real calc.sh fixture");

        let (greet_deploy, greet_executor) = greet.into_parts();
        let (calc_deploy, calc_executor) = calc.into_parts();

        let mut multi = MultiExecutor::new();
        multi.add("greet", greet_executor);
        multi.add("add", calc_executor);
        assert_eq!(multi.nouns(), vec!["greet", "add"]);

        let greet_invocation = greet_deploy
            .schema()
            .build_invocation(
                "greet",
                &serde_json::json!({"name": "World"}).as_object().cloned().unwrap(),
            )
            .expect("real greet invocation");
        let calc_invocation = calc_deploy
            .schema()
            .build_invocation(
                "add",
                &serde_json::json!({"a": "2", "b": "3"}).as_object().cloned().unwrap(),
            )
            .expect("real calc invocation");

        let greet_result = multi.execute(&greet_invocation).expect("routes to greet.sh");
        assert_eq!(greet_result.stdout, "Hello, World!\n");

        let calc_result = multi.execute(&calc_invocation).expect("routes to calc.sh");
        assert_eq!(calc_result.stdout, "5\n");
    }

    #[test]
    fn multi_executor_refuses_an_invocation_naming_no_registered_noun() {
        let multi = MultiExecutor::new();
        let error = multi
            .execute(&Invocation::new(vec!["nonexistent".to_owned()]))
            .expect_err("no registered noun must be refused");
        assert!(matches!(error, MultiExecutorError::UnknownNoun(noun) if noun == "nonexistent"));
    }

    #[test]
    fn merge_schemas_unions_commands_from_every_input_schema() {
        let fixtures = fixtures_dir();
        let greet =
            wrap(fixtures.join("greet.sh").into_os_string(), &fixtures.join("cnv-any.json"))
                .expect("wrap real greet.sh fixture");
        let calc = wrap(fixtures.join("calc.sh").into_os_string(), &fixtures.join("calc.json"))
            .expect("wrap real calc.sh fixture");

        let merged = merge_schemas(
            "multi-cli-demo",
            Some("Serves greet.sh and calc.sh from one schema".to_owned()),
            &[greet.deploy().schema().clone(), calc.deploy().schema().clone()],
        );

        assert_eq!(merged.name, "multi-cli-demo");
        assert_eq!(merged.commands.len(), 2);
        assert!(merged.commands.iter().any(|c| c.tool_name() == "greet"));
        assert!(merged.commands.iter().any(|c| c.tool_name() == "add"));
    }
}
