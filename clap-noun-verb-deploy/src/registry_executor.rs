//! Closes the real gap the "unwired deployment crate" innovation finding
//! surfaced: `Deploy::from_command`/`from_registry` already project a real
//! `clap_noun_verb` command graph into a schema, but nothing in this crate
//! actually EXECUTED a call back through the real, production
//! `clap_noun_verb::cli::CommandRegistry` dispatch path (with its real
//! `Guard`/`Receipt`/OCEL machinery already wired in) -- every existing
//! `Executor` (`ProcessExecutor`) only spawns a separate child process.
//!
//! [`RegistryExecutor`] is the missing in-process half: it drives a real
//! invocation straight through `CommandRegistry::execute_single_step`, the
//! exact same argv-to-`HandlerOutput` pipeline `cli::run()` itself uses --
//! so serving a `clap-noun-verb` binary's own registry over MCP/HTTP needs
//! no subprocess at all.

use crate::{Execution, Executor, Invocation};
use clap_noun_verb::cli::CommandRegistry;
use thiserror::Error;

/// An [`Executor`] that dispatches directly through the real, in-process,
/// process-wide `clap_noun_verb::cli::CommandRegistry` -- no subprocess.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegistryExecutor;

#[derive(Debug, Error)]
pub enum RegistryExecutorError {
    /// The process-wide registry `Mutex` was poisoned by an earlier panic
    /// in another caller -- surfaced explicitly rather than silently
    /// recovering, since a poisoned registry may hold inconsistent state.
    #[error("the clap-noun-verb registry lock was poisoned by an earlier panic")]
    LockPoisoned,
}

impl Executor for RegistryExecutor {
    type Error = RegistryExecutorError;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
        let registry_lock = CommandRegistry::get();
        let registry =
            registry_lock.lock().map_err(|_| RegistryExecutorError::LockPoisoned)?;

        // `execute_single_step` expects a real argv, args[0] being the
        // (unused, for parsing purposes only) binary name -- exactly what
        // clap::Command::try_get_matches_from expects.
        let mut argv = vec!["clap-noun-verb-registry-executor".to_owned()];
        argv.extend(invocation.args.iter().cloned());

        match registry.execute_single_step(argv) {
            Ok(output) => Ok(Execution {
                exit_code: 0,
                stdout: output.to_json().unwrap_or_default(),
                stderr: String::new(),
            }),
            Err(error) => Ok(Execution { exit_code: 1, stdout: String::new(), stderr: error.to_string() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap_noun_verb::logic::{HandlerInput, HandlerOutput};

    #[test]
    fn registry_executor_dispatches_a_real_registered_verb_in_process() {
        // Arrange: a real verb registered on the real, process-wide
        // cli::CommandRegistry, under a real noun. Note: `register_noun`
        // must be called explicitly -- `register_verb` alone only
        // populates the registry's `verbs` map (used by direct
        // `execute_verb` calls, e.g. tests/guard_dispatch_integration.rs),
        // not the separate `nouns` map `build_command()` walks to emit
        // clap subcommands, so a verb registered without also calling
        // `register_noun` is invisible to real argv-based dispatch --
        // exactly the real gap `RegistryExecutor` (which dispatches via
        // argv, like a real MCP `tools/call`) would hit without this.
        CommandRegistry::register_noun(
            "registry_executor_probe_noun",
            "A real noun proving RegistryExecutor dispatches in-process",
        );
        CommandRegistry::register_verb(
            "registry_executor_probe_noun",
            "ping",
            "A real verb proving RegistryExecutor dispatches in-process",
            |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
                HandlerOutput::from_data(serde_json::json!({"pong": true}))
            },
        );

        // Act: the real Executor::execute path, no subprocess.
        let executor = RegistryExecutor;
        let invocation =
            Invocation::new(vec!["registry_executor_probe_noun".to_owned(), "ping".to_owned()]);
        let execution = executor.execute(&invocation).expect("real in-process dispatch");

        // Assert
        assert_eq!(execution.exit_code, 0);
        assert!(execution.success());
        let parsed: serde_json::Value =
            serde_json::from_str(&execution.stdout).expect("real JSON output");
        assert_eq!(parsed["pong"], serde_json::json!(true));
    }

    #[test]
    fn registry_executor_surfaces_a_real_dispatch_error_without_panicking() {
        let executor = RegistryExecutor;
        let invocation = Invocation::new(vec!["definitely_not_a_registered_noun".to_owned()]);
        let execution = executor.execute(&invocation).expect("real in-process dispatch");

        assert!(!execution.success());
        assert_eq!(execution.exit_code, 1);
        assert!(!execution.stderr.is_empty());
    }
}
