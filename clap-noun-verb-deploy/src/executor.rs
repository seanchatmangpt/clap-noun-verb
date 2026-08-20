use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::process::Command;
use thiserror::Error;

/// A bounded request to invoke the configured CLI executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invocation {
    /// Arguments after argv[0].
    pub args: Vec<String>,
    /// Explicit environment additions for this invocation.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl Invocation {
    /// Construct an invocation with no environment additions.
    #[must_use]
    pub fn new(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self { args: args.into_iter().map(Into::into).collect(), env: BTreeMap::new() }
    }

    /// Add one explicit environment value. Admission and executor policy still apply.
    #[must_use]
    pub fn with_env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(name.into(), value.into());
        self
    }
}

/// Receipt-shaped execution result returned by an [`Executor`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Execution {
    /// Child-process exit code. Signal termination maps to a non-zero value.
    pub exit_code: i32,
    /// Captured stdout as UTF-8 with replacement for invalid byte sequences.
    pub stdout: String,
    /// Captured stderr as UTF-8 with replacement for invalid byte sequences.
    pub stderr: String,
}

impl Execution {
    /// Whether the process returned exit code zero.
    #[must_use]
    pub const fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Sole actuation boundary used by protocol adapters.
pub trait Executor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error>;
}

/// Executes a deployed CLI as one explicitly configured child process.
#[derive(Debug, Clone)]
pub struct ProcessExecutor {
    executable: OsString,
    clear_env: bool,
    base_env: BTreeMap<String, String>,
    invocation_env_allowlist: BTreeSet<String>,
}

impl ProcessExecutor {
    /// Pin the executor to one executable. Per-invocation environment is denied by default.
    #[must_use]
    pub fn new(executable: impl Into<OsString>) -> Self {
        Self {
            executable: executable.into(),
            clear_env: false,
            base_env: BTreeMap::new(),
            invocation_env_allowlist: BTreeSet::new(),
        }
    }

    /// Clear the inherited process environment before applying explicit values.
    #[must_use]
    pub const fn clear_env(mut self, clear: bool) -> Self {
        self.clear_env = clear;
        self
    }

    /// Add an environment value controlled by the embedding application.
    #[must_use]
    pub fn with_env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.base_env.insert(name.into(), value.into());
        self
    }

    /// Permit one per-invocation environment name at the final process boundary.
    #[must_use]
    pub fn allow_invocation_env(mut self, name: impl Into<String>) -> Self {
        self.invocation_env_allowlist.insert(name.into());
        self
    }
}

#[derive(Debug, Error)]
pub enum ProcessExecutionError {
    #[error("invocation environment variable '{0}' is not allowed by the executor")]
    EnvironmentRefused(String),
    #[error("failed to execute deployed CLI: {0}")]
    Io(#[from] std::io::Error),
}

impl Executor for ProcessExecutor {
    type Error = ProcessExecutionError;

    fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
        if let Some(name) = invocation
            .env
            .keys()
            .find(|name| !self.invocation_env_allowlist.contains(*name))
        {
            return Err(ProcessExecutionError::EnvironmentRefused(name.clone()));
        }

        let mut command = Command::new(&self.executable);
        if self.clear_env {
            command.env_clear();
        }
        let output =
            command.args(&invocation.args).envs(&self.base_env).envs(&invocation.env).output()?;

        Ok(Execution {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
